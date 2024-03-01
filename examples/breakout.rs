use bevy::prelude::*;
use clap::Parser;
use framegame::matrix::{FrameBuffer, FrameLimit, MatrixPlugin};
use framegame::schedule::Draw;

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

/// Draw solid background to buffer.
fn draw_background(mut frame_buffer: ResMut<FrameBuffer>) {
    let length = frame_buffer.0.len();
    frame_buffer.0.copy_from_slice(&[0x00].repeat(length));
}

/// Draw objects to buffer.
fn draw_objects(
    mut frame_buffer: ResMut<FrameBuffer>,
) {
    /* for (position, size, color) in &query {
         // frame_buffer.set_pixel(2, 33, 0xff)
         //frame_buffer.set_pixel(position.x, position.y, color.0)
         let y_offset = position.y as u16;
         let height_bytes = size.height as u16;
         let object_col = &[color.0].repeat(size.height as usize);

         for x in position.x as u16..(position.x as u16 + size.width as u16) {
             let x_offset = x * HEIGHT as u16;
             let i = x_offset + y_offset;
             let j = i + height_bytes;

             frame_buffer.0[i as usize..j as usize].copy_from_slice(object_col);
         }
     }*/
}

#[derive(Resource)]
struct AnimationTimer(Timer);

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let args = ClapCli::parse_from(args);

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(MatrixPlugin { list: args.list, verbose: args.verbose, serial_dev: args.serial_dev, wait_for_device: args.wait_for_device })
        // .add_systems(Startup, setup)
        .insert_resource(FrameLimit(Timer::from_seconds(0.1, TimerMode::Repeating)))
        .insert_resource(AnimationTimer(Timer::from_seconds(0.3, TimerMode::Repeating)))
        .add_systems(Draw, (draw_background, draw_objects).chain())
        .run();
}
