use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::{Read, Write};
use std::net::{IpAddr, Shutdown, TcpStream};

#[derive(Debug, Clone)]
pub struct DCCStream {
    pub filename: String,
    pub ip: IpAddr,
    pub port: usize,
    pub file_size: usize,
}

pub fn download_anime(anime: &DCCStream) -> std::result::Result<(), std::io::Error> {
    let bar = ProgressBar::new(anime.file_size as u64);
    bar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} {msg} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .progress_chars("#>-"));
    let filename = anime.filename.to_string();
    bar.set_message(&filename);
    let mut file = File::create(&anime.filename).unwrap();
    let mut stream = TcpStream::connect(format!("{}:{}", anime.ip, anime.port)).unwrap();
    let mut buffer = [0; 4096];
    let mut progress: usize = 0;

    while progress < anime.file_size {
        let count = stream.read(&mut buffer[..]).unwrap();
        file.write(&mut buffer[..count]).unwrap();
        progress += count;
        bar.set_position(progress as u64);
        // println!("download for {} is at {}/{}", anime.filename, progress, anime.file_size);
    }
    stream.shutdown(Shutdown::Both).unwrap();
    file.flush().unwrap();
    Ok(())
}
