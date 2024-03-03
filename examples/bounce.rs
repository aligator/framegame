use bevy::prelude::*;
use clap::Parser;
use rand::random;

use framegame::matrix::{FrameBuffer, FrameLimit, HEIGHT, MatrixPlugin, WIDTH};
use framegame::schedule::Draw;

use crate::common::{ClapCli, Color, ObjectBundle, Size, Velocity};

mod common;

/// Spawn object.
fn setup(mut commands: Commands) {
    let box_object = ObjectBundle {
        position: TransformBundle::from_transform(Transform::from_xyz(5.0, 5.0, 0.0)),
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
    mut query: Query<(&Transform, &mut Velocity, &Size, &mut Color)>,
) {
    for (position, mut velocity, size, mut color) in &mut query {
        let mut bounce = false;
        if position.translation.x == 0.0 && velocity.x < 0 {
            velocity.x *= -1;
            bounce = true;
        }
        if position.translation.x + size.width as f32 == WIDTH as f32 && velocity.x > 0 {
            velocity.x *= -1;
            bounce = true;
        }
        if position.translation.y == 0.0 && velocity.y < 0 {
            velocity.y *= -1;
            bounce = true;
        }
        if position.translation.y + size.height as f32 == HEIGHT as f32 && velocity.y > 0 {
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
            mut query: Query<(&mut Transform, &Velocity, &Size)>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    for (mut position, velocity, size) in &mut query {
        position.translation.x = (position.translation.x + velocity.x as f32).clamp(0.0, WIDTH as f32 - size.width as f32);
        position.translation.y =
            (position.translation.y + velocity.y as f32).clamp(0.0, HEIGHT as f32 - size.height as f32);
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
    query: Query<(&GlobalTransform, &Size, &Color)>,
) {
    for (position, size, color) in &query {
        // frame_buffer.set_pixel(2, 33, 0xff)
        //frame_buffer.set_pixel(position.x, position.y, color.0)
        let y_offset = position.translation().y as u16;
        let height_bytes = size.height as u16;
        let object_col = &[color.0].repeat(size.height as usize);

        for x in position.translation().x as u16..(position.translation().x as u16 + size.width as u16) {
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
        .add_plugins((
            // Use minimal here, as we don't want a window and need no user input. So for this basic example this is enough.
            MinimalPlugins,

            // To set the global transform from the local transform.
            // We could have used a custom Transform component instead or use Transform for drawing and not the global transform.
            TransformPlugin,
        ))
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