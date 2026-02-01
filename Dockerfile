FROM rust:1.92-alpine AS backend_base
RUN apk add --no-cache musl-dev
RUN rustup target add aarch64-unknown-linux-musl
WORKDIR /app
RUN cargo install cargo-chef


FROM backend_base AS backend_plan
COPY ./backend/Cargo.toml ./backend/Cargo.lock .
COPY ./backend/src src
COPY ./backend/.sqlx .sqlx
RUN --mount=type=cache,id=cargo_registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=cargo_target,target=./backend/target \
    cargo chef prepare --recipe-path recipe.json --bin backend


FROM backend_base AS backend_build
COPY --from=backend_plan /app/recipe.json recipe.json
RUN --mount=type=cache,id=cargo_registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=cargo_target,target=./backend/target \
    cargo chef cook --release --recipe-path recipe.json
COPY ./backend/Cargo.toml ./backend/Cargo.lock .
COPY ./backend/src src
COPY ./backend/.sqlx .sqlx
ENV SQLX_OFFLINE=true
RUN --mount=type=cache,id=cargo_registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=cargo_target,target=./backend/target \
    cargo build --release && \
    cp ./target/release/backend backend
RUN chmod +x backend


FROM node:25-slim AS frontend_build
WORKDIR /app
RUN npm uni -g pnpm yarn
RUN rm -rf /usr/local/bin/yarnpkg
RUN rm -rf /usr/local/bin/yarn
RUN npm i -g corepack
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
COPY ./frontend/package.json   package.json
COPY ./frontend/pnpm-lock.yaml pnpm-lock.yaml
RUN corepack enable
COPY ./frontend/.npmrc         .npmrc
RUN --mount=type=cache,id=pnpm,target=/pnpm/store \
    pnpm install
COPY ./frontend/index.html        index.html
COPY ./frontend/src               src
COPY ./frontend/public            public
COPY ./frontend/postcss.config.js postcss.config.js
COPY ./frontend/tsconfig.json     tsconfig.json
COPY ./frontend/vite.config.ts    vite.config.ts
RUN pnpm build


FROM scratch AS runtime
COPY --from=backend_build /app/backend /usr/local/bin/backend
COPY --from=frontend_build /app/dist /app/frontend
ENV FRONTEND_DIR=/app/frontend
EXPOSE 8000
ENTRYPOINT ["backend"]
