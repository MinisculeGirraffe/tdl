extern crate crypto;

mod client;
mod config;
mod decryption;
mod download;
mod login;
mod models;
use download::download_track;

use crate::client::{get_album, get_items, get_stream_url};
use crate::login::*;
use crate::models::Track;

#[tokio::main]
async fn main() {
    match login().await {
        Ok(res) => println!("Logged in: {}", res),
        Err(e) => eprintln!("{}", e.to_string()),
    }
    //https://tidal.com/browse/album/86697999
    let album = get_album(86697999).await.unwrap();
    let url = format!("https://api.tidal.com/v1/albums/{}/items", album.id);
    let tracks = get_items::<Track>(&url).await.unwrap();
    for track in tracks {
        download_track(track).await;
    }
   // let resp = get_stream_url(38519997).await.unwrap();
   // println!("{:?}", resp);
}
