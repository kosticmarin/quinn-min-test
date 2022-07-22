use std::{net::SocketAddr, sync::Arc};

use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use quinn::{ClientConfig, Endpoint, ServerConfig};
use rustls_pemfile::Item;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let socket: SocketAddr = args[2].parse()?;

    match args[1].as_ref() {
        "server" => server(socket).await,
        "client" => client(socket).await,
        _ => Err(anyhow!("First argument needs to be server or client.")),
    }
}

async fn client(socket: SocketAddr) -> Result<()> {
    let mut cert = include_str!("../certs/root.pem").as_bytes();
    let cert = rustls_pemfile::read_one(&mut cert)?.unwrap();
    let cert = match cert {
        Item::X509Certificate(cert) => Ok(rustls::Certificate(cert)),
        _ => Err(anyhow!("Not a cert")),
    }?;
    let mut certs = rustls::RootCertStore::empty();
    certs.add(&cert)?;
    let client_crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(certs)
        .with_no_client_auth();
    let client_config = ClientConfig::new(Arc::new(client_crypto));
    let mut endpoint = Endpoint::client("[::]:0".parse().unwrap())?;
    endpoint.set_default_client_config(client_config);

    let new_conn = endpoint.connect(socket, "server")?.await?;
    let quinn::NewConnection {
        connection: conn, ..
    } = new_conn;
    let (mut send, mut recv) = conn
        .open_bi()
        .await
        .map_err(|e| anyhow!("failed to open stream: {}", e))?;

    for i in 0..16 {
        let msg = vec![0; (2usize).pow(i)];
        let size = (msg.len() as u32).to_le_bytes();
        println!("Message {i}: {} bytes", msg.len());
        send.write_all(&size).await?;
        send.write_all(&msg).await?;

        let mut size = [0u8; 4];
        recv.read_exact(&mut size).await?;
        let size = u32::from_le_bytes(size);
        println!("Reading msg with {} bytes", size);
        let mut buffer = vec![0u8; size as usize];
        recv.read_exact(&mut buffer).await?;
        println!("Received msg with {} bytes", buffer.len());
    }
    Ok(())
}

async fn server(socket: SocketAddr) -> Result<()> {
    let mut cert = include_str!("../certs/server.pem").as_bytes();
    let mut key = include_str!("../certs/server.pk.pem").as_bytes();
    let pk = rustls_pemfile::read_one(&mut key)?.unwrap();
    let pk = match pk {
        Item::PKCS8Key(key) => Ok(rustls::PrivateKey(key)),
        _ => Err(anyhow!("Not a pk")),
    }?;
    let cert = rustls_pemfile::read_one(&mut cert)?.unwrap();
    let cert = match cert {
        Item::X509Certificate(cert) => Ok(rustls::Certificate(cert)),
        _ => Err(anyhow!("Not a cert")),
    }?;
    let cert = vec![cert];
    let server_crypto = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert, pk)?;
    let mut server_config = ServerConfig::with_crypto(Arc::new(server_crypto));
    Arc::get_mut(&mut server_config.transport)
        .unwrap()
        .max_concurrent_uni_streams(0_u8.into());

    let (endpoint, mut incoming) = quinn::Endpoint::server(server_config, socket)?;
    println!("listening on {}", endpoint.local_addr()?);

    while let Some(conn) = incoming.next().await {
        println!("connection incoming");
        tokio::spawn(handle_conn_wrapper(conn));
    }
    Ok(())
}

async fn handle_conn_wrapper(conn: quinn::Connecting) {
    if let Err(e) = handle_conn(conn).await {
        println!("Error handling connection {e}");
    }
}

#[allow(unused)]
async fn handle_conn(connection: quinn::Connecting) -> Result<()> {
    let quinn::NewConnection {
        connection,
        mut bi_streams,
        ..
    } = connection.await?;
    while let Some(stream) = bi_streams.next().await {
        let (mut send, mut recv) = stream?;
        loop {
            let mut size = [0u8; 4];
            recv.read_exact(&mut size).await?;
            let size = u32::from_le_bytes(size);
            println!("Reading msg with {} bytes", size);
            let mut buffer = vec![0u8; size as usize];
            recv.read_exact(&mut buffer).await?;
            println!("Recv msg with {} bytes", buffer.len());

            let msg: Vec<u8> = vec![1, 2, 3, 4];
            let size = (msg.len() as u32).to_le_bytes();
            send.write_all(&size).await?;
            send.write_all(&msg).await?;
        }
    }
    Ok(())
}
