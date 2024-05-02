use game::GameManager;
use network::get_stream;
use raylib::{camera::Camera3D, math::Vector3, shaders::RaylibShader};
use tokio::io::AsyncWriteExt;

pub mod gui;
pub mod session;
pub mod game;
pub mod lights;
pub mod network;
pub mod objects;
pub mod player;
pub mod reader;

#[tokio::main]
async fn main() {
    //delete_models();
    //build_models("static/models/scene.obj", "static/models/scene.mtl");
    let (mut handle, thread) = raylib::init()
        .height(768)
        .width(1366)
        .title("Física MUITO FODA")
        .vsync()
        .msaa_4x()
        .resizable()
        .build();
    handle.set_target_fps(60);
    handle.gui_enable();

    let sky_shader = handle
        .load_shader(&thread, None, Some("static/shaders/shader.fs"))
        .unwrap();
    let mut light_shader = handle
        .load_shader(
            &thread,
            Some("static/shaders/lighting.vs"),
            Some("static/shaders/lightning.fs"),
        )
        .unwrap();

    let player_model = handle
        .load_model(&thread, "static/models/ball.obj")
        .unwrap();

    let camera = Camera3D::perspective(Vector3::zero(), Vector3::zero(), Vector3::up(), 90.0);
    let mut manager = GameManager::new(sky_shader, camera, &mut handle, &thread, player_model);

    let mut stream = get_stream().await;

    while !handle.window_should_close() {
        manager.update(&mut handle, &thread, &mut stream).await;
    }
    stream.flush().await.unwrap();
}
