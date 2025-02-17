use anyhow::anyhow;
use std::io::Read;
use std::net::TcpStream;
use std::time::{Duration, Instant};

pub mod video_hub;

#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => (if ::std::cfg!(debug_assertions) { ::std::println!($($arg)*); })
}

pub fn read_to_newline(s: &mut TcpStream, timeout: Option<Duration>) -> anyhow::Result<String> {
    let mut result = String::new();
    s.set_nonblocking(true)?;
    let timeout = timeout.unwrap_or(Duration::from_millis(1000));
    let start = Instant::now();

    loop {
        const BUF_SIZE: usize = 256;
        let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
        let size = s.read(&mut buf).unwrap_or_default();
        if size > 0 {
            let str = core::str::from_utf8(&buf[..size])?;
            debug_println!("read data: len={}, {:?}", size, &buf[..size]);
            result += &str;
        }

        if result.ends_with("\n\n") && size != BUF_SIZE {
            debug_println!("reached double newline");
            break;
        }
        if start.elapsed() > timeout {
            debug_println!("read timed out (total={}), msg={:?}", result.len(), result);
            return Err(anyhow!("Read timed out"));
        }
    }

    s.set_nonblocking(false)?;

    Ok(result)
}
