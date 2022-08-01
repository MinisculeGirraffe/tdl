mod api;
mod cli;
mod config;
mod download;
mod login;
mod models;

use std::io;

use crate::config::CONFIG;
use crate::login::*;

use api::auth::AuthClient;
use api::models::{Album, Artist, Track};

use clap::ArgMatches;
use clap_complete::{generate, Shell};
use clap_complete_fig::Fig;
use cli::{cli, parse_config_flags};
use download::dispatch_downloads;
use env_logger::Env;
use futures::future::join_all;
use futures::{ StreamExt};

use download::ReceiveChannel;
use tokio::join;
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
        Some(("login", _)) => {
            login().await;
        }
        Some(("logout", _)) => logout().await,
        Some(("autocomplete", matches)) => autocomplete(matches),
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
    }
}

async fn get(matches: &ArgMatches) {
    parse_config_flags(matches).await;
    if let Some(urls) = matches.get_many::<String>("URL") {
        let client = login().await;
        let url: Vec<String> = urls.map(|i| i.to_owned()).collect();
        let (handles, download, worker) = dispatch_downloads(url, client)
            .await
            .expect("Unable to dispatch download thread");
        let config = CONFIG.read().await;
        join!(
            join_all(handles),
            consume_channel(download, config.downloads.into(),),
            consume_channel(worker, config.workers.into())
        );
    }
}

async fn consume_channel(channel: ReceiveChannel, concurrency: usize) {
    //The channel receives an unexecuted future as a stream
    ReceiverStream::new(channel)
        //execute that future in a greenthread
        .map(|i| async { tokio::task::spawn(i).await })
        //up to a maximum concurrent tasks at a single time
        .buffer_unordered(concurrency)
        .for_each(|r| async {
            match r {
                Ok(l) => match l {
                    Ok(_) => {}
                    //if the task failed
                    Err(f) => eprintln!("{f}"),
                },
                // if we failed to launch the task
                Err(e) => eprintln!("{e}"),
            }
        })
        .await;
}

async fn search(matches: &ArgMatches) {
    let client = login().await;
    if let Some(query) = matches.get_one::<String>("query") {
        let max = matches.get_one::<u32>("max").cloned();
        let result = match matches.get_one::<String>("filter") {
            Some(filter) => match filter.as_str() {
                "artist" => {
                    client
                        .search
                        .search_content::<Artist>("artists", query, max)
                        .await
                }
                "track" => {
                    client
                        .search
                        .search_content::<Track>("tracks", query, max)
                        .await
                }
                "album" => {
                    client
                        .search
                        .search_content::<Album>("albums", query, max)
                        .await
                }
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

async fn logout() {
    let config = CONFIG.read().await;

    match config.login_key.access_token.clone() {
        Some(token) => match AuthClient::new(config.api_key.clone())
            .logout(token.to_owned())
            .await
        {
            Ok(_) => println!("Logout Sucessful"),
            Err(e) => eprintln!("Error Logging out: {e}"),
        },
        None => println!("No Auth Token is configured to logout with"),
    }
}

fn autocomplete(matches: &ArgMatches) {
    let mut cmd = cli();
    if let Some(shell) = matches.get_one::<Shell>("shell") {
        generate(
            shell.to_owned(),
            &mut cmd,
            env!("CARGO_PKG_NAME"),
            &mut io::stdout(),
        )
    }

    if matches.contains_id("fig") {
        generate(Fig, &mut cmd, env!("CARGO_PKG_NAME"), &mut io::stdout())
    }
}
