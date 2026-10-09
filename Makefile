.DEFAULT_GOAL := help
.PHONY: help up down reset logs ps psql rebuild server worker migrate check

S ?=   # service for `make logs S=server`

help:
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "} {printf "  \033[36m%-10s\033[0m %s\n", $$1, $$2}'

up: ## Build and start postgres + server, wait until ready
	docker compose up -d --build --wait
	@docker compose ps

down: ## Stop containers, keep the database
	docker compose down

reset: ## Stop and DELETE the database volume, then start fresh
	@read -p "This deletes all local Flare data. Continue? [y/N] " ans && [ "$$ans" = y ] || [ "$$ans" = Y ]
	docker compose down -v
	docker compose up -d --build --wait

logs: ## Follow logs (all, or one service: make logs S=server)
	docker compose logs -f $(S)

ps: ## Show container status
	docker compose ps

psql: ## Open psql in the postgres container
	docker compose exec postgres psql -U flare -d flare

rebuild: ## Rebuild and restart only the server container
	docker compose up -d --build --no-deps --wait server

# --- native development --------------------------------------------------

server: ## Run the server natively (needs postgres: make up or docker compose up -d postgres)
	cargo run -p flare-cli -- server

worker: ## Run a worker natively against localhost:8080
	cargo run -p flare-cli -- worker --url http://localhost:8080

migrate: ## Run sqlx migrations against DATABASE_URL
	sqlx migrate run --source crates/flare-server/migrations

check: ## fmt + clippy + tests, same as CI
	cargo fmt --all --check
	cargo clippy --workspace --all-targets -- -D warnings
	cargo test --workspace
