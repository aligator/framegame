use std::thread;
use std::time::Duration;

use serialport::{SerialPort, SerialPortInfo, SerialPortType};

const FWK_MAGIC: &[u8] = &[0x32, 0xAC];
pub const FRAMEWORK_VID: u16 = 0x32AC;
pub const LED_MATRIX_PID: u16 = 0x0020;
pub const B1_LCD_PID: u16 = 0x0021;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Command {
    Brightness = 0x00,
    Pattern = 0x01,
    Bootloader = 0x02,
    Sleeping = 0x03,
    Animate = 0x04,
    Panic = 0x05,
    DisplayBwImage = 0x06,
    SendCol = 0x07,
    CommitCols = 0x08,
    _B1Reserved = 0x09,
    StartGame = 0x10,
    GameControl = 0x11,
    _GameStatus = 0x12,
    SetColor = 0x13,
    DisplayOn = 0x14,
    InvertScreen = 0x15,
    SetPixelColumn = 0x16,
    FlushFramebuffer = 0x17,
    ClearRam = 0x18,
    ScreenSaver = 0x19,
    Fps = 0x1A,
    PowerMode = 0x1B,
    AnimationPeriod = 0x1C,
    PwmFreq = 0x1E,
    DebugMode = 0x1F,
    Version = 0x20,
}

const SERIAL_TIMEOUT: Duration = Duration::from_millis(20);

fn match_serialdevs(
    ports: &[SerialPortInfo],
    requested: &Option<String>,
    pid: Option<u16>,
) -> Vec<String> {
    if let Some(requested) = requested {
        for p in ports {
            if requested == &p.port_name {
                return vec![p.port_name.clone()];
            }
        }
        vec![]
    } else {
        let mut compatible_devs = vec![];
        let pids = if let Some(pid) = pid {
            vec![pid]
        } else {
            // By default accept any type
            vec![LED_MATRIX_PID, B1_LCD_PID, 0x22, 0xFF]
        };
        // Find all supported Framework devices
        for p in ports {
            if let SerialPortType::UsbPort(usbinfo) = &p.port_type {
                if usbinfo.vid == FRAMEWORK_VID && pids.contains(&usbinfo.pid) {
                    compatible_devs.push(p.port_name.clone());
                }
            }
        }
        compatible_devs
    }
}

pub fn find_serialdevs(list: bool, verbose: bool, serial_dev: Option<String>, wait_for_device: bool) -> (Vec<String>, bool) {
    let mut serialdevs: Vec<String>;
    let mut waited = false;
    loop {
        let ports = serialport::available_ports().expect("No ports found!");
        if list || verbose {
            for p in &ports {
                match &p.port_type {
                    SerialPortType::UsbPort(usbinfo) => {
                        println!("{}", p.port_name);
                        println!("  VID     {:#06X}", usbinfo.vid);
                        println!("  PID     {:#06X}", usbinfo.pid);
                        if let Some(sn) = &usbinfo.serial_number {
                            println!("  SN      {}", sn);
                        }
                        if let Some(product) = &usbinfo.product {
                            // TODO: Seems to replace the spaces with underscore, not sure why
                            println!("  Product {}", product);
                        }
                    }
                    _ => {
                        //println!("{}", p.port_name);
                        //println!("  Unknown (PCI Port)");
                    }
                }
            }
        }
        serialdevs = match_serialdevs(
            &ports,
            &serial_dev,
            Some(LED_MATRIX_PID),
        );
        if serialdevs.is_empty() {
            if wait_for_device {
                // Waited at least once, that means the device was not present
                // when the program started
                waited = true;

                // Try again after short wait
                thread::sleep(Duration::from_millis(100));
                continue;
            } else {
                return (vec![], waited);
            }
        } else {
            break;
        }
    }
    (serialdevs, waited)
}

pub fn connect(list: bool, verbose: bool, serial_dev: Option<String>, wait_for_device: bool) -> Option<Box<dyn SerialPort>> {
    let (serialdevs, waited): (Vec<String>, bool) = find_serialdevs(list, verbose, serial_dev, wait_for_device);
    if serialdevs.is_empty() {
        println!("Failed to find serial devivce. Please manually specify with --serial-dev");
        return None;
    } else if wait_for_device && !waited {
        println!("Device already present.");
        thread::sleep(Duration::from_millis(2000));
    }


    if let Some(serialdev) = serialdevs.first() {
        let mut port = serialport::new(serialdev, 115_200)
            .timeout(SERIAL_TIMEOUT)
            .open()
            .expect("Failed to open port");

        println!("found device {}", get_device_version(&mut port));

        return Some(port);
    }

    return None;
}

pub fn simple_cmd(port: &mut Box<dyn SerialPort>, command: Command, args: &[u8], sleep: bool) {
    let mut buffer: [u8; 64] = [0; 64];
    buffer[..2].copy_from_slice(FWK_MAGIC);
    buffer[2] = command as u8;
    buffer[3..3 + args.len()].copy_from_slice(args);
    port.write_all(&buffer[..3 + args.len()])
        .expect("Write failed!");

    if sleep {
        // wait a bit
        thread::sleep(SERIAL_TIMEOUT);
    }
}

fn get_device_version(port: &mut Box<dyn SerialPort>) -> String {
    simple_cmd(port, Command::Version, &[], true);

    let mut response: Vec<u8> = vec![0; 32];
    port.read_exact(response.as_mut_slice())
        .expect("Found no data!");

    let major = response[0];
    let minor = (response[1] & 0xF0) >> 4;
    let patch = response[1] & 0x0F;
    let pre_release = response[2] == 1;

    let mut version = format!("{major}.{minor}.{patch}");
    if pre_release {
        version = format!("{version}-rc")
    }

    return version;
}
