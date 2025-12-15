
### Running locally

Example env vars in [`.env.example`](/.env.example). Both frontend and backend dev servers can read variables from .env files (frontend/.env and backend/.env)

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
