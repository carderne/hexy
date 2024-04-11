use rocket::fs::{relative, FileServer};
use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{get, uri, Error, Ignite, Rocket};
use rocket_dyn_templates::{context, Template};
use rocket_okapi::settings::UrlObject;
use rocket_okapi::{openapi, openapi_get_routes, rapidoc::*};
use std::env;

use crate::models::{is_dt_past, ts_to_dt, Data, User, UserDb};
use crate::{data, db, h3, strava};

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
        logged_out,
        get_data,
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
fn authed_index(user: User) -> Template {
    let User { id } = user;
    let os_key = env::var("OS_KEY").unwrap();
    Template::render("map", context! { id, os_key })
}

#[openapi(skip)]
#[get("/data")]
async fn get_data(user: User) -> Json<Data> {
    let User { id } = user;

    let user = db::get_user(id);
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
        db::save_user(&token_response);
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
        .map(|cell_index| format!("\"{:x}\"", cell_index))
        .collect();

    let activities = data::to_geojson(activities);
    Json(Data { activities: Some(activities), cells })
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
    let token_response = strava::get_token(code, strava::GrantType::Auth).await;
    db::save_user(&token_response);

    let mut c_id: Cookie = Cookie::new("id", token_response.athlete.id.to_string());
    // This happens after the OAuth flow and if SameSite::Strict
    // the cookies somehow dont get sent with the first / request
    // as there is some Strava/Google analytics stuff in between??
    c_id.set_same_site(SameSite::Lax);
    jar.add_private(c_id);

    Redirect::to(uri!(authed_index))
}

#[openapi(skip)]
#[get("/logout")]
fn logout(jar: &CookieJar<'_>) -> Redirect {
    jar.remove_private("id");
    Redirect::to(uri!(logged_out))
}

#[openapi(skip)]
#[get("/logged-out")]
async fn logged_out() -> Template {
    Template::render("logged-out", ())
}
