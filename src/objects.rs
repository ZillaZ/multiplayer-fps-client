use deku::prelude::*;
use raylib::math::{Vector3, Vector4};
use raylib::models::Model;

#[derive(Debug, DekuRead, DekuWrite, Clone, Hash, Eq, PartialEq)]
#[deku(type = "u8")]
pub enum ObjectType {
    #[deku(id = "0x1")]
    GROUND,
    #[deku(id = "0x2")]
    PLAYER,
    #[deku(id = "0x3")]
    BALL,
    #[deku(id = "0x4")]
    RING
}

#[derive(Debug, DekuRead, DekuWrite, Clone)]
pub struct NetworkObject {
    pub model_type: ObjectType,
    position: [f32; 3],
    rotation: [f32; 4],
    pub id: u64,
}

impl NetworkObject {
    pub fn to_object(
        &self,
        handle: &mut raylib::prelude::RaylibHandle,
        thread: &raylib::prelude::RaylibThread,
        models_map: &std::collections::HashMap<ObjectType, String>,
    ) -> Object {
        let model = handle
            .load_model(thread, models_map.get(&self.model_type).unwrap())
            .unwrap();
        Object::new(
            handle,
            thread,
            model,
            Vector3::new(self.position[0], self.position[1], self.position[2]),
            Vector4::new(
                self.rotation[0],
                self.rotation[1],
                self.rotation[2],
                self.rotation[3],
            ),
        )
    }
}

#[derive(Debug)]
pub struct Object {
    pub model: Model,
    pub position: Vector3,
    pub rotation: Vector4,
}

impl Object {
    pub fn new(
        handle: &mut raylib::prelude::RaylibHandle,
        thread: &raylib::prelude::RaylibThread,
        model: Model,
        position: Vector3,
        rotation: Vector4,
    ) -> Self {
        Self {
            model,
            position,
            rotation,
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
