use anyhow::Error;

use indicatif::{MultiProgress, ProgressStyle};

use crate::api::models::Track;
use std::ops::Deref;
use std::{fmt, str::FromStr};

pub struct ProgressBar(indicatif::ProgressBar);

impl ProgressBar {
    pub fn new(parent: MultiProgress, id: usize) -> Self {
        let pb = parent.add(indicatif::ProgressBar::new(0));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg}\n{spinner:.green}")
                .expect("Progress Bar Template is Invalid"),
        );
        pb.set_message(format!("Getting Track Details: {}", id));

        Self(pb)
    }

    pub fn start_download(&self, length: u64, track: &Track) {
        self.set_length(length);
        self.set_style(ProgressStyle::default_bar()
                        .template("{wide_msg}\n{spinner:.green} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec:4}, ETA: {eta:2})").expect("Progress Bar Template is invalid")
                        .progress_chars("#>-"));
        self.set_message(format!("Downloading File | {}", track.get_info()));
    }
}

impl Deref for ProgressBar {
    type Target = indicatif::ProgressBar;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct Action {
    pub kind: ActionKind,
    pub id: String,
}
impl FromStr for Action {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url_parts: Vec<&str> = s.split('/').collect();
        let [kind, id]: [_; 2] = url_parts[url_parts.len() - 2..].try_into()?;
        Ok(Self {
            kind: ActionKind::from_str(kind)?,
            id: id.into(),
        })
    }
}
#[derive(Debug)]
pub enum ActionKind {
    Track,
    Album,
    Artist,
    Playlist,
}
impl FromStr for ActionKind {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "track" => Ok(ActionKind::Track),
            "album" => Ok(ActionKind::Album),
            "artist" => Ok(ActionKind::Artist),
            "playlist" => Ok(ActionKind::Playlist),
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
            ActionKind::Playlist => "playlist",
        };
        fmt.write_str(str)?;
        Ok(())
    }
}
