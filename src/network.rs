// NETWORK.RS

pub use crate::types::NetMessage;
use crate::types::StatusMessage;

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub fn run_server(tx: Sender<NetMessage>, rx_out: Receiver<NetMessage>) -> std::io::Result<()> {
    let server_address = "0.0.0.0:7878";
    let listener = TcpListener::bind(server_address)?;

    let _ = tx.send(NetMessage::Status(StatusMessage {
        text: "Waiting for player...".into(),
    }));

    let (stream, _) = listener.accept()?;

    let _ = tx.send(NetMessage::Status(StatusMessage {
        text: "Player connected!".into(),
    }));

    handle_connection(stream, tx, rx_out)
}

pub fn run_client(tx: Sender<NetMessage>, rx_out: Receiver<NetMessage>) -> std::io::Result<()> {
    // For testing on the same computer.
    let server_address = "127.0.0.1:7878";
    let stream = TcpStream::connect(server_address)?;

    let _ = tx.send(NetMessage::Status(StatusMessage {
        text: "Connected to server!".into(),
    }));

    handle_connection(stream, tx, rx_out)
}

fn handle_connection(
    stream: TcpStream,
    tx: Sender<NetMessage>,
    rx_out: Receiver<NetMessage>,
) -> std::io::Result<()> {
    // One thread writes messages while this thread reads them.
    let mut write_stream = stream.try_clone()?;

    thread::spawn(move || {
        while let Ok(msg) = rx_out.recv() {
            let mut json_data = match serde_json::to_string(&msg) {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("Failed to serialize message: {err}");
                    continue;
                }
            };

            // Newline separates JSON messages.
            json_data.push('\n');

            if let Err(err) = write_stream.write_all(json_data.as_bytes()) {
                eprintln!("Failed to send message: {err}");
                break;
            }
        }
    });

    let mut reader = BufReader::new(stream);

    loop {
        let mut line = String::new();

        match reader.read_line(&mut line) {
            Ok(0) => {
                let _ = tx.send(NetMessage::Status(StatusMessage {
                    text: "Disconnected.".into(),
                }));
                break;
            }

            Ok(_) => match serde_json::from_str::<NetMessage>(line.trim()) {
                Ok(msg) => {
                    let _ = tx.send(msg);
                }

                Err(err) => {
                    eprintln!("Received invalid message: {err}");
                }
            },

            Err(err) => {
                eprintln!("Failed to read from connection: {err}");
                break;
            }
        }
    }

    Ok(())
}
