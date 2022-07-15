use crate::client::{get_album, get_cover_data, get_items, get_track, ItemResponseItem};
use crate::models::Album;
use crate::{
    client,
    config::CONFIG,
    decryption::{decrypt_file, decrypt_security_token},
    models::{PlaybackManifest, Track},
};
use anyhow::anyhow;
use anyhow::Error;
use futures::stream;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info};
use metaflac::block::PictureType::CoverFront;
use metaflac::Tag;
use regex::{Captures, Regex};
use std::cmp::min;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

async fn _remove_encryption(
    stream: PlaybackManifest,
    src: Vec<u8>,
    dst: &Path,
) -> Result<(), Error> {
    tokio::fs::create_dir_all(dst.parent().unwrap()).await?;
    if stream.key_id.is_some() {
        let (key, nonce) = decrypt_security_token(&stream.key_id.unwrap())?;
        let res = decrypt_file(src, key, nonce).await.unwrap();
        tokio::fs::write(dst, res).await?;
        return Ok(());
    }
    debug!("No encryption key. Writing {} bytes directly", src.len());
    tokio::fs::write(dst, src).await?;
    Ok(())
}

pub async fn download_track(id: usize) -> Result<bool, Error> {
    let pb = ProgressBar::new(0);
    pb.set_style(ProgressStyle::default_bar()
    .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
    .progress_chars("#>-"));

    let config = CONFIG.read().await;
    let track = get_track(id).await?;
    pb.set_message(format!(
        "Downloading {} - {}",
        track.artist.name, track.title
    ));

    let path_str = get_path(&track).await?;

    if config.download_cover {
        let _ = download_cover(&track).await;
    }

    let dl_path = Path::new(&path_str);
    if dl_path.exists() {
        pb.finish_with_message(format!(
            "File Already Exists | {} | Elapsed: {:.2?}",
            &path_str,
            pb.elapsed()
        ));
        return Ok(false);
    }
    let stream = client::get_stream_url(track.id).await?;
    let dl_url = &stream.urls[0];
    let response = reqwest::Client::new().get(dl_url).send().await?;

    let total_size: u64 = response
        .content_length()
        .ok_or(anyhow!("Failed to get content length from {}", dl_url))?
        .try_into()?;

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
    pb.finish_with_message(format!(
        "Download Complete | {} - {} | Elapsed: {:.2?}",
        track.artist.name,
        track.title,
        pb.elapsed()
    ));
    get_meta(track, dl_path).await?;
    Ok(true)
}

pub async fn download_album(id: usize) -> Result<bool, Error> {
    let config = CONFIG.read().await;
    //https://tidal.com/browse/album/86697999
    let album = get_album(id).await.unwrap();
    let url = format!("https://api.tidal.com/v1/albums/{}/items", album.id);
    let tracks = get_items::<ItemResponseItem<Track>>(&url).await?;

    stream::iter(tracks)
        .map(|track| tokio::task::spawn(download_track(track.item.id)))
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
    let regex_raw = r"(\{artist\}|\{artist_id\}|\{album\}|\{album_id\}|\{track_num\}|\{track_name\}|\{quality\})";
    let re = Regex::new(regex_raw).unwrap();
    let track_num_str = &track.track_number.to_string();
    let track_quality = &track.audio_quality.to_string();
    let track_id = &track.id.to_string();
    let artist_id = &track.artist.id.to_string();
    let album_id = &track.album.id.to_string();

    let replaced = re.replace_all(&shell_path, |cap: &Captures| match &cap[0] {
        "{artist}" => sanitize_filename::sanitize(&track.artist.name),
        "{artist_id}" => sanitize_filename::sanitize(artist_id),
        "{album}" => sanitize_filename::sanitize(&track.album.title),
        "{album_id}" => sanitize_filename::sanitize(album_id),
        "{track_num}" => sanitize_filename::sanitize(track_num_str),
        "{track_name}" => sanitize_filename::sanitize(&track.title),
        "{track_id}" => sanitize_filename::sanitize(track_id),
        "{quality}" => sanitize_filename::sanitize(track_quality),
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
