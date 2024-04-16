use anyhow::Context;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use log::{debug, warn};
use rocket::{Build, Rocket};
use rocket_sync_db_pools::database;

use crate::crypto::Crypto;
use crate::error;
use crate::models::UserDb;
use crate::schema::users::dsl::*;
use crate::{schema, strava};

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

pub async fn save_user(db: &Db, t: &strava::TokenResponse) -> Result<usize, error::Error> {
    let ref_token = Crypto::default().encrypt(&t.refresh_token);
    let user = UserDb {
        id: t.athlete.id,
        refresh_token: ref_token,
        access_token: t.access_token.clone(),
        expires_at: t.expires_at,
    };
    debug!("inserting user {}", t.athlete.id);
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
            .with_context(|| "db::get_user".to_string())
            .map_err(error::Error::from)
    })
    .await
}

pub async fn get_user(db: &Db, user_id: i32) -> Result<UserDb, error::Error> {
    let user = db
        .run(move |c| {
            users
                .find(user_id)
                .select(UserDb::as_select())
                .first(c)
                .with_context(|| "db::get_user".to_string())
                .map_err(error::Error::from)
        })
        .await?;
    let ref_token = Crypto::default().decrypt_fallback(&user.refresh_token);
    let user = UserDb {
        id: user.id,
        access_token: user.access_token,
        refresh_token: ref_token,
        expires_at: user.expires_at,
    };
    Ok(user)
}

/// These pragmas hopefully prevent the DB from locking up
/// Source: https://github.com/the-lean-crate/criner/issues/1
pub async fn prep_db(db: &Db) -> Result<(), error::Error> {
    db.run(|c| {
        c.batch_execute(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA wal_autocheckpoint = 100;
            PRAGMA wal_checkpoint(TRUNCATE);
        ",
        )
        .map_err(|err| {
            warn!("Failed to prep db");
            error::Error::from(err)
        })
    })
    .await
}
