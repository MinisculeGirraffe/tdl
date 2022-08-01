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
use log::{debug, info};
use metaflac::block::PictureType::CoverFront;
use metaflac::Tag;
use regex::{Captures, Regex};
use sanitize_filename::sanitize;
use std::cmp::min;
use std::path::Path;
use std::str::FromStr;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{self};

async fn download_file(
    track: Track,
    mp: MultiProgress,
    path: String,
) -> Result<bool, anyhow::Error> {
    let info = track.get_info();
    let pb = ProgressBar::new(mp, track.id);
    let playback_manifest = get_stream_url(track.id).await?;

    let track_path = format!(
        "{path}{}",
        playback_manifest
            .get_file_extension()
            .expect("Unable to determine track file extension")
    );
    let stream_url = &playback_manifest.urls[0];
    let dl_path = Path::new(&track_path);
    let response = REQ.get(stream_url).send().await?;
    let total_size: u64 = response
        .content_length()
        .ok_or_else(|| anyhow!("Failed to get content length from {}", stream_url))?;
    pb.start_download(total_size, &track);
    debug!("Got Content Length: {total_size} for {}", track.get_info());
    tokio::fs::create_dir_all(dl_path.parent().unwrap()).await?;
    let file = File::create(dl_path).await?;
    // 1 MiB Write buffer to minimize syscalls for slow i/o
    // Reduces write CPU time from 24% to 7%.
    let mut writer = tokio::io::BufWriter::with_capacity(1024 * 1000 * 1000, file);
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    while let Some(item) = stream.next().await {
        let chunk = item?;
        downloaded = min(downloaded + (chunk.len() as u64), total_size);
        pb.set_position(downloaded);
        writer.write_all(&chunk).await?;
    }

    //flush buffer to disk;
    pb.set_message(format!("Writing to Disk | {info}"));
    writer.flush().await?;

    pb.set_message(format!("Writing metadata | {info}"));
    write_metadata(track, path).await.ok();
    pb.println(format!("Download Complete | {info}"));

    Ok(true)
}

async fn download_track(id: String, task: DownloadTask) -> Result<bool, Error> {
    let config = CONFIG.read().await;
    let track = get_track(&id).await?;
    let path_str = get_path(&track).await?;
    if config.download_cover {
        //spawn a green thread as to not block the current download
        //failure doesn't really matter so result is unchecked
        tokio::task::spawn(download_cover(track.to_owned(), path_str.to_owned()))
            .await
            .ok();
    }
    let dl_path = Path::new(&path_str);
    if dl_path.exists() {
        task.progress
            .println(format!("File Exists | {}", track.get_info()))?;
        // Exit early if the file already exists
        return Ok(false);
    }
    task.progress.println(format!(
        "Submitting Track to Download Queue: {}",
        track.get_info()
    ))?;
    let download = download_file(track, task.progress, path_str);
    match task.dl_channel.send(Box::pin(download)).await {
        Ok(_) => Ok(true),
        Err(_) => Err(anyhow!("Submitting Download Task failed")),
    }
}

async fn download_list(kind: ActionKind, id: String, task: DownloadTask) -> Result<bool, Error> {
    let url = format!("https://api.tidal.com/v1/{kind}s/{id}/items",);
    let tracks = get_items::<ItemResponseItem<Track>>(&url, None, None).await?;
    for track in tracks {
        task.progress
            .println(format!("Getting Track Info for: {}", track.item.get_info()))?;
        let future = Box::pin(download_track(track.item.id.to_string(), task.clone()));
        match task.worker_channel.send(future).await {
            Ok(_) => continue,
            Err(_) => return Err(anyhow!("Error Submitting download_track")),
        }
    }
    Ok(true)
}
async fn download_artist(id: String, task: DownloadTask) -> Result<bool, Error> {
    task.progress.println("Getting Artist Albums")?;
    let albums = get_album_items(&id).await?;
    for album in albums {
        task.progress.println(format!(
            "Getting Tracks for Album: {}",
            album.title.unwrap_or_else(|| "".into())
        ))?;
        download_list(ActionKind::Album, album.id.to_string(), task.clone()).await?;
    }
    Ok(true)
}

pub async fn dispatch_downloads(
    urls: ValuesRef<'_, String>,
) -> Result<(ReceiveChannel, ReceiveChannel), Error> {
    let config = CONFIG.read().await;
    let progress = setup_multi_progress(config.show_progress, config.progress_refresh_rate);

    // the maximum amount of items that can be buffered by the rx channel
    // this should be equal to the total number of of work items possible at a single time
    // the actual concurrent requests will be limited by the consumer.
    let buffer_size = config.workers as usize + config.downloads as usize;
    let (dl_tx, dl_rx) = mpsc::channel(buffer_size);
    let (worker_tx, worker_rx) = mpsc::channel(config.workers as usize);

    // for every url supplied to the get command
    for url in urls {
        let action = match Action::from_str(url) {
            Ok(a) => a,
            Err(_) => continue, // skip the current url if it's not valid.
        };
        let id = action.id;
        let task = DownloadTask {
            dl_channel: dl_tx.clone(),
            worker_channel: worker_tx.clone(),
            progress: progress.clone(),
        };

        //spawn the download task for each URL in a new thread
        tokio::task::spawn(async move {
            let res = match action.kind {
                ActionKind::Track => {
                    let channel = task.worker_channel.clone();
                    let job = Box::pin(download_track(id, task));
                    match channel.send(job).await {
                        Ok(_) => Ok(true),
                        Err(_) => Err(anyhow!("Error submitting track to worker queue")),
                    }
                }
                ActionKind::Album => download_list(ActionKind::Album, id, task).await,
                ActionKind::Artist => download_artist(id, task).await,
                ActionKind::Playlist => download_list(ActionKind::Playlist, id, task).await,
            };
            match res {
                Ok(_) => {}
                Err(e) => eprint!("{e}"),
            };
        });
    }

    Ok((dl_rx, worker_rx))
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

    Ok(replaced.to_string())
}

async fn write_metadata(track: Track, path: String) -> Result<(), Error> {
    let mut tag =
        tokio::task::spawn_blocking(move || Tag::read_from_path(Path::new(&path))).await??;
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

    tokio::task::spawn_blocking(move || tag.save()).await??;
    info!("Metadata written to file");
    Ok(())
}

pub async fn download_cover(track: Track, folder: String) -> Result<bool, Error> {
    let dl_path = Path::new(&folder).parent().unwrap().join("cover.jpg");
    if dl_path.exists() {
        return Ok(false);
    }
    let cover = &track
        .album
        .cover
        .as_ref()
        .ok_or_else(|| Error::msg("No Cover available for Album"))?;
    let pic = get_cover_data(cover).await?;
    tokio::fs::write(dl_path, pic.data).await?;
    info!("Write cover to disk");
    Ok(true)
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
