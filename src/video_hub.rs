use crate::debug_println;
use std::io::{BufRead, BufReader, ErrorKind};
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
}

#[derive(Debug)]
enum HubMessage {
    Preamble(Preamble),
    DeviceInfo(DeviceInfo),
    InputLabels(LabelList),
    OutputLabels(LabelList),
    VideoRouting(VideoRouting),
    PreludeEnd,
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
            
            list.labels.push(Label {
                name: line[(delim + 1)..].to_owned(),
                index: line[..delim].parse()?,
            });
        }
        Ok(list)
    }
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

            routing.routes.push(Route {
                source: parts[1].parse()?,
                destination: parts[0].parse()?,
            });
        }
        Ok(HubMessage::VideoRouting(routing))
    }
}

impl HubMessage {
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
                Ok(msg) => parsed_messages.push(msg)
            }
        }
        Ok(parsed_messages)
    }
}

impl VideoHub {
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
    pub fn new(addr: SocketAddrV4) -> anyhow::Result<VideoHub> {
        let stream =
            TcpStream::connect_timeout(&SocketAddr::from(addr), Duration::from_secs(5))?;
        stream.set_read_timeout(Some(Duration::from_millis(200)))?;

        println!("Connected to VideoHub at {}", addr);

        let mut hub = VideoHub::default(stream);

        let hello_msg = hub.read_all();

        let blocks = HubMessage::parse_blocks(&hello_msg)?;

        if blocks.len() == 0 {
            return Err(anyhow::anyhow!("Failed to parse blocks from hello"));
        }

        debug_println!("parsed blocks: {:?}", blocks);

        if let Some(HubMessage::DeviceInfo(device_info)) = blocks.iter().find(|x| matches!(x, HubMessage::DeviceInfo(_))) {
            if device_info.present == "false" || device_info.present == "needs_update" {
                debug_println!("Present device present: {}", device_info.present);
            }
            if device_info.input_count != device_info.output_count {
                panic!("Input count and output count on video router are not equal: {} != {}", device_info.input_count, device_info.output_count);
            }
        } else {
            return Err(anyhow::anyhow!("Failed to find device info block"));
        }

        for block in blocks {
            match block {
                HubMessage::Preamble(preamble) => {
                    debug_println!("Preamble: {:?}", preamble);
                }
                HubMessage::DeviceInfo(device_info) => {
                    debug_println!("DeviceInfo: {:?}", device_info);
                    hub.input_count = device_info.input_count;
                    hub.output_count = device_info.output_count;
                    
                    hub.input_labels.resize(device_info.input_count, String::new());
                    hub.output_labels.resize(device_info.input_count, String::new());
                    hub.video_routes.resize(device_info.input_count, 0);
                    
                    hub.model = device_info.model;
                }
                HubMessage::InputLabels(input_labels) => {
                    debug_println!("InputLabels: {:?}", input_labels);
                    for label in input_labels.labels {
                        hub.input_labels[label.index] = label.name;
                    }
                }
                HubMessage::OutputLabels(output_labels) => {
                    debug_println!("OutputLabels: {:?}", output_labels);
                    for label in output_labels.labels {
                        hub.input_labels[label.index] = label.name;
                    }
                }
                HubMessage::VideoRouting(routing) => {
                    debug_println!("VideoRouting: {:?}", routing);
                    for route in routing.routes {
                        hub.video_routes[route.destination] = route.source;
                    }
                }
                _ => continue,
            }
        }
        Ok(hub)
    }
}
