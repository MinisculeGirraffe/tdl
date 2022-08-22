use crate::api::{models::*, TidalClient, CLIENT};
use crate::config::CONFIG;

use crate::config::DownloadPath;
use crate::models::*;
use anyhow::{anyhow, Error};
use futures::Future;
use indicatif::{MultiProgress, ProgressDrawTarget};
use log::{debug, info};
use metaflac::block::PictureType::CoverFront;
use metaflac::Tag;
use std::cmp::min;
use std::path::Path;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;

pub type ChannelValue = Pin<Box<dyn Future<Output = Result<bool, Error>> + Send>>;
pub type ReceiveChannel = Receiver<ChannelValue>;

pub async fn dispatch_downloads(
    urls: Vec<String>,
    client: TidalClient,
) -> Result<(Vec<JoinHandle<()>>, ReceiveChannel, ReceiveChannel), Error> {
    let config = CONFIG.read().await;
    let progress = setup_multi_progress(config.show_progress, config.progress_refresh_rate);
    let client = Arc::new(client);
    // the maximum amount of items that can be buffered by the rx channel
    // this should be equal to the total number of of work items possible at a single time
    // the actual concurrent requests will be limited by the consumer.
    let buffer_size = config.workers as usize + config.downloads as usize;
    let (dl_tx, dl_rx) = mpsc::channel(buffer_size);
    let (worker_tx, worker_rx) = mpsc::channel(config.workers as usize);

    let task = DownloadTask {
        dl_channel: dl_tx,
        worker_channel: worker_tx,
        client,
        progress,
    };
    debug!("Download Task");
    let mut handles = Vec::with_capacity(urls.len());
    // for every url supplied to the get command
    for url in urls {
        let action = match Action::from_str(&url) {
            Ok(a) => a,
            Err(_) => continue, // skip the current url if it's not valid.
        };
        let id = action.id;
        let task = task.clone();

        //spawn the download task for each URL in a new thread
        let handle = tokio::task::spawn(async move {
            let res = match action.kind {
                ActionKind::Track => {
                    let channel = task.worker_channel.clone();
                    let job = Box::pin(task.download_track(id));
                    match channel.send(job).await {
                        Ok(_) => Ok(true),
                        Err(_) => Err(anyhow!("Error submitting track to worker queue")),
                    }
                }
                ActionKind::Album => task.download_list(ActionKind::Album, id).await,
                ActionKind::Artist => task.download_artist(id).await,
                ActionKind::Playlist => task.download_list(ActionKind::Playlist, id).await,
            };
            match res {
                Ok(_) => {}
                Err(e) => eprint!("{e}"),
            };
        });

        handles.push(handle)
    }

    Ok((handles, dl_rx, worker_rx))
}

#[derive(Clone)]
pub struct DownloadTask {
    pub progress: MultiProgress,
    pub dl_channel: Sender<ChannelValue>,
    pub worker_channel: Sender<ChannelValue>,
    pub client: Arc<TidalClient>,
}

impl DownloadTask {
    async fn download_artist(&self, id: String) -> Result<bool, Error> {
        self.progress.println("Getting Artist Albums")?;
        let albums = self.client.media.get_artist_albums(&id).await?;
        for album in albums {
            self.download_list(ActionKind::Album, album.id.to_string())
                .await?;
        }
        Ok(true)
    }

    async fn download_list(&self, kind: ActionKind, id: String) -> Result<bool, Error> {
        let url = format!("https://api.tidal.com/v1/{kind}s/{id}/items",);
        let tracks = self
            .client
            .media
            .get_items::<ItemResponseItem<Track>>(&url, None, None)
            .await?;
        for track in tracks {
            self.progress
                .println(format!("Getting Track Info for: {}", track.item.get_info()))?;
            let future = Box::pin(self.clone().download_track(track.item.id.to_string()));
            match self.clone().worker_channel.send(future).await {
                Ok(_) => continue,
                Err(_) => return Err(anyhow!("Error Submitting download_track")),
            }
        }
        Ok(true)
    }

