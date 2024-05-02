use deku::{DekuContainerRead, DekuContainerWrite};
use raylib::ffi::rAudioBuffer;
use raylib::prelude::*;
use raylib::{camera::Camera3D, drawing::RaylibMode3DExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt, Join};
use std::collections::HashMap;
use tokio::net::TcpStream;

use crate::gui::Draw;
use crate::network::{get_stream, Reason, ResponseSignal, ServerResponse};
use crate::player::Player;
use crate::session::{JoinSessionRequest, NewSessionRequest, ServerRequest};
use crate::{lights, network, objects::*};

#[derive(PartialEq, Eq)]
enum GameState {
    MainMenu,
    CreateMenu,
    JoinMenu,
    InGame,
    ErrorMessage
}

pub struct GameManager {
    pub players: Vec<ResponseSignal>,
    pub objects: HashMap<String, Object>,
    sky_shader: Shader,
    pub player: Player,
    state: GameState,
    once_game: bool,
    draw: Draw,
    server_error: Option<Reason>
}

impl GameManager {
    pub async fn update(
        &mut self,
        handle: &mut raylib::RaylibHandle,
        thread: &raylib::RaylibThread,
        stream: &mut TcpStream,
    ) {
        use GameState::*;
        match self.state {
            MainMenu => {
                self.draw_main_menu(handle, thread);
            },
            CreateMenu => {
                self.draw_new_game_menu(handle, thread, stream).await;
            },
            JoinMenu => {
                self.draw_join_game_menu(handle, thread, stream).await;
            }
            InGame => {
                if !self.once_game {
                    handle.disable_cursor();
                    self.once_game = true
                }
                self.do_game_logic(handle, thread, stream).await;
                self.draw_game(handle, thread);
            },
            ErrorMessage => {
                self.draw_server_error(handle, thread);
            }
        }
    }
    fn draw_main_menu(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread) {
        let mut handle = clear_screen(handle, thread);
        let handle = &mut handle;
        
        self.draw.draw_label("Jogo Fodakkj", handle, [0.0, 25.0]);
        if self.draw.draw_button("Create Session", handle, [0.0, 0.0]) {
            self.state = GameState::CreateMenu;
        }
        if self.draw.draw_button("Join Session", handle, [0.0, -15.0]) {
            self.state = GameState::JoinMenu;
        }
    }
    async fn draw_new_game_menu(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread, stream: &mut TcpStream) {
        let mut handle = clear_screen(handle, thread);
        let handle = &mut handle;
        
        self.draw.draw_label("Create Game", handle, [0.0, 25.0]);
        self.draw.draw_label("Session ID:", handle, [0.0, 5.0]);
        self.draw.draw_textbox("id", handle, [0.0, 0.0]);
        self.draw.draw_label("Session Password:", handle, [0.0, -15.0]);
        self.draw.draw_textbox("passwd", handle, [0.0, -20.0]);
        if self.draw.draw_button("Back to Main Menu", handle, [-15.0, -30.0]) {
            self.state = GameState::MainMenu;
        }
        if self.draw.draw_button("Create Session", handle, [15.0, -30.0]) {
            match self.create_game(stream).await {
                ServerResponse::Ok(signal) => {
                    self.state = GameState::InGame;
                    self.server_error = None;
                },
                ServerResponse::InvalidRequest(reason) => {
                    self.state = GameState::ErrorMessage;
                    self.server_error = Some(reason);
                    *stream = get_stream().await;
                }
            }
        }
    }
    async fn create_game(&mut self, stream: &mut TcpStream) -> ServerResponse {
        let id = String::from_utf8(self.draw.buffers.get("id").unwrap().1.to_vec()).unwrap();
        let passwd = String::from_utf8(self.draw.buffers.get("passwd").unwrap().1.to_vec()).unwrap();
        let request = ServerRequest::NewSession(NewSessionRequest::new(&id, &passwd));
        stream.write(&request.to_bytes().unwrap()).await.unwrap();
        stream.flush().await.unwrap();
        let mut buffer = [0;1024];
        stream.read(&mut buffer).await.unwrap();
        ServerResponse::from_bytes((&buffer, 0)).unwrap().1
    }

    async fn draw_join_game_menu(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread, stream: &mut TcpStream) {
        let mut handle = clear_screen(handle, thread);
        let handle = &mut handle;

        self.draw.draw_label("Join Game", handle, [0.0, 25.0]);
        self.draw.draw_label("Session ID:", handle, [0.0, 5.0]);
        self.draw.draw_textbox("id", handle, [0.0, 0.0]);
        self.draw.draw_label("Session Password:", handle, [0.0, -15.0]);
        self.draw.draw_textbox("passwd", handle, [0.0, -20.0]);
        if self.draw.draw_button("Back to Main Menu", handle, [-15.0, -30.0]) {
            self.state = GameState::MainMenu;
        }
        if self.draw.draw_button("Join Session", handle, [15.0, -30.0]) {
            match self.join_game(stream).await {
                ServerResponse::Ok(signal) => {
                    self.state = GameState::InGame;
                    self.server_error = None;
                },
                ServerResponse::InvalidRequest(reason) => {
                    self.state = GameState::ErrorMessage;
                    self.server_error = reason.into();
                    *stream = get_stream().await;
                }
            }
        }
    }
    
