use futures::prelude::*;
use irc::client::prelude::*;
use regex::Regex;
use std::env;
use std::net::{IpAddr, Ipv4Addr};
mod download;

#[derive(Debug, Clone)]
struct HSQueryResults {
    bot: String,
    pack: u32,
    size: u32,
    filename: String,
}

use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::style;

const TERMINAL_SIZE_OFFSET: usize = 4;
//  /msg Ginpachi-Sensei xdcc send #10199

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    let mut current_anime : usize = 0;
    let mut started_downloading: bool = false;

    let anime_list = choose_download().await;

    let re = Regex::new(r#""(.*)" (\d*) (\d*) (\d*)"#).unwrap();
    let config = Config {
        nickname: Some("soudini-awfuldl".to_owned()),
        server: Some("irc.rizon.net".to_owned()),
        ..Config::default()
    };
    println!("starting client and identifying");
    let mut client = Client::from_config(config).await?;
    client.identify()?;
    println!("starting streaming");

    let mut stream = client.stream()?;
    println!("streaming");


    while let Some(message) = stream.next().await.transpose()? {
        if current_anime >= anime_list.len() {
            break;
        }
        match &message.command {
            Command::PRIVMSG(_, message) => {
                println!("Getting PRIVMSG {:#?}", &message);
                if re.is_match(&message) {
                    for cap in re.captures_iter(&message) {
                        download::download_anime(&download::DCCStream {
                            filename: cap[1].to_string(),
                            ip: IpAddr::V4(Ipv4Addr::from(cap[2].parse::<u32>().unwrap())),
                            port: cap[3].parse::<usize>().unwrap(),
                            file_size: cap[4].parse::<usize>().unwrap(),
                        })
                        .unwrap();
                        current_anime += 1;
                        if current_anime >= anime_list.len() {
                            break;
                        }
                        send_dl_request(&client, &anime_list[current_anime])
                            .await
                            .unwrap();
                    }
                }
            }
            Command::PONG(_, _) => {
                //println!("Getting PONG msg : {:#?}", &message);
                if !started_downloading {
                    send_dl_request(&client, &anime_list[current_anime])
                        .await
                        .unwrap();
                    started_downloading = true;
                }
            }
            _ => {}
        }
    }
    Ok(())
}


async fn send_dl_request(client: &Client, anime: &HSQueryResults) -> Result<(), failure::Error> {
    println!("starting to send messages");
    println!("asking for {} from {}", &anime.filename, &anime.bot);
    client
        .send_privmsg(&anime.bot, format!("xdcc send #{}", &anime.pack))
        .unwrap();
    Ok(())
}

async fn choose_download() -> Vec<HSQueryResults> {
    let args: Vec<String> = env::args().collect();
    // Get the standard input stream.
    let stdin = stdin();
    // Get the standard output stream and go to raw mode.
    let mut stdout = stdout().into_raw_mode().unwrap();

    write!(
        stdout,
        "{}{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        termion::cursor::Hide
    )
    .unwrap();
    stdout.flush().unwrap();

    let mut cursor_position: usize = 0;
    let anime_list: Vec<HSQueryResults> = get_anime_list(&args[1]).await.unwrap();
    let texts = anime_list
        .iter()
        .map(|anime| &anime.filename)
        .collect::<Vec<&String>>();
    let mut selection = anime_list.iter().map(|_| false).collect::<Vec<bool>>();

    let mut display_window: [usize; 2] = [
        0,
        termion::terminal_size().unwrap().1 as usize - TERMINAL_SIZE_OFFSET - 1,
    ];

    display_text(&texts, cursor_position, &selection, &display_window);
    for c in stdin.keys() {
        write!(
            stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
        )
        .unwrap();

        match c.unwrap() {
            Key::Esc => {
                write!(stdout, "{}{}", termion::cursor::Show, termion::clear::All).unwrap();

                stdout.flush().unwrap();
                std::mem::drop(stdout);
                std::process::exit(0)
            }
            Key::Char('z') => {
                move_cursor(&mut cursor_position, -1, texts.len(), &mut display_window)
            }
            Key::Char('s') => {
                move_cursor(&mut cursor_position, 1, texts.len(), &mut display_window)
            }
            Key::Char('Z') => {
                selection[cursor_position] = true;
                move_cursor(&mut cursor_position, -1, texts.len(), &mut display_window);
            }
            Key::Char('S') => {
                selection[cursor_position] = true;
                move_cursor(&mut cursor_position, 1, texts.len(), &mut display_window);
            }
            Key::Ctrl('z') => {
                selection[cursor_position] = false;
                move_cursor(&mut cursor_position, -1, texts.len(), &mut display_window);
            }
            Key::Ctrl('s') => {
                selection[cursor_position] = false;
                move_cursor(&mut cursor_position, 1, texts.len(), &mut display_window);
            }
            Key::Char(' ') => {
                selection[cursor_position] = !selection[cursor_position];
            }
            Key::Char('\n') => break,
            _ => {}
        }

        display_text(&texts, cursor_position, &selection, &display_window);
        // Flush again.
        stdout.flush().unwrap();
    }
    write!(stdout, "{}", termion::cursor::Show).unwrap();
    return anime_list
        .iter()
        .enumerate()
        .filter(|&(i, _)| selection[i])
        .map(|(_, anime)| anime)
        .cloned()
        .collect::<Vec<HSQueryResults>>();
}

