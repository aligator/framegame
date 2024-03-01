use std::sync::{Arc, Mutex};
use std::thread;

use bevy::app::MainScheduleOrder;
use bevy::ecs::schedule::ExecutorKind;
use bevy::prelude::*;
use serialport::SerialPort;
use crate::schedule::{Draw, Render};
use crate::serial::{Command, connect, simple_cmd};

pub const WIDTH: u8 = 9;
pub const HEIGHT: u8 = 34;

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
                    let mut col: [u8; HEIGHT as usize + 1] = [0x00; HEIGHT as usize + 1];
                    // Set column number.
                    col[0] = u8::try_from(x).unwrap();

                    // Set column pixels.
                    let col_range = (x as usize * HEIGHT as usize)..((x + 1) as usize * HEIGHT as usize);
                    col[1..].copy_from_slice(&frame_buffer.0[col_range]);
                    
                    // Reverse y-axis.
                    // This matches the bevy coordinate system. (left bottom is 0, 0)
                    col[1..].reverse();
                    
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
pub struct FrameLimit(pub Timer);

#[derive(Resource)]
pub struct FrameBuffer(pub [u8; WIDTH as usize * HEIGHT as usize]);

impl Default for FrameBuffer {
    fn default() -> Self {
        Self([0x00; WIDTH as usize * HEIGHT as usize])
    }
}

impl FrameBuffer {
    pub fn set_pixel(&mut self, x: u8, y: u8, value: u8) {
        self.0[x as usize * HEIGHT as usize + y as usize] = value;
    }
}


fn render(time: Res<Time>, timer: Option<ResMut<FrameLimit>>, mut light_matrix: NonSendMut<LightMatrix>, frame_buffer: Res<FrameBuffer>) {
    if let Some(mut timer) = timer {
        if !timer.0.tick(time.delta()).just_finished() {
            return;
        }
    }

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
        let mut draw_schedule = Schedule::new(Draw);
        draw_schedule.set_executor_kind(ExecutorKind::SingleThreaded);

        let mut render_schedule = Schedule::new(Render);
        render_schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        render_schedule.add_systems(render);

        app.insert_non_send_resource(LightMatrix::new(self.list, self.verbose, self.serial_dev.clone(), self.wait_for_device))
            .insert_resource(FrameBuffer::default())
            .add_schedule(draw_schedule)
            .add_schedule(render_schedule);

        // Ensure `Draw` and `Render` schedules execute at the correct moment.
        let mut order = app.world.resource_mut::<MainScheduleOrder>();
        order.insert_after(PostUpdate, Draw);
        order.insert_after(Draw, Render);
    }
}