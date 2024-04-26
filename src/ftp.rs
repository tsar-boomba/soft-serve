use std::{net::SocketAddr, path::Path, sync::Arc};

use async_tftp::server::TftpServerBuilder;
use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use unftp_sbe_fs::ServerExt;

pub async fn serve(root_path: Arc<Path>, listen_addr: SocketAddr) -> Result<()> {
    let server = libunftp::Server::with_fs(root_path.to_path_buf());

    tracing::info!("FTP server listening on {}", listen_addr.to_string().cyan().underline());
    server.listen(listen_addr.to_string()).await?;
    Ok(())
}

pub async fn serve_trivial(root_path: Arc<Path>, listen_addr: SocketAddr) -> Result<()> {
    let tftpd = TftpServerBuilder::with_dir_ro(root_path)?
        .bind(listen_addr)
        .build()
        .await?;

    tracing::info!("TFTP server listening on {}", listen_addr.to_string().cyan().underline());
    tftpd.serve().await?;
    Ok(())
}