fn move_cursor(
    cursor_position: &mut usize,
    cursor_move: isize,
    position_max: usize,
    display_window: &mut [usize; 2],
) {
    if (cursor_move == -1) & (*cursor_position > 0) {
        *cursor_position -= 1;
        if *cursor_position <= display_window[0] {
            *display_window = [
                *cursor_position,
                termion::terminal_size().unwrap().1 as usize - 1 + *cursor_position
                    - TERMINAL_SIZE_OFFSET,
            ];
        }
    } else if (cursor_move == 1) & (*cursor_position < position_max - 1) {
        *cursor_position += 1;
        if *cursor_position >= display_window[1] {
            *display_window = [
                *cursor_position + 1 + TERMINAL_SIZE_OFFSET
                    - termion::terminal_size().unwrap().1 as usize,
                *cursor_position,
            ];
        }
    }
}

fn display_text(
    texts: &Vec<&String>,
    cursor_position: usize,
    selection: &Vec<bool>,
    display_window: &[usize; 2],
) {
    println!(
        "{bold}<ESC> = <quit> <Enter> = <start download> <space> = <toggle> <z/s> = <Up/Down> <MAJ+z/MAJ+s> = <Select + Up/Select + Down> <CTRL+z/CTRL+s> = <Unselect + Up/Unselect + Down>{reset}\r",
        bold = style::Bold,
        reset = style::Reset,
    );
    for i in display_window[0]..=std::cmp::min(display_window[1], texts.len() - 1) {
        if i == cursor_position {
            println!(
                "{bold}{color}{line}{reset}\r",
                color = if selection[i] { "\u{1b}[31m" } else { "" },
                bold = style::Bold,
                line = texts[i],
                reset = style::Reset,
            );
        } else {
            println!(
                "{color}{line}{reset}\r",
                color = if selection[i] { "\u{1b}[31m" } else { "" },
                line = texts[i],
                reset = style::Reset,
            );
        }
    }
}

async fn get_anime_list(search: &str) -> Result<Vec<HSQueryResults>, Box<dyn std::error::Error>> {
    let re = Regex::new(r#"b:"(.*)", n:(\d*), s:(\d*), f:"(.*)"}"#).unwrap();
    let resp = reqwest::get(&format!(
        "https://xdcc.horriblesubs.info/search.php?t={}",
        search
    ))
    .await?
    .text()
    .await?;

    let list: Vec<&str> = resp.split(";\n").collect();
    let mut v: Vec<HSQueryResults> = Vec::new();
    for elem in list.iter() {
        for cap in re.captures_iter(elem) {
            v.push(HSQueryResults {
                bot: cap[1].to_string(),
                pack: cap[2].parse::<u32>().unwrap(),
                size: cap[3].parse::<u32>().unwrap(),
                filename: cap[4].to_string(),
            })
        }
    }
    Ok(v)
}
