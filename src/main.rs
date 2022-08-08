use std::{net::SocketAddr, sync::Arc, time::Instant};

use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use quinn::{ClientConfig, Endpoint, ServerConfig};
use rand::{distributions::Alphanumeric, Rng};
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

    let byte_sizes: Vec<usize> = (2..8)
        .collect::<Vec<u32>>()
        .iter()
        .map(|i| 10_usize.pow(*i))
        .collect();
    let request_nums: Vec<usize> = (2..5)
        .collect::<Vec<u32>>()
        .iter()
        .map(|i| 10_usize.pow(*i))
        .collect();

    let mut rtts = vec![];

    for size in byte_sizes {
        for num in &request_nums {
            for _ in 0..*num {
                let msg: String = rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(size)
                    .map(char::from)
                    .collect();
                let t = Instant::now();
                let msg = bincode::serialize(&msg).unwrap();
                let size = (msg.len() as u32).to_le_bytes();
                send.write_all(&size).await?;
                send.write_all(&msg).await?;

                let mut size = [0u8; 4];
                recv.read_exact(&mut size).await?;
                let size = u32::from_le_bytes(size);
                let mut buffer = vec![0u8; size as usize];
                recv.read_exact(&mut buffer).await?;
                let _ = bincode::deserialize::<String>(&buffer).unwrap();
                rtts.push(t.elapsed().as_secs_f32());
            }
            let bs = size as f32 / (rtts.iter().sum::<f32>() / *num as f32);
            let mbs = bs / 1_000_000 as f32;
            let b = size as f32 / 1_000_000 as f32;
            println!("sending {b} Mb, requests {num}, throughtput {mbs} Mbs");
        }
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
            let s: String = bincode::deserialize(&buffer).unwrap();
            println!("Recv msg with {} bytes", buffer.len());

            let buffer = bincode::serialize(&s).unwrap();
            let size = (buffer.len() as u32).to_le_bytes();
            send.write_all(&size).await?;
            send.write_all(&buffer).await?;
        }
    }
    Ok(())
}
