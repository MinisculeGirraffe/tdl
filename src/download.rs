use crate::client::{get_album, get_cover_data, get_items, get_track, ItemResponseItem};
use crate::models::Album;
use crate::{client, config::CONFIG, models::Track};
use anyhow::anyhow;
use anyhow::Error;
use futures::stream;
use futures::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
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
    let pb: ProgressBar;

    let mut multi_progress = false;

    if let Some(mpb) = mp.as_ref() {
        pb = mpb.add(ProgressBar::new(0));
        multi_progress = true;
    } else {
        pb = ProgressBar::new(0);
    }

    pb.set_style(ProgressStyle::default_bar()
    .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, ETA: {eta})")?
    .progress_chars("#>-"));

    let config = CONFIG.read().await;
    let track = get_track(id).await?;
    pb.set_message(format!(
        "Downloading File | [{}] {} - {}",
        track.track_number, track.artist.name, track.title
    ));

    let path_str = get_path(&track).await?;

    if config.download_cover {
        let _ = download_cover(&track).await;
    }

    let dl_path = Path::new(&path_str);
    if dl_path.exists() {
        return Ok(false);
    }
    let stream = client::get_stream_url(track.id).await?;
    let dl_url = &stream.urls[0];
    let response = reqwest::Client::new().get(dl_url).send().await?;

    let total_size: u64 = response
        .content_length()
        .ok_or_else(|| anyhow!("Failed to get content length from {}", dl_url))?;

    pb.set_length(total_size);

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

    let finish_text = format!(
        "Download Complete | [{}] {} - {} | Elapsed: {:.2?}",
        track.track_number,
        track.artist.name,
        track.title,
        pb.elapsed()
    );
    if multi_progress {
        mp.unwrap().println(finish_text)?;
    } else {
        pb.finish_with_message(finish_text);
    }

    get_meta(track, dl_path).await?;
    Ok(true)
}

pub async fn download_album(id: usize) -> Result<bool, Error> {
    let config = CONFIG.read().await;
    //https://tidal.com/browse/album/86697999
    let album = get_album(id).await.unwrap();
    let url = format!("https://api.tidal.com/v1/albums/{}/items", album.id);
    let tracks = get_items::<ItemResponseItem<Track>>(&url).await?;
    let mp = MultiProgress::new();
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
    let url = format!("https://api.tidal.com/v1/artists/{}/albums", id);
    let albums = get_items::<Album>(&url).await?;
    debug!("Got Albums successfully");
    for album in albums {
        download_album(album.id).await?;
    }
    Ok(true)
}

async fn get_path(track: &Track) -> Result<String, Error> {
    let config = &CONFIG.read().await;
    let dl_path = &config.download_path;
    let shell_path = shellexpand::full(&dl_path)?;
    let album_re = r"\{album\}|\{album_id\}|\{album_release\}|\{album_release_year\}";
    let artist_re = r"\{artist\}|\{artist_id\}";
    let track_re = r"\{track_num\}|\{track_name\}|\{quality\}";
    let master_re = format!("({}|{}|{})", artist_re, album_re, track_re);
    let re = Regex::new(&master_re).unwrap();

    let album = get_album(track.album.id).await?;
    let track_num_str = &track.track_number.to_string();
    let track_quality = &track.audio_quality.to_string();
    let track_id = &track.id.to_string();
    let artist_id = &track.artist.id.to_string();
    let album_id = &track.album.id.to_string();
    let release = album.release_date.unwrap();
    let ymd: Vec<&str> = release.splitn(3, '-').collect();
    let replaced = re.replace_all(&shell_path, |cap: &Captures| match &cap[0] {
        "{artist}" => sanitize(&track.artist.name),
        "{artist_id}" => sanitize(artist_id),
        "{album}" => sanitize(&track.album.title),
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
    tag.set_vorbis("ALBUM", vec![track.album.title]);
    tag.set_vorbis("COPYRIGHT", vec![track.copyright]);
    tag.set_vorbis("ISRC", vec![track.isrc]);
    let cover = get_cover_data(&track.album.cover).await?;
    tag.add_picture(cover.content_type, CoverFront, cover.data);
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
    let cover = get_cover_data(&track.album.cover).await?;
    tokio::fs::write(dl_path, cover.data).await?;
    info!("Write cover to disk");
    Ok(())
}
