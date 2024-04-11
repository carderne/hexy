use geojson::GeoJson;
use rocket::http::Status;
use rocket::request::Outcome;
use rocket::request::{FromRequest, Request};
use serde::Serialize;

#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub token: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<User, Self::Error> {
        let jar = request.cookies();
        let id = jar
            .get_private("id")
            .and_then(|cookie| cookie.value().parse::<i32>().ok());
        let token = jar
            .get_private("token")
            .map(|cookie| cookie.value().to_string());
        match (id, token) {
            (Some(id), Some(token)) => Outcome::Success(User { id, token }),
            _ => Outcome::Forward(Status::Unauthorized),
        }
    }
}

#[derive(Serialize)]
pub struct Data {
    pub activities: GeoJson,
    pub cells: Vec<String>,
}
