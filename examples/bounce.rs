use bevy::prelude::*;
use clap::Parser;
use rand::random;

use framegame::matrix::{FrameBuffer, FrameLimit, HEIGHT, MatrixPlugin, WIDTH};
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

#[derive(Component, Debug)]
struct Position {
    x: u8,
    y: u8,
}

#[derive(Component, Debug)]
struct Velocity {
    x: i16,
    y: i16,
}

#[derive(Component, Debug)]
struct Size {
    width: u8,
    height: u8,
}

#[derive(Component, Debug)]
struct Color(u8);


#[derive(Bundle, Debug)]
struct ObjectBundle {
    position: Position,
    velocity: Velocity,
    size: Size,
    color: Color,
}


/// Spawn object.
fn setup(mut commands: Commands) {
    let box_object = ObjectBundle {
        position: Position { x: 1, y: 1 },
        velocity: Velocity { x: 1, y: 1 },
        size: Size {
            width: 3,
            height: 3,
        },
        color: Color(0xff),
    };
    commands.spawn(box_object);
}

/// Bounce object off edges of buffer.
fn bounce(
    mut query: Query<(&Position, &mut Velocity, &Size, &mut Color)>,
) {
    for (position, mut velocity, size, mut color) in &mut query {
        let mut bounce = false;
        if position.x == 0 && velocity.x < 0 {
            velocity.x *= -1;
            bounce = true;
        }
        if position.x + size.width == WIDTH && velocity.x > 0 {
            velocity.x *= -1;
            bounce = true;
        }
        if position.y == 0 && velocity.y < 0 {
            velocity.y *= -1;
            bounce = true;
        }
        if position.y + size.height == HEIGHT && velocity.y > 0 {
            velocity.y *= -1;
            bounce = true;
        }
        if bounce {
            color.0 = random();
        }
    }
}

/// Move object based on current velocity.
fn movement(time: Res<Time>, mut timer: ResMut<AnimationTimer>,
            mut query: Query<(&mut Position, &Velocity, &Size)>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    for (mut position, velocity, size) in &mut query {
        position.x = ((position.x as i16 + velocity.x) as u8).clamp(0, WIDTH - size.width);
        position.y =
            ((position.y as i16 + velocity.y) as u8).clamp(0, HEIGHT - size.height);
    }
}

/// Draw solid background to buffer.
fn draw_background(mut frame_buffer: ResMut<FrameBuffer>) {
    let length = frame_buffer.0.len();
    frame_buffer.0.copy_from_slice(&[0x11].repeat(length));
}

/// Draw objects to buffer.
fn draw_objects(
    mut frame_buffer: ResMut<FrameBuffer>,
    query: Query<(&Position, &Size, &Color)>,
) {
    for (position, size, color) in &query {
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
    }
}

#[derive(Resource)]
struct AnimationTimer(Timer);

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let args = ClapCli::parse_from(args);

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(MatrixPlugin { list: args.list, verbose: args.verbose, serial_dev: args.serial_dev, wait_for_device: args.wait_for_device })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (bounce, movement).chain(),
        )
        .insert_resource(FrameLimit(Timer::from_seconds(0.1, TimerMode::Repeating)))
        .insert_resource(AnimationTimer(Timer::from_seconds(0.3, TimerMode::Repeating)))
        .add_systems(Draw, (draw_background, draw_objects).chain())
        .run();
}
