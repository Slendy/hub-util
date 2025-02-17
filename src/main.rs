use hub_util::video_hub::VideoHub;

use clap::{arg, command, Parser, Subcommand};

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(about = "", long_about = None, version)]

struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Debug, Subcommand)]
enum Commands {
    Test {},
    Dump {
        #[arg(short, long)]
        ip: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Test {} => {}
        Commands::Dump { ip } => {
            let router = VideoHub::new(ip.parse().expect("Invalid IP address"));
            let json = match router {
                Err(e) => panic!("{e}"),
                Ok(router) => router.dump_json().unwrap_or("".to_string()),
            };
            println!("{}", json);
        }
    }

    println!("{:?}", cli);
    // let args: Vec<String> = env::args().collect();
    // let router = VideoHub::new("172.16.160.9:9990".parse().unwrap());
    // debug_println!("router: {:?}", router);

    // println!("Hello, world!");
}
