Simple feed of RSS/Atom feeds +
- Text search over entry titles and URLs
- Supports comments link when given, e.g. on Hackernews
- OPML import/export
- [More screenshots](#more-screenshots)

<table>
  <tr>
    <td><img src="https://github.com/kveeti/rss/raw/main/.readme_assets/unread-light.webp" alt="Light theme unread page showing unread Hackernews and Lobste.rs posts" width="460"></td>
    <td><img src="https://github.com/kveeti/rss/raw/main/.readme_assets/unread-dark.webp" alt="Dark theme unread page showing unread Hackernews and Lobste.rs posts" width="460"></td>
  </tr>
</table>

### Running locally

Example env vars in [`.env.example`](/.env.example). Both frontend and backend dev servers can read variables from .env files (frontend/.env and backend/.env)

With Nix:
- Enter the dev shell. It starts Postgres on a free port and sets `DATABASE_URL`
    ```bash
    nix develop
    ```
- The backend runs on port `8000` and the frontend on port `3000`

Backend:
- Db (postgres): e.g. with docker compose [`compose.yml`](/compose.yml)
    ```bash
    docker compose up -d
    ```
- Backend: Run backend using cargo. Db migrations are executed on startup
    ```bash
    cd backend
    cargo run
    ```

Frontend:
- Install deps using pnpm and run dev script
    ```bash
    cd frontend
    pnpm install
    pnpm run dev
    ```

[`Makefile`](/Makefile) has some commands using tools like [`cargo-watch`](https://github.com/watchexec/cargo-watch) for server auto restart and [`sqlx-cli`](https://github.com/launchbadge/sqlx/blob/e8384f2a00173c2b120eea72e99d120557fced8b/sqlx-cli/README.md) for db management. Those commands export vars from ./.env

Running locally with `make dev`:
```bash
# run db
docker compose up -d

# install cargo-watch
cargo install cargo-watch --locked

# copy example env vars
cp ./.env.example ./.env

# install frontend deps
cd frontend && pnpm install && cd -

# run backend and frontend
make dev
```

### More screenshots

<table>
  <tr><th align="left">Feeds page</th></tr>
  <tr><td><img src="https://github.com/kveeti/rss/raw/main/.readme_assets/feeds.webp" alt="Feeds page showing Hackernews and Lobste.rs as added feeds" width="460"></td></tr>
</table>

<table>
  <tr><th align="left">Entries page with search</th></tr>
  <tr><td><img src="https://github.com/kveeti/rss/raw/main/.readme_assets/entries.webp" alt="Entries page showing the search input, filters and results" width="460"></td></tr>
</table>

<table>
  <tr><th align="left">New feed page</th></tr>
  <tr><td><img src="https://github.com/kveeti/rss/raw/main/.readme_assets/new-feed.webp" alt="New feed page showing the feed form" width="460"></td></tr>
</table>
