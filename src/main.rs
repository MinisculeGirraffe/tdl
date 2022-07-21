mod api;
mod cli;
mod config;
mod download;
mod login;
mod models;
use crate::login::*;
use crate::{api::search::search_track, config::CONFIG};
use api::search::search_album;
use api::{auth::logout, search::search_artist};
use clap::ArgMatches;
use cli::cli;
use download::{download_album, download_artist, download_track};
//use env_logger::Env;
use models::{Action, ActionKind};
use std::str::FromStr;
use tabled::TableIteratorExt;

#[tokio::main]
async fn main() {
    // read from config to always trigger initialization, then release immediately lock
    {
        CONFIG.read().await;
    }
    // env_logger::Builder::from_env(Env::default().default_filter_or("none")).init();
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("get", get_matches)) => get(get_matches).await,
        Some(("search", search_matches)) => search(search_matches).await,
        Some(("login", _)) => login().await,
        Some(("logout", _)) => logout().await.unwrap(),
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
    }
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

async fn search(matches: &ArgMatches) {
    if let Some(query) = matches.get_one::<String>("query") {
        let q = query.to_string();
        let max = matches.get_one::<u32>("max").cloned();
        let table = match matches.get_one::<String>("filter") {
            Some(filter) => match filter.as_str() {
                "artist" => search_artist(q, max).await.unwrap().table(),
                "track" => search_track(q, max).await.unwrap().table(),
                "album" => search_album(q, max).await.unwrap().table(),
                _ => unreachable!(),
            },
            None => todo!(), //search all
        };
        println!("{}", table)
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
