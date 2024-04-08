# hexy
Playing around with the Strava API

```bash
make lint
make
cargo run
```

Go to [localhost:8000](http://localhost:8000) and follow the OAuth flow.

Need the following in your `.env`:
```bash
DATABASE_URL=''
REDIRECT_URI='http://localhost:8000/exchange'

STRAVA_BASE='https://www.strava.com'
STRAVA_CLIENT_ID=''
STRAVA_CLIENT_SECRET=''

OS_KEY=''
```
