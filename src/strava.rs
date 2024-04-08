use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use std::env;
use url::{ParseError, Url};

#[derive(Deserialize, Debug)]
pub struct Athlete {
    pub id: i32,
}

#[derive(Deserialize, Debug)]
pub struct TokenResponse {
    pub athlete: Athlete,
    pub refresh_token: String,
    pub access_token: String,
    pub expires_at: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Map {
    pub polyline: Option<String>,
    pub summary_polyline: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActivitiesResponse {
    pub id: i64,
    pub name: String,
    pub distance: f64,
    pub moving_time: i64,
    pub elapsed_time: i64,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub kudos_count: i32,
    pub average_speed: f64,

    #[serde(rename = "type")]
    pub ac_type: String,
    pub sport_type: String,

    pub map: Map,
}

impl ActivitiesResponse {
    pub fn to_json_object(&self) -> Option<geojson::JsonObject> {
        let mut value = serde_json::to_value(self).unwrap();
        if let geojson::JsonValue::Object(ref mut obj) = value {
            obj.remove("map");
            Some(obj.clone())
        } else {
            None
        }
    }
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

fn create_token_url(code: &str) -> Result<String, ParseError> {
    let base = env::var("STRAVA_BASE").unwrap();
    let client_id = env::var("STRAVA_CLIENT_ID").unwrap();
    let client_secret = env::var("STRAVA_CLIENT_SECRET").unwrap();

    let mut url = Url::parse(&base)?;
    let path = "/oauth/token";
    url = url.join(path)?;
    url.query_pairs_mut()
        .append_pair("client_id", &client_id)
        .append_pair("client_secret", &client_secret)
        .append_pair("code", code)
        .append_pair("grant_type", "authorization_code");
    Ok(url.to_string())
}

pub async fn get_activities(token: &str) -> Vec<ActivitiesResponse> {
    let url = create_activities_url().unwrap();
    let client = reqwest::Client::new();
    let bearer = format!("Bearer {}", token);
    client
        .get(url)
        .header(AUTHORIZATION, bearer)
        .send()
        .await
        .unwrap()
        .json::<Vec<ActivitiesResponse>>()
        .await
        .unwrap()
}

pub async fn get_token(code: &str) -> TokenResponse {
    let url = create_token_url(code).unwrap();
    let client = reqwest::Client::new();
    client
        .post(url)
        .send()
        .await
        .unwrap()
        .json::<TokenResponse>()
        .await
        .unwrap()
}
