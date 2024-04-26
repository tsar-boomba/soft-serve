use std::{
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
};

use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;

#[derive(Debug, Parser)]
#[command(name = "sfs - Soft Serve", version, about = "Easily serve your filesystem over HTTP, FTP, or TFTP. Defaults to HTTP.", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short, long, default_value = "5001")]
    port: u16,

    #[arg(short, long, default_value = "127.0.0.1")]
    ip: IpAddr,

    /// When serving over HTTP, the base path will be treated as /index.html for convenience when serving websites
    #[arg(long)]
    no_index_convenience: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Ftp {
        /// The path to serve files from
        #[arg(default_value = ".")]
        path: PathBuf,

        #[arg(short, long, default_value = "5002")]
        port: u16,

        #[arg(short, long, default_value = "127.0.0.1")]
        ip: IpAddr,

        #[arg(short, long)]
        trivial: bool,
    },
    Http {
        /// The path to serve files from
        #[arg(default_value = ".")]
        path: PathBuf,

        #[arg(short, long, default_value = "5001")]
        port: u16,

        #[arg(short, long, default_value = "127.0.0.1")]
        ip: IpAddr,

        /// When serving over HTTP, the base path will be treated as /index.html for convenience when serving websites
        #[arg(long)]
        no_index_convenience: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    if cfg!(debug_assertions) || std::env::var("RUST_LOG").is_ok() {
        tracing_subscriber::fmt()
            // Use RUST_LOG env var or debug in debug build
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug")),
            )
            .init();
    }

    let cli = Cli::parse();

    tracing::debug!("Args: {cli:#?}");

    match cli.command {
        Some(Command::Http {
            path,
            port,
            ip,
            no_index_convenience,
        }) => {
            let root_path = Arc::<Path>::from(path.canonicalize()?.as_path());

            tracing::debug!("Root path: {}", root_path.display());

            soft_serve::http::serve(root_path, SocketAddr::new(ip, port), no_index_convenience)
                .await?
        }
        Some(Command::Ftp {
            path,
            port,
            ip,
            trivial,
        }) => {
            let root_path = Arc::<Path>::from(path.canonicalize()?.as_path());

            tracing::debug!("Root path: {}", root_path.display());

            if !trivial {
                soft_serve::ftp::serve(root_path, SocketAddr::new(ip, port)).await?
            } else {
                soft_serve::ftp::serve_trivial(root_path, SocketAddr::new(ip, port)).await?
            }
        }
        _ => {}
    }

    let root_path = Arc::<Path>::from(cli.path.canonicalize()?.as_path());

    tracing::debug!("Root path: {}", root_path.display());

    soft_serve::http::serve(
        root_path,
        SocketAddr::new(cli.ip, cli.port),
        cli.no_index_convenience,
    )
    .await?;

    Ok(())
}
