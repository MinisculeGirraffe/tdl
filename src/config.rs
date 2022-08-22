use crate::api::models::Album;
use crate::api::models::Artist;
use crate::api::models::AudioQuality;
use crate::api::models::Track;
use anyhow::Error;
use config::{Config, File, FileFormat};
use phf::phf_map;
use sanitize_filename::sanitize;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::NoneAsEmptyString;
use std::env::var;
use std::io::Write;
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub download_path: FilePath,
    pub audio_quality: AudioQuality,
    pub show_progress: bool,
    pub progress_refresh_rate: u8,
    pub include_singles: bool,
    pub downloads: u8,
    pub workers: u8,
    pub download_cover: bool,
    pub cache_dir: String,
    pub db_path: String,
    pub login_key: LoginKey,
    pub api_key: ApiKey,
}

impl Settings {
    pub fn save(&self) -> Result<(), Error> {
        let config_file = get_config_file();
        let config_dir = get_config_dir();
        let cache_dir = get_cache_dir();

        std::fs::create_dir_all(config_dir)?;
        std::fs::create_dir_all(cache_dir)?;

        let mut file = std::fs::File::create(config_file)?;
        let config_str = toml::to_string_pretty(&self)?;
        file.write_all(config_str.as_bytes())?;
        Ok(())
    }
}
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginKey {
    #[serde_as(as = "NoneAsEmptyString")]
    pub device_code: Option<String>,
    pub user_id: Option<i64>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub country_code: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub access_token: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub refresh_token: Option<String>,
    pub expires_after: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiKey {
    pub client_id: String,
    pub client_secret: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilePath {
    pub base_path: String,
    pub artist: String,
    pub album: String,
    pub track: String,
}

pub trait DownloadPath<T>
where
    Self: Sized + Clone,
    T: TokenMap<Self> + 'static + Copy,
{
    fn replace_path(self, path: &str) -> String {
        let map = T::token_map();
        let mut x = path.to_string();
        map.entries().for_each(|entry| {
            if x.contains(entry.0) {
                x = x.replace(entry.0, &entry.1.get_token(&self));
            };
        });
        x
    }
}

impl DownloadPath<ArtistTokens> for Artist {}
impl DownloadPath<AlbumTokens> for Album {}
impl DownloadPath<TrackTokens> for Track {}

pub trait TokenMap<T>
where
    Self: Sized,
    T: Clone,
{
    fn token_map() -> &'static phf::Map<&'static str, Self>;

    fn get_token(self, _: &T) -> String;
}

static ARTIST_TOKEN_MAP: phf::Map<&'static str, ArtistTokens> = phf_map! {
    "{artist_name}" =>  ArtistTokens::Name,
    "{artist_id}" => ArtistTokens::ID
};

#[derive(Clone, Copy)]
pub enum ArtistTokens {
    ID,
    Name,
}

impl TokenMap<Artist> for ArtistTokens {
    fn token_map() -> &'static phf::Map<&'static str, Self> {
        &ARTIST_TOKEN_MAP
    }

    fn get_token(self, a: &Artist) -> String {
        let val = match self {
            ArtistTokens::ID => a.id.to_string(),
            ArtistTokens::Name => a.name.to_string(),
        };
        sanitize(val)
    }
}

static ALBUM_TOKEN_MAP: phf::Map<&'static str, AlbumTokens> = phf_map! {
    "{album_id}" => AlbumTokens::ID,
    "{album_name}" => AlbumTokens::Title,
    "{album_duration}" => AlbumTokens::Duration,
    "{album_tracks}" => AlbumTokens::NumberOfTracks,
    "{album_explicit}" => AlbumTokens::Explicit,
    "{album_quality}" => AlbumTokens::AudioQuality,
    "{album_release}" => AlbumTokens::ReleaseDate,
    "{album_release_year}" => AlbumTokens::ReleaseYear,
};
impl TokenMap<Album> for AlbumTokens {
    fn token_map() -> &'static phf::Map<&'static str, Self> {
        &ALBUM_TOKEN_MAP
    }

