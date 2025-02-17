extern crate hub_util;

use hub_util::video_hub::{VideoHub, VideoHubLabelType};
use serde_json::Value;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::{self};
use std::time::Duration;

fn spawn_test_server<F: FnOnce(&mut TcpStream) -> () + Send + Copy + 'static>(func: Option<F>) -> i32 {
    let random_port = rand::random_range(1024..9990);
    let socket = TcpListener::bind(format!("127.0.0.1:{}", random_port)).expect("Could not start test TCP server");

    thread::spawn(move || {
        // loop {
            let (mut client, _) = socket.accept().expect("Could not accept connection");
            client.set_read_timeout(Some(Duration::from_millis(200))).expect("Failed to set Unit Test server read timeout");
            client
                .write(
                    r#"PROTOCOL PREAMBLE:
Version: 2.8

VIDEOHUB DEVICE:
Device present: true
Model name: Blackmagic Smart Videohub 20 x 20
Friendly name: Smart Videohub 20 x 20
Unique ID: 7C2E0D03192A
Video inputs: 20
Video processing units: 0
Video outputs: 20
Video monitoring outputs: 0
Serial ports: 0

INPUT LABELS:
0 Input 1
1 Input 2
2 Input 3
3 Input 4
4 Input 5
5 Input 6
6 Input 7
7 Input 8
8 Input 9
9 Input 10
10 Input 11
11 Input 12
12 Input 13
13 Input 14
14 Input 15
15 Input 15
16 Input 17
17 Input 18
18 Input 19
19 Input 20

OUTPUT LABELS:
0 Output 1
1 Output 2
2 Output 3
3 Output 4
4 Output 5
5 Output 6
6 Output 7
7 Output 8
8 Output 9
9 Output 10
10 Output 11
11 Output 12
12 Output 13
13 Output 14
14 Output 15
15 Output 16
16 Output 17
17 Output 18
18 Output 19
19 Output 20

VIDEO OUTPUT LOCKS:
0 U
1 U
2 U
3 U
4 U
5 U
6 U
7 U
8 U
9 U
10 U
11 U
12 U
13 U
14 U
15 U
16 U
17 U
18 U
19 U

VIDEO OUTPUT ROUTING:
0 0
1 1
2 2
3 3
4 4
5 5
6 6
7 7
8 8
9 9
10 10
11 11
12 12
13 13
14 14
15 15
16 16
17 17
18 18
19 19

CONFIGURATION:
Take Mode: true

END PRELUDE:
            "#
                        .as_bytes(),
                )
                .expect("Failed to write initial message to socket");

            if let Some(ref server_func) = func {
                server_func(&mut client);
                println!("server func ran");
            }

            // wait for client to close socket
            let _ = client.read_to_end(&mut vec![]);
        // }
    });
    random_port
}

const EMPTY_FUNC: Option<fn(&mut TcpStream)> = None::<fn(&mut TcpStream) -> ()>;

#[test]
fn videohub_does_parse_hello_message() {
    let port = spawn_test_server(EMPTY_FUNC);

    let hub = VideoHub::new(format!("127.0.0.1:{}", &port).parse().expect("Failed to parse server IP"))
        .expect("failed to parse videohub");
    assert_eq!(hub.input_count(), 20);
    assert_eq!(hub.output_count(), 20);
    assert_eq!(hub.model(), "Blackmagic Smart Videohub 20 x 20");
}

#[test]
fn videohub_does_dump_json() {
    let port = spawn_test_server(EMPTY_FUNC);

    let hub = VideoHub::new(format!("127.0.0.1:{}", &port).parse().expect("Failed to parse server IP"))
        .expect("failed to parse videohub");

    let json = hub.dump_json().expect("failed to dump json");

    let deserialized: Value = serde_json::from_str(&json).expect("failed to parse json");

    assert_eq!(deserialized["name"], "Blackmagic Smart Videohub 20 x 20");
    assert_eq!(deserialized["sources"].as_array().expect("failed to parse inputs").len(), 20);
}

#[test]
fn videohub_does_send_command() {
    let port = spawn_test_server(Some(|client: &mut TcpStream| {
        let mut data: Vec<u8> = Vec::new();
        let len = client.read_to_end(&mut data).unwrap_or_default();
        assert_ne!(len, 0);
        client.write("ACK".as_bytes()).expect("failed to send");
    }));

    let mut hub = VideoHub::new(format!("127.0.0.1:{}", &port).parse().expect("Failed to parse server IP"))
        .expect("failed to parse videohub");

    hub.set_label(VideoHubLabelType::Input, 0, "test label").expect("Failed to set label");
}