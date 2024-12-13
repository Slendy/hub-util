use std::io::Write;
use std::net::{SocketAddr, SocketAddrV4, TcpStream};
use std::time::Duration;
use crate::debug_println;

#[derive(Debug)]
pub struct VideoHub {
    addr: SocketAddrV4,
    stream: TcpStream,
    model: String,
}

impl VideoHub {
    pub fn new(addr: SocketAddrV4) -> VideoHub {
        let mut stream = match TcpStream::connect_timeout(
            &SocketAddr::from(addr),
            Duration::new(10, 0)) {
            Ok(s) => s,
            Err(_) => panic!("Failed to connect to VideoHub : {}", addr),
        };

        // stream.read

        println!("Connected to VideoHub");
        debug_println!("Connected to VideoHub at {}", addr);

        stream.write(&[1, 2, 3, 4]).expect("TODO: panic message");

        VideoHub { addr, stream, model: String::from("test") }
    }

    pub fn test(&mut self){
        self.stream.write(&[1, 2, 3, 4]).expect("TODO: panic message");
    }
}