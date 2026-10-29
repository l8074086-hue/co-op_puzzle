// MAIN.RS

mod network;
mod types;

use network::NetMessage;
use raylib::prelude::*;
use std::sync::mpsc;
use std::thread;

#[derive(Clone, Copy, PartialEq)]
enum Role {
    Client,
    Server,
}

const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 600.0;

const PLAYER_RADIUS: f32 = 20.0;
const PLAYER_SPEED: f32 = 180.0;

const BUTTON_RADIUS: f32 = 50.0;

// Server sends 20 snapshots per second.
const SNAPSHOT_INTERVAL: f32 = 1.0 / 20.0;

// Client sends input 30 times per second.
const INPUT_INTERVAL: f32 = 1.0 / 30.0;

// How quickly the displayed remote player catches up.
const REMOTE_SMOOTHING: f32 = 15.0;

// How quickly the local player's predicted position is corrected.
const RECONCILE_STRENGTH: f32 = 0.2;

fn clamp_player(x: &mut f32, y: &mut f32) {
    *x = x.clamp(PLAYER_RADIUS, SCREEN_WIDTH - PLAYER_RADIUS);
    *y = y.clamp(PLAYER_RADIUS, SCREEN_HEIGHT - PLAYER_RADIUS);
}

fn overlaps_button(x: f32, y: f32, bx: f32, by: f32) -> bool {
    let dx = x - bx;
    let dy = y - by;
    let radius = PLAYER_RADIUS + BUTTON_RADIUS;

    dx * dx + dy * dy <= radius * radius
}

