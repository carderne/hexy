use anyhow::Context;
use reqwest::header::AUTHORIZATION;
use serde::Deserialize;
use std::env;
use url::{ParseError, Url};

use crate::error::Error;

pub enum GrantType {
    Auth,
    Refresh,
}

#[derive(Deserialize)]
pub struct Athlete {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct TokenResponse {
    pub athlete: Athlete,
    pub refresh_token: String,
    pub access_token: String,
    pub expires_at: i32,
}

#[derive(Deserialize)]
pub struct Map {
    pub polyline: Option<String>,
    pub summary_polyline: Option<String>,
}

#[derive(Deserialize)]
pub struct ActivityResponse {
    pub id: i64,
    pub name: String,
    pub distance: f64,
    pub moving_time: i64,
    pub elapsed_time: i64,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub kudos_count: i32,
    pub average_speed: f64,
    pub sport_type: String,
    pub map: Map,
}

fn create_activities_url() -> Result<String, ParseError> {
    let base = env::var("STRAVA_BASE").unwrap();

    let mut url = Url::parse(&base)?;
    let path = "api/v3/athlete/activities";
    url = url.join(path)?;
    url.query_pairs_mut().append_pair("per_page", "200");
    Ok(url.to_string())
}

pub fn create_oauth_url() -> Result<String, ParseError> {
    let base = env::var("STRAVA_BASE").unwrap();
    let client_id = env::var("STRAVA_CLIENT_ID").unwrap();
    let redirect_uri = env::var("REDIRECT_URI").unwrap();

    let mut url = Url::parse(&base)?;
    let path = "/oauth/authorize";
    url = url.join(path)?;
    url.query_pairs_mut()
        .append_pair("client_id", &client_id)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("approval_prompt", "force")
        .append_pair("scope", "read,activity:read");
    Ok(url.to_string())
}

fn create_token_url(code: &str, grant_type: GrantType) -> Result<String, ParseError> {
    let base = env::var("STRAVA_BASE").unwrap();
    let client_id = env::var("STRAVA_CLIENT_ID").unwrap();
    let client_secret = env::var("STRAVA_CLIENT_SECRET").unwrap();
    let grant_type = match grant_type {
        GrantType::Auth => "authorization_code",
        GrantType::Refresh => "refresh_token",
    };

    let mut url = Url::parse(&base)?;
    let path = "/oauth/token";
    url = url.join(path)?;
    url.query_pairs_mut()
        .append_pair("client_id", &client_id)
        .append_pair("client_secret", &client_secret)
        .append_pair("code", code)
        .append_pair("grant_type", grant_type);
    Ok(url.to_string())
}

pub async fn get_activities(token: &str) -> Result<Vec<ActivityResponse>, Error> {
    let url = create_activities_url()?;
    let client = reqwest::Client::new();
    let bearer = format!("Bearer {}", token);
    let response = client
        .get(url)
        .header(AUTHORIZATION, bearer)
        .send()
        .await?
        .error_for_status()?;
    let body = response
        .json::<Vec<ActivityResponse>>()
        .await
        .with_context(|| "strava::get_activities".to_string())?;
    Ok(body)
}

pub async fn get_token(code: &str, grant_type: GrantType) -> Result<TokenResponse, Error> {
    let url = create_token_url(code, grant_type)?;
    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .send()
        .await?
        .error_for_status()?;
    let body = response
        .json::<TokenResponse>()
        .await
        // TODO insert this function context automatically
        .with_context(|| "strava::get_token".to_string())?;
    Ok(body)
}
