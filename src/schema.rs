// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Integer,
        refresh_token -> Text,
        access_token -> Text,
        expires_at -> Integer,
    }
}
