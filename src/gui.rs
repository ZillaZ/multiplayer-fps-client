use std::ffi::CString;
use std::collections::HashMap;

use raylib::prelude::*;

pub struct Draw {
    screen_x: f32,
    screen_y: f32,
    button_width: f32,
    button_height: f32,
    label_width: f32,
    label_height: f32,
    textbox_height: f32,
    textbox_width: f32,
    pub buffers: HashMap<String, (bool, [u8; 1024])>
}

impl Draw {
    pub fn new(handle: &mut RaylibHandle) -> Self {
        let (x, y) = (handle.get_screen_width() as f32, handle.get_screen_height() as f32);
        Self {
            screen_x: x,
            screen_y: y,
            button_height: y * (5.0 / 100.0),
            button_width: x * (10.0 / 100.0),
            label_height: y * (5.0 / 100.0),
            label_width: x * (30.0 / 100.0),
            textbox_height: y * (5.0 / 100.0),
            textbox_width: x * (50.0 / 100.0),
            buffers: HashMap::new()
        }
    }
    pub fn draw_label(&self, text: &str, handle: &mut RaylibDrawHandle, position: [f32;2]) -> bool{
        handle.gui_label(
            build_rectangle(self.screen_x, self.screen_y, position, self.label_width, self.label_height),
            Some(CString::new(text).unwrap().as_c_str())
        )
    }
    pub fn draw_button(&mut self, text: &str, handle: &mut RaylibDrawHandle, position: [f32;2]) -> bool {
        let rtn = handle.gui_button(
            build_rectangle(self.screen_x, self.screen_y, position, self.button_width, self.button_height),
            Some(CString::new(text).unwrap().as_c_str())
        );
        if rtn {
            for (_key, (state, _buffer)) in self.buffers.iter_mut() {
                *state = false;
            }
        }
        rtn
    }
    pub fn draw_textbox(&mut self, id: &str, handle: &mut RaylibDrawHandle, position: [f32;2]) -> bool {
        let (height, width) = (self.textbox_height, self.textbox_width);
        let (x, y) = (self.screen_x, self.screen_y);
        let (_state, buffer) = match self.buffers.get(id) {
            Some(buffer) => buffer,
            None => {
                self.buffers.insert(id.into(), (false, [0; 1024]));
                self.buffers.get(id).unwrap()
            }
        };
        let res = self.buffers.get(id).unwrap().0;
        if res {
            for (key, (state, _buffer)) in self.buffers.iter_mut() {
                if key == id {continue;}
                *state = false;
            }
        }else{
            self.buffers.insert(id.into(), (handle.gui_text_box(
                build_rectangle(x, y, position, width, height),
                &mut [0],
                false
            ), *buffer));
        }
        handle.clear_background(Color::WHITE);
        let (state, _buffer) = match self.buffers.get(id) {
            Some(buffer) => buffer,
            None => {
                self.buffers.insert(id.into(), (false, [0; 1024]));
                self.buffers.get(id).unwrap()
            }
        };
        if *state {
            for (key, (state, _buffer)) in self.buffers.iter_mut() {
                if key == id {continue;}
                *state = false;
            }
        }
        let (state, buffer) = match self.buffers.get_mut(id) {
            Some(buffer) => buffer,
            None => {
                self.buffers.insert(id.into(), (false, [0; 1024]));
                self.buffers.get_mut(id).unwrap()
            }
        };
        let len = buffer.iter().filter(|x| x != &&0).collect::<Vec<&u8>>().len();
        handle.gui_text_box(
            build_rectangle(x, y, position, width, height),
            buffer,
            *state && len <= 8
        )
    }

}

fn calc_position(screen_x: f32, screen_y: f32, position: [f32;2], width: f32, height: f32) -> (f32, f32) {
    ((screen_x / 2.0 - width / 2.0) - screen_x * (position[0] / 100.0) * -1.0,
    (screen_y / 2.0 - height / 2.0) - screen_y * (position[1] / 100.0))
}

fn build_rectangle(screen_x: f32, screen_y: f32, position: [f32;2], width: f32, height: f32) -> Rectangle {
    let (x, y) = calc_position(screen_x, screen_y, position, width, height);
    Rectangle::new(
        x,
        y,
        width,
        height,
    )
}