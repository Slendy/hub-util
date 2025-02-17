use anyhow::anyhow;

use crate::debug_println;
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Write};
use std::net::{SocketAddr, SocketAddrV4, TcpStream};
use std::time::Duration;

#[derive(Debug)]
pub struct VideoHub {
    stream: TcpStream,
    model: String,
    input_count: usize,
    input_labels: Vec<String>,
    output_count: usize,
    output_labels: Vec<String>,
    video_routes: Vec<usize>,
}

include!("hub_json.rs");

impl VideoHub {
    fn default(tcp_stream: TcpStream) -> Self {
        Self {
            stream: tcp_stream,
            model: "".to_string(),
            input_count: 0,
            input_labels: vec![],
            output_count: 0,
            output_labels: vec![],
            video_routes: vec![],
        }
    }
    pub fn input_count(&self) -> usize {
        self.input_count
    }
    pub fn output_count(&self) -> usize {
        self.output_count
    }
    pub fn input_labels(&self) -> &Vec<String> {
        &self.input_labels
    }
    pub fn output_labels(&self) -> &Vec<String> {
        &self.output_labels
    }
    pub fn model(&self) -> &str {
        &self.model
    }
    pub fn video_routes(&self) -> &Vec<usize> {
        &self.video_routes
    }
    pub fn set_label(
        &mut self,
        label_type: VideoHubLabelType,
        index: usize,
        label: &str,
    ) -> anyhow::Result<()> {
        let labels = LabelList {
            labels: vec![Label {
                index,
                name: label.to_string(),
            }],
        };

        match label_type {
            VideoHubLabelType::Input => self.send_message(HubMessage::InputLabels(labels)),
            VideoHubLabelType::Output => self.send_message(HubMessage::OutputLabels(labels)),
        }
    }
    pub fn set_labels(
        &mut self,
        label_type: VideoHubLabelType,
        labels: Vec<VideoHubLabel>,
    ) -> anyhow::Result<()> {
        let labels = LabelList {
            labels: labels
                .iter()
                .map(|label| Label {
                    name: label.name.to_owned(),
                    index: label.id,
                })
                .collect(),
        };
        match label_type {
            VideoHubLabelType::Input => self.send_message(HubMessage::InputLabels(labels)),
            VideoHubLabelType::Output => self.send_message(HubMessage::OutputLabels(labels)),
        }
    }
    pub fn set_routes(&mut self, routes: Vec<VideoHubRoute>) -> anyhow::Result<()> {
        let routes = VideoRouting {
            routes: routes
                .iter()
                .map(|route| Route {
                    source: route.source_id,
                    destination: route.destination_id,
                })
                .collect(),
        };

        self.send_message(HubMessage::VideoRouting(routes))
    }
}

#[derive(Debug)]
enum HubMessage {
    Preamble(Preamble),
    DeviceInfo(DeviceInfo),
    InputLabels(LabelList),
    OutputLabels(LabelList),
    VideoRouting(VideoRouting),
    PreludeEnd,
    Acknowledge,
    NoAcknowledge,
    TODO,
}

#[derive(Debug, Default)]
struct Preamble {
    version: f32,
}

#[derive(Debug, Default)]
struct DeviceInfo {
    present: String,
    model: String,
    uuid: String,

    input_count: usize,
    output_count: usize,
}

#[derive(Debug, Default)]
struct LabelList {
    labels: Vec<Label>,
}

#[derive(Debug)]
struct Label {
    name: String,
    index: usize,
}

#[derive(Default, Debug)]
struct VideoRouting {
    routes: Vec<Route>,
}

#[derive(Debug)]
struct Route {
    destination: usize,
    source: usize,
}

impl Preamble {
    fn parse(lines: &Vec<&str>) -> anyhow::Result<HubMessage> {
        let mut preamble: Preamble = Preamble::default();
        for line in lines {
            match line {
                line if line.starts_with("Version: ") => {
                    preamble.version = line["Version: ".len()..].parse()?;
                }
                _ => continue,
            }
        }
        Ok(HubMessage::Preamble(preamble))
    }
}

