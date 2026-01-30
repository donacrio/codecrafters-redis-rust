mod command;
mod resp;
mod store;

use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::command::{execute, interpret_command};
use crate::resp::parse_resp;
use crate::store::Store;

#[tokio::main]
async fn main() -> Result<()> {
    let store = Arc::new(Store::default());

    let listener = TcpListener::bind("127.0.0.1:6379")
        .await
        .context("failed to start TCP listener on port 6379")?;

    loop {
        let (stream, addr) = listener
            .accept()
            .await
            .context("failed to accept connection")?;
        println!("Socket {} connected", addr);

        let store = store.clone();
        tokio::spawn(async move {
            process_stream(stream, addr, store)
                .await
                .context("Error processing stream")
        });
    }
}

async fn process_stream(mut stream: TcpStream, addr: SocketAddr, store: Arc<Store>) -> Result<()> {
    let mut buffer: [u8; 512] = [0; 512];

    loop {
        let bytes_read = stream
            .read(&mut buffer)
            .await
            .context("failed to read from socket")?;

        if bytes_read == 0 {
            println!("Socket {} closed", addr);
            break;
        }

        let bytes = &buffer[..bytes_read];

        let result = match parse_resp(bytes) {
            Ok(value) => match interpret_command(value) {
                Ok(command) => execute(command, &store),
                Err(e) => e.into(),
            },
            Err(e) => e.into(),
        };

        match stream.write_all(result.to_string().as_bytes()).await {
            Ok(()) => {
                continue;
            }
            Err(e) => {
                eprintln!("Error writing to {}: {}", addr, e);
                break;
            }
        }
    }
    Ok(())
}
