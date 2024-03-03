use bevy::prelude::{Bundle, Component, TransformBundle};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, arg_required_else_help = true)]
pub struct ClapCli {
    /// List connected HID devices
    #[arg(short, long)]
    pub list: bool,

    /// Verbose outputs to the console
    #[arg(short, long)]
    pub verbose: bool,

    /// Serial device, like /dev/ttyACM0 or COM0
    #[arg(long)]
    pub serial_dev: Option<String>,

    /// Retry connecting to the device until it works
    #[arg(long)]
    pub wait_for_device: bool,
}

#[derive(Component, Debug)]
pub struct Velocity {
    pub x: i16,
    pub y: i16,
}

#[derive(Component, Debug)]
pub struct Size {
    pub width: u8,
    pub height: u8,
}

#[derive(Component, Debug)]
pub struct Color(pub u8);

#[derive(Bundle, Debug)]
pub struct ObjectBundle {
    pub position: TransformBundle,
    pub velocity: Velocity,
    pub size: Size,
    pub color: Color,
}

// Just for RustRover to be happy...
fn main() {}