#[test]
fn test_preamble_parse() {
    let msg = Preamble::parse(&vec!["Version: 2.7"]).expect("Failed to parse version");
    if let HubMessage::Preamble(preamble) = msg {
        assert_eq!(preamble.version, 2.7);
    } else {
        panic!("Parsed message is not preamble, {:?}", msg);
    }
}

impl DeviceInfo {
    // Format style:
    // (Key): (Value)
    // ...
    fn parse(lines: &Vec<&str>) -> anyhow::Result<HubMessage> {
        let mut device_info: DeviceInfo = DeviceInfo::default();
        for line in lines {
            let parts: Vec<&str> = line.split(": ").collect();
            if parts.len() != 2 {
                debug_println!("Malformed line: {}", line);
                continue;
            }

            match line {
                s if s.starts_with("Device present: ") => {
                    device_info.present = parts[1].to_owned();
                }
                s if s.starts_with("Model name: ") => {
                    device_info.model = parts[1].to_owned();
                }
                s if s.starts_with("Video inputs: ") => {
                    device_info.input_count = parts[1].parse()?;
                }
                s if s.starts_with("Video outputs: ") => {
                    device_info.output_count = parts[1].parse()?;
                }
                s if s.starts_with("Unique ID: ") => {
                    device_info.uuid = parts[1].to_owned();
                }
                _ => continue,
            }
        }
        Ok(HubMessage::DeviceInfo(device_info))
    }
}

#[test]
fn test_device_info_parse() {
    let msg = DeviceInfo::parse(&vec![
        "Device present: true",
        "Model name: test",
        "Video inputs: 37",
        "Video outputs: 37",
        "Unique ID: test",
    ])
    .expect("Failed to parse version info");
    if let HubMessage::DeviceInfo(device_info) = msg {
        assert_eq!(device_info.input_count, 37);
        assert_eq!(device_info.output_count, 37);
        assert_eq!(device_info.present, "true");
        assert_eq!(device_info.uuid, "test");
        assert_eq!(device_info.model, "test");
    } else {
        panic!("Parsed message is not device info, {:?}", msg);
    }
}

impl LabelList {
    // Example format:
    // 0 Input 1
    // 1 Input 2
    // ...
    fn parse(lines: &Vec<&str>) -> anyhow::Result<LabelList> {
        let mut list: LabelList = LabelList { labels: Vec::new() };
        for line in lines {
            let delim = match line.find(' ') {
                Some(i) => i,
                None => break,
            };

            let index: i32 = line[..delim].parse()?;

            if index < 0 {
                continue;
            }

            list.labels.push(Label {
                name: line[(delim + 1)..].to_owned(),
                index: index as usize,
            });
        }
        Ok(list)
    }
    fn serialize(&self) -> String {
        let mut serialized = String::new();
        for label in &self.labels {
            serialized += &format!("{} {}\n", label.index, label.name)
        }
        serialized
    }
}

#[test]
fn test_label_list_parse() {
    let msg = LabelList::parse(&vec![
        "-1 test 1",
        "0 test 1",
        "1 test 2",
        "2 test 3",
        "3 test 4",
        "4 test 5",
    ])
    .expect("Failed to parse label list");
    assert_eq!(msg.labels.len(), 5);
    assert_eq!(msg.labels[0].name, "test 1");
}

#[test]
fn test_label_list_serialize() {
    let list = LabelList {
        labels: vec![
            Label {
                index: 0,
                name: "test 1".to_string(),
            },
            Label {
                index: 15,
                name: "test 16".to_string(),
            },
        ],
    };
    let serialized = list.serialize();
    assert_eq!(serialized, "0 test 1\n15 test 16\n");
}

impl VideoRouting {
    // Example format:
    // 0 0 (input 0 routed to output 0)
    // 1 1 (input 1 routed to output 1)
    // ...
    fn parse(lines: &Vec<&str>) -> anyhow::Result<HubMessage> {
        let mut routing: VideoRouting = VideoRouting::default();
        for line in lines {
            let parts: Vec<&str> = line.split(" ").collect();

            let src: i32 = parts[1].parse()?;
            let dest: i32 = parts[0].parse()?;

            if src < 0 || dest < 0 {
                continue;
            }

            routing.routes.push(Route {
                source: src as usize,
                destination: dest as usize,
            });
        }
        Ok(HubMessage::VideoRouting(routing))
    }
    fn serialize(&self) -> String {
        let mut serialized = String::new();
        for routing in &self.routes {
            serialized += &format!("{} {}\n", routing.destination, routing.source);
        }
        serialized
    }
}