fn main() {
    let (mut rl, raylib_thread) = raylib::init().size(800, 600).title("Rust Co-op").build();

    rl.set_target_fps(60);

    let (tx_in, rx_in) = mpsc::channel::<NetMessage>();
    let (tx_out, rx_out) = mpsc::channel::<NetMessage>();

    let args: Vec<String> = std::env::args().collect();

    let role = match args.get(1).map(String::as_str).unwrap_or("client") {
        "server" => Role::Server,
        "client" => Role::Client,
        _ => {
            eprintln!("Usage: cargo run -- server");
            eprintln!("   or: cargo run -- client");
            return;
        }
    };

    // Start the networking code away from the rendering thread.
    let _network_thread = match role {
        Role::Server => thread::spawn(move || {
            if let Err(err) = network::run_server(tx_in, rx_out) {
                eprintln!("Server error: {err}");
            }
        }),

        Role::Client => thread::spawn(move || {
            if let Err(err) = network::run_client(tx_in, rx_out) {
                eprintln!("Client error: {err}");
            }
        }),
    };

    // Player 1 is controlled by the server.
    // Player 2 is controlled by the client.
    let (mut x, mut y) = match role {
        Role::Server => (200.0_f32, 300.0_f32),
        Role::Client => (600.0_f32, 300.0_f32),
    };

    // Position of the other player as currently displayed.
    let (mut peer_x, mut peer_y) = match role {
        Role::Server => (600.0_f32, 300.0_f32),
        Role::Client => (200.0_f32, 300.0_f32),
    };

    // Latest authoritative position received for the remote player.
    let mut target_peer_x = peer_x;
    let mut target_peer_y = peer_y;

    let mut red_button = types::button {
        pressed: false,
        x: 100,
        y: 300,
        rad: BUTTON_RADIUS,
        color: Color::RED,
    };

    let blue_button = types::button {
        pressed: false,
        x: 400,
        y: 200,
        rad: BUTTON_RADIUS,
        color: Color::BLUE,
    };

    let mut blue_door_open = false;
    let mut status = String::from("Connecting...");
    let mut peer_connected = false;

    // Latest client input, used only by the server.
    let mut remote_move_x = 0.0_f32;
    let mut remote_move_y = 0.0_f32;

    let mut snapshot_timer = 0.0_f32;
    let mut input_timer = 0.0_f32;

    while !rl.window_should_close() {
        // Avoid giant movement jumps if a frame stalls.
        let dt = rl.get_frame_time().clamp(0.0, 0.05);

        // ---------------------------------------------------------
        // 1. Receive network messages.
        // ---------------------------------------------------------
        while let Ok(msg) = rx_in.try_recv() {
            match msg {
                NetMessage::Status(message) => {
                    status = message.text.clone();

                    if role == Role::Server && message.text == "Player connected!" {
                        peer_connected = true;
                    }

                    if message.text == "Disconnected." {
                        peer_connected = false;
                        remote_move_x = 0.0;
                        remote_move_y = 0.0;
                    }
                }

                NetMessage::PlayerInput(input) => {
                    if role == Role::Server {
                        remote_move_x = input.move_x.clamp(-1.0, 1.0);
                        remote_move_y = input.move_y.clamp(-1.0, 1.0);
                    }
                }

                NetMessage::PlayerState(state) => {
                    if role == Role::Client {
                        match state.player_id {
                            // Server-controlled player.
                            1 => {
                                target_peer_x = state.x;
                                target_peer_y = state.y;
                            }

                            // Our own position, validated by the server.
                            2 => {
                                // Correct small prediction errors gradually
                                // instead of snapping to every snapshot.
                                x += (state.x - x) * RECONCILE_STRENGTH;
                                y += (state.y - y) * RECONCILE_STRENGTH;
                            }

                            _ => {}
                        }
                    }
                }

                NetMessage::MapState(map) => {
                    if role == Role::Client {
                        red_button.pressed = map.red_button_down;
                        blue_door_open = map.blue_door_open;

                        red_button.color = if map.red_button_down {
                            Color::WHITE
                        } else {
                            Color::RED
                        };
                    }
                }
            }
        }

        // ---------------------------------------------------------
        // 2. Smooth the remote player's displayed position.
        // ---------------------------------------------------------
        if role == Role::Client {
            let alpha = 1.0 - (-REMOTE_SMOOTHING * dt).exp();

            peer_x += (target_peer_x - peer_x) * alpha;
            peer_y += (target_peer_y - peer_y) * alpha;
        }

        // ---------------------------------------------------------
        // 3. Read keyboard input.
        // ---------------------------------------------------------
        let mut move_x = 0.0_f32;
        let mut move_y = 0.0_f32;

        if rl.is_key_down(KeyboardKey::KEY_A) {
            move_x -= 1.0;
        }

        if rl.is_key_down(KeyboardKey::KEY_D) {
            move_x += 1.0;
        }

        if rl.is_key_down(KeyboardKey::KEY_W) {
            move_y -= 1.0;
        }

        if rl.is_key_down(KeyboardKey::KEY_S) {
            move_y += 1.0;
        }

        // Normalize diagonal movement.
        let magnitude = (move_x * move_x + move_y * move_y).sqrt();

        if magnitude > 0.0 {
            move_x /= magnitude;
            move_y /= magnitude;
        }

        // ---------------------------------------------------------
        // 4. Apply movement.
        // ---------------------------------------------------------
        match role {
            Role::Server => {
                // Player 1 moves immediately on the server.
                x += move_x * PLAYER_SPEED * dt;
                y += move_y * PLAYER_SPEED * dt;

                // Player 2 moves according to the client's latest input.
                if peer_connected {
                    peer_x += remote_move_x * PLAYER_SPEED * dt;
                    peer_y += remote_move_y * PLAYER_SPEED * dt;
                }

                clamp_player(&mut x, &mut y);
                clamp_player(&mut peer_x, &mut peer_y);

                // The server is the authority on puzzle state.
                let player1_pressed =
                    overlaps_button(x, y, red_button.x as f32, red_button.y as f32);

                let player2_pressed = peer_connected
                    && overlaps_button(peer_x, peer_y, red_button.x as f32, red_button.y as f32);

                red_button.pressed = player1_pressed || player2_pressed;

                red_button.color = if red_button.pressed {
                    Color::WHITE
                } else {
                    Color::RED
                };

                blue_door_open = red_button.pressed;

                // Send authoritative state at a fixed rate.
                snapshot_timer += dt;

                if peer_connected && snapshot_timer >= SNAPSHOT_INTERVAL {
                    snapshot_timer %= SNAPSHOT_INTERVAL;

                    let _ = tx_out.send(NetMessage::PlayerState(types::PlayerState {
                        player_id: 1,
                        x,
                        y,
                    }));

                    let _ = tx_out.send(NetMessage::PlayerState(types::PlayerState {
                        player_id: 2,
                        x: peer_x,
                        y: peer_y,
                    }));

                    let _ = tx_out.send(NetMessage::MapState(types::MapState {
                        red_button_down: red_button.pressed,
                        blue_door_open,
                    }));
                }
            }

            Role::Client => {
                // Client-side prediction: move our own character
                // immediately, without waiting for a network round trip.
                x += move_x * PLAYER_SPEED * dt;
                y += move_y * PLAYER_SPEED * dt;

                clamp_player(&mut x, &mut y);

                // Send movement input at a controlled rate.
                input_timer += dt;

                if input_timer >= INPUT_INTERVAL {
                    input_timer %= INPUT_INTERVAL;

                    let _ = tx_out.send(NetMessage::PlayerInput(types::PlayerInput {
                        move_x,
                        move_y,
                    }));
                }
            }
        }

        // ---------------------------------------------------------
        // 5. Render.
        // ---------------------------------------------------------
        let mut d = rl.begin_drawing(&raylib_thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text("Rust Co-op", 20, 20, 30, Color::DARKBLUE);
        d.draw_text("Move with WASD", 20, 60, 20, Color::DARKGRAY);

        let role_name = match role {
            Role::Server => "Server",
            Role::Client => "Client",
        };

        d.draw_text(&format!("Role: {role_name}"), 20, 90, 20, Color::DARKGRAY);

        d.draw_text(&format!("Network: {status}"), 20, 120, 18, Color::MAROON);

        d.draw_circle(red_button.x, red_button.y, red_button.rad, red_button.color);

        if blue_door_open {
            d.draw_circle(
                blue_button.x,
                blue_button.y,
                blue_button.rad,
                blue_button.color,
            );
        }

        // Player 1 is blue. Player 2 is red.
        match role {
            Role::Server => {
                d.draw_circle(x as i32, y as i32, PLAYER_RADIUS, Color::BLUE);

                d.draw_circle(peer_x as i32, peer_y as i32, PLAYER_RADIUS, Color::RED);
            }

            Role::Client => {
                d.draw_circle(x as i32, y as i32, PLAYER_RADIUS, Color::RED);

                d.draw_circle(peer_x as i32, peer_y as i32, PLAYER_RADIUS, Color::BLUE);
            }
        }

        d.draw_text(
            if blue_door_open {
                "Door: OPEN"
            } else {
                "Door: CLOSED"
            },
            20,
            155,
            20,
            Color::DARKGREEN,
        );
    }
}
