use crate::api::media::{get_album, get_album_items, get_cover_data, get_stream_url, get_track};
use crate::api::models::*;
use crate::api::{get_items, REQ};
use crate::config::CONFIG;
use anyhow::anyhow;
use anyhow::Error;
use futures::future::join_all;
use futures::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use log::{debug, info};
use metaflac::block::PictureType::CoverFront;
use metaflac::Tag;
use regex::{Captures, Regex};
use sanitize_filename::sanitize;
use std::cmp::min;
use std::path::Path;
use std::pin::Pin;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::Sender;

use tokio::try_join;

type DownloadTask =
    Sender<Pin<Box<dyn futures::Future<Output = Result<bool, anyhow::Error>> + Sync + Send>>>;

pub async fn download_track(id: usize, mp: MultiProgress) -> Result<bool, Error> {
    let config = CONFIG.read().await;
    let (pb, track) = try_join!(setup_progress(mp, id), get_track(id))?;
    let track_info = format!(
        "[{}] {} - {}",
        track.track_number, track.artist.name, track.title
    );
    let path_str = get_path(&track).await?;
    let dl_path = Path::new(&path_str);
    if dl_path.exists() {
        pb.println("File Exists");
        // Exit early if the file already exists
        return Ok(false);
    }

    if config.download_cover {
        //spawn a green thread as to not block the current download
        //failure doesn't really matter so result is unchecked
        tokio::task::spawn(download_cover(track.to_owned(), path_str.to_owned()));
    }

    let stream_url = &get_stream_url(track.id).await?.urls[0];
    let response = REQ.get(stream_url).send().await?;
    let total_size: u64 = response
        .content_length()
        .ok_or_else(|| anyhow!("Failed to get content length from {}", stream_url))?;

    pb.set_length(total_size);
    pb.set_style(ProgressStyle::default_bar()
                    .template("{msg}\n{spinner:.green} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, ETA: {eta})")?
                    .progress_chars("#>-"));
    pb.set_message(format!("Downloading File | {}", track_info));

    debug!("Creating File path {}", &dl_path.display());
    tokio::fs::create_dir_all(dl_path.parent().unwrap()).await?;
    let mut file = File::create(dl_path).await?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk).await?;
        downloaded = min(downloaded + (chunk.len() as u64), total_size);
        pb.set_position(downloaded)
    }

    pb.set_message(format!("Writing Metadata | {}", track_info));
    write_metadata(track, dl_path).await?;
    pb.println(format!("Download Complete | {}", track_info));
    Ok(true)
}

pub async fn download_album(id: usize, mp: MultiProgress, tx: DownloadTask) -> Result<bool, Error> {
    let url = format!("https://api.tidal.com/v1/albums/{}/items", id);
    let tracks = get_items::<ItemResponseItem<Track>>(&url, None, None).await?;
    for track in tracks {
        let handle = download_track(track.item.id, mp.clone());
        if (tx.send(Box::pin(handle)).await).is_err() {
            panic!("receiver dropped");
        }
    }
    Ok(true)
}
pub async fn download_artist(
    id: usize,
    mp: MultiProgress,
    tx: DownloadTask,
) -> Result<bool, Error> {
    let albums = get_album_items(id).await?;

    let mut tasks = Vec::new();
    for album in albums {
        tasks.push(download_album(album.id, mp.clone(), tx.clone()));
    }
    // await all the album tasks concurrently
    join_all(tasks).await;
    Ok(true)
}

//Compile the regex once per invocation
lazy_static! {
    pub static ref RE: Regex = Regex::new(r"(\{album\}|\{album_id\}|\{album_release\}|\{album_release_year\}|\{artist\}|\{artist_id\}|\{track_num\}|\{track_name\}|\{quality\})").unwrap();
}

async fn get_path(track: &Track) -> Result<String, Error> {
    let config = &CONFIG.read().await;
    let dl_path = &config.download_path;
    let shell_path = shellexpand::full(&dl_path)?;

    let album = get_album(track.album.id).await?;
    let album_name = album.title.unwrap();
    let track_num_str = &track.track_number.to_string();
    let track_quality = &track.audio_quality.to_string();
    let track_id = &track.id.to_string();
    let artist_id = &track.artist.id.to_string();
    let album_id = &track.album.id.to_string();
    let release = album.release_date.unwrap();
    let ymd: Vec<&str> = release.splitn(3, '-').collect();
    let replaced = RE.replace_all(&shell_path, |cap: &Captures| match &cap[0] {
        "{artist}" => sanitize(&track.artist.name),
        "{artist_id}" => sanitize(artist_id),
        "{album}" => sanitize(&album_name),
        "{album_id}" => sanitize(album_id),
        "{track_num}" => sanitize(track_num_str),
        "{track_name}" => sanitize(&track.title),
        "{track_id}" => sanitize(track_id),
        "{quality}" => sanitize(track_quality),
        "{album_release}" => sanitize(&release),
        "{album_release_year}" => sanitize(ymd[0]),
        _ => panic!("matched no tokens on download_path string"),
    });

    let with_ext = format!("{}.flac", replaced);

    Ok(with_ext)
}

async fn write_metadata(track: Track, path: &Path) -> Result<(), Error> {
    let mut tag = Tag::read_from_path(path)?;
    tag.set_vorbis("TITLE", vec![track.title]);
    tag.set_vorbis("TRACKNUMBER", vec![track.track_number.to_string()]);
    tag.set_vorbis("ARTIST", vec![track.artist.name]);
    tag.set_vorbis("ALBUM", vec![track.album.title.unwrap_or_default()]);
    tag.set_vorbis("COPYRIGHT", vec![track.copyright]);
    tag.set_vorbis("ISRC", vec![track.isrc]);
    if let Some(cover) = &track.album.cover {
        let cover = get_cover_data(cover).await?;
        tag.add_picture(cover.content_type, CoverFront, cover.data);
    }

    tag.save()?;
    info!("Metadata written to file");
    Ok(())
}

pub async fn download_cover(track: Track, folder: String) -> Result<(), Error> {
    let dl_path = Path::new(&folder).parent().unwrap().join("cover.jpg");
    if dl_path.exists() {
        return Ok(());
    }
    let cover = &track
        .album
        .cover
        .as_ref()
        .ok_or_else(|| Error::msg("No Cover available for Album"))?;
    let pic = get_cover_data(cover).await?;
    tokio::fs::write(dl_path, pic.data).await?;
    info!("Write cover to disk");
    Ok(())
}

async fn setup_progress(mp: MultiProgress, id: usize) -> Result<ProgressBar, Error> {
    //Initialize Progress bar
    let pb = mp.add(ProgressBar::new(0));
    pb.set_style(ProgressStyle::default_bar().template("{msg}\n{spinner:.green}")?);
    pb.set_message(format!("Getting Track Details: {}", id));
    Ok(pb)
}
