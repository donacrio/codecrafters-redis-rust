mod command;
mod resp;
mod store;

use crate::command::handle_value;
use crate::resp::{parse_resp, ParseError, Value};
use crate::store::Store;
use anyhow::{Context, Result};
use std::fmt::Write;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::{TcpListener, TcpStream};

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
        eprintln!("[info] socket {} connected", addr);

        let store = store.clone();
        tokio::spawn(async move {
            if let Err(e) = process_stream(stream, &store).await {
                eprintln!("[error] socket {}: {:?}", addr, e);
            }
            eprintln!("[info] socket {} closed", addr);
        });
    }
}

async fn process_stream(stream: TcpStream, store: &Store) -> Result<()> {
    let (read_stream, write_stream) = stream.into_split();
    let mut reader = BufReader::new(read_stream);
    let mut writer = BufWriter::new(write_stream);

    let mut read_buffer = Vec::with_capacity(4096);
    let mut write_buffer = String::new();

    loop {
        let bytes_read = reader
            .read_buf(&mut read_buffer)
            .await
            .context("failed to read from socket")?;

        if bytes_read == 0 {
            break;
        }

        process_buffer(&mut read_buffer, &mut write_buffer, &mut writer, store).await?;
    }
    Ok(())
}

async fn process_buffer(
    read_buffer: &mut Vec<u8>,
    write_buffer: &mut String,
    writer: &mut (impl AsyncWriteExt + Unpin),
    store: &Store,
) -> Result<()> {
    loop {
        match parse_resp(read_buffer) {
            Ok((value, bytes_consumed)) => {
                let result = handle_value(value, store);
                write_response(writer, write_buffer, &result).await?;
                read_buffer.drain(..bytes_consumed);
            }
            Err(ParseError::UnexpectedEOF) => break,
            Err(e) => {
                let result = Value::from(e);
                write_response(writer, write_buffer, &result).await?;
                read_buffer.clear();
                break;
            }
        }
    }
    Ok(())
}

async fn write_response(
    writer: &mut (impl AsyncWriteExt + Unpin),
    buf: &mut String,
    value: &Value,
) -> Result<()> {
    buf.clear();
    write!(buf, "{}", value).expect("write to String is infallible");
    writer
        .write_all(buf.as_bytes())
        .await
        .context("error writing to socket")?;
    writer.flush().await.context("error writing to socket")
}
