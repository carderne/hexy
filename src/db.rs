use diesel::prelude::*;
use diesel::{Connection, SqliteConnection};
use std::env;

use crate::{models, schema, strava};

fn establish_connection() -> SqliteConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn save_user(t: strava::TokenResponse) {
    let user = models::User {
        id: t.athlete.id,
        refresh_token: t.refresh_token,
        access_token: t.access_token,
        expires_at: t.expires_at,
    };

    let conn = &mut establish_connection();
    diesel::insert_into(schema::users::table)
        .values(&user)
        .execute(conn)
        .expect("Error saving new post");
}

pub fn get_user(user_id: i32) -> models::User {
    use self::schema::users::dsl::*;
    let conn = &mut establish_connection();
    users
        .find(user_id)
        .select(models::User::as_select())
        .first(conn)
        .unwrap()
}
