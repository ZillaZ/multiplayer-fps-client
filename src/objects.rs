use deku::prelude::*;
use raylib::math::{Vector3, Vector4};
use raylib::models::Model;

#[derive(Debug, DekuRead, DekuWrite, Clone)]
pub struct NetworkObject {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    #[deku(update = "self.id.len()")]
    id_len: usize,
    #[deku(count = "id_len")]
    pub id: Vec<u8>,
}

#[derive(Debug)]
pub struct Object {
    pub model: Model,
    pub id: String,
    pub position: Vector3,
    pub rotation: Vector4,
}

impl Object {
    pub fn new(
        handle: &mut raylib::prelude::RaylibHandle,
        thread: &raylib::prelude::RaylibThread,
        id: String,
        position: [f32; 3],
        rotation: [f32; 4],
    ) -> Self {
        Self {
            id: id.clone(),
            model: handle
                .load_model(
                    thread,
                    &format!(
                        "static/models/{}.obj",
                        id.clone()
                            .trim_matches(|x: char| x.to_string().parse::<i32>().is_ok())
                    ),
                )
                .unwrap(),
            position: Vector3::new(position[0], position[1], position[2]),
            rotation: Vector4::new(rotation[0], rotation[1], rotation[2], rotation[3]),
        }
    }

    pub fn update(&mut self, new_state: &NetworkObject) {
        self.position = Vector3::new(
            new_state.position[0],
            new_state.position[1],
            new_state.position[2],
        );
        self.rotation = Vector4::new(
            new_state.rotation[0],
            new_state.rotation[1],
            new_state.rotation[2],
            new_state.rotation[3],
        );
    }
}
