use rocket::fs::{relative, FileServer};
use rocket::response::Redirect;
use rocket::{get, uri, Error, Ignite, Rocket};
use rocket_dyn_templates::{context, Template};
use rocket_okapi::settings::UrlObject;
use rocket_okapi::{openapi, openapi_get_routes, rapidoc::*};
use std::env;

use crate::{geo, strava};

pub async fn build() -> Result<Rocket<Ignite>, Error> {
    rocket::build()
        .attach(Template::custom(|engines| {
            engines
                .handlebars
                // otherwise it mangles the GeoJSON
                .register_escape_fn(handlebars::no_escape)
        }))
        .mount("/static", FileServer::from(relative!("static")))
        .mount(
            "/",
            openapi_get_routes![health, index, auth, exchange, user],
        )
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

#[openapi(tag = "Health")]
#[get("/health")]
fn health() -> &'static str {
    "ok"
}

#[openapi(tag = "Main")]
#[get("/")]
fn index() -> Redirect {
    Redirect::to(uri!(auth()))
}

#[openapi(tag = "OAuth")]
#[get("/auth")]
fn auth() -> Redirect {
    let url = strava::create_oauth_url().unwrap();
    Redirect::to(url)
}

#[openapi(tag = "OAuth")]
#[get("/exchange?<code>")]
async fn exchange(code: &str) -> Redirect {
    let t = strava::get_token(code).await;
    let redir_url = uri!(user(t.athlete.id, &t.access_token));
    // Not saving users, rather just redirect with param
    // until I implement sessions and whatnot
    // db::save_user(t);
    Redirect::to(redir_url)
}

#[openapi(skip)]
#[get("/user/<_id>?<access_token>")]
async fn user(_id: i32, access_token: &str) -> Template {
    // Should get user from db/session
    // but for now just using code directly
    // let user = db::get_user(id);
    let activities = strava::get_activities(access_token).await;

    // Could just return Json here and use a `fetch` call from UI
    // Json(geo::decode_all(activities))

    // But instead return template with injected GeoJSON
    let gj = geo::decode_all(activities);
    let os_key = env::var("OS_KEY").unwrap();
    Template::render("map", context! { gj: gj.to_string(), os_key })
}
