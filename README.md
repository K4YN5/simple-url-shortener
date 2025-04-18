# rust url shortener

It's a url shortener made with rust.

## Libraries used

- Rust
- Axum
- Tokio
- SQLx
- SQLite
- mini-moka

## How to run

Make sure rust is installed.

`cargo run`

Server starts on port `3000`. It makes a data folder on pwd with a `db.sqlite` file.

## How to use

POST / with json like {"http://some-url.com"}. Gives back a short id.

GET /{id} redirects to the original url.

GET /length tells you how many urls are stored.

That's pretty much it. It's for learning async rust and simple architecture mostly.
