use crate::models::UserDb;
use crate::{schema, strava};
use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use rocket::{Build, Rocket};
use rocket_sync_db_pools::database;

#[database("db")]
pub struct Db(diesel::SqliteConnection);

// This macro from `diesel_migrations` defines an `embedded_migrations` module
// containing a function named `run`. This allows the example to be run and
// tested without any outside setup of the database.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub async fn migrate(rocket: Rocket<Build>) -> Result<Rocket<Build>, Rocket<Build>> {
    let db = Db::get_one(&rocket).await.expect("database connection");
    db.run(|conn| match conn.run_pending_migrations(MIGRATIONS) {
        Ok(_) => Ok(rocket),
        Err(e) => {
            println!("Failed to run database migrations: {:?}", e);
            Err(rocket)
        }
    })
    .await
}

pub async fn save_user(db: &Db, t: &strava::TokenResponse) {
    let user = UserDb {
        id: t.athlete.id,
        refresh_token: t.refresh_token.clone(),
        access_token: t.access_token.clone(),
        expires_at: t.expires_at,
    };
    db.run(move |c| {
        diesel::insert_into(schema::users::table)
            .values(&user)
            .on_conflict(schema::users::id)
            .do_update()
            .set((
                schema::users::refresh_token.eq(&user.refresh_token),
                schema::users::access_token.eq(&user.access_token),
                schema::users::expires_at.eq(user.expires_at),
            ))
            .execute(c)
            .expect("Error saving new post");
    })
    .await;
}

pub async fn get_user(db: &Db, user_id: i32) -> Option<UserDb> {
    use self::schema::users::dsl::*;
    db.run(move |c| {
        users
            .find(user_id)
            .select(UserDb::as_select())
            .first(c)
            .ok()
    })
    .await
}
