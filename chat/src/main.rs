use std::{env, io::Cursor};
use anyhow::Result;
use bytes::{Buf, BufMut, BytesMut};
use tokio::{
    io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{tcp::{OwnedReadHalf, OwnedWriteHalf}, TcpListener, TcpStream},
    sync::broadcast,
};

const SERVER_ADDR: &str = "0.0.0.0:8080";
const CLIENT_ADDR: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        Some("server") => run_server(SERVER_ADDR).await?,
        Some("client") => run_client(CLIENT_ADDR).await?,
        _ => println!("Usage: chat server | chat client"),
    }

    Ok(())
}

async fn run_server(addr: &str) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("Server listening on {addr}");

    let (tx, _rx) = broadcast::channel::<(u64, String)>(128);

    let mut next_client_id: u64 = 1;
    loop {
        let (stream, peer) = listener.accept().await?;
        println!("New client: {peer}");

        let (reader, writer) = stream.into_split();
        let tx_client = tx.clone();
        let rx = tx.subscribe();
        let client_id = next_client_id;
        next_client_id += 1;

        tokio::spawn(async move {
            let res: Result<()> = tokio::select! {
                res = reader_task(client_id, reader, tx_client) => { res }
                res = writer_task(client_id, writer, rx) => { res }
            };
            if let Err(err) = res {
                println!("Client {client_id} error: {err:?}");
            }
            println!("Client {client_id} disconnected");
        });
    }
}

async fn run_client(addr: &str) -> Result<()> {
    let stream = TcpStream::connect(addr).await?;
    println!("Connected to {addr}");

    let (reader, writer) = stream.into_split();

    tokio::select! {
        res = send_stdin(writer) => { res?; }
        res = read_socket(reader) => { res?; }
    }

    Ok(())
}

async fn send_stdin(mut writer: OwnedWriteHalf) -> Result<()> {
    let buffer = &mut BytesMut::new();
    let mut lines = BufReader::new(io::stdin()).lines();

    while let Some(line) = lines.next_line().await? {
        buffer.put_u32(line.len() as u32);
        buffer.put_slice(line.as_bytes());
        writer.write_all_buf(buffer).await?;
    }

    Ok(())
}

async fn read_socket(mut reader: OwnedReadHalf) -> Result<()> {
    let buffer = &mut BytesMut::new();
    while reader.read_buf(buffer).await? != 0 {
        while let Some(message) = parse_message(buffer) {
            println!("{message}");
        }
    }
    println!("Connection closed by server");

    Ok(())
}

async fn writer_task(
    client_id: u64,
    mut writer: OwnedWriteHalf,
    mut rx: broadcast::Receiver<(u64, String)>,
) -> Result<()> {
    let buffer = &mut BytesMut::new();
    while let Ok((from, text)) = rx.recv().await {
        if from == client_id {
            continue;
        }
        buffer.put_u32(text.len() as u32);
        buffer.put_slice(text.as_bytes());
        writer.write_all_buf(buffer).await?;
    }
    Ok(())
}

async fn reader_task(
    client_id: u64,
    mut reader: OwnedReadHalf,
    tx: broadcast::Sender<(u64, String)>,
) -> Result<()> {
    let buffer = &mut BytesMut::new();
    while reader.read_buf(buffer).await? != 0 {
        while let Some(message) = parse_message(buffer) {
            let _ = tx.send((client_id, message));
        }
    }
    Ok(())
}

fn parse_message(buffer: &mut BytesMut) -> Option<String> {
    let cursor = &mut Cursor::new(&buffer);

    let len = cursor.try_get_u32().ok()? as usize;
    if len > cursor.remaining() {
        return None;
    }

    let mut vec = vec![0; len];
    cursor.try_copy_to_slice(&mut vec).ok()?;

    buffer.advance(cursor.position() as usize);

    let message = String::from_utf8(vec).ok()?;
    Some(message)
}
