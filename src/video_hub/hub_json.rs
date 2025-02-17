use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct VideoHubDump {
    time: u128,
    name: String,
    sources: Vec<VideoHubLabel>,
    destinations: Vec<VideoHubLabel>,
    routes: Vec<VideoHubRoute>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VideoHubLabel {
    id: usize,
    name: String,
}

pub enum VideoHubLabelType {
    Input,
    Output,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VideoHubRoute {
    destination_id: usize,
    source_id: usize,
}

impl VideoHub {
    pub fn import_dump(&mut self, json: &str) -> anyhow::Result<()> {
        let dump: VideoHubDump = serde_json::from_str(json)?;

        if dump.sources.len() >= self.input_count() {
            return Err(anyhow!("Dump contains {} inputs but VideoHub contains {} inputs", dump.sources.len(), self.input_count()));
        }

        if dump.destinations.len() >= self.output_count() {
            return Err(anyhow!("Dump contains {} outputs but VideoHub contains {} outputs", dump.sources.len(), self.input_count()));
        }

        self.set_labels(VideoHubLabelType::Input, dump.sources).expect("TODO: panic message");
        self.set_labels(VideoHubLabelType::Output, dump.destinations).expect("TODO: panic message");

        self.set_routes(dump.routes).expect("TODO: panic message");

        Ok(())
    }
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
