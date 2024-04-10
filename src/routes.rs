use rocket::fs::{relative, FileServer};
use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::response::Redirect;
use rocket::{get, uri, Error, Ignite, Rocket};
use rocket_dyn_templates::{context, Template};
use rocket_okapi::settings::UrlObject;
use rocket_okapi::{openapi, openapi_get_routes, rapidoc::*};
use std::env;
use time::{Duration, OffsetDateTime};

use crate::models::User;
use crate::{data, h3, strava};

pub async fn build() -> Result<Rocket<Ignite>, Error> {
    rocket::build()
        .attach(Template::custom(|engines| {
            engines
                .handlebars
                // otherwise it mangles the GeoJSON
                .register_escape_fn(handlebars::no_escape)
        }))
        .mount("/static", FileServer::from(relative!("static")))
        .mount("/", routes())
        .mount(
            "/rapidoc",
            make_rapidoc(&RapiDocConfig {
                general: GeneralConfig {
                    spec_urls: vec![UrlObject::new("General", "../openapi.json")],
                    ..Default::default()
                },
                hide_show: HideShowConfig {
                    allow_spec_url_load: false,
                    allow_spec_file_load: false,
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .launch()
        .await
}

fn routes() -> Vec<rocket::Route> {
    openapi_get_routes![
        health,
        index,
        auth,
        callback,
        authed_index,
        logout,
        logged_out
    ]
}

#[openapi(tag = "Health")]
#[get("/health")]
fn health() -> &'static str {
    "ok"
}

#[openapi(skip)]
#[get("/", rank = 2)]
fn index() -> Redirect {
    Redirect::to(uri!(auth()))
}

#[openapi(skip)]
#[get("/")]
async fn authed_index(user: User) -> Template {
    let User { id, token } = user;
    // Should get user from db/session
    // but for now just using code directly
    // let user = db::get_user(id);

    // TODO flash message on error (eg expired token)
    let activities = strava::get_activities(&token).await;

    // Could just return Json here and use a `fetch` call from UI
    // Json(geo::decode_all(activities))

    // But instead return template with injected GeoJSON
    let activities = data::decode_all(activities);

    let mut cells = h3::polyfill_all(&activities);
    cells.sort();
    cells.dedup();
    let cells: Vec<String> = cells
        .iter()
        .map(|cell_index| format!("\"{:x}\"", cell_index))
        .collect();

    let gj = data::to_geojson(activities).to_string();
    let os_key = env::var("OS_KEY").unwrap();
    Template::render("map", context! { gj, os_key, id, cells })
}

#[openapi(tag = "OAuth")]
#[get("/auth")]
fn auth() -> Redirect {
    let url = strava::create_oauth_url().unwrap();
    Redirect::to(url)
}

#[openapi(tag = "OAuth")]
#[get("/callback?<code>")]
async fn callback(code: &str, jar: &CookieJar<'_>) -> Redirect {
    let token_response = strava::get_token(code).await;

    let mut c_id: Cookie = Cookie::new("id", token_response.athlete.id.to_string());
    // This happens after the OAuth flow and if SameSite::Strict
    // the cookies somehow dont get sent with the first / request
    // as there is some Strava/Google analytics stuff in between??
    c_id.set_same_site(SameSite::Lax);
    jar.add_private(c_id);

    // The Strava token is valid for six hours
    // so expire the cookie after 5!
    let mut c_token: Cookie = Cookie::new("token", token_response.access_token);
    c_token.set_expires(OffsetDateTime::now_utc() + Duration::hours(5));
    // Same comment as above about Strict
    c_token.set_same_site(SameSite::Lax);
    jar.add_private(c_token);

    // Not saving users, rather just redirect with cookies
    // db::save_user(token_response);
    use std::{thread, time};
    let ten_millis = time::Duration::from_millis(1000);
    thread::sleep(ten_millis);
    Redirect::to(uri!(authed_index))
}

#[openapi(skip)]
#[get("/logout")]
fn logout(jar: &CookieJar<'_>) -> Redirect {
    jar.remove_private("id");
    jar.remove_private("token");
    Redirect::to(uri!(logged_out))
}

#[openapi(skip)]
#[get("/logged-out")]
async fn logged_out() -> Template {
    Template::render("logged-out", ())
}
