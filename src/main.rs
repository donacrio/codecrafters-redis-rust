mod command;
mod resp;

use anyhow::{Context, Result};
use core::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::command::execute;
use crate::resp::parse_command;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")
        .await
        .context("failed to start TCP listener on port 6379")?;

    loop {
        let (stream, addr) = listener
            .accept()
            .await
            .context("failed to accept connection")?;

        tokio::spawn(async move {
            if let Err(e) = process_stream(stream, addr).await {
                println!("Error: {}", e);
            }
        });
    }
}

async fn process_stream(mut stream: TcpStream, addr: SocketAddr) -> Result<()> {
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

        let response = match parse_command(bytes).and_then(execute) {
            Ok(v) => v,
            Err(e) => e.into(),
        };

        match stream.write_all(response.to_string().as_bytes()).await {
            Ok(()) => {
                continue;
            }
            Err(e) => {
                println!("Error writing to {}: {}", addr, e);
                break;
            }
        }
    }
    Ok(())
}
