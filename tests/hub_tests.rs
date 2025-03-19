extern crate hub_util;

use hub_util::video_hub::{VideoHub, VideoHubLabelType};
use hub_util::read_to_newline;
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
fn videohub_does_not_import_broken_json() {
    let port = spawn_test_server(EMPTY_FUNC);

    let mut hub = VideoHub::new(format!("127.0.0.1:{}", &port).parse().expect("Failed to parse server IP"))
        .expect("failed to parse videohub");

    let json = r#"{
    "timestamp": 1741618011000,
    "sources": [],
    "destinations": [],
    "routes": []
}"#;

    let result = hub.import_dump(json);
    assert_eq!(result.is_ok(), false);
}

#[test]
fn videohub_does_import_json() {
    let port = spawn_test_server(Some(|client: &mut TcpStream| {
        loop {
            println!("serv: waiting for client command");
            let cmd = read_to_newline(client, None).unwrap_or_default();
            assert_ne!(cmd.len(), 0);
            println!("serv: client command: {:?}", cmd);
            client.write("ACK\n\n".as_bytes()).expect("failed to send");
            println!("serv: wrote ack");
            if cmd.contains("LABELS") || cmd.contains("ROUTING") {
                // server will send back changes for clients to update
                client.write(cmd.as_bytes()).expect("failed to send");
            }
        }
    }));

    let mut hub = VideoHub::new(format!("127.0.0.1:{}", &port).parse().expect("Failed to parse server IP"))
        .expect("failed to parse videohub");

    let json = r#"{"time":1742323854265,"name":"Blackmagic Smart Videohub 20 x 20","sources":[{"id":0,"name":"Src 1"},{"id":1,"name":"Src 2"},{"id":2,"name":"Src 3"},{"id":3,"name":"Src 4"},{"id":4,"name":"Src 5"},{"id":5,"name":"Src 6"},{"id":6,"name":"Src 7"},{"id":7,"name":"Src 8"},{"id":8,"name":"Src 9"},{"id":9,"name":"Src 10"},{"id":10,"name":"Src 11"},{"id":11,"name":"Src 12"},{"id":12,"name":"Src 13"},{"id":13,"name":"Src 14"},{"id":14,"name":"Src 15"},{"id":15,"name":"Src 15"},{"id":16,"name":"Src 17"},{"id":17,"name":"Src 18"},{"id":18,"name":"Src 19"},{"id":19,"name":"Src 20"}],"destinations":[{"id":0,"name":"Dest 1"},{"id":1,"name":"Dest 2"},{"id":2,"name":"Dest 3"},{"id":3,"name":"Dest 4"},{"id":4,"name":"Dest 5"},{"id":5,"name":"Dest 6"},{"id":6,"name":"Dest 7"},{"id":7,"name":"Dest 8"},{"id":8,"name":"Dest 9"},{"id":9,"name":"Dest 10"},{"id":10,"name":"Dest 11"},{"id":11,"name":"Dest 12"},{"id":12,"name":"Dest 13"},{"id":13,"name":"Dest 14"},{"id":14,"name":"Dest 15"},{"id":15,"name":"Dest 16"},{"id":16,"name":"Dest 17"},{"id":17,"name":"Dest 18"},{"id":18,"name":"Dest 19"},{"id":19,"name":"Dest 20"}],"routes":[{"destinationId":0,"sourceId":0},{"destinationId":1,"sourceId":1},{"destinationId":2,"sourceId":2},{"destinationId":3,"sourceId":3},{"destinationId":4,"sourceId":4},{"destinationId":5,"sourceId":5},{"destinationId":6,"sourceId":6},{"destinationId":7,"sourceId":7},{"destinationId":8,"sourceId":8},{"destinationId":9,"sourceId":9},{"destinationId":10,"sourceId":10},{"destinationId":11,"sourceId":11},{"destinationId":12,"sourceId":12},{"destinationId":13,"sourceId":13},{"destinationId":14,"sourceId":14},{"destinationId":15,"sourceId":15},{"destinationId":16,"sourceId":16},{"destinationId":17,"sourceId":17},{"destinationId":18,"sourceId":18},{"destinationId":19,"sourceId":19}]}"#;

    let result = hub.import_dump(json);

    assert_eq!(result.is_ok(), true);
    assert_eq!(hub.input_labels()[0], "Src 1");
    assert_eq!(hub.output_labels()[0], "Dest 1");
}

#[test]
fn videohub_does_send_command() {
    let port = spawn_test_server(Some(|client: &mut TcpStream| {
        println!("serv: waiting for client command");
        let cmd = read_to_newline(client, None).unwrap_or_default();
        assert_ne!(cmd.len(), 0);
        println!("serv: client command: {:?}", cmd);
        client.write("ACK\n\n".as_bytes()).expect("failed to send");
        println!("serv: wrote ack");
    }));

    let mut hub = VideoHub::new(format!("127.0.0.1:{}", &port).parse().expect("Failed to parse server IP"))
        .expect("failed to parse videohub");

    hub.set_label(VideoHubLabelType::Input, 0, "test label").expect("Failed to set label");
}