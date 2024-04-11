use rocket::fairing::AdHoc;
use rocket::fs::{relative, FileServer};
use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{get, routes, uri, Error, Ignite, Rocket};
use rocket_dyn_templates::{context, Template};
use std::env;

use crate::db::Db;
use crate::models::{is_dt_past, ts_to_dt, Data, User, UserDb};
use crate::{data, db, h3, strava};

pub async fn build() -> Result<Rocket<Ignite>, Error> {
    rocket::build()
        .attach(Db::fairing())
        .attach(AdHoc::try_on_ignite("Migrations", db::migrate))
        .attach(Template::fairing())
        .mount("/static", FileServer::from(relative!("static")))
        .mount("/", routes())
        .launch()
        .await
}

fn routes() -> Vec<rocket::Route> {
    routes![
        health,
        index,
        auth,
        callback,
        authed_index,
        logout,
        logged_out,
        get_data,
    ]
}

#[get("/health")]
fn health() -> &'static str {
    "ok"
}

#[get("/", rank = 2)]
fn index() -> Redirect {
    Redirect::to(uri!(auth()))
}

#[get("/")]
fn authed_index(user: User) -> Template {
    let User { id } = user;
    let os_key = env::var("OS_KEY").unwrap();
    Template::render("map", context! { id, os_key })
}

#[get("/data")]
async fn get_data(conn: Db, user: User) -> Json<Data> {
    let User { id } = user;

    let user = db::get_user(&conn, id).await;
    let user: UserDb = match user {
        Some(user) => user,
        None => {
            // TODO do something in the UI to handle this
            return Json(Data {
                activities: None,
                cells: vec![],
            });
        }
    };

    let expiry = ts_to_dt(user.expires_at);
    let expired = is_dt_past(expiry);

    // previously I was just getting a token out of the cookie
    // which was quite elegant, but didn't provide for refreshing...
    let token = if expired {
        // get a new token (using refresh_token) if this one expired
        let token_response =
            strava::get_token(&user.refresh_token, strava::GrantType::Refresh).await;
        db::save_user(&conn, &token_response).await;
        token_response.access_token
    } else {
        // otherwise use the current one
        user.access_token
    };

    let activities = strava::get_activities(&token).await;
    let activities = data::decode_all(activities);
    let mut cells = h3::polyfill_all(&activities);
    cells.sort();
    cells.dedup();
    let cells: Vec<String> = cells
        .iter()
        .map(|cell_index| format!("{:x}", cell_index))
        .collect();

    let activities = data::to_geojson(activities);
    Json(Data {
        activities: Some(activities),
        cells,
    })
}

#[get("/auth")]
fn auth() -> Redirect {
    let url = strava::create_oauth_url().unwrap();
    Redirect::to(url)
}

#[get("/callback?<code>")]
async fn callback(conn: Db, code: &str, jar: &CookieJar<'_>) -> Redirect {
    let token_response = strava::get_token(code, strava::GrantType::Auth).await;
    db::save_user(&conn, &token_response).await;

    let mut c_id: Cookie = Cookie::new("id", token_response.athlete.id.to_string());
    // This happens after the OAuth flow and if SameSite::Strict
    // the cookies somehow dont get sent with the first / request
    // as there is some Strava/Google analytics stuff in between??
    c_id.set_same_site(SameSite::Lax);
    jar.add_private(c_id);

    Redirect::to(uri!(authed_index))
}

#[get("/logout")]
fn logout(jar: &CookieJar<'_>) -> Redirect {
    jar.remove_private("id");
    Redirect::to(uri!(logged_out))
}

#[get("/logged-out")]
async fn logged_out() -> Template {
    Template::render("logged-out", ())
}
