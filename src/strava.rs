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

pub struct StravaClient {
    base: Url,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

impl Default for StravaClient {
    fn default() -> Self {
        let base = env::var("STRAVA_BASE").unwrap();
        let client_id = env::var("STRAVA_CLIENT_ID").unwrap();
        let client_secret = env::var("STRAVA_CLIENT_SECRET").unwrap();
        let redirect_uri = env::var("REDIRECT_URI").unwrap();
        Self::new(&base, &client_id, &client_secret, &redirect_uri)
    }
}

impl StravaClient {
    pub fn new(base: &str, client_id: &str, client_secret: &str, redirect_uri: &str) -> Self {
        let base = Url::parse(base).unwrap();
        StravaClient {
            base,
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            redirect_uri: redirect_uri.to_string(),
        }
    }

    fn create_activities_url(&self) -> Result<String, ParseError> {
        let mut url = self.base.clone();
        let path = "api/v3/athlete/activities";
        url = url.join(path)?;
        url.query_pairs_mut().append_pair("per_page", "200");
        Ok(url.to_string())
    }

    pub fn create_oauth_url(&self) -> Result<String, ParseError> {
        let mut url = self.base.clone();
        let path = "/oauth/authorize";
        url = url.join(path)?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("response_type", "code")
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("approval_prompt", "force")
            .append_pair("scope", "read,activity:read");
        Ok(url.to_string())
    }

    fn create_token_url(&self, code: &str, grant_type: GrantType) -> Result<String, ParseError> {
        let grant_type = match grant_type {
            GrantType::Auth => "authorization_code",
            GrantType::Refresh => "refresh_token",
        };

        let mut url = self.base.clone();
        let path = "/oauth/token";
        url = url.join(path)?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("client_secret", &self.client_secret)
            .append_pair("code", code)
            .append_pair("grant_type", grant_type);
        Ok(url.to_string())
    }

    pub async fn get_activities(&self, token: &str) -> Result<Vec<ActivityResponse>, Error> {
        let url = self.create_activities_url()?;
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

    pub async fn get_token(
        &self,
        code: &str,
        grant_type: GrantType,
    ) -> Result<TokenResponse, Error> {
        let url = self.create_token_url(code, grant_type)?;
        let client = reqwest::Client::new();
        let response = client.post(url).send().await?.error_for_status()?;
        let body = response
            .json::<TokenResponse>()
            .await
            // TODO insert this function context automatically
            .with_context(|| "strava::get_token".to_string())?;
        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use httpmock::prelude::*;
    use tokio;
    use super::*;

    #[tokio::test]
    async fn test_get_activities() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/api/v3/athlete/activities");
            then.status(200)
                .header("content-type", "text/html; charset=UTF-8")
                .body(r#"[]"#);
        });

        let sc = StravaClient::new(&server.url("/"), "", "", "");
        let res = sc.get_activities("").await.unwrap();

        mock.assert();
        assert!(res.len() == 0);
    }
}
