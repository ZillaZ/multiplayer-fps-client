use std::collections::HashMap;

use raylib::prelude::*;
use raylib::{camera::Camera3D, drawing::RaylibMode3DExt};
use tokio::net::TcpStream;

use crate::network::ResponseSignal;
use crate::player::Player;
use crate::{lights, network, objects::*};

pub struct GameManager {
    pub players: Vec<ResponseSignal>,
    pub objects: HashMap<u64, Object>,
    sky_shader: Shader,
    pub player: Player,
}

impl GameManager {
    pub async fn update(
        &mut self,
        handle: &mut raylib::RaylibHandle,
        thread: &raylib::RaylibThread,
        stream: &mut TcpStream,
        models_map: &HashMap<ObjectType, String>,
    ) {
        let new_state = network::connect(self, stream, handle).await;
        println!("{:?}", new_state);
        self.player.set_state(new_state.clone());
        for x in new_state.objects.iter() {
            if self.objects.contains_key(&x.id) {
                self.objects.get_mut(&x.id).unwrap().update(x);
            } else {
                let obj = x.to_object(handle, thread, &models_map);
                self.objects.insert(x.id, obj);
            }
        }
        self.players = new_state.players;
        self.draw(handle, thread);
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
                Object::new(handle, thread, model, Vector3::zero()),
                Vector3::zero(),
            ),
        }
    }

    fn draw(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread) {
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
