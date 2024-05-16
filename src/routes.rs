use log::info;
use rocket::fairing::AdHoc;
use rocket::fs::{relative, FileServer};
use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{get, routes, uri, Build, Rocket};
use rocket_dyn_templates::context;
use rocket_dyn_templates::Template;
use std::env;

use crate::db::Db;
use crate::error;
use crate::models::{is_dt_past, ts_to_dt, Data, User};
use crate::{db, geo, h3, strava};

pub fn build(prep_db: bool) -> Rocket<Build> {
    let mut s = rocket::build()
        .attach(db::Db::fairing())
        .attach(Template::fairing())
        .mount("/static", FileServer::from(relative!("static")))
        .mount("/", routes());

    if prep_db {
        s = s
            .attach(AdHoc::try_on_ignite("Migrations", db::migrate)) // Database migrations
            .attach(AdHoc::on_liftoff("Startup Check", |rocket| {
                Box::pin(async move {
                    let d = db::Db::get_one(rocket).await.unwrap();
                    db::prep_db(&d).await.expect("Failed to prep db");
                })
            }));
    }
    s
}

fn routes() -> Vec<rocket::Route> {
    routes![
        health,
        authed_index,
        unauthed_index,
        get_data,
        auth,
        callback,
        logout,
        home,
        privacy,
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
        info!("getting new token for id {}", id);
        let token_response = strava::StravaClient::default()
            .get_token(&user.refresh_token, strava::GrantType::Refresh)
            .await?;
        db::save_user(&conn, &token_response).await?;
        token_response.access_token
    } else {
        // otherwise use the current one
        user.access_token
    };

    let activities = strava::StravaClient::default()
        .get_activities(&token)
        .await?;
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
    let url = strava::StravaClient::default().create_oauth_url().unwrap();
    Redirect::to(url)
}

#[get("/callback?<code>")]
async fn callback(conn: Db, code: &str, jar: &CookieJar<'_>) -> Result<Redirect, error::Error> {
    let token_response = strava::StravaClient::default()
        .get_token(code, strava::GrantType::Auth)
        .await?;
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

#[get("/home")]
fn home() -> Template {
    Template::render("home", ())
}

#[get("/privacy")]
fn privacy() -> Template {
    Template::render("privacy", ())
}

