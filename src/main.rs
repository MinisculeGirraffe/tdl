use crate::config::CONFIG;
use anyhow::Error;

use tokio::time::Instant;
mod client;
mod config;

#[tokio::main]
async fn main() {


}


async fn login_web() -> Result<bool, Error> {
    let code = client::get_device_code().await?;
    println!(
        "Please Login to Tidal: http://{}",
        code.verification_uri_complete
    );
    let now = Instant::now();
    while now.elapsed().as_secs() < code.expires_in.try_into()? {
        let login = client::check_auth_status(&code.device_code).await;
        let now = chrono::Utc::now().timestamp();
        if login.is_err() {
            tokio::time::sleep(tokio::time::Duration::from_secs(code.interval.try_into()?)).await;
            continue;
        }
        let mut config = CONFIG.write().await;
        let login_results = login?;
        config.login_key.device_code = Some(code.device_code);
        config.login_key.access_token = Some(login_results.access_token);
        config.login_key.refresh_token = Some(login_results.refresh_token);
        config.login_key.expires_after = Some(login_results.expires_in + now);
        config.login_key.user_id = Some(login_results.user.user_id);
        config.login_key.country_code = Some(login_results.user.country_code);
        config.save()?;

        return Ok(true);
    }

    Ok(false)
}
