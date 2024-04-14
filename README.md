# hexy
A simple webapp that loads your Strava data, displays it on a map, and let you fill in hexagons with your activities.

Try it out: [hexy.rdrn.me](https://hexy.rdrn.me)

<img width="1094" alt="Screenshot 2024-04-13 at 21 24 17" src="https://github.com/carderne/hexy/assets/19817302/bd6acec8-2d4d-436e-947e-a0dc852a86a7">

## Stack
Rust API with the following bits:
- `rocket`: web framework, partially chosen for OpenAPI compatibility with `okapi` but I've ripped that out anyway. Easy HTML templating and slightly-too-clever db pool.
- `diesel`: db queries to on-disk SQLite. Will probably use `sqlx` next time because why not. Like diesel's in-process migrations!
- `geo` with `h3o`, `polyline`, `dbscan` for the fun geo bits!
- `serde`, `reqwest`, `chrono`, `anyhow` for the usual.

Bit of plain HTML, JavaScript and Tailwind for the frontend.
I'd like to find a nice setup for using compiled TypeScript for the frontend (React or Svelte) together with a backend-first (single server, lots of HTTP) compiled language approach but I haven't found it yet...

Running on Render.com with SQLite DB on a persistent disk.

## Todo
- [ ] Write some tests
- [ ] Compact hexagons into bigger ones
- [ ] Maybe squares are just better?
- [ ] Improve hexagon styling
- [x] Highlight selected activity

## Development
You'll need to create a `.env` file with the following:
```bash
ROCKET_DATABASES='{db={url="db.sqlite"}}'
ROCKET_SECRET_KEY=''
REDIRECT_URI='http://localhost:8000/callback'

STRAVA_BASE='https://www.strava.com'
STRAVA_CLIENT_ID=''
STRAVA_CLIENT_SECRET=''

OS_KEY=''
```

The usual:
```
cargo fmt
      check
      clippy
      build
      test
      run
```

Once running, go to [localhost:8000](http://localhost:8000) and follow the prompts.
