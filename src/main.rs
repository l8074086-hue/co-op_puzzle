mod network;

use raylib::prelude::*;
use std::sync::mpsc;
use std::thread;

fn main() {
    let (mut rl, thread) = raylib::init().size(800, 600).title("Rust Co-op").build();

    rl.set_target_fps(60);

    let (tx, rx) = mpsc::channel::<String>();

    let args: Vec<String> = std::env::args().collect();
    let role = args.get(1).map(String::as_str).unwrap_or("client");

    // Networking runs separately from the game loop.
    let network_thread = match role {
        "server" => thread::spawn(move || {
            if let Err(error) = network::run_server(tx) {
                eprintln!("Server error: {error}");
            }
        }),
        _ => thread::spawn(move || {
            if let Err(error) = network::run_client(tx) {
                eprintln!("Client error: {error}");
            }
        }),
    };

    let mut x = 400.0;
    let mut y = 300.0;
    let mut status = String::from("Connecting...");

    while !rl.window_should_close() {
        // Receive messages without blocking the game.
        while let Ok(message) = rx.try_recv() {
            status = message;
        }

        let speed = 3.0;

        if rl.is_key_down(KeyboardKey::KEY_W) {
            y -= speed;
        }
        if rl.is_key_down(KeyboardKey::KEY_S) {
            y += speed;
        }
        if rl.is_key_down(KeyboardKey::KEY_A) {
            x -= speed;
        }
        if rl.is_key_down(KeyboardKey::KEY_D) {
            x += speed;
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        d.draw_text("Rust Co-op", 20, 20, 30, Color::DARKBLUE);
        d.draw_text("Move with WASD", 20, 65, 20, Color::DARKGRAY);
        d.draw_text(&format!("Network: {status}"), 20, 100, 20, Color::MAROON);

        d.draw_circle(x as i32, y as i32, 20.0, Color::BLUE);
    }

    // Don't join here: the network thread may be blocked waiting for a connection.
    drop(network_thread);
}
