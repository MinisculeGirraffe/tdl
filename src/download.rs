use anyhow::Error;
use metaflac::block::PictureType::CoverFront;
use metaflac::Tag;
use regex::{Captures, Regex};
use std::path::Path;

use crate::client::{get_album, get_cover_data, get_items};
use crate::{
    client,
    config::CONFIG,
    decryption::{decrypt_file, decrypt_security_token},
    models::{PlaybackManifest, Track},
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
        tokio::fs::write(dst, res).await?;
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
    println!("Downloaded {:.2} MiB", response.len() as f64 / 1.049e6);
    remove_encryption(stream, response, dl_path).await?;
    get_meta(track, dl_path).await?;
    Ok(())
}

pub async fn download_album(id: i64) -> Result<bool, Error> {
    //https://tidal.com/browse/album/86697999
    let album = get_album(id).await.unwrap();
    let url = format!("https://api.tidal.com/v1/albums/{}/items", album.id);
    let tracks = get_items::<Track>(&url).await.unwrap();
    for track in tracks {
        download_track(track).await?;
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
        "{artist}" => &track.artist.name,
        "{artist_id}" => artist_id,
        "{album}" => &track.album.title,
        "{album_id}" => album_id,
        "{track_num}" => track_num_str,
        "{track_name}" => &track.title,
        "{track_id}" => track_id,
        "{quality}" => track_quality,
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
    println!("Metadata written to file");
    Ok(())
}
