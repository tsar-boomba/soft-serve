use std::{
    io::{self, ErrorKind},
    net::SocketAddr,
    path::Path,
    sync::Arc,
};

use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use http_body_util::{combinators::BoxBody, BodyExt, Full, StreamBody};
use hyper::{
    body::{Bytes, Frame, Incoming},
    header::CONTENT_TYPE,
    service::service_fn,
    Request, Response, StatusCode,
};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder,
};
use tokio::{io::BufReader, net::TcpListener};
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;

pub async fn serve(
    root_path: Arc<Path>,
    listen_addr: SocketAddr,
    no_index_convenience: bool,
) -> Result<()> {
    let tcp_listener = TcpListener::bind(listen_addr).await?;
    println!(
        "HTTP server listening on {}",
        format!("http://{listen_addr}").cyan().underline()
    );

    loop {
        let root_path = root_path.clone();
        let (stream, addr) = match tcp_listener.accept().await {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Failed to accept connection: {e}");
                continue;
            }
        };

        let serve_connection = async move {
            let result = Builder::new(TokioExecutor::new())
                .serve_connection(
                    TokioIo::new(stream),
                    service_fn({
                        move |req: hyper::Request<Incoming>| {
                            handle_request(root_path.clone(), no_index_convenience, req)
                        }
                    }),
                )
                .await;

            if let Err(e) = result {
                eprintln!("Error serving {addr}: {:?}", e.source());
            }
        };

        tokio::spawn(serve_connection);
    }
}

pub async fn handle_request(
    root_path: Arc<Path>,
    no_index_convenience: bool,
    req: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, io::Error>>> {
    let path = req.uri().path();
    tracing::debug!("Path: {}", path);

    let raw_file_path = root_path.join(&path[1..]);

    tracing::debug!("Root + path: {}", raw_file_path.display());

    let mut file_path = match raw_file_path.canonicalize() {
        Ok(file_path) => file_path,
        Err(err) => match err.kind() {
            ErrorKind::NotFound => {
                println!("{} {}", "404".yellow().bold(), path);
                return Ok(not_found(path));
            }
            _ => {
                if let Some(err_no) = err.raw_os_error() {
                    if err_no == 20 || err_no == 21 {
                        // Is/Not a dir
                        println!("{} {}", "404".yellow().bold(), path);
                        return Ok(not_found(path));
                    }
                }

                return Err(err.into());
            }
        },
    };

    tracing::debug!("Canonicalized req path: {}", raw_file_path.display());

    if !file_path.starts_with(root_path.as_ref()) {
        // Someone is trying to access files outside of the root directory

        tracing::warn!("Malicious request path: {}", path);
        return Ok(not_found(path));
    }

    let mut file = tokio::fs::OpenOptions::new()
        .read(true)
        .open(&file_path)
        .await?;

    let meta = file.metadata().await?;

    if meta.is_dir() {
        if no_index_convenience {
            return Ok(not_found(path));
        } else {
            // Convenience for serving websites
            let new_file_path = file_path.join("index.html");
            file = match tokio::fs::OpenOptions::new()
                .read(true)
                .open(&new_file_path)
                .await
            {
                Ok(file) => file,
                Err(_) => {
                    println!("{} {}", "404".yellow().bold(), path);
                    return Ok(not_found(path));
                }
            };

            if file.metadata().await?.is_dir() {
                println!("{} {}", "404".yellow().bold(), path);
                return Ok(not_found(path));
            }

            file_path = new_file_path;
            tracing::debug!("Remapped to {}", file_path.display());
        }
    }

    let stream = ReaderStream::new(BufReader::with_capacity(16 * 1024, file))
        .map(|read_res| read_res.map(Frame::data));
    let mime = mime_guess::from_path(&file_path).first_or_text_plain();

    println!("{} {}", "200".green().bold(), path);

    let response = Response::builder()
        .header(CONTENT_TYPE, mime.essence_str())
        .body(StreamBody::new(stream).boxed())
        .expect("values provided to the builder should be valid");

    Ok(response)
}

fn not_found<E>(path: &str) -> Response<BoxBody<Bytes, E>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(CONTENT_TYPE, "text/plain")
        .body(
            Full::new(Bytes::from(format!("Not found: {path}")))
                .map_err(|_| unreachable!("Creating not found body cannot fail."))
                .boxed(),
        )
        .unwrap()
}