    fn draw_server_error(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread) {
        let mut handle = clear_screen(handle, thread);
        let handle = &mut handle;
        self.draw.draw_label(&self.server_error.as_ref().unwrap().to_string(), handle, [0.0, 30.0]);
        if self.draw.draw_button("Back", handle, [0.0, 0.0]) {
            self.state = GameState::MainMenu;
        }
    }

    async fn join_game(&mut self, stream: &mut TcpStream) -> ServerResponse {
        let id = String::from_utf8(self.draw.buffers.get("id").unwrap().1.to_vec()).unwrap();
        let passwd = String::from_utf8(self.draw.buffers.get("passwd").unwrap().1.to_vec()).unwrap();
        let request = ServerRequest::JoinSession(JoinSessionRequest::new(&id, &passwd));
        stream.write(&request.to_bytes().unwrap()).await.unwrap();
        let mut buffer = [0;1024];
        stream.read(&mut buffer).await.unwrap();
        ServerResponse::from_bytes((&buffer, 0)).unwrap().1
    }

    async fn do_game_logic(
        &mut self,
        handle: &mut RaylibHandle,
        thread: &RaylibThread,
        stream: &mut TcpStream,
    ) {
        let new_state = network::connect(self, stream, handle).await;
        self.player.set_state(new_state.clone());
        for x in new_state.objects.iter() {
            if let Some(object) = self
                .objects
                .get_mut(&String::from_utf8(x.id.clone()).unwrap())
            {
                object.update(x);
            } else {
                let object = Object::new(
                    handle,
                    thread,
                    String::from_utf8(x.id.clone()).unwrap(),
                    x.position,
                    x.rotation,
                );
                self.objects
                    .insert(String::from_utf8(x.id.clone()).unwrap(), object);
            }
        }
        self.players = new_state.players;
        /*for player in self.players.iter() {
            let mut draw_handle = handle.begin_drawing(thread);
            let mut draw_handle = draw_handle.begin_mode3D(self.player.camera);
            println!("drawing players");
            draw_handle.draw_model(
                &self.player.object.model,
                Vector3::new(
                    player.translation[0],
                    player.translation[1],
                    player.translation[2],
                ),
                1.0,
                Color::WHITE,
            );
        }*/
    }
    pub fn new(
        sky_shader: Shader,
        camera: Camera3D,
        handle: &mut RaylibHandle,
        thread: &RaylibThread,
        model: Model,
    ) -> Self {
        Self {
            sky_shader,
            players: Vec::new(),
            objects: HashMap::new(),
            player: Player::new(
                camera,
                1.0,
                Object::new(handle, thread, "DCPlayer".into(), [0.0; 3], [0.0; 4]),
                Vector3::zero(),
            ),
            state: GameState::MainMenu,
            once_game: false,
            draw: Draw::new(handle),
            server_error: None
        }
    }

    fn draw_game(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread) {
        let mut draw_handle = handle.begin_drawing(thread);
        draw_handle.clear_background(Color::WHITE);

        self.draw_sky(&mut draw_handle);
        self.draw_objects(&mut draw_handle);
        self.draw_lights(&mut draw_handle);
    }

    fn draw_sky(&mut self, handle: &mut RaylibDrawHandle) {
        let mut shader_mode = handle.begin_shader_mode(&self.sky_shader);
        shader_mode.draw_rectangle(
            0,
            0,
            shader_mode.get_screen_width(),
            shader_mode.get_screen_height(),
            Color::WHITE,
        );
    }

    fn draw_objects(&mut self, handle: &mut RaylibDrawHandle) {
        let mut draw = handle.begin_mode3D(&self.player.camera);

        for object in self.objects.values_mut() {
            object.model.set_transform(&object.rotation.to_matrix());
            draw.draw_model(&object.model, object.position, 1.0, Color::WHITE);
        }

        for player in self.players.iter() {
            draw.draw_model(
                &self.player.object.model,
                Vector3::new(
                    player.translation[0],
                    player.translation[1],
                    player.translation[2],
                ),
                1.0,
                Color::WHITE,
            );
        }
    }

    pub fn ambient_light(
        &mut self,
        handle: &mut RaylibHandle,
        thread: &RaylibThread,
        light_shader: &mut Shader,
    ) {
        let ambient_loc = light_shader.get_shader_location("ambient");
        println!("AMBIENT: {}", ambient_loc);
        *light_shader.locs_mut().get_mut(11).unwrap() = light_shader.get_shader_location("viewPos");
        light_shader.set_shader_value(*light_shader.locs().get(11).unwrap(), [0.0, 0.0, 0.0]);
        light_shader.set_shader_value(ambient_loc, [0.1, 0.1, 0.1, 1.0]);
    }

    pub fn create_light(
        &mut self,
        handle: &mut RaylibHandle,
        thread: &RaylibThread,
        light_shader: &mut Shader,
    ) {
        let light = lights::Light::new(
            1,
            Vector3::up() * 10.0,
            Vector3::zero(),
            Color::RED,
            light_shader,
        );
    }

    fn draw_lights(&mut self, handle: &mut RaylibDrawHandle) {}
}

fn clear_screen<'a>(handle: &'a mut RaylibHandle, thread: &'a RaylibThread) -> RaylibDrawHandle<'a> {
    let mut handle = handle.begin_drawing(thread);
    handle.clear_background(Color::WHITE);
    handle
}
