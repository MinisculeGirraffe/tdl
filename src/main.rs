extern crate crypto;

mod client;
mod config;
mod decryption;
mod download;
mod login;
mod models;
use std::fmt;

use std::str::FromStr;

use crate::login::*;
use anyhow::Error;
use clap::{arg, Command};
use download::{download_album, download_artist, download_track};
use env_logger::Env;

#[tokio::main]
async fn main() {
    let matches = Command::new("tdl")
        .version("0.1")
        .author("Daniel Norred")
        .about("Command Line Tidal Song Downloader")
        .arg(arg!(--url <VALUE>).help("Tidal URL to Song/Album/Artist"))
        .get_matches();

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    match login().await {
        Ok(res) => println!("Logged in: {}", res),
        Err(e) => eprintln!("{}", e),
    };
    let url = matches.get_one::<String>("url").expect("required");
    let action = Action::from_str(url).expect("invalid URL supplied");
    println!("{:?}", action);
    dispatch_action(action).await.unwrap();
}
#[derive(Debug)]
struct Action {
    kind: ActionKind,
    id: i64,
}
impl FromStr for Action {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url_parts: Vec<&str> = s.split('/').collect();
        let [kind, id]: [_; 2] = url_parts[url_parts.len() - 2..].try_into()?;
        Ok(Self {
            kind: ActionKind::from_str(kind)?,
            id: i64::from_str(id)?,
        })
    }
}
#[derive(Debug)]
enum ActionKind {
    Track,
    Album,
    Artist,
}
impl FromStr for ActionKind {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "track" => Ok(ActionKind::Track),
            "album" => Ok(ActionKind::Album),
            "artist" => Ok(ActionKind::Artist),
            _ => Err(Error::msg("No action kind for type")),
        }
    }
}

impl fmt::Display for ActionKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            ActionKind::Track => "track",
            ActionKind::Album => "album",
            ActionKind::Artist => "artist",
        };
        fmt.write_str(str)?;
        Ok(())
    }
}

async fn dispatch_action(action: Action) -> Result<bool, Error> {
    // let url = format!("https://api.tidal.com/v1/{}/{}",action.kind.to_string(),action.id);

    match action.kind {
        ActionKind::Track => download_track(action.id).await,
        ActionKind::Album => download_album(action.id).await,
        ActionKind::Artist => download_artist(action.id).await,
    }
}
