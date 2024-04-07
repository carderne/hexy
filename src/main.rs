use dotenvy::dotenv;
use geojson::GeoJson;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{get, launch, routes, uri};

use hexy::{geo, strava};

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    rocket::build().mount("/", routes![health, index, user, auth, exchange])
}

#[get("/health")]
fn health() -> &'static str {
    "ok"
}

#[get("/")]
fn index() -> Redirect {
    Redirect::to(uri!(auth()))
}

#[get("/auth")]
fn auth() -> Redirect {
    let url = strava::create_oauth_url().unwrap();
    Redirect::to(url)
}

#[get("/exchange?<code>")]
async fn exchange(code: &str) -> Redirect {
    let t = strava::get_token(code).await;
    let redir_url = uri!(user(t.athlete.id, &t.access_token));
    // Not saving users, rather just redirect with param
    // until I implement sessions and whatnot
    // db::save_user(t);
    Redirect::to(redir_url)
}

#[get("/user/<id>?<access_token>")]
async fn user(id: i32, access_token: &str) -> Json<GeoJson> {
    // Should get user from db/session
    // but for now just using code directly
    // let user = db::get_user(id);
    let activities = strava::get_activities(access_token).await;

    println!("get activities for user {}", id);
    Json(geo::decode_all(activities))
}
