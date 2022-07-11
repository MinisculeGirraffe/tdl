use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(Serialize, Deserialize, Debug)]
pub struct Artist {
    pub id: i64,
    pub name: String,
    #[serde(alias = "type")]
    pub artist_type: String,
    pub picture: String,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Album {
    pub id: i64,
    pub title: String,
    pub duration: Option<i64>,
    pub number_of_tracks: Option<i64>,
    pub number_of_videos: Option<i64>,
    pub number_of_volumes: Option<i64>,
    pub release_date: Option<String>,
    #[serde(alias = "type")]
    pub album_type: Option<String>,
    pub version: Option<String>,
    pub cover: String,
    pub video_cover: Option<String>,
    pub explicit: Option<bool>,
    pub audio_quality: Option<AudioQuality>,
    pub audio_modes: Option<Vec<AudioMode>>,
    pub artist: Option<Artist>,
    pub artists: Option<Vec<Artist>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Track {
    pub id: i64,
    pub title: String,
    pub duration: i64,
    pub track_number: i64,
    pub volume_number: i64,
    pub track_number_on_playlist: Option<i64>,
    pub isrc: String,
    pub explicit: bool,
    pub audio_quality: AudioQuality,
    pub copyright: String,
    pub artist: Artist,
    pub artists: Vec<Artist>,
    pub album: Album,
    pub allow_streaming: bool,
    pub playlist: Option<String>,
    pub mixes: TrackMix,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "UPPERCASE"))]
pub struct TrackMix {
    master_track_mix: Option<String>,
    track_mix: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PlaybackManifest {
    pub mime_type: String,
    pub codecs: String,
    pub encryption_type: EncryptionType,
    pub key_id: Option<String>,
    pub urls: Vec<String>,
}
impl fmt::Display for PlaybackManifest {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let json = serde_json::to_string(&self).unwrap();
        let encode = base64::encode(&json);
        fmt.write_str(&encode)?;
        Ok(())
    }
}

impl FromStr for PlaybackManifest {
    type Err = anyhow::Error;
    fn from_str(input: &str) -> Result<PlaybackManifest, Self::Err> {
        let decode = base64::decode(input)?;
        let json = String::from_utf8(decode)?;
        let parsed: PlaybackManifest = serde_json::from_str(&json)?;
        Ok(parsed)
    }
}

#[derive(SerializeDisplay, DeserializeFromStr, Debug)]
///LOW(96kbps AAC)
///HIGH(320kbps AAC)
///LOSSLESS(1411kbps|16bit/44.1kHz FLAC/ALAC)
///HI_RES(24bit/96kHz MQA encoded FLAC)
pub enum AudioQuality {
    Low,
    High,
    Lossless,
    HiRes,
}
impl fmt::Display for AudioQuality {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            AudioQuality::Low => "LOW",
            AudioQuality::High => "HIGH",
            AudioQuality::Lossless => "LOSSLESS",
            AudioQuality::HiRes => "HI_RES",
        };
        fmt.write_str(str)?;
        Ok(())
    }
}
impl FromStr for AudioQuality {
    type Err = String;
    fn from_str(input: &str) -> Result<AudioQuality, Self::Err> {
        match input {
            "LOW" => Ok(AudioQuality::Low),
            "HIGH" => Ok(AudioQuality::High),
            "LOSSLESS" => Ok(AudioQuality::Lossless),
            "HI_RES" => Ok(AudioQuality::HiRes),
            _ => Err("Error".to_string()),
        }
    }
}

#[derive(SerializeDisplay, DeserializeFromStr, Debug)]
pub enum AudioMode {
    Stereo,
    DolbyAtmos,
}
impl fmt::Display for AudioMode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            AudioMode::Stereo => "STEREO",
            AudioMode::DolbyAtmos => "DOLBY_ATMOS",
        };
        fmt.write_str(str)?;
        Ok(())
    }
}

impl FromStr for AudioMode {
    type Err = String;
    fn from_str(input: &str) -> Result<AudioMode, Self::Err> {
        match input {
            "STEREO" => Ok(AudioMode::Stereo),
            "DOLBY_ATMOS" => Ok(AudioMode::DolbyAtmos),
            _ => Err("Error".to_string()),
        }
    }
}

#[derive(SerializeDisplay, DeserializeFromStr, Debug)]
pub enum PlaybackMode {
    Stream,
    Offline,
}

impl fmt::Display for PlaybackMode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            PlaybackMode::Stream => "STREAM",
            PlaybackMode::Offline => "OFFLINE",
        };
        fmt.write_str(str)?;
        Ok(())
    }
}

impl FromStr for PlaybackMode {
    type Err = String;
    fn from_str(input: &str) -> Result<PlaybackMode, Self::Err> {
        match input {
            "STREAM" => Ok(PlaybackMode::Stream),
            "OFFLINE" => Ok(PlaybackMode::Offline),
            _ => Err("Error".to_string()),
        }
    }
}

#[derive(SerializeDisplay, DeserializeFromStr, Debug)]
pub enum AssetPresentation {
    Full,
    Preview,
}

impl fmt::Display for AssetPresentation {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            AssetPresentation::Full => "FULL",
            AssetPresentation::Preview => "PREVIEW",
        };
        fmt.write_str(str)?;
        Ok(())
    }
}

impl FromStr for AssetPresentation {
    type Err = String;
    fn from_str(input: &str) -> Result<AssetPresentation, Self::Err> {
        match input {
            "FULL" => Ok(AssetPresentation::Full),
            "PREVIEW" => Ok(AssetPresentation::Preview),
            _ => Err("Error".to_string()),
        }
    }
}

#[derive(SerializeDisplay, DeserializeFromStr, Debug)]
pub enum EncryptionType {
    None,
}

impl fmt::Display for EncryptionType {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            EncryptionType::None => "NONE",
        };
        fmt.write_str(str)?;
        Ok(())
    }
}

impl FromStr for EncryptionType {
    type Err = String;
    fn from_str(input: &str) -> Result<EncryptionType, Self::Err> {
        match input {
            "NONE" => Ok(EncryptionType::None),
            _ => Err("Error".to_string()),
        }
    }
}
