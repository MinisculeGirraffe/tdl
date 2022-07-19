mod api;
mod config;
mod download;
mod login;
mod models;

use crate::config::CONFIG;
use crate::login::*;
use api::auth::logout;
use clap::{arg, Command};
use clap::{value_parser, ArgMatches};
use download::{download_album, download_artist, download_track};
use models::{Action, ActionKind};
use std::env;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    // read from config to always trigger initialization, then release immediately lock
    {
        CONFIG.read().await;
    }

    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("get", get_matches)) => get(get_matches).await,
        Some(("login", _)) => login().await,
        Some(("logout", _)) => logout().await.unwrap(),
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
    }
}

fn cli() -> Command<'static> {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand_required(true)
        .subcommand(
            Command::new("get")
                .arg(
                    arg!(<URL>)
                        .multiple_values(true)
                        .required(true)
                        .min_values(1)
                        .value_parser(value_parser!(String))
                        .help("The Tidal URL to download"),
                )
                .arg(
                    arg!(--concurrent <VALUE>)
                        .short('c')
                        .required(false)
                        .help("Number of songs to download concurrently"),
                ),
        )
        .subcommand(Command::new("login"))
        .subcommand(Command::new("logout"))
}

async fn get(matches: &ArgMatches) {
    login().await;
    if let Some(concurrent) = matches.get_one::<String>("concurrent") {
        CONFIG.write().await.concurrency = usize::from_str(concurrent).unwrap();
    }

    if let Some(urls) = matches.get_many::<String>("URL") {
        for url in urls {
            let action = Action::from_str(url).expect("invalid URL supplied");
            let _ = match action.kind {
                ActionKind::Track => download_track(action.id, None).await,
                ActionKind::Album => download_album(action.id).await,
                ActionKind::Artist => download_artist(action.id).await,
            };
        }
    }
}

pub async fn login() {
    let cfg_login = login_config().await;
    if cfg_login.is_ok() {
        return;
    }
    let web_login = login_web().await;
    if web_login.is_ok() {
        return;
    }
    panic!("All Login methods failed")
}
