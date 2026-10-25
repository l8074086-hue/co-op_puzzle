use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Sender;

pub fn run_server(tx: Sender<String>) -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;

    let _ = tx.send("Waiting for player...".into());

    let (stream, _) = listener.accept()?;

    let _ = tx.send("Player connected!".into());

    handle_connection(stream, tx)
}

pub fn run_client(tx: Sender<String>) -> std::io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:7878")?;

    let _ = tx.send("Connected to server!".into());

    handle_connection(stream, tx)
}

fn handle_connection(mut stream: TcpStream, tx: Sender<String>) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);

    loop {
        let mut message = String::new();
        let bytes_read = reader.read_line(&mut message)?;

        if bytes_read == 0 {
            let _ = tx.send("Disconnected.".into());
            break;
        }

        let message = message.trim().to_string();
        let _ = tx.send(format!("Peer: {message}"));

        stream.write_all(b"Hello from peer!\n")?;
    }

    Ok(())
}
