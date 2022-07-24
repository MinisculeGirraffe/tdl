use crate::api::media::{get_album, get_album_items, get_cover_data, get_stream_url, get_track};
use crate::api::models::*;
use crate::api::{get_items, REQ};
use crate::config::CONFIG;
use crate::models::DownloadTask;
use crate::models::*;
use anyhow::{anyhow, Error};
use clap::parser::ValuesRef;
use futures::StreamExt;
use indicatif::{MultiProgress, ProgressDrawTarget};
use lazy_static::lazy_static;
use log::info;
use metaflac::block::PictureType::CoverFront;
use metaflac::Tag;
use regex::{Captures, Regex};
use sanitize_filename::sanitize;
use std::cmp::min;
use std::path::Path;
use std::str::FromStr;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

async fn download_file(track: Track, mp: MultiProgress, path: String) -> Result<bool, Error> {
    let info = track.get_info();
    let pb = ProgressBar::new(mp, track.id);
    let dl_path = Path::new(&path);
    let stream_url = &get_stream_url(track.id).await?.urls[0];
    let response = REQ.get(stream_url).send().await?;
    let total_size: u64 = response
        .content_length()
        .ok_or_else(|| anyhow!("Failed to get content length from {}", stream_url))?;
    pb.start_download(total_size, &track);

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

    write_metadata(track, dl_path).await?;
    pb.println(format!("Download Complete | {}", info));
    Ok(true)
}

async fn download_track(id: usize, task: DownloadTask) -> Result<bool, Error> {
    let config = CONFIG.read().await;

    let track = get_track(id).await?;
    let path_str = get_path(&track).await?;
    if config.download_cover {
        //spawn a green thread as to not block the current download
        //failure doesn't really matter so result is unchecked
        tokio::task::spawn(download_cover(track.to_owned(), path_str.to_owned()));
    }
    let dl_path = Path::new(&path_str);
    if dl_path.exists() {
        task.progress
            .println(format!("File Exists | {}", track.get_info()))?;
        // Exit early if the file already exists
        return Ok(false);
    }

    let download = download_file(track, task.progress, path_str);
    match task.channel.send(Box::pin(download)).await {
        Ok(_) => Ok(true),
        Err(_) => Err(anyhow!("Submitting Download Task failed")),
    }
}

async fn download_album(id: usize, task: DownloadTask) -> Result<bool, Error> {
    let url = format!("https://api.tidal.com/v1/albums/{}/items", id);
    let tracks = get_items::<ItemResponseItem<Track>>(&url, None, None).await?;
    for track in tracks {
        download_track(track.item.id, task.clone()).await?;
    }
    Ok(true)
}
async fn download_artist(id: usize, task: DownloadTask) -> Result<bool, Error> {
    let albums = get_album_items(id).await?;
    for album in albums {
        download_album(album.id, task.clone()).await?;
    }
    Ok(true)
}

pub async fn dispatch_downloads(urls: ValuesRef<'_, String>) -> ReceiveChannel {
    let config = CONFIG.read().await;
    let progress = setup_multi_progress(config.show_progress, config.progress_refresh_rate);

    // the maximum amount of items that can be buffered by the rx channel
    // we want this larger than our total download concurrency
    // that way when a track finishes, the next buffered task is already ready to start the DL
    let max_buffered = config.concurrency as usize * 2;
    let (tx, rx) = mpsc::channel(max_buffered);

    // for every url supplied to the get command
    for url in urls {
        let action = match Action::from_str(url) {
            Ok(a) => a,
            Err(_) => continue, // skip the current url if it's not valid.
        };
        let id = action.id;
        let task = DownloadTask {
            channel: tx.clone(),
            progress: progress.clone(),
        };
        //spawn the download task for each URL in a new thread
        tokio::task::spawn(async move {
            let res = match action.kind {
                ActionKind::Track => download_track(id, task).await,
                ActionKind::Album => download_album(id, task).await,
                ActionKind::Artist => download_artist(id, task).await,
            };
            match res {
                Ok(_) => {}
                Err(e) => eprint!("{e}"),
            };
        });
    }
    rx
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

fn setup_multi_progress(show_progress: bool, refresh_rate: u8) -> MultiProgress {
    let mp = MultiProgress::new();
    let draw_target = match show_progress {
        true => ProgressDrawTarget::stdout_with_hz(refresh_rate),
        false => ProgressDrawTarget::hidden(),
    };
    mp.set_draw_target(draw_target);
    mp
}
