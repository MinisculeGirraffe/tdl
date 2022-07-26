use anyhow::Error;
use futures::Future;
use indicatif::{MultiProgress, ProgressStyle};
use tokio::sync::mpsc::{Receiver, Sender};

use std::{fmt, pin::Pin, str::FromStr};

use crate::api::models::Track;

pub type ChannelValue = Pin<Box<dyn Future<Output = Result<bool, Error>> + Sync + Send>>;
pub type ReceiveChannel = Receiver<ChannelValue>;

#[derive(Clone)]
pub struct DownloadTask {
    pub progress: MultiProgress,
    pub channel: Sender<ChannelValue>,
}

pub struct ProgressBar(indicatif::ProgressBar);

impl ProgressBar {
    pub fn new(parent: MultiProgress, id: usize) -> Self {
        let pb = parent.add(indicatif::ProgressBar::new(0));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg}\n{spinner:.green}")
                .unwrap(),
        );
        pb.set_message(format!("Getting Track Details: {}", id));

        Self(pb)
    }

    pub fn start_download(&self, length: u64, track: &Track) {
        self.0.set_length(length);
        self.0.set_style(ProgressStyle::default_bar()
                        .template("{msg}\n{spinner:.green} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, ETA: {eta})").unwrap()
                        .progress_chars("#>-"));
        self.0
            .set_message(format!("Downloading File | {}", track.get_info()));
    }
    pub fn set_message (&self,message: String) {
        self.0.set_message(message)
    }
    pub fn println(&self, s: impl ToString) {
        let _ = &self.0.println(s.to_string());
    }
    pub fn set_position(&self, i: u64) {
        let _ = &self.0.set_position(i);
    }
}

#[derive(Debug)]
pub struct Action {
    pub kind: ActionKind,
    pub id: usize,
}
impl FromStr for Action {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url_parts: Vec<&str> = s.split('/').collect();
        let [kind, id]: [_; 2] = url_parts[url_parts.len() - 2..].try_into()?;
        Ok(Self {
            kind: ActionKind::from_str(kind)?,
            id: usize::from_str(id)?,
        })
    }
}
#[derive(Debug)]
pub enum ActionKind {
    Track,
    Album,
    Artist,
}
impl FromStr for ActionKind {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "track" => Ok(ActionKind::Track),
            "album" => Ok(ActionKind::Album),
            "artist" => Ok(ActionKind::Artist),
            _ => Err(Error::msg("No action kind for type")),
        }
    }
}

impl fmt::Display for ActionKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            ActionKind::Track => "track",
            ActionKind::Album => "album",
            ActionKind::Artist => "artist",
        };
        fmt.write_str(str)?;
        Ok(())
    }
}
