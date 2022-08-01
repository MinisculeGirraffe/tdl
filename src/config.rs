use crate::api::models::AudioQuality;
use anyhow::Error;
use config::{Config, File, FileFormat};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::NoneAsEmptyString;
use std::env::var;
use std::io::Write;
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub download_path: String,
    pub audio_quality: AudioQuality,
    pub show_progress: bool,
    pub progress_refresh_rate: u8,
    pub include_singles: bool,
    pub downloads: u8,
    pub workers: u8,
    pub download_cover: bool,
    pub cache_dir: String,
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

pub fn get_config() -> Result<Settings, Error> {
    let config = Config::builder()
        .set_default(
            "download_path",
            "$HOME/Music/{artist}/{album} [{album_id}] [{album_release_year}]/{track_num} - {track_name}",
        )?
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

lazy_static::lazy_static! {
   pub static ref CONFIG_HOME: String = get_config_dir();
   pub static ref CONFIG_FILE: String = get_config_file();
   pub static ref CONFIG: RwLock<Settings> = RwLock::new(get_config().expect("Unable to get configuration"));
}
