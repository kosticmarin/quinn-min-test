use anyhow::{anyhow, Result};
use std::{net::SocketAddr, str::FromStr, time::Duration};

mod client;
mod common;
mod manager;
mod message;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    let server_addr = SocketAddr::from_str("127.0.0.1:10410")?;
    let client_addr = SocketAddr::from_str("0.0.0.0:8000")?;
    let (server_endpoint, cert) =
        common::make_server_endpoint(server_addr).map_err(|e| anyhow!("{e}"))?;
    let client_endpoint =
        common::make_client_endpoint(client_addr, &[&cert]).map_err(|e| anyhow!("{e}"))?;

    println!("Starting Server");
    let server_handle = tokio::spawn(server::server_node(server_endpoint));
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("Starting Client");
    tokio::spawn(client::client_node(client_endpoint, server_addr));

    server_handle.await?
}
