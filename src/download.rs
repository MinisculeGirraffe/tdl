use anyhow::Error;
use metaflac::block::PictureType::CoverFront;
use metaflac::{Block, BlockType, Tag};
use regex::{Captures, Regex};
use std::path::Path;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::client::get_cover_data;
use crate::{
    client,
    config::CONFIG,
    decryption::{decrypt_file, decrypt_security_token},
    models::{Artist, PlaybackManifest, Track},
};

async fn remove_encryption(
    stream: PlaybackManifest,
    src: Vec<u8>,
    dst: &Path,
) -> Result<(), Error> {
    tokio::fs::create_dir_all(dst.parent().unwrap()).await?;
    if stream.key_id.is_some() {
        let (key, nonce) = decrypt_security_token(&stream.key_id.unwrap())?;
        let res = decrypt_file(src, key, nonce).await.unwrap();
        File::create(&dst).await?.write(&res).await?;
        return Ok(());
    }
    println!("No encryption key. Writing {} bytes directly", src.len());
    tokio::fs::write(dst, src).await?;
    Ok(())
}

pub async fn download_track(track: Track) -> Result<(), Error> {
    let path_str = get_path(&track).await?;
    let dl_path = Path::new(&path_str);
    if dl_path.exists() {
        println!("File already downloaded");
        return Ok(());
    }
    let stream = client::get_stream_url(track.id).await?;
    println!("Downloading {} - {}", track.artist.name, track.title);
    let dl_url = &stream.urls[0];
    let response = reqwest::Client::new()
        .get(dl_url)
        .send()
        .await?
        .bytes()
        .await?
        .to_vec();
    println!("Downloaded {} MiB", response.len() as f64 / 1.049e6);
    remove_encryption(stream, response, &dl_path).await?;
    get_meta(track,dl_path).await?;
    Ok(())
}

async fn get_path(track: &Track) -> Result<String, Error> {
    let config = &CONFIG.read().await;
    let dl_path = &config.download_path;
    let shell_path = shellexpand::full(&dl_path)?;
    let regex_raw = r"(\{artist\}|\{album\}|\{track_num\}|\{track_name\})";

    let re = Regex::new(&regex_raw).unwrap();
    let track_num_str = &track.track_number.to_string();
    let replaced = re.replace_all(&shell_path, |cap: &Captures| match &cap[0] {
        "{artist}" => &track.artist.name,
        "{album}" => &track.album.title,
        "{track_num}" => &track_num_str,
        "{track_name}" => &track.title,
        _ => panic!("matched no tokens on download_path string"),
    });

    let with_ext = format!("{}.flac", replaced);

    Ok(with_ext)
}

async fn get_meta(track: Track,path: &Path) -> Result<(), Error> {
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
    println!("Metadata written to file");
    Ok(())
}
