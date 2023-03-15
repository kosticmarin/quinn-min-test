use std::{net::SocketAddr, time::Duration};

use anyhow::Result;
use quinn::Endpoint;

pub async fn client(endpont: Endpoint, server_addr: SocketAddr) -> Result<()> {
    println!("Client: connecting to server {server_addr}");
    let conn = endpont.connect(server_addr, "localhost")?.await?;
    println!("Client: connected to server {server_addr}");
    let _ = conn.open_bi().await?;
    println!("Client: stream opened to server {server_addr}");
    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok(())
}
