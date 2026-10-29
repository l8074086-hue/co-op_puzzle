// TYPES.RS
use raylib::prelude::*;
use serde::{Deserialize, Serialize};

pub struct button {
    pub pressed: bool,
    pub x: i32,
    pub y: i32,
    pub rad: f32,
    pub color: Color,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetMessage {
    PlayerInput(PlayerInput),
    PlayerState(PlayerState),
    MapState(MapState),
    Status(StatusMessage),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerInput {
    pub move_x: f32,
    pub move_y: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerState {
    pub player_id: u32,
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MapState {
    pub red_button_down: bool,
    pub blue_door_open: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatusMessage {
    pub text: String,
}
