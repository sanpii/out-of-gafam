# Out Of GAFAM

[GAFAM](https://fr.wikipedia.org/wiki/GAFAM) try to enclose you in their gilded
prison, mainly by removing the web foundations: interoperability.

This project help you to evade from its prisons by allowing you to keep
up to date about new contents without account, just with a good old RSS feed.


[<img title="Home" src="https://raw.githubusercontent.com/sanpii/out-of-gafam/master/screenshots/home.png" width="300px" />](https://raw.githubusercontent.com/sanpii/out-of-gafam/master/screenshots/home.png)
[<img title="User page" src="https://raw.githubusercontent.com/sanpii/out-of-gafam/master/screenshots/page.png" width="300px" />](https://raw.githubusercontent.com/sanpii/out-of-gafam/master/screenshots/page.png)
[<img title="Feed" src="https://raw.githubusercontent.com/sanpii/out-of-gafam/master/screenshots/feed.png" width="300px" />](https://raw.githubusercontent.com/sanpii/out-of-gafam/master/screenshots/feed.png)

## Compilation

```
make
```

## Installation

Create a new PostgreSQL database:

```
createdb oog
psql -f src/sql/structure.sql oog
```

Here an example of systemd service:

```
[Unit]
Description=out of gafam service

[Service]
ExecStart=/home/git/public_repositories/out-of-gafam/current/target/release/oog
WorkingDirectory=/home/git/public_repositories/out-of-gafam/current
Restart=on-failure
Environment="LISTEN_IP=127.0.0.1"
Environment="LISTEN_PORT=8000"
Environment="DATABASE_URL=postgresql://localhost/oog"
Environment="RUST_LOG=warn"
```
