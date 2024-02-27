use std::sync::{Arc, Mutex};
use std::thread;

use bevy::app::FixedMain;
use bevy::prelude::*;
use serialport::SerialPort;

use crate::serial::{Command, connect, simple_cmd};

pub const WIDTH: usize = 9;
pub const HEIGHT: usize = 34;

struct LightMatrix {
    port: Arc<Mutex<Box<dyn SerialPort>>>,
}

impl LightMatrix {
    pub fn new(list: bool, verbose: bool, serial_dev: Option<String>, wait_for_device: bool) -> Self {
        Self {
            port: Arc::new(Mutex::new(connect(list, verbose, serial_dev, wait_for_device).unwrap())),
        }
    }

    pub fn draw(&mut self, frame_buffer: FrameBuffer) {
        // Draws the current frame in a separate thread to avoid blocking the main thread.
        let port = self.port.clone();
        let _ = thread::spawn(move || -> () {
            if let Ok(port) = &mut port.try_lock() {
                for x in 0..WIDTH {
                    let mut col: [u8; HEIGHT + 1] = [0x00; HEIGHT + 1];
                    // Set column number.
                    col[0] = u8::try_from(x).unwrap();

                    // Set column pixels.
                    col[1..].copy_from_slice(&frame_buffer.0[x * HEIGHT..(x + 1) * HEIGHT]);

                    // Send column.
                    simple_cmd(port, Command::SendCol, &col, true);
                }

                // Commit frame.
                simple_cmd(port, Command::CommitCols, &[], true);
            } else {
                // println!("skip frame");
            }
        });
    }
}

#[derive(Resource)]
pub struct FrameBuffer(pub [u8; WIDTH * HEIGHT]);

impl Default for FrameBuffer {
    fn default() -> Self {
        Self([0x00; WIDTH * HEIGHT])
    }
}

impl FrameBuffer {
    pub fn set_pixel(&mut self, x: usize, y: usize, value: u8) {
        self.0[y * WIDTH + x] = value;
    }
}


fn draw_frame(mut light_matrix: NonSendMut<LightMatrix>, frame_buffer: Res<FrameBuffer>) {
    let buffer_copy = FrameBuffer(frame_buffer.0.clone());
    light_matrix.draw(buffer_copy);
}

pub struct MatrixPlugin {
    pub list: bool,
    pub verbose: bool,
    pub serial_dev: Option<String>,
    pub wait_for_device: bool,
}

impl Plugin for MatrixPlugin {
    fn build(&self, app: &mut App) {
        app.insert_non_send_resource(LightMatrix::new(self.list, self.verbose, self.serial_dev.clone(), self.wait_for_device))
            .insert_resource(FrameBuffer::default())
            .add_systems(FixedMain, draw_frame);
    }
}