#[test]
fn test_video_routing_parse() {
    let msg = VideoRouting::parse(&vec!["-1 -1", "0 0", "1 1", "2 2", "3 3", "4 4"])
        .expect("Failed to parse label list");
    if let HubMessage::VideoRouting(routing) = msg {
        assert_eq!(routing.routes.len(), 5);
        assert_eq!(routing.routes[0].destination, 0);
    } else {
        panic!("Parsed message is not video routing");
    }
}

#[test]
fn test_video_routing_serialize() {
    let list = VideoRouting {
        routes: vec![
            Route {
                source: 0,
                destination: 0,
            },
            Route {
                source: 1,
                destination: 1,
            },
        ],
    };
    let serialized = list.serialize();
    assert_eq!(serialized, "0 0\n1 1\n");
}

impl HubMessage {
    pub fn get_header(&self) -> String {
        match self {
            HubMessage::Preamble(_) => "PROTOCOL PREAMBLE:".to_string(),
            HubMessage::DeviceInfo(_) => "VIDEOHUB DEVICE:".to_string(),
            HubMessage::InputLabels(_) => "INPUT LABELS:".to_string(),
            HubMessage::OutputLabels(_) => "OUTPUT LABELS:".to_string(),
            HubMessage::VideoRouting(_) => "VIDEO OUTPUT ROUTING:".to_string(),
            _ => "TODO".to_string(),
        }
    }
    pub fn parse_blocks(msg: &str) -> anyhow::Result<Vec<HubMessage>> {
        let mut parsed_messages: Vec<HubMessage> = Vec::new();
        let blocks: Vec<&str> = msg.split("\n\n").collect();
        for block in blocks {
            let lines: Vec<&str> = block.lines().collect();
            if lines.len() < 1 {
                continue;
            }
            let header = lines[0];

            let lines = lines[1..].to_vec();

            let hub_message = match header {
                "PROTOCOL PREAMBLE:" => Preamble::parse(&lines),
                "VIDEOHUB DEVICE:" => DeviceInfo::parse(&lines),
                "INPUT LABELS:" => LabelList::parse(&lines).map(|i| HubMessage::InputLabels(i)),
                "OUTPUT LABELS:" => LabelList::parse(&lines).map(|i| HubMessage::OutputLabels(i)),
                "VIDEO OUTPUT LOCKS:" => Ok(HubMessage::TODO),
                "VIDEO OUTPUT ROUTING:" => VideoRouting::parse(&lines),
                "CONFIGURATION:" => Ok(HubMessage::TODO),
                "END PRELUDE:" => Ok(HubMessage::PreludeEnd),
                "ACK" => Ok(HubMessage::Acknowledge),
                "NACK" => Ok(HubMessage::NoAcknowledge),
                _ => {
                    debug_println!("Encountered unknown block type: {}", header);
                    continue;
                }
            };
            match hub_message {
                Err(e) => {
                    debug_println!("Failed to process message block ({}): {}", header, e);
                    continue;
                }
                Ok(msg) => parsed_messages.push(msg),
            }
        }
        Ok(parsed_messages)
    }
}

impl VideoHub {
    fn write(&self, msg: &str) -> anyhow::Result<()> {
        let mut writer = BufWriter::new(&self.stream);
        writer
            .write_all(msg.as_bytes())
            .with_context(|| "Failed to write message to stream")
    }
    fn read_all(&self) -> String {
        let mut reader = BufReader::new(&self.stream);
        let mut input = String::new();
        loop {
            let len = match reader.read_line(&mut input) {
                Ok(len) => len,
                Err(e) => {
                    // a read timeout returns WouldBlock on Unix-like systems and TimedOut on Windows
                    if e.kind() != ErrorKind::WouldBlock && e.kind() != ErrorKind::TimedOut {
                        debug_println!("Err ({}): {}", e.kind(), e);
                    }
                    break;
                }
            };
            if len == 0 {
                break;
            }
        }
        input
    }

