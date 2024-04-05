use crate::*;
use objects::*;
use raylib::consts::KeyboardKey::*;
use raylib::prelude::*;

use self::network::{PlayerSignal, ResponseSignal};

pub struct Player {
    pub camera: Camera3D,
    pub object: Object,
    pub position: Vector3,
    pub fwd: Vector3,
    rotation: Vector3,
    speed: f32,
    pitch: f32,
    yaw: f32,
    right: Vector3,
}

impl Player {
    pub fn new(camera: Camera3D, speed: f32, object: Object, position: Vector3) -> Self {
        Self {
            pitch: 0.0,
            yaw: 0.0,
            camera,
            speed,
            position,
            fwd: Vector3::forward(),
            right: Vector3::right(),
            object,
            rotation: Vector3::zero(),
        }
    }

    fn get_input(&mut self, rl: &mut RaylibHandle) -> Vector3 {
        let mut movement_vector = Vector3::zero();
        let fwd = (self.fwd - Vector3::new(0.0, self.fwd.y, 0.0)).normalized();
        if rl.is_key_down(KEY_W) {
            movement_vector += fwd;
        }
        if rl.is_key_down(KEY_A) {
            movement_vector -= self.right;
        }
        if rl.is_key_down(KEY_S) {
            movement_vector -= fwd;
        }
        if rl.is_key_down(KEY_D) {
            movement_vector += self.right;
        }
        if rl.is_key_down(KEY_SPACE) {
            movement_vector += Vector3::up();
        }
        if rl.is_key_down(KEY_LEFT_SHIFT) {
            movement_vector -= Vector3::up();
        }
        movement_vector.normalize();
        movement_vector * self.speed
    }

    pub fn update(&mut self, handle: &mut RaylibHandle) -> PlayerSignal {
        let desired_mov = self.get_input(handle);
        let desired_rot = self.update_camera(handle);
        PlayerSignal::new(desired_mov, desired_rot, handle.get_frame_time())
    }

    pub fn update_camera(&mut self, rl: &mut RaylibHandle) -> Vector2 {
        rl.get_mouse_delta()
    }

    pub fn set_state(&mut self, new_state: ResponseSignal) {
        self.position = Vector3::new(
            new_state.translation[0],
            new_state.translation[1],
            new_state.translation[2],
        );
        self.camera.position = Vector3::new(
            new_state.camera_pos[0],
            new_state.camera_pos[1],
            new_state.camera_pos[2],
        );
        self.camera.target = Vector3::new(
            new_state.camera_target[0],
            new_state.camera_target[1],
            new_state.camera_target[2],
        );
        self.fwd = Vector3::new(new_state.fwd[0], new_state.fwd[1], new_state.fwd[2]);
        self.right = Vector3::new(new_state.right[0], new_state.right[1], new_state.right[2])
    }
}
