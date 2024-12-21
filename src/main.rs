use hub_util::{debug_println, video_hub::VideoHub};

fn main() {
    // let args: Vec<String> = env::args().collect();
    let router = VideoHub::new("172.16.160.9:9990".parse().unwrap());
    debug_println!("router: {:?}", router);

    println!("Hello, world!");
}
