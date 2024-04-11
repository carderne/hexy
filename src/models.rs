use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;
use geojson::GeoJson;
use rocket::http::Status;
use rocket::request::Outcome;
use rocket::request::{FromRequest, Request};
use serde::Serialize;

#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UserDb {
    pub id: i32,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i32,
}

pub struct User {
    pub id: i32,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<User, Self::Error> {
        let jar = request.cookies();
        let id = jar
            .get_private("id")
            .and_then(|cookie| cookie.value().parse::<i32>().ok());
        match id {
            Some(id) => Outcome::Success(User { id }),
            _ => Outcome::Forward(Status::Unauthorized),
        }
    }
}

#[derive(Serialize)]
pub struct Data {
    pub activities: Option<GeoJson>,
    pub cells: Vec<String>,
}

pub fn ts_to_dt(timestamp: i32) -> NaiveDateTime {
    DateTime::from_timestamp(timestamp as i64, 0)
        .unwrap()
        .naive_utc()
}

pub fn is_dt_past(datetime: NaiveDateTime) -> bool {
    let now_plus_one_hour = Utc::now().naive_local() + chrono::Duration::hours(1);
    datetime < now_plus_one_hour
}
