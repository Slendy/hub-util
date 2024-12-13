extern crate hub_util;

use hub_util::video_hub::VideoHub;
use std::io::Write;
use std::net::TcpListener;
use std::thread;

#[test]
fn socket_does_connect(){
    thread::spawn(|| {
        let socket = TcpListener::bind("127.0.0.1:9990").expect("Could not start test TCP server");
        loop {
            let (mut client, _) = socket.accept().expect("Could not accept connection");
            let preamble = "PROTOCOL PREAMBLE:\nVersion: 2.3\n\n";
            let mut write = | msg: &str  | {
                client.write(msg.as_bytes()).expect("Could not write to test socket");
            };
            write(preamble);
            let device_information = "\
VIDEOHUB DEVICE:
Device present: true
Model name: Blackmagic Smart Videohub
Video inputs: 16
Video processing units: 0
Video outputs: 16
Video monitoring outputs: 0
Serial ports: 0
\n
            ";
            write(device_information);
        }
    });
    
    VideoHub::new("127.0.0.1:9990".parse().expect("Failed to parse server IP"));
}