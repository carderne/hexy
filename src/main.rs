use dotenvy::dotenv;

use hexy::routes::build;

#[rocket::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    let launch_result = build(true).launch().await;
    match launch_result {
        Ok(_) => println!("Rocket shut down gracefully."),
        Err(err) => println!("Rocket had an error: {}", err),
    };
}
