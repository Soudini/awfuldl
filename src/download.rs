use pbr::{MultiBar, Pipe, ProgressBar, Units};
use regex::Regex;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Shutdown, TcpStream};
use std::str::from_utf8;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone)]
pub struct DCCStream {
    pub filename: String,
    pub ip: IpAddr,
    pub port: usize,
    pub file_size: usize,
}

pub fn download_anime(
    anime: &DCCStream,
    progress_bar: &mut ProgressBar<Pipe>,
) -> std::result::Result<(), std::io::Error> {
    progress_bar.set_units(Units::Bytes);
    progress_bar.message(&format!("{}: ", &anime.filename));
    let filename = anime.filename.to_string();
    let mut file = File::create(&anime.filename).unwrap();
    let mut stream = TcpStream::connect(format!("{}:{}", anime.ip, anime.port)).unwrap();
    let mut buffer = [0; 4096];
    let mut progress: usize = 0;

    while progress < anime.file_size {
        let count = stream.read(&mut buffer[..]).unwrap();
        file.write(&mut buffer[..count]);
        progress += count;
        progress_bar.set(progress as u64);
        progress_bar.finish();
        stream.shutdown(Shutdown::Both);
        file.flush();
    }
    Ok(())
}
