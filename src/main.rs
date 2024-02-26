use std::ops::Deref;

use bevy::app::FixedMain;
use bevy::prelude::*;
use clap::Parser;
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
    port: Option<Box<dyn SerialPort>>,
    
    frame_buffer: [u8; WIDTH * HEIGHT]
}

impl LightMatrix {
    pub fn new(args: ClapCli) -> Self {
        Self {
            port: connect(args.list, args.verbose, args.serial_dev, args.wait_for_device),
            frame_buffer: [0x00; WIDTH * HEIGHT],
        }
    }

    pub fn draw(&mut self) {
        if let Some(port) = &mut self.port {
            let mut row: [u8; HEIGHT + 1] = [0xff; HEIGHT + 1];
            for x in 0..WIDTH {
                row[0] = u8::try_from(x).unwrap();
                simple_cmd(port, Command::SendCol, &row.clone(), true);
            }

            simple_cmd(port, Command::CommitCols, &[], true);
        }
    }
}

fn draw_frame(mut light_matrix: NonSendMut<LightMatrix>) {
    light_matrix.draw();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let args = ClapCli::parse_from(args);

    App::new()
        .add_plugins(MinimalPlugins)
        .insert_non_send_resource(LightMatrix::new(args))
        .add_systems(FixedMain, draw_frame)
        .run();

    /* let port = connect(args.list, args.verbose, args.serial_dev, args.wait_for_device);
     if let Some(mut port) = port {
         run(&mut port);
     }*/
}
