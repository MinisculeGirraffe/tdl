use crate::config::CONFIG;
use anyhow::Error;
use indicatif::TermLike;
use log::info;
use tokio::task::JoinHandle;

use crate::client::{self, DeviceAuthResponse};
use console::{measure_text_width, Emoji, Term};
use console::{pad_str, style};
use tokio::time::{interval, sleep, Duration, Instant};
pub async fn login() -> Result<bool, Error> {
    let cfg_login = login_config().await;
    if cfg_login.is_ok() {
        return Ok(true);
    }
    let web_login = login_web().await;
    if web_login.is_ok() {
        return Ok(true);
    }
    Err(Error::msg("All Login methods failed"))
}

async fn login_web() -> Result<bool, Error> {
    let code = client::get_device_code().await?;
    let term = Term::stdout();
    let now = Instant::now();
    let task = display_login_prompt(code.clone(), now);

    while now.elapsed().as_secs() <= code.expires_in {
        let login = client::check_auth_status(&code.device_code).await;
        if login.is_err() {
            sleep(Duration::from_secs(code.interval)).await;
            continue;
        }
        task.abort();
        term.show_cursor()?;
        let timestamp = chrono::Utc::now().timestamp();
        let mut config = CONFIG.write().await;
        let login_results = login?;
        config.login_key.device_code = Some(code.device_code);
        config.login_key.access_token = Some(login_results.access_token);
        config.login_key.refresh_token = Some(login_results.refresh_token);
        config.login_key.expires_after = Some(login_results.expires_in + timestamp);
        config.login_key.user_id = Some(login_results.user.user_id);
        config.login_key.country_code = Some(login_results.user.country_code);
        config.save()?;
        return Ok(true);
    }
    task.abort();
    term.show_cursor()?;
    println!("Login Request timed out. Please generate a new code");
    Ok(false)
}

async fn login_config() -> Result<bool, Error> {
    let mut config = CONFIG.write().await;
    if let Some(access_token) = config.login_key.access_token.as_ref() {
        if client::verify_access_token(access_token).await? {
            println!("Access Token Valid");
            return Ok(true);
        }
    }

    if let Some(refresh_token) = config.login_key.refresh_token.as_ref() {
        let refresh = client::refresh_access_token(refresh_token).await?;
        let now = chrono::Utc::now().timestamp();
        config.login_key.expires_after = Some(refresh.expires_in + now);
        config.login_key.access_token = Some(refresh.access_token);
        config.login_key.refresh_token = Some(refresh.refresh_token);
        config.save()?;
        info!("Access Token Refreshed with Refresh Token");
        return Ok(true);
    }

    Err(Error::msg(
        "Unable to authenticate with both client and refresh token",
    ))
}

fn display_login_prompt(code: DeviceAuthResponse, instant: Instant) -> JoinHandle<()> {
    tokio::task::spawn(async move {
        let clocks = vec![
            "🕛", "🕐", "🕑", "🕒", "🕓", "🕔", "🕕", "🕖", "🕗", "🕘", "🕙", "🕚",
        ];
        let mut animation_index = 0;
        let term = Term::stdout();
        term.hide_cursor().ok();
        let mut interval = interval(Duration::from_millis(83));
        let url = format!("https://{}", code.verification_uri_complete);
        let hyperlink = format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b", url, url);
        let login_str = format!(
            "Please Login to Tidal: {}",
            style(hyperlink).underlined().bold()
        );
        let login_str_width = measure_text_width(&login_str);
        term.write_line(&login_str).ok();
        loop {
            interval.tick().await;
            let timeleft = code.expires_in - instant.elapsed().as_secs();
            let mut time_str = format!(
                "{} {}:{} ",
                Emoji(clocks[animation_index], ""),
                (timeleft / 60) % 60,
                timeleft % 60,
            );
            let term_width: usize = term.width().into();
            if term_width > login_str_width {
                time_str =
                    pad_str(&time_str, term_width, console::Alignment::Right, None).to_string();
            }
            term.clear_line().ok();
            term.write_str(&time_str).ok();
            animation_index += 1;
            if animation_index > clocks.len() - 1 {
                animation_index = 0
            }
        }
    })
}
