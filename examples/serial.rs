// Add this to your Cargo.toml file under [dependencies]
// serialport = "4"

use std::io::{self, Write};
use std::path::Path;
use std::ffi::{OsStr, OsString};
use itertools::Itertools;
use serialport::{self, SerialPortBuilder};

type SerialError = Box<dyn std::error::Error>;

fn get_device_names() -> Vec<String> {
    Path::new("/dev/")
        .read_dir()
        .unwrap()
        .filter_map(|e| 
                    { 
                        let fname = e.as_ref().unwrap().path().file_name().unwrap().to_owned().into_string().unwrap();
                      fname.starts_with("tty.usb").then_some(fname)
                    })
    .collect()
}

fn get_beaglebone() -> String {
    let ports = serialport::available_ports().expect("Unable to get serial connection info!");
    ports.iter().find(
        |port| 
            match &port.port_type {
                serialport::SerialPortType::UsbPort(info) => {
                    info.manufacturer.as_ref().is_some_and(
                        |manufacturer| manufacturer == "BeagleBoard.org")
                },
                _ => false
            }
        ).expect("No Beaglebone USB connection detected!").port_name.to_owned()
}

fn main() -> io::Result<()> {
    let port_name = get_beaglebone();
    // Configure serial port settings
    use std::time::Duration;

    let five_seconds = Duration::new(5, 0);

    //open the serial port
    let mut port = serialport::new(port_name, 9600)
    .timeout(five_seconds).open().expect("Failed to open port");

    let mut serial_buf: Vec<u8> = vec![0; 128]; // Buffer to store received data
    // let bytes_read = port.read(serial_buf.as_mut_slice())?;
    // println!("Read {} bytes: {}", bytes_read, std::str::from_utf8(&serial_buf[..bytes_read]).unwrap());

    // Write data to the serial port (host to client)
    let user = "debian\n";
    let passwd = "temppwd\n";
    port.write_all(user.as_bytes())?;
    port.write_all(passwd.as_bytes())?;
    //
    // Read data from the serial port (client to host)
    let bytes_read = port.read(serial_buf.as_mut_slice())?;
    println!("Read {} bytes: {}", bytes_read, std::str::from_utf8(&serial_buf[..bytes_read]).unwrap());

    Ok(())
}