    fn send_message(&mut self, msg: HubMessage) -> anyhow::Result<()> {
        let serialized = match &msg {
            HubMessage::InputLabels(labels) => Ok(labels.serialize()),
            HubMessage::OutputLabels(labels) => Ok(labels.serialize()),
            HubMessage::VideoRouting(routes) => Ok(routes.serialize()),
            _ => Err(anyhow!("Cannot serialize this type")),
        }?;

        let header = msg.get_header();

        // header does not contain newline, and message must be terminated with 2 newlines
        let serialized = format!("{}\n{}\n", header, serialized);

        self.write(&serialized)?;

        let response = self.read_all();

        let blocks = HubMessage::parse_blocks(&response)?;

        self.update(&blocks)?;

        // Return an error if server returns 'NACK' or fails to send an 'ACK'
        if blocks
            .iter()
            .any(|x| matches!(x, HubMessage::NoAcknowledge))
            || !blocks.iter().any(|x| matches!(x, HubMessage::Acknowledge))
        {
            return Err(anyhow!("Server did not acknowledge request: {}", response));
        }

        Ok(())
    }
    fn update(&mut self, blocks: &Vec<HubMessage>) -> anyhow::Result<()> {
        for block in blocks {
            match block {
                HubMessage::Preamble(preamble) => {
                    debug_println!("Preamble: {:?}", preamble);
                }
                HubMessage::DeviceInfo(device_info) => {
                    debug_println!("DeviceInfo: {:?}", device_info);
                    self.input_count = device_info.input_count;
                    self.output_count = device_info.output_count;

                    self.input_labels
                        .resize(device_info.input_count, "".to_string());
                    self.output_labels
                        .resize(device_info.input_count, "".to_string());
                    self.video_routes.resize(device_info.input_count, 0);

                    self.model = device_info.model.clone();
                }
                HubMessage::InputLabels(input_labels) => {
                    debug_println!("InputLabels: {:?}", input_labels);
                    for label in &input_labels.labels {
                        self.input_labels[label.index] = label.name.clone();
                    }
                }
                HubMessage::OutputLabels(output_labels) => {
                    debug_println!("OutputLabels: {:?}", output_labels);
                    for label in &output_labels.labels {
                        self.output_labels[label.index] = label.name.clone();
                    }
                }
                HubMessage::VideoRouting(routing) => {
                    debug_println!("VideoRouting: {:?}", routing);
                    for route in &routing.routes {
                        self.video_routes[route.destination] = route.source;
                    }
                }
                _ => continue,
            }
        }
        Ok(())
    }
    pub fn new(addr: SocketAddrV4) -> anyhow::Result<VideoHub> {
        let stream = TcpStream::connect_timeout(&SocketAddr::from(addr), Duration::from_secs(5))?;
        stream.set_read_timeout(Some(Duration::from_millis(200)))?;

        println!("Connected to VideoHub at {}", addr);

        let mut hub = VideoHub::default(stream);

        let hello_msg = hub.read_all();

        let blocks = HubMessage::parse_blocks(&hello_msg)?;

        if blocks.len() == 0 {
            return Err(anyhow::anyhow!("Failed to parse blocks from hello"));
        }

        if let Some(HubMessage::DeviceInfo(device_info)) = blocks
            .iter()
            .find(|x| matches!(x, HubMessage::DeviceInfo(_)))
        {
            if device_info.present == "false" || device_info.present == "needs_update" {
                debug_println!("Present device present: {}", device_info.present);
            }
            if device_info.input_count != device_info.output_count {
                panic!(
                    "Input count and output count on video router are not equal: {} != {}",
                    device_info.input_count, device_info.output_count
                );
            }
        } else {
            return Err(anyhow::anyhow!("Failed to find device info block"));
        }

        hub.update(&blocks)?;

        Ok(hub)
    }
}