    fn get_token(self, a: &Album) -> String {
        let a = match self {
            AlbumTokens::ID => a.id.to_string(),
            AlbumTokens::Title => a.title.as_ref().unwrap().clone(),
            AlbumTokens::Duration => a.duration.unwrap().to_string(),
            AlbumTokens::NumberOfTracks => a.number_of_tracks.unwrap().to_string(),
            AlbumTokens::Explicit => match a.explicit.unwrap() {
                true => String::from("[E]"),
                false => String::new(),
            },
            AlbumTokens::AudioQuality => a.audio_quality.unwrap().to_string(),
            AlbumTokens::ReleaseDate => a.release_date.as_ref().unwrap().clone(),
            AlbumTokens::ReleaseYear => a
                .release_date
                .as_ref()
                .unwrap()
                .split('-')
                .next()
                .unwrap()
                .to_string(),
        };
        sanitize(a)
    }
}
#[derive(Clone, Copy)]
pub enum AlbumTokens {
    ID,
    Title,
    Duration,
    NumberOfTracks,
    Explicit,
    AudioQuality,
    ReleaseDate,
    ReleaseYear,
}

static TRACK_TOKEN_MAP: phf::Map<&'static str, TrackTokens> = phf_map! {
   "{track_id}" => TrackTokens::ID,
   "{track_name}" => TrackTokens::Title,
   "{track_duration}" => TrackTokens::Duration,
   "{track_num}" => TrackTokens::TrackNumber,
   "{track_volume}" => TrackTokens::VolumeNumber,
   "{track_isrc}" => TrackTokens::ISRC,
   "{track_explicit}" => TrackTokens::Explicit,
   "{track_quality}" => TrackTokens::AudioQuality,
};

#[derive(Clone, Copy)]
pub enum TrackTokens {
    ID,
    Title,
    Duration,
    TrackNumber,
    VolumeNumber,
    ISRC,
    Explicit,
    AudioQuality,
}
impl TokenMap<Track> for TrackTokens {
    fn token_map() -> &'static phf::Map<&'static str, Self> {
        &TRACK_TOKEN_MAP
    }

    fn get_token(self, v: &Track) -> String {
        let a = match self {
            TrackTokens::ID => v.id.to_string(),
            TrackTokens::Title => v.title.clone(),
            TrackTokens::Duration => v.duration.to_string(),
            TrackTokens::TrackNumber => v.track_number.to_string(),
            TrackTokens::VolumeNumber => v.volume_number.to_string(),
            TrackTokens::ISRC => v.isrc.to_string(),
            TrackTokens::Explicit => match v.explicit {
                true => String::from("[E]"),
                false => String::new(),
            },
            TrackTokens::AudioQuality => v.audio_quality.to_string(),
        };
        sanitize(a)
    }
}

pub fn get_config() -> Result<Settings, Error> {
    let config = Config::builder()
        .set_default("audio_quality", "HI_RES")?
        .set_default("show_progress", true)?
        .set_default("include_singles", true)?
        .set_default("progress_refresh_rate", 5)?
        .set_default("login_key.device_code", "")?
        .set_default("login_key.country_code", "")?
        .set_default("download_cover", true)?
        .set_default("downloads", 1)?
        .set_default("workers", 1)?
        .set_default("cache_dir", get_cache_dir())?
        .set_default("login_key.access_token", "")?
        .set_default("login_key.refresh_token", "")?
        .set_default("login_key.expires_after", 0)?
        .set_default("api_key.client_id", "zU4XHVVkc2tDPo4t")?
        .set_default(
            "api_key.client_secret",
            "VJKhDFqJPqvsPVNBV6ukXTJmwlvbttP7wlMlrc72se4=",
        )?
        .set_default("db_path", get_db_dir())?
        .set_default("download_path.base_path", "$HOME/Music")?
        .set_default("download_path.artist", "{artist_name}")?
        .set_default(
            "download_path.album",
            "{album_name} [ {album_id} ] [{album_release_year}] ",
        )?
        .set_default("download_path.track", "{track_num} - {track_name}")?
        .add_source(File::new(CONFIG_FILE.as_str(), FileFormat::Toml).required(false))
        .build()?;
    let settings: Settings = config.try_deserialize()?;
    settings.save()?;

    Ok(settings)
}

fn get_config_dir() -> String {
    let config_dir =
        var("XDG_CONFIG_HOME").unwrap_or_else(|_| var("HOME").unwrap_or_else(|_| "".to_string()));
    format!("{}/.config/tdl", config_dir)
}

fn get_cache_dir() -> String {
    format!("{}/cache", get_config_dir())
}

fn get_config_file() -> String {
    format!("{}/config.toml", get_config_dir())
}

fn get_db_dir() -> String {
    format!("{}/db.sqlite", get_config_dir())
}

lazy_static::lazy_static! {
   pub static ref CONFIG_HOME: String = get_config_dir();
   pub static ref CONFIG_FILE: String = get_config_file();
   pub static ref CONFIG: RwLock<Settings> = RwLock::new(get_config().expect("Unable to get configuration"));
}
