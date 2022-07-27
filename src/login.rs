use crate::api::auth::{
    check_auth_status, get_device_code, refresh_access_token, verify_access_token,
};
use crate::api::models::DeviceAuthResponse;
use crate::config::CONFIG;
use anyhow::Error;
use indicatif::TermLike;
use log::debug;
use tokio::task::JoinHandle;

use console::{measure_text_width, Emoji, Term};
use console::{pad_str, style};
use tokio::time::{interval, sleep, Duration, Instant};

pub async fn login_web() -> Result<bool, Error> {
    let code = get_device_code().await?;
    let now = Instant::now();
    let prompt = show_prompt(code.clone(), now);

    while now.elapsed().as_secs() <= code.expires_in {
        let login = check_auth_status(&code.device_code).await;
        if login.is_err() {
            sleep(Duration::from_secs(code.interval)).await;
            continue;
        }
        hide_prompt(prompt);
        let timestamp = chrono::Utc::now().timestamp();
        let mut config = CONFIG.write().await;
        //login will not be error if this is reached.
        let login_results = login?;
        config.login_key.device_code = Some(code.device_code);
        config.login_key.access_token = Some(login_results.access_token);
        config.login_key.refresh_token = login_results.refresh_token;
        config.login_key.expires_after = Some(login_results.expires_in + timestamp);
        config.login_key.user_id = Some(login_results.user.user_id);
        config.login_key.country_code = Some(login_results.user.country_code);
        config.save()?;
        return Ok(true);
    }
    hide_prompt(prompt);
    println!("Login Request timed out. Please generate a new code");
    Ok(false)
}

pub async fn login_config() -> Result<bool, Error> {
    let config = CONFIG.read().await;
    if let Some(access_token) = config.login_key.access_token.as_ref() {
        debug!("Attempting to validate access token");
        if verify_access_token(access_token).await? {
            println!("Access Token Valid");
            return Ok(true);
        }
    }

    if let Some(refresh_token) = config.login_key.refresh_token.as_ref() {
        debug!("Attempting to refresh access token");
        let refresh = refresh_access_token(refresh_token).await?;
        drop(config);
        debug!("Access token refreshed");
        let now = chrono::Utc::now().timestamp();
        {
            let mut config = CONFIG.write().await;
            config.login_key.expires_after = Some(refresh.expires_in + now);
            config.login_key.access_token = Some(refresh.access_token);
            debug!("Attempting to save access token");
            config.save()?;
            println!("Access Token Refreshed with Refresh Token");
            return Ok(true);
        }
    }
    debug!("All methods failed");
    Err(Error::msg(
        "Unable to authenticate with both client and refresh token",
    ))
}

fn show_prompt(code: DeviceAuthResponse, instant: Instant) -> JoinHandle<()> {
    tokio::task::spawn(async move {
        let clocks = vec![
            "🕛", "🕐", "🕑", "🕒", "🕓", "🕔", "🕕", "🕖", "🕗", "🕘", "🕙", "🕚",
        ];
        let mut animation_index = 0;
        let term = Term::stdout();
        term.hide_cursor().ok();
        let mut interval = interval(Duration::from_millis(83));
        let login_str = fmt_login(&code.verification_uri_complete);
        let login_str_width = measure_text_width(&login_str);
        term.write_line(&login_str).ok();
        loop {
            interval.tick().await;
            // re-calc terminal size every tick
            let term_width: usize = term.width().into();
            // calculate the time left in the login prompt at the current tick
            let sec_left = code.expires_in - instant.elapsed().as_secs();

            let mut time_str = format!(
                "{} {}",
                // display the current frame of the clock spinning or fallback to empty string
                Emoji(clocks[animation_index], ""),
                fmt_time_left(sec_left)
            );

            // if our terminal is wider than our base text, right align the time
            if term_width > login_str_width {
                time_str =
                    pad_str(&time_str, term_width, console::Alignment::Right, None).to_string();
            }
            // clear the last frame and re-draw
            term.clear_line().ok();
            term.write_str(&time_str).ok();
            animation_index += 1;
            if animation_index > clocks.len() - 1 {
                animation_index = 0
            }
        }
    })
}

fn hide_prompt(task: JoinHandle<()>) {
    task.abort();
    let _ = Term::stdout().show_cursor();
}

fn fmt_login(uri: &str) -> String {
    let url = format!("https://{}", uri);
    // ANSI Hyperlink format.
    
    format!(
        "Please Login to Tidal: {}",
        style(url).underlined().bold()
    )
}

// formats a clickable hyperlink in a terminal
// https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda
// https://en.wikipedia.org/wiki/ANSI_escape_code
fn _fmt_ansi_url(display: &str, url: &str) -> String {
    format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b", display, url)
}

fn fmt_time_left(sec_left: u64) -> String {
    let seconds = sec_left % 60;
    let mins = (sec_left / 60) % 60;

    format!("{}:{}", pad_zero(mins), pad_zero(seconds))
}

fn pad_zero(s: impl ToString) -> String {
    let str = s.to_string();
    if str == "0" {
        "00".to_string()
    } else {
        str
    }
}
