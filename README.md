# hexy
Playing around with the Strava API

```bash
cargo fmt
cargo clippy
cargo check
cargo build
cargo run
```

Go to [localhost:8000](http://localhost:8000) and follow the OAuth flow.

API docs at [localhost:8000/rapidoc](http://localhost:8000/rapidoc).

Need the following in your `.env`:
```bash
DATABASE_URL=''
REDIRECT_URI='http://localhost:8000/callback'

STRAVA_BASE='https://www.strava.com'
STRAVA_CLIENT_ID=''
STRAVA_CLIENT_SECRET=''

OS_KEY=''
```
