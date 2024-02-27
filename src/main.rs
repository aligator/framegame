use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;

use bevy::app::FixedMain;
use bevy::prelude::*;
use bevy::utils::tracing::Instrument;
use clap::Parser;
use rand::random;
use serialport::SerialPort;

use crate::serial::{Command, connect, simple_cmd};

mod serial;

#[derive(Parser, Debug)]
#[command(version, arg_required_else_help = true)]
pub struct ClapCli {
    /// List connected HID devices
    #[arg(short, long)]
    list: bool,

    /// Verbose outputs to the console
    #[arg(short, long)]
    verbose: bool,

    /// Serial device, like /dev/ttyACM0 or COM0
    #[arg(long)]
    pub serial_dev: Option<String>,

    /// Retry connecting to the device until it works
    #[arg(long)]
    wait_for_device: bool,
}

const WIDTH: usize = 9;
const HEIGHT: usize = 34;

struct LightMatrix {
    port: Arc<Mutex<Box<dyn SerialPort>>>,
}

impl LightMatrix {
    pub fn new(args: ClapCli) -> Self {
        Self {
            port: Arc::new(Mutex::new(connect(args.list, args.verbose, args.serial_dev, args.wait_for_device).unwrap())),
        }
    }

    pub fn draw(&mut self, frame_buffer: FrameBuffer) {
        let port = self.port.clone();
        let _ = thread::spawn(move || -> () {
            if let Ok(port) = &mut port.try_lock() {
                for x in 0..WIDTH {
                    let mut col: [u8; HEIGHT + 1] = [0x00; HEIGHT + 1];
                    col[0] = u8::try_from(x).unwrap();
                    col[1..].copy_from_slice(&frame_buffer.0[x * HEIGHT..(x + 1) * HEIGHT]);

                    simple_cmd(port, Command::SendCol, &col, true);
                }

                simple_cmd(port, Command::CommitCols, &[], true);
            } else {
                // println!("skip frame");
            }
        });
    }
}

#[derive(Resource)]
struct FrameBuffer([u8; WIDTH * HEIGHT]);

impl Default for FrameBuffer {
    fn default() -> Self {
        Self([0x00; WIDTH * HEIGHT])
    }
}

#[derive(Resource)]
struct SpawnTimer(Timer);

fn draw_frame(mut light_matrix: NonSendMut<LightMatrix>, frame_buffer: Res<FrameBuffer>) {
    let buffer_copy = FrameBuffer(frame_buffer.0.clone());
    light_matrix.draw(buffer_copy);
}

fn set_random_pixel(time: Res<Time>, mut timer: ResMut<SpawnTimer>, mut frame_buffer: ResMut<FrameBuffer>) {
    // update our timer with the time elapsed since the last update
    // if that caused the timer to finish, we say hello to everyone
    if timer.0.tick(time.delta()).just_finished() {
        let x = random::<usize>() % WIDTH;
        let y = random::<usize>() % HEIGHT;
        let mut new_frame_buffer = frame_buffer.0.clone();
        new_frame_buffer[y * WIDTH + x] = 0xff;
        *frame_buffer = FrameBuffer(new_frame_buffer);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let args = ClapCli::parse_from(args);

    App::new()
        .add_plugins(MinimalPlugins)
        .insert_non_send_resource(LightMatrix::new(args))
        .insert_resource(FrameBuffer::default())
        .insert_resource(SpawnTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
        .add_systems(FixedMain, draw_frame)
        .add_systems(Update, set_random_pixel)
        .run();

    /* let port = connect(args.list, args.verbose, args.serial_dev, args.wait_for_device);
     if let Some(mut port) = port {
         run(&mut port);
     }*/
}
