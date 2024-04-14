use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;
use geo::{LineString, Point};
use geojson::GeoJson;
use geojson::{JsonObject, JsonValue};
use polyline;
use rocket::http::Status;
use rocket::request::Outcome;
use rocket::request::{FromRequest, Request};
use serde::Serialize;

use crate::strava::ActivityResponse;

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
    pub centroid: Option<Point>,
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

#[derive(Debug, PartialEq, Serialize)]
pub struct Activity {
    pub id: i64,
    pub name: String,
    pub distance: f64,
    pub moving_time: i64,
    pub elapsed_time: i64,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub kudos_count: i32,
    pub average_speed: f64,
    pub sport_type: String,
    pub linestring: Option<LineString>,
}

impl Activity {
    pub fn from_response(obj: ActivityResponse) -> Activity {
        let poly = obj.map.summary_polyline;
        let linestring = poly.map(|poly| polyline::decode_polyline(&poly, 5).unwrap());
        Activity {
            id: obj.id,
            name: obj.name,
            distance: obj.distance,
            moving_time: obj.moving_time,
            elapsed_time: obj.elapsed_time,
            start_date: obj.start_date,
            kudos_count: obj.kudos_count,
            average_speed: obj.average_speed,
            sport_type: obj.sport_type,
            linestring,
        }
    }
    pub fn to_properties(&self) -> Option<JsonObject> {
        let mut value = serde_json::to_value(self).unwrap();
        if let JsonValue::Object(ref mut obj) = value {
            obj.remove("linestring");
            Some(obj.clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::strava;
    use chrono::{NaiveDate, NaiveTime};

    use super::*;

    #[test]
    fn test_ts_to_dt() {
        let d = NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
        let t = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        let want = NaiveDateTime::new(d, t);
        let got = ts_to_dt(1711929600);
        assert_eq!(want, got);
    }

    #[test]
    fn test_is_dt_past() {
        let d = NaiveDate::from_ymd_opt(1980, 1, 1).unwrap();
        let t = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        let dt = NaiveDateTime::new(d, t);
        let want = true;
        let got = is_dt_past(dt);
        assert_eq!(want, got);
    }

    #[test]
    fn activity_from_response() {
        let d = NaiveDate::from_ymd_opt(1980, 1, 1).unwrap();
        let t = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        let dt = NaiveDateTime::new(d, t);
        let dt = dt.and_utc();
        let want = Activity {
            id: 0,
            name: "".to_string(),
            distance: 0.0,
            moving_time: 0,
            elapsed_time: 0,
            start_date: dt,
            kudos_count: 0,
            average_speed: 0.0,
            sport_type: "Ride".to_string(),
            linestring: None,
        };
        let map = strava::Map {
            summary_polyline: None,
        };
        let res = strava::ActivityResponse {
            id: 0,
            name: "".to_string(),
            distance: 0.0,
            moving_time: 0,
            elapsed_time: 0,
            start_date: dt,
            kudos_count: 0,
            average_speed: 0.0,
            sport_type: "Ride".to_string(),
            map,
        };
        let got = Activity::from_response(res);
        assert_eq!(want, got);
    }
}
