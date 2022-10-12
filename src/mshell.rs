use clap::Args;
use mavlink::common::{MavMessage, SERIAL_CONTROL_DATA};

#[derive(Args)]
pub struct Options {
    /// Serial port over which to connect
    port: String,
    /// Baudrate of the connection.
    /// If none is provided, the program will try 3000000 and 57600.
    #[arg(short, long)]
    baudrate: Option<u32>,
}

pub fn try_connect_with_baud<M: mavlink::Message>(
    port: &str,
    baudrate: u32,
) -> Option<Box<dyn mavlink::MavConnection<M>>> {
    let address = format!("serial:{}:{}", port, baudrate);
    let connection = mavlink::connect(&address);

    if let Ok(mut connection) = connection {
        connection.set_protocol_version(mavlink::MavlinkVersion::V1); // XXX: somehow it does not work for me with V2
        let msg = connection.recv().unwrap(); // Make sure we can receive a message.
                                                    // This would benefit from a timeout, but that is currently not supported in the
                                                    // mavlink library.

        Some(connection)
    } else {
        None
    }
}

/// Tries to connect to a mavlink device on a serial port.
/// If no baudrate is given 3000000 and 57600 are tried.
pub fn try_connect<M: mavlink::Message>(
    port: &str,
    baudrate: Option<u32>,
) -> Option<Box<dyn mavlink::MavConnection<M>>> {
    if let Some(baud) = baudrate {
        try_connect_with_baud(port, baud)
    } else {
        let baudrates = vec![3000000, 57600];

        for baud in baudrates {
            if let Some(connection) = try_connect_with_baud(port, baud) {
                return Some(connection);
            }
        }

        None
    }
}

fn build_msg(mut line: String) -> MavMessage {
    let line = String::from("ver all\n");
    let msg = MavMessage::SERIAL_CONTROL(SERIAL_CONTROL_DATA{
        baudrate: 0,
        timeout: 0,
        device: mavlink::common::SerialControlDev::SERIAL_CONTROL_DEV_SHELL,
        flags: mavlink::common::SerialControlFlag::SERIAL_CONTROL_FLAG_RESPOND |
                mavlink::common::SerialControlFlag::SERIAL_CONTROL_FLAG_MULTI |
                mavlink::common::SerialControlFlag::SERIAL_CONTROL_FLAG_EXCLUSIVE,
        data: line.as_bytes().to_vec(),
        count: line.len() as u8,
    });
    msg
}

pub fn run(options: Options) {
    let connection = try_connect::<mavlink::common::MavMessage>(&options.port, options.baudrate);
    if connection.is_none() {
        println!("Could not connect to device.");
        return;
    }

    let connection = connection.unwrap();
    let mut console = MavConsole::new();

    loop {
        let line = console.readline();
        let msg = build_msg(line);
        connection.send_default(&msg).unwrap();

        let (_, msg) = connection.recv().unwrap();

        if let MavMessage::SERIAL_CONTROL(data) = msg {
            console.handle_data(&data);
        }
    }
}

struct MavConsole {
    stdin: std::io::Stdin,
}

impl MavConsole {
    fn new() -> Self {
        Self{
            stdin: std::io::stdin(),
        }
    }

    fn handle_data(&mut self, data: &SERIAL_CONTROL_DATA) {
        let len = data.count as usize;
        let data = &data.data;

        for c in &data[..len] {
            print!("{}", *c as char);
        }
    }

    fn readline(&self) -> String {
        let mut buf = String::new();
        self.stdin.read_line(&mut buf).unwrap();
        buf
    }
}

impl Drop for MavConsole {
    fn drop(&mut self) {
    }
}
