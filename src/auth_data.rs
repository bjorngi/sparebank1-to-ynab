use crate::config::Config;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Sparebanken1AuthDataResponse {
    access_token: String,
    refresh_token: String,
}

fn get_refresh_token(config: &Config) -> Result<String, String> {
    match fs::read_to_string(config.refresh_token_file_path.clone()) {
        Ok(refresh_token) => Ok(refresh_token),
        Err(_) => Ok(config.initial_refresh_token.clone()),
    }
}

fn save_refresh_token(refresh_token_file_path: &str, new_refresh_token: String) -> Result<(), std::io::Error> {
    fs::write(refresh_token_file_path, new_refresh_token)
}

async fn refresh_access_token(config: &Config, refresh_token: String) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url: &str = "https://api-auth.sparebank1.no/oauth/token";

    let body = format!("grant_type=refresh_token&refresh_token={}&client_id={}&client_secret={}", refresh_token, config.sparebank1_client_id, config.sparebank1_client_secret);

    let response: Sparebanken1AuthDataResponse = client.post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?
        .json::<Sparebanken1AuthDataResponse>()
        .await?;

    let _ = save_refresh_token(&config.refresh_token_file_path, response.refresh_token);

    Ok(response.access_token)
}

pub async fn get_access_token(config: &Config) -> Result<String, Box<dyn std::error::Error>> {
    let refresh_token = get_refresh_token(config)?;
    let access_token = refresh_access_token(config, refresh_token);
    return Ok(access_token.await?);
}
