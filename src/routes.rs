use log::info;
use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{get, routes, uri};
use rocket_dyn_templates::{context, Template};
use std::env;

use crate::db::Db;
use crate::error;
use crate::models::{is_dt_past, ts_to_dt, Data, User};
use crate::{db, geo, h3, strava};

pub fn routes() -> Vec<rocket::Route> {
    routes![
        health,
        authed_index,
        unauthed_index,
        get_data,
        auth,
        callback,
        logout,
    ]
}

#[get("/health")]
fn health() -> &'static str {
    "ok"
}

#[get("/")]
fn authed_index(user: User) -> Template {
    let User { id } = user;
    let os_key = env::var("OS_KEY").unwrap();
    let logged_in = true;
    Template::render("index", context! { id, os_key, logged_in })
}

#[get("/", rank = 2)]
fn unauthed_index() -> Template {
    let id = "";
    let os_key = env::var("OS_KEY").unwrap();
    let logged_in = false;
    Template::render("index", context! { id, os_key, logged_in })
}

#[get("/data")]
async fn get_data(conn: Db, user: User) -> Result<Json<Data>, error::Error> {
    let User { id } = user;

    let user = db::get_user(&conn, id).await?;
    let expiry = ts_to_dt(user.expires_at);
    let expired = is_dt_past(expiry);

    // previously I was just getting a token out of the cookie
    // which was quite elegant, but didn't provide for refreshing...
    let token = if expired {
        // get a new token (using refresh_token) if this one expired

        info!("getting new refresh token for id {}", id);
        let token_response =
            strava::get_token(&user.refresh_token, strava::GrantType::Refresh).await?;
        db::save_user(&conn, &token_response).await?;
        token_response.access_token
    } else {
        // otherwise use the current one
        user.access_token
    };

    let activities = strava::get_activities(&token).await?;
    let activities = geo::decode_all(activities);
    let centroid = geo::get_useful_centroid(&activities);
    let cells = h3::polyfill_all(&activities);
    let cells: Vec<String> = cells
        .iter()
        .map(|cell_index| format!("{:x}", cell_index))
        .collect();

    let activities = geo::to_geojson(activities);
    Ok(Json(Data {
        activities: Some(activities),
        cells,
        centroid,
    }))
}

#[get("/auth")]
fn auth() -> Redirect {
    let url = strava::create_oauth_url().unwrap();
    Redirect::to(url)
}

#[get("/callback?<code>")]
async fn callback(conn: Db, code: &str, jar: &CookieJar<'_>) -> Result<Redirect, error::Error> {
    let token_response = strava::get_token(code, strava::GrantType::Auth).await?;
    db::save_user(&conn, &token_response).await?;

    let mut c_id: Cookie = Cookie::new("id", token_response.athlete.id.to_string());
    // This happens after the OAuth flow and if SameSite::Strict
    // the cookies somehow dont get sent with the first / request
    // as there is some Strava/Google analytics stuff in between??
    c_id.set_same_site(SameSite::Lax);
    jar.add_private(c_id);

    Ok(Redirect::to(uri!(authed_index)))
}

#[get("/logout")]
fn logout(jar: &CookieJar<'_>) -> Redirect {
    jar.remove_private("id");
    Redirect::to(uri!(unauthed_index))
}
