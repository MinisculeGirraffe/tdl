use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{fmt, str::FromStr};
use tabled::Tabled;

#[derive(Serialize, Deserialize, Debug)]
pub struct RefreshResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct UserResponse {
    pub user_id: i64,
    pub country_code: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct DeviceAuthResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceAuthRequest {
    pub client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_code: Option<String>,
}

impl Default for DeviceAuthRequest {
    fn default() -> Self {
        DeviceAuthRequest {
            client_id: "".to_string(),
            client_secret: None,
            refresh_token: None,
            scope: None,
            grant_type: None,
            device_code: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ItemResponse<T> {
    pub limit: usize,
    pub offset: usize,
    pub total_number_of_items: usize,
    pub items: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemResponseItem<T> {
    pub item: T,
    #[serde(alias = "type")]
    pub item_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PlaybackInfoPostPaywallRes {
    pub track_id: usize,
    pub asset_presentation: AssetPresentation,
    pub audio_quality: AudioQuality,
    pub manifest_mime_type: String,
    pub manifest: String,
}

pub struct Cover {
    pub content_type: String,
    pub data: Vec<u8>,
}

trait Named {
    fn get_name(&self) -> &str;
}

#[derive(Serialize, Deserialize, Debug, Clone, Tabled)]
pub struct Artist {
    pub id: usize,
    pub name: String,
    #[serde(alias = "type")]
    #[tabled(skip)]
    pub artist_type: Option<String>,
    #[tabled(skip)]
    pub artist_types: Option<Vec<String>>, //todo change to enum
    #[tabled(skip)]
    pub picture: Option<String>,
    #[tabled(display_with = "display_option")]
    pub popularity: Option<i32>,
    #[tabled(skip)]
    pub artist_roles: Option<Vec<ArtistRole>>,
}
impl Named for Artist {
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Tabled)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Album {
    pub id: usize,
    #[tabled(display_with = "display_option")]
    pub title: Option<String>,
    #[tabled(skip)]
    pub duration: Option<i64>,
    #[tabled(skip)]
    pub number_of_tracks: Option<i64>,
    #[tabled(skip)]
    pub number_of_videos: Option<i64>,
    #[tabled(skip)]
    pub number_of_volumes: Option<i64>,
    #[tabled(display_with = "display_option")]
    pub release_date: Option<String>,
    #[serde(alias = "type")]
    #[tabled(skip)]
    pub album_type: Option<String>,
    #[tabled(skip)]
    pub version: Option<String>,
    #[tabled(skip)]
    pub cover: Option<String>,
    #[tabled(skip)]
    pub video_cover: Option<String>,
    #[tabled(display_with = "display_option")]
    pub explicit: Option<bool>,
    #[tabled(display_with = "display_option")]
    pub audio_quality: Option<AudioQuality>,
    #[tabled(skip)]
    pub audio_modes: Option<Vec<AudioMode>>,
    #[tabled(display_with = "display_option_named")]
    pub artist: Option<Artist>,
    #[tabled(skip)]
    pub artists: Option<Vec<Artist>>,
}

impl Named for Album {
    fn get_name(&self) -> &str {
        match &self.title {
            Some(t) => t.as_str(),
            None => "",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Tabled)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Track {
    pub id: usize,
    pub title: String,
    pub duration: usize,
    #[tabled(skip)]
    pub track_number: usize,
    #[tabled(skip)]
    pub volume_number: usize,
    #[tabled(skip)]
    pub track_number_on_playlist: Option<usize>,
    pub isrc: String,
    pub explicit: bool,
    pub audio_quality: AudioQuality,
    #[tabled(skip)]
    pub copyright: String,
    #[tabled(display_with = "display_name")]
    pub artist: Artist,
    #[tabled(skip)]
    pub artists: Vec<Artist>,
    #[tabled(display_with = "display_name")]
    pub album: Album,
    #[tabled(skip)]
    pub allow_streaming: bool,
    #[tabled(skip)]
    pub playlist: Option<String>,
    #[tabled(skip)]
    pub mixes: TrackMix,
}

impl Track {
    pub fn get_info(&self) -> String {
        format!(
            "[{}] {} - {}",
            self.track_number, self.artist.name, self.title
        )
    }
}
impl Named for Track {
    fn get_name(&self) -> &str {
        &self.title
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all(deserialize = "UPPERCASE"))]
pub struct TrackMix {
    master_track_mix: Option<String>,
    track_mix: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Playlist {
    uuid: String,
    title: String,
    number_of_tracks: usize,
    number_of_videos: usize,
    creator: PlaylistCreator,
    description: String,
    duration: usize,
    promoted_artists: Vec<Artist>,
}
impl Named for Playlist {
    fn get_name(&self) -> &str {
        &self.title
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlaylistCreator {
    id: usize,
    name: String,
    #[serde(alias = "type")]
    creator_type: Option<String>,
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

impl PlaybackManifest {
    pub fn get_file_extension(&self) -> Option<&str> {
        match self.mime_type.as_str() {
            "audio/mp4" => Some(".m4a"),
            "audio/flac" => Some(".flac"),
            _ => None,
        }
    }
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

#[derive(SerializeDisplay, DeserializeFromStr, Clone, Debug, Copy)]
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

impl clap::ValueEnum for AudioQuality {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Low, Self::High, Self::Lossless, Self::HiRes]
    }

    fn to_possible_value<'a>(&self) -> Option<clap::PossibleValue<'a>> {
        match self {
            Self::HiRes => Some(clap::PossibleValue::new("max")),
            Self::Lossless => Some(clap::PossibleValue::new("lossless")),
            Self::High => Some(clap::PossibleValue::new("high")),
            Self::Low => Some(clap::PossibleValue::new("low")),
        }
    }
}

#[derive(SerializeDisplay, DeserializeFromStr, Clone, Debug)]
pub enum AudioMode {
    Stereo,
    DolbyAtmos,
    Sony360RA,
}
impl fmt::Display for AudioMode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            AudioMode::Stereo => "STEREO",
            AudioMode::DolbyAtmos => "DOLBY_ATMOS",
            AudioMode::Sony360RA => "SONY_360RA",
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
            "SONY_360RA" => Ok(AudioMode::Sony360RA),
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
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub enum ArtistType {
    Artist,
    Contributor,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ArtistRole {
    pub category_id: i32,
    pub category: ArtistRoleCategory,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub enum ArtistRoleCategory {
    Artist,
    Performer,
    Producer,
    Songwriter,
    Engineer,
    Misc,
}

fn display_option(o: &Option<impl ToString>) -> String {
    match o {
        Some(val) => val.to_string(),
        None => String::new(),
    }
}

fn display_option_named(o: &Option<impl Named>) -> String {
    match o {
        Some(n) => n.get_name().to_string(),
        None => String::new(),
    }
}

fn display_name(n: &impl Named) -> String {
    n.get_name().to_string()
}
