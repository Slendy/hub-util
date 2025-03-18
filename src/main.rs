use hub_util::video_hub::VideoHub;
use std::fs;

use clap::{arg, command, Parser, Subcommand};

#[derive(Debug, Parser)] // requires `derive` feature
#[command(about = "A CLI tool written in Rust for interacting with Blackmagic Videohub devices", long_about = None, version)]

struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Debug, Subcommand)]
enum Commands {
    Test {},
    /// Saves all relevant Videohub information into a single file that can be re-imported
    Dump {
        #[arg(short, long)]
        ip: String,
    },
    /// Loads all parameters from a Videohub dump file to a Videohub device
    Import {
        #[arg(short, long)]
        ip: String,
        #[arg(short, long)]
        file: String,
    },
}

#[crate_type = "bin"]
fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Test {} => {}
        Commands::Dump { ip } => {
            let mut ip = ip.to_owned();
            if !ip.contains(":") {
                ip = format!("{ip}:9990");
            }
            let router = VideoHub::new(ip.parse().expect("Invalid IP address"));
            let json = match router {
                Err(e) => panic!("{e}"),
                Ok(router) => router.dump_json().unwrap_or("".to_string()),
            };
            println!("{}", json);
        }
        Commands::Import { ip, file } => {
            let mut ip = ip.to_owned();
            if !ip.contains(":") {
                ip = format!("{ip}:9990");
            }
            let dump = fs::read_to_string(file).expect("Failed to read file");

            let mut router = VideoHub::new(ip.parse().expect("Invalid IP address")).expect("Failed to connect to router");

            router.import_dump(&dump).expect("Failed to import dump");
        }
    }
}
