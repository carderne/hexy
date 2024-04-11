CREATE TABLE users (
  id            INTEGER PRIMARY KEY NOT NULL,
  refresh_token TEXT    NOT NULL,
  access_token  TEXT    NOT NULL,
  expires_at    INTEGER NOT NULL
);
