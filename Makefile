default: setup dev

.PHONY: setup dev

setup:
	@echo "Setting up environment..."
	@docker compose down
	@rm -rf data
	@docker compose up -d
	@sleep 5
	@sqlx database setup

dev:
	cargo watch -x 'run --release'