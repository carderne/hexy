use dotenvy::dotenv;
use rocket::fairing::AdHoc;
use rocket::fs::{relative, FileServer};
use rocket::{Error, Ignite, Rocket};
use rocket_dyn_templates::Template;

use hexy::{db, routes};

#[rocket::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    let launch_result = build().await;
    match launch_result {
        Ok(_) => println!("Rocket shut down gracefully."),
        Err(err) => println!("Rocket had an error: {}", err),
    };
}

async fn build() -> Result<Rocket<Ignite>, Error> {
    rocket::build()
        .attach(db::Db::fairing())
        .attach(AdHoc::try_on_ignite("Migrations", db::migrate))
        .attach(AdHoc::on_liftoff("Startup Check", |rocket| {
            Box::pin(async move {
                let d = db::Db::get_one(rocket).await.unwrap();
                db::prep_db(&d).await.expect("Failed to prep db");
            })
        }))
        .attach(Template::fairing())
        .mount("/static", FileServer::from(relative!("static")))
        .mount("/", routes::routes())
        .launch()
        .await
}
