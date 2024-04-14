use dotenvy;
use rocket::{http::Status, local::blocking::Client};

use hexy::routes;

#[test]
fn test_create_user() {
    dotenvy::from_filename("test.env").ok();
    let s = routes::build(false);
    let client = Client::tracked(s).unwrap();
    let req = client.get("/auth");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::SeeOther);
}
