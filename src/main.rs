mod api;
mod cli;
mod config;
mod download;
mod login;
mod models;
use crate::config::CONFIG;
use crate::login::*;
use api::auth::logout;
use api::models::{Album, Artist, Track};
use api::search::search_content;
use clap::ArgMatches;
use cli::cli;
use download::{download_album, download_artist, download_track};
use env_logger::Env;
use futures::StreamExt;
use indicatif::{MultiProgress, ProgressDrawTarget};
use models::{Action, ActionKind};
use std::str::FromStr;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

#[tokio::main]
async fn main() {
    // read from config to always trigger initialization, then release immediately lock
    {
        CONFIG.read().await;
    }
    env_logger::Builder::from_env(Env::default().default_filter_or("none")).init();
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
    if let Some(concurrent) = matches.get_one::<u8>("concurrent") {
        CONFIG.write().await.concurrency = *concurrent;
    }

    if let Some(urls) = matches.get_many::<String>("URL") {
        let (tx, rx) = mpsc::channel(100);
        let progress = setup_progress().await;

        for url in urls {
            let action = Action::from_str(url).expect("invalid URL supplied");
            let bar = progress.clone();
            let tx = tx.clone();
            let _ = tokio::task::spawn(async move {
                let _ = match action.kind {
                    ActionKind::Track => download_track(action.id, bar).await,
                    ActionKind::Album => download_album(action.id, bar, tx).await,
                    ActionKind::Artist => download_artist(action.id, bar, tx).await,
                };
            });
        }
        //drop the tx channel spawned in this thread to prevent indefinite blocking
        drop(tx);

        let stream = ReceiverStream::new(rx);
        stream
            .map(|i| async { i.await })
            .buffer_unordered(3)
            .for_each(|r| async {
                match r {
                    Ok(_) => {}
                    Err(e) => eprintln!("{e}"),
                }
            })
            .await;
    }
}

async fn search(matches: &ArgMatches) {
    if let Some(query) = matches.get_one::<String>("query") {
        let max = matches.get_one::<u32>("max").cloned();
        let result = match matches.get_one::<String>("filter") {
            Some(filter) => match filter.as_str() {
                "artist" => search_content::<Artist>("artists", query, max).await,
                "track" => search_content::<Track>("tracks", query, max).await,
                "album" => search_content::<Album>("albums", query, max).await,
                _ => unreachable!(),
            },
            None => todo!(), //search all
        };
        match result {
            Ok(t) => println!("{t}"),
            Err(e) => eprintln!("{e}"),
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

async fn setup_progress() -> MultiProgress {
    let config = CONFIG.read().await;
    let mp = MultiProgress::new();
    mp.set_draw_target(get_draw_target(
        config.show_progress,
        config.progress_refresh_rate,
    ));
    mp
}

fn get_draw_target(show_progress: bool, refresh_rate: u8) -> ProgressDrawTarget {
    match show_progress {
        true => ProgressDrawTarget::stdout_with_hz(refresh_rate),
        false => ProgressDrawTarget::hidden(),
    }
}
