use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct VideoHubDump {
    time: u128,
    name: String,
    sources: Vec<VideoHubLabel>,
    destinations: Vec<VideoHubLabel>,
    routes: Vec<VideoHubRoute>,
}

#[derive(Serialize, Deserialize, Debug)]
struct VideoHubLabel {
    id: usize,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct VideoHubRoute {
    destination_id: usize,
    source_id: usize,
}

impl VideoHub {
    pub fn dump_json(&self) -> anyhow::Result<String> {
        let dump = VideoHubDump {
            time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards").as_millis(),
            name: self.model().to_owned(),
            sources: self.input_labels().iter().enumerate().map(|(i, label)| {
                VideoHubLabel {
                    id: i,
                    name: label.to_owned(),
                }
            }).collect(),
            destinations: self.output_labels().iter().enumerate().map(|(i, label)| {
                VideoHubLabel {
                    id: i,
                    name: label.to_owned(),
                }
            }).collect(),
            routes: self.video_routes().iter().enumerate().map(|(dest_id, source_id)| {
                VideoHubRoute {
                    destination_id: dest_id,
                    source_id: *source_id,
                }
            }).collect(),
        };

        serde_json::to_string_pretty(&dump).with_context(|| "Failed to create JSON dump")
    }
}
