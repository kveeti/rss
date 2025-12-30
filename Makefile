# Export variables from ./.env if it exists
ifneq (,$(wildcard ./.env))
	include .env
	export
endif

.PHONY: all
MAKEFLAGS += -j

frontdev:
	@cd frontend && pnpm run dev
backdev:
	@cd backend && cargo watch -x run
dev: backdev frontdev

frontbuild:
	@cd frontend && pnpm run build
backbuild:
	@cd backend && cargo build --release
build: backbuild frontbuild

frontpreview:
	@cd frontend && pnpm run build && pnpm run preview
backpreview:
	@cd backend && cargo run --release
preview: backpreview frontpreview

dbreset:
	@cargo sqlx db reset --force --source ./backend/src/db/migrations

dbnewmigration:
	@cargo sqlx migrate add --source ./backend/db/migrations
