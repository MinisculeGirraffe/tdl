use crate::api::get_items;
use crate::api::media::{get_album, get_cover_data, get_stream_url, get_track};
use crate::api::models::*;
use crate::config::CONFIG;
use anyhow::anyhow;
use anyhow::Error;
use futures::stream;
use futures::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use lazy_static::lazy_static;
use log::{debug, info};
use metaflac::block::PictureType::CoverFront;
use metaflac::Tag;
use regex::{Captures, Regex};
use sanitize_filename::sanitize;
use std::cmp::min;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub async fn download_track(id: usize, mp: Option<MultiProgress>) -> Result<bool, Error> {
    let config = CONFIG.read().await;

    //Initialize Progress bar
    let pb: ProgressBar;
    if config.show_progress {
        if let Some(mpb) = mp.as_ref() {
            pb = mpb.add(ProgressBar::new(0));
        } else {
            pb = ProgressBar::new(0);
            pb.set_draw_target(ProgressDrawTarget::stdout_with_hz(
                config.progress_refresh_rate,
            ))
        }
    } else {
        pb = ProgressBar::new(0);
        pb.set_draw_target(ProgressDrawTarget::hidden())
    }
    pb.set_style(ProgressStyle::default_bar().template("{msg}\n{spinner:.green}")?);
    pb.set_message(format!("Getting Track Details | {}", id));

    let track = get_track(id).await?;

    let track_info = format!(
        "[{}] {} - {}",
        track.track_number, track.artist.name, track.title
    );
    let path_str = get_path(&track).await?;
    if config.download_cover {
        let _ = download_cover(&track).await;
    }
    let dl_path = Path::new(&path_str);
    if dl_path.exists() {
        pb.println(format!("File Exists {}", track_info));
        // Exit early if the file already exists
        return Ok(false);
    }

    let stream = get_stream_url(track.id).await?;
    let dl_url = &stream.urls[0];
    let response = reqwest::get(dl_url).await?;
    let total_size: u64 = response
        .content_length()
        .ok_or_else(|| anyhow!("Failed to get content length from {}", dl_url))?;

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
    get_meta(track, dl_path).await?;
    pb.println(format!("Download Complete | {}", track_info));
    Ok(true)
}

pub async fn download_album(id: usize) -> Result<bool, Error> {
    let config = CONFIG.read().await;
    //https://tidal.com/browse/album/86697999
    let album = get_album(id).await.unwrap();
    let url = format!("https://api.tidal.com/v1/albums/{}/items", album.id);
    let tracks = get_items::<ItemResponseItem<Track>>(&url, None, None).await?;
    let mp = MultiProgress::new();
    mp.set_draw_target(ProgressDrawTarget::stdout_with_hz(
        config.progress_refresh_rate,
    ));
    stream::iter(tracks)
        .map(|track| tokio::task::spawn(download_track(track.item.id, Some(mp.clone()))))
        .buffer_unordered(config.concurrency)
        .for_each(|r| async {
            match r {
                Ok(_) => {}
                Err(_e) => panic!("download failed"),
            }
        })
        .await;

    Ok(true)
}
pub async fn download_artist(id: usize) -> Result<bool, Error> {
    let config = CONFIG.read().await;
    let url = format!("https://api.tidal.com/v1/artists/{}/albums", id);
    let mut albums = get_items::<Album>(&url, None, None).await?;

    if config.include_singles {
        let filter = vec![("filter".to_string(), "EPSANDSINGLES".to_string())];
        let mut singles = get_items::<Album>(&url, Some(filter), None).await?;
        albums.append(&mut singles);
    }

    debug!("Got Albums successfully");
    for album in albums {
        download_album(album.id).await?;
    }
    Ok(true)
}

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

async fn get_meta(track: Track, path: &Path) -> Result<(), Error> {
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

pub async fn download_cover(track: &Track) -> Result<(), Error> {
    let path_str = get_path(track).await?;
    let dl_path = Path::new(&path_str).parent().unwrap().join("cover.jpg");
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
