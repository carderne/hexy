use dotenvy::dotenv;

use hexy::routes;

#[rocket::main]
async fn main() {
    dotenv().ok();
    let launch_result = routes::build().await;
    match launch_result {
        Ok(_) => println!("Rocket shut down gracefully."),
        Err(err) => println!("Rocket had an error: {}", err),
    };
}
