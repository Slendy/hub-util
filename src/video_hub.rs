use crate::debug_println;
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{SocketAddr, SocketAddrV4, TcpStream};
use std::ops::Index;
use std::time::Duration;

#[derive(Debug)]
pub struct VideoHub {
    stream: TcpStream,
    model: String,
}

#[derive(Debug, Default)]
struct Preamble {
    version: f32,
}

#[derive(Debug, Default)]
struct DeviceInfo {
    present: bool,
    model: String,
    uuid: String,

    input_count: i16,
    output_count: i16,
}

#[derive(Debug, Default)]
struct LabelList {
    labels: Vec<String>,
}

impl Preamble {
    fn new() -> Self {
        Default::default()
    }
    fn parse(lines: &Vec<&str>) -> anyhow::Result<HubMessage> {
        let mut preamble: Preamble = Default::default();
        debug_println!("{:?}", lines);
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
    fn new() -> Self {
        Default::default()
    }
    fn parse(lines: &Vec<&str>) -> anyhow::Result<HubMessage> {
        let mut device_info: DeviceInfo = Default::default();
        for line in lines {
            let parts: Vec<&str> = line.split(": ").collect();
            if parts.len() != 2 {
                debug_println!("Malformed line: {}", line);
                continue;
            }

            match line {
                s if s.starts_with("Device present: ") => {
                    device_info.present = parts[1].parse()?;
                }
                s if s.starts_with("Model name: ") => {
                    device_info.model = String::from(parts[1]);
                }
                s if s.starts_with("Video inputs: ") => {
                    device_info.input_count = parts[1].parse()?;
                }
                s if s.starts_with("Video outputs: ") => {
                    device_info.output_count = parts[1].parse()?;
                }
                s if s.starts_with("Unique ID: ") => {
                    device_info.uuid = String::from(parts[1]);
                }
                _ => continue,
            }
        }
        Ok(HubMessage::DeviceInfo(device_info))
    }
}

impl LabelList {
    fn new() -> Self {
        Default::default()
    }
    fn parse(lines: &Vec<&str>) -> anyhow::Result<LabelList> {
        let mut list: LabelList = LabelList { labels: Vec::new() };
        for line in lines {
            let delim = match line.find(' ') {
                Some(i) => i,
                None => break,
            };
            list.labels.push(String::from(&line[(delim + 1)..]));
        }
        Ok(list)
    }
}

#[derive(Debug)]
enum HubMessage {
    Preamble(Preamble),
    DeviceInfo(DeviceInfo),
    InputLabels(LabelList),
    OutputLabels(LabelList),
    PreludeEnd,
    TODO,
}

impl HubMessage {
    pub fn find_block<T: Default>(blocks: &Vec<T>) -> Option<&T> {
        let generic_type: T = Default::default();
        blocks
            .into_iter()
            .filter(|x| std::mem::discriminant(&generic_type) == std::mem::discriminant(x))
            .next()
    }
    pub fn is_preamble() {}
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
                "OUTPUT LABELS:" => LabelList::parse(&lines).map(|i| HubMessage::InputLabels(i)),
                "VIDEO OUTPUT LOCKS:" => Ok(HubMessage::PreludeEnd),
                "VIDEO OUTPUT ROUTING:" => Ok(HubMessage::PreludeEnd),
                "CONFIGURATION:" => Ok(HubMessage::PreludeEnd),
                "END PRELUDE:" => Ok(HubMessage::PreludeEnd),
                _ => {
                    debug_println!("Encountered unknown block type: {}", lines[0]);
                    continue;
                }
            };
            if let Err(e) = hub_message {
                debug_println!("Failed to process message block {}: {}", lines[1], e);
                continue;
            }

            parsed_messages.push(hub_message.unwrap());
        }
        Ok(parsed_messages)
    }
}

impl VideoHub {
    pub fn new(addr: SocketAddrV4) -> anyhow::Result<VideoHub> {
        let mut stream =
            TcpStream::connect_timeout(&SocketAddr::from(addr), Duration::from_secs(5))?;

        println!("Connected to VideoHub at {}", addr);

        stream.set_read_timeout(Some(Duration::from_millis(200)))?;
        let mut reader = BufReader::new(&stream);

        let mut input = String::new();

        loop {
            let len = match reader.read_line(&mut input) {
                Ok(len) => len,
                Err(e) => {
                    if e.kind() != ErrorKind::WouldBlock {
                        debug_println!("Err: {}", e);
                    }
                    break;
                }
            };
            if len == 0 {
                break;
            }
        }

        let blocks = HubMessage::parse_blocks(&input)?;

        if blocks.len() == 0 {
            return Err(anyhow::anyhow!("Failed to parse blocks from hello"));
        }

        let mut blocks_iter = (&blocks).into_iter();

        if let Some(HubMessage::Preamble(preamble)) =
            blocks_iter.find(|x| matches!(x, HubMessage::Preamble(_)))
        {
            println!("{:?}", preamble);
        } else {
            return Err(anyhow::anyhow!("Missing preamble block from hello"));
        }

        debug_println!("parsed blocks: {:?}", blocks);

        Ok(VideoHub {
            stream,
            model: String::from("test"),
        })
    }

    pub fn test(&mut self) {
        self.stream
            .write(&[1, 2, 3, 4])
            .expect("TODO: panic message");
    }
}
