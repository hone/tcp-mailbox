use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

#[derive(Debug, Eq, PartialEq)]
enum Request {
    Publish(String),
    Retrieve,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878").await?;
    let storage = Arc::new(Mutex::new(VecDeque::new()));

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let mut thread_handle = Arc::clone(&storage);
                tokio::spawn(async move { handle_client(stream, &mut thread_handle).await });
            }
            Err(e) => eprintln!("Error connecting: {}", e),
        }
    }
}

async fn handle_client(
    mut stream: TcpStream,
    storage: &Mutex<VecDeque<String>>,
) -> anyhow::Result<()> {
    println!("Client connected!");
    let line = read_line(&mut stream).await?;
    let request = parse_request(line);

    match request {
        Request::Publish(msg) => {
            let mut guard = storage.lock().unwrap();
            guard.push_back(msg);
        }
        Request::Retrieve => {
            let maybe_msg = storage.lock().unwrap().pop_front();
            match maybe_msg {
                Some(msg) => stream.write_all(msg.as_bytes()).await?,
                None => stream.write_all(b"no message available").await?,
            }
        }
    }

    Ok(())
}

fn parse_request(line: String) -> Request {
    let trimmed = line.trim_end();

    if trimmed == "" {
        Request::Retrieve
    } else {
        Request::Publish(String::from(trimmed))
    }
}

async fn read_line(stream: &mut TcpStream) -> anyhow::Result<String> {
    let mut buffered_reader = BufReader::new(stream);

    let mut buf = String::new();
    buffered_reader.read_line(&mut buf).await?;

    Ok(buf)
}
