extern crate hub_util;

use hub_util::video_hub::VideoHub;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread::{self};

#[test]
fn videohub_does_parse_hello_message() {
    thread::spawn(|| {
        let socket = TcpListener::bind("127.0.0.1:9990").expect("Could not start test TCP server");
        loop {
            let (mut client, _) = socket.accept().expect("Could not accept connection");
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
0 CG1
1 CG1 Alpha
2 Resi 1
3 Resi 2
4 Grass Valley 1
5 Grass Valley 2
6 7
7 8
8 9
9 West Live
10 Livestream PGM
11 12
12 13
13 14
14 15
15 15
16 17
17 18
18 19
19 20

OUTPUT LABELS:
0 ATEM 1 - CG
1 ATEM 2
2 ATEM 3 - Resi 1
3 ATEM 4 - Resi - 2
4 ATEM 5 - Grass Valley 1
5 ATEM 6 - Grass Valley 2
6 ATEM 7
7 ATEM 8
8 ATEM 9
9 ATEM 10 - West Live
10 11
11 12
12 13
13 14
14 15
15 16
16 17
17 18
18 19
19 Audio Monitor

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
5 9
6 4
7 8
8 9
9 9
10 9
11 5
12 1
13 17
14 16
15 5
16 10
17 14
18 8
19 10

CONFIGURATION:
Take Mode: true

END PRELUDE:
            "#
                    .as_bytes(),
                )
                .expect("Failed to write initial message to socket");
            // wait for client to close socket
            let _ = client.read_to_end(&mut vec![]);
        }
    });

    let hub = VideoHub::new("127.0.0.1:9990".parse().expect("Failed to parse server IP"))
        .expect("failed to parse videohub");
    assert_eq!(hub.input_count(), 20);
    assert_eq!(hub.output_count(), 20);
    assert_eq!(hub.model(), "Blackmagic Smart Videohub 20 x 20");
    println!("videohub {:?}", hub)
}
