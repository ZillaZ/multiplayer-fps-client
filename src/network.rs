use crate::*;
use deku::prelude::*;
use raylib::{math::*, RaylibHandle};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use self::objects::NetworkObject;

#[derive(DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum Reason {
    #[deku(id = "0x1")]
    IdInUse,
    #[deku(id = "0x2")]
    InvalidRequestFormat,
    #[deku(id = "0x3")]
    InvalidIdFormat,
    #[deku(id = "0x4")]
    InvalidPassword,
    #[deku(id = "0x5")]
    IdDoesntExist,
    #[deku(id = "0x6")]
    WrongPassword
}

impl ToString for Reason {
    fn to_string(&self) -> String {
        use Reason::*;
        match self {
            IdInUse => "The given ID is already in use.".into(),
            InvalidRequestFormat => "The request is invalid.".into(),
            InvalidIdFormat => "The given ID is invalid".into(),
            InvalidPassword => "The given password is invalid".into(),
            IdDoesntExist => "There is no session with the given ID".into(),
            WrongPassword => "The given password is incorrect".into()
        }
    }
}

#[derive(DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum ServerResponse {
    #[deku(id = "0x1")]
    Ok(ResponseSignal),
    #[deku(id = "0x2")]
    InvalidRequest(Reason),
}
#[derive(Clone, DekuRead, DekuWrite)]
pub struct PlayerSignal {
    desired_mov: [f32; 3],
    desired_rot: [f32; 2],
    camera_radius: f32,
}

impl PlayerSignal {
    pub fn new(desired_mov: Vector3, desired_rot: Vector2, camera_radius: f32) -> Self {
        Self {
            desired_mov: desired_mov.to_array(),
            desired_rot: [desired_rot.x, desired_rot.y],
            camera_radius,
        }
    }
}


#[derive(Clone, Debug, DekuRead, DekuWrite)]
pub struct ResponseSignal {
    #[deku(update = "self.players.len()")]
    pub player_count: usize,
    #[deku(update = "self.objects.len()")]
    pub object_count: usize,
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
    pub fn new(
        translation: Vector3,
        camera_pos: Vector3,
        camera_target: Vector3,
        fwd: Vector3,
        right: Vector3,
    ) -> Self {
        Self {
            player_count: 0,
            object_count: 0,
            translation: translation.to_array(),
            camera_pos: camera_pos.to_array(),
            camera_target: camera_target.to_array(),
            fwd: fwd.to_array(),
            right: right.to_array(),
            players: Vec::new(),
            objects: Vec::new(),
        }
    }
}

impl Default for ResponseSignal {
    fn default() -> Self {
        Self::new(
            Vector3::zero(),
            Vector3::zero(),
            Vector3::zero(),
            Vector3::forward(),
            Vector3::forward(),
        )
    }
}

pub async fn connect(
    manager: &mut GameManager,
    stream: &mut TcpStream,
    handle: &mut RaylibHandle,
) -> ResponseSignal {
    let mut buf = [0; 1024];
    let update_data = manager.player.update(handle);
    stream
        .write(&update_data.to_bytes().unwrap())
        .await
        .unwrap();
    let _response = stream.read(&mut buf).await.unwrap();
    let parsed = ResponseSignal::from_bytes((&buf, 0)).unwrap();
    println!("{:?}", parsed.1);
    parsed.1
}

pub async fn get_stream() -> TcpStream {
    TcpStream::connect("127.0.0.1:9001").await.unwrap()
}