    async fn download_track(self, id: String) -> Result<bool, Error> {
        let track = self.client.media.get_track(&id).await?;
        let path_str = self.get_path(&track).await?;
        self.progress.println(format!(
            "Submitting Track to Download Queue: {}",
            track.get_info()
        ))?;
        let download = Box::pin(self.clone().download_file(track, path_str));
        match &self.dl_channel.send(download).await {
            Ok(_) => Ok(true),
            Err(_) => Err(anyhow!("Submitting Download Task failed")),
        }
    }

    async fn download_file(self, track: Track, path: String) -> Result<bool, anyhow::Error> {
        let info = track.get_info();
        let pb = ProgressBar::new(self.progress.clone(), track.id);
        let playback_manifest = self.client.media.get_stream_url(track.id).await?;

        let track_path = format!(
            "{path}{}",
            playback_manifest
                .get_file_extension()
                .expect("Unable to determine track file extension")
        );

        let stream_url = &playback_manifest.urls[0];
        let dl_path = Path::new(&track_path);

        if dl_path.exists() {
            debug!("Path exists");
            self.progress
                .println(format!("File Exists | {}", track.get_info()))?;
            // Exit early if the file already exists
            return Ok(false);
        }

        let response = CLIENT.get(stream_url).send().await?;
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
        self.write_metadata(track, track_path).await?;
        pb.println(format!("Download Complete | {info}"));

        Ok(true)
    }

    async fn write_metadata(&self, track: Track, path: String) -> Result<(), Error> {
        let fp = path.clone();
        debug!("{fp}");
        let mut tag =
            tokio::task::spawn_blocking(move || Tag::read_from_path(Path::new(&fp))).await??;
        tag.set_vorbis("TITLE", vec![track.title]);
        tag.set_vorbis("TRACKNUMBER", vec![track.track_number.to_string()]);
        tag.set_vorbis("ARTIST", vec![track.artist.name]);
        tag.set_vorbis("ALBUM", vec![track.album.title.unwrap_or_default()]);
        tag.set_vorbis("COPYRIGHT", vec![track.copyright]);
        tag.set_vorbis("ISRC", vec![track.isrc]);
        if let Some(cover_id) = &track.album.cover {
            let cover = self.get_cover_data(path.clone(), cover_id).await?;
            tag.add_picture(cover.content_type, CoverFront, cover.data);
        }

        tokio::task::spawn_blocking(move || tag.save()).await??;
        info!("Metadata written to file");
        Ok(())
    }

    pub async fn get_cover_data(&self, path: String, cover_id: &str) -> Result<Cover, Error> {
        let dl_path = Path::new(&path).parent().unwrap().join("cover.jpg");
        if dl_path.exists() {
            let cover = Cover {
                content_type: "application/jpeg".to_string(),
                data: tokio::fs::read(dl_path).await?,
            };
            return Ok(cover);
        }

        let pic = self.client.media.get_cover_data(cover_id).await?;
        tokio::fs::write(dl_path, pic.data.clone()).await?;
        info!("Write cover to disk");
        Ok(pic)
    }

    async fn get_path(&self, track: &Track) -> Result<String, Error> {
        let config = &CONFIG.read().await;
        let dl_path = &config.download_paths;
        let base_path = shellexpand::full(&dl_path.base_path)?.to_string();

        let album = self.client.media.get_album(track.album.id).await?;
        let artist = self
            .client
            .media
            .get_artist(&track.artist.id.to_string())
            .await?;
        let album_path = album.replace_path(&dl_path.album);
        let artist_path = artist.replace_path(&dl_path.artist);
        let track_path = track.clone().replace_path(&dl_path.track);

        Ok(Path::new("")
            .join(base_path)
            .join(artist_path)
            .join(album_path)
            .join(track_path)
            .display()
            .to_string())
    }
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
