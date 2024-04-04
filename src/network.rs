use crate::*;
use deku::prelude::*;
use raylib::{math::*, RaylibHandle};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use self::objects::NetworkObject;

#[derive(Clone, DekuRead, DekuWrite)]
pub struct PlayerSignal {
    desired_mov: [f32; 3],
    desired_rot: [f32; 2],
    pub dt: f32,
}

impl PlayerSignal {
    pub fn new(desired_mov: Vector3, desired_rot: Vector2, dt: f32) -> Self {
        Self {
            desired_mov: desired_mov.to_array(),
            desired_rot: [desired_rot.x, desired_rot.y],
            dt,
        }
    }
}

#[derive(Clone, Debug, DekuRead, DekuWrite)]
pub struct ResponseSignal {
    #[deku(update = "self.players.len()")]
    player_count: u8,
    #[deku(update = "self.objects.len()")]
    object_count: u8,
    pub translation: [f32; 3],
    pub camera_pos: [f32; 3],
    pub camera_target: [f32; 3],
    pub fwd: [f32; 3],
    pub right: [f32; 3],
    #[deku(count = "player_count")]
    pub players: Vec<ResponseSignal>,
    #[deku(count = "object_count")]
    pub objects: Vec<NetworkObject>,
}

impl ResponseSignal {
    pub fn new(translation: Vector3, camera_pos: Vector3, camera_target: Vector3) -> Self {
        Self {
            player_count: std::u8::MAX,
            object_count: std::u8::MAX,
            translation: translation.to_array(),
            camera_pos: camera_pos.to_array(),
            camera_target: camera_target.to_array(),
            fwd: Vector3::forward().to_array(),
            right: Vector3::right().to_array(),
            players: Vec::new(),
            objects: Vec::new(),
        }
    }
}

pub async fn connect(
    manager: &mut GameManager,
    stream: &mut TcpStream,
    handle: &mut RaylibHandle,
) -> ResponseSignal {
    let mut buf = [0; 2048];
    let update_data = manager.player.update(handle);
    stream
        .write(&update_data.to_bytes().unwrap())
        .await
        .unwrap();
    let _response = stream.read(&mut buf).await.unwrap();
    let parsed = ResponseSignal::from_bytes((&buf, 0)).unwrap();
    parsed.1
}

pub async fn get_stream() -> TcpStream {
    TcpStream::connect("127.0.0.1:9001").await.unwrap()
}
