use std::sync::Arc;

use anyhow::Result;
use quinn::{Connecting, Endpoint, RecvStream, SendStream};

#[derive(Clone)]
pub struct ServerCtx {
    inner: Arc<()>,
}

pub async fn server(endpoint: Endpoint) -> Result<()> {
    let ctx = ServerCtx {
        inner: Arc::new(()),
    };
    while let Some(conn) = endpoint.accept().await {
        tokio::spawn(handle_server_connection(conn, ctx.clone()));
    }
    Ok(())
}

async fn handle_server_connection(conn: Connecting, ctx: ServerCtx) {
    let remote_addr = conn.remote_address();
    match conn.await {
        Ok(conn) => match conn.accept_bi().await {
            Ok((send, recv)) => {
                tokio::spawn(handle_server_stream(send, recv, ctx));
            }
            Err(e) => println!("Server: stream accept failed from {remote_addr}, reason: {e}"),
        },
        Err(e) => println!("Server: connection refused from {remote_addr}, reason: {e}"),
    };
}

async fn handle_server_stream(_: SendStream, _: RecvStream, _: ServerCtx) {
    println!("Server: stream accepted");
}
