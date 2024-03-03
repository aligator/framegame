use bevy::prelude::*;
use clap::Parser;

use common::*;
use framegame::matrix::{FrameBuffer, FrameLimit, HEIGHT, MatrixPlugin, WIDTH};
use framegame::schedule::Draw;

mod common;

const PADDLE_COLOR: common::Color = common::Color(0xff);
const PADDLE_SIZE: u8 = 2;
const PADDLE_SPEED: f32 = 8.0;
const BOTTOM_WALL: i32 = 0;
const GAP_BETWEEN_PADDLE_AND_FLOOR: i32 = 1;

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Collider;

// Add the game's entities to our world
fn setup(
    mut commands: Commands,
) {
    // Paddle
    let paddle_y = BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR;
    let paddle = ObjectBundle {
        position: TransformBundle::from_transform(Transform::from_xyz(1.0, paddle_y as f32, 0.0)),
        velocity: Velocity { x: 0, y: 0 },
        size: Size {
            width: PADDLE_SIZE,
            height: 1,
        },
        color: PADDLE_COLOR,
    };
    commands.spawn((paddle, Paddle, Collider));
}

fn move_paddle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Paddle>>,
    time: Res<Time>,
) {
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        direction += 1.0;
    }

    // Calculate the new horizontal paddle position based on player input
    let new_paddle_position =
        paddle_transform.translation.x + (direction * PADDLE_SPEED * time.delta_seconds());

    // Update the paddle position,
    // making sure it doesn't cause the paddle to leave the arena
    let left_bound = 0;
    let right_bound = WIDTH - PADDLE_SIZE;

    paddle_transform.translation.x = new_paddle_position.clamp(left_bound as f32, right_bound as f32);
}

/// Draw solid background to buffer.
fn draw_background(mut frame_buffer: ResMut<FrameBuffer>) {
    let length = frame_buffer.0.len();
    frame_buffer.0.copy_from_slice(&[0x00].repeat(length));
}

/// Draw objects to buffer.
fn draw_objects(
    mut frame_buffer: ResMut<FrameBuffer>,
    query: Query<(&GlobalTransform, &Size, &common::Color)>,
) {
    for (position, size, color) in &query {
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
        .add_plugins(DefaultPlugins)
        .add_plugins(MatrixPlugin { list: args.list, verbose: args.verbose, serial_dev: args.serial_dev, wait_for_device: args.wait_for_device })
        .add_systems(Startup, setup)
        .insert_resource(FrameLimit(Timer::from_seconds(0.1, TimerMode::Repeating)))
        .insert_resource(AnimationTimer(Timer::from_seconds(0.3, TimerMode::Repeating)))
        .add_systems(
            FixedUpdate,
            (
                move_paddle,
            )
                // `chain`ing systems together runs them in order
                .chain(),
        )
        .add_systems(Draw, (draw_background, draw_objects).chain())
        .run();
}
