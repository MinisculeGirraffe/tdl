mod client;
mod config;
mod download;
mod login;
mod models;

use crate::config::CONFIG;
use crate::login::*;
use anyhow::Error;
use clap::{arg, Command};
use download::{download_album, download_artist, download_track};
use log::info;
use std::env;
use std::fmt;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    {
        // read from config to always trigger initialization.
        CONFIG.read().await;
    }

    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(arg!(--url <VALUE>).help("Tidal URL to Song/Album/Artist"))
        .arg(
            arg!(--concurrent <VALUE>)
                .required(false)
                .help("Number of songs to download concurrently"),
        )
        .get_matches();

    //env_logger::Builder::from_env(Env::default().default_filter_or("none")).init();

    match login().await {
        Ok(res) => info!("Logged in: {}", res),
        Err(e) => eprintln!("{}", e),
    };
    let url = matches.get_one::<String>("url").expect("required");
    let action = Action::from_str(url).expect("invalid URL supplied");
    let concurrent = matches.get_one::<String>("concurrent");
    if let Some(val) = concurrent {
        println!("Got value");
        CONFIG.write().await.concurrency = usize::from_str(val).unwrap();
    }

    dispatch_action(action).await.unwrap();
}
#[derive(Debug)]
struct Action {
    kind: ActionKind,
    id: usize,
}
impl FromStr for Action {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url_parts: Vec<&str> = s.split('/').collect();
        let [kind, id]: [_; 2] = url_parts[url_parts.len() - 2..].try_into()?;
        Ok(Self {
            kind: ActionKind::from_str(kind)?,
            id: usize::from_str(id)?,
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
    match action.kind {
        ActionKind::Track => download_track(action.id, None).await,
        ActionKind::Album => download_album(action.id).await,
        ActionKind::Artist => download_artist(action.id).await,
    }
}
