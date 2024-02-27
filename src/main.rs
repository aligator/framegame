use bevy::prelude::*;
use clap::Parser;
use rand::random;

use crate::matrix::{FrameBuffer, HEIGHT, MatrixPlugin, WIDTH};

mod matrix;
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

#[derive(Resource)]
struct SpawnTimer(Timer);

fn set_random_pixel(time: Res<Time>, mut timer: ResMut<SpawnTimer>, mut frame_buffer: ResMut<FrameBuffer>) {
    // update our timer with the time elapsed since the last update
    // if that caused the timer to finish, we say hello to everyone
    if timer.0.tick(time.delta()).just_finished() {
        let x = random::<usize>() % WIDTH;
        let y = random::<usize>() % HEIGHT;
        frame_buffer.set_pixel(x, y, 0xff);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let args = ClapCli::parse_from(args);

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(MatrixPlugin{ list: args.list, verbose: args.verbose, serial_dev: args.serial_dev, wait_for_device: args.wait_for_device })
        .insert_resource(SpawnTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
        .add_systems(Update, set_random_pixel)
        .run();
}
