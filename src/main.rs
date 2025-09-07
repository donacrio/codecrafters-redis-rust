#![allow(unused_imports)]
use core::net::SocketAddr;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                tokio::spawn(async move {
                    if let Err(e) = process_stream(stream, addr).await {
                        println!("Error: {}", e);
                    }
                });
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}

async fn process_stream(mut stream: TcpStream, addr: SocketAddr) -> io::Result<()> {
    let mut buffer: [u8; 512] = [0; 512];

    loop {
        stream.readable().await?;

        match stream.read(&mut buffer).await {
            Ok(0) => {
                println!("Socket {} closed", addr);
                break;
            }
            Ok(n) => {
                println!("Socket {} read {} bytes", addr, n);
                stream.writable().await?;
                match stream.write(b"+PONG\r\n").await {
                    Ok(0) => {
                        println!("Socket {} closed", addr);
                        break;
                    }
                    Ok(_) => continue,
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        println!("Error writing to {}: {}", addr, e);
                        break;
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                println!("Error reading from {}: {}", addr, e);
                break;
            }
        }
    }
    Ok(())
}
