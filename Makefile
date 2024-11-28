default: setup dev

.PHONY: setup dev

setup:
	@echo "Setting up environment..."
	@rm -rf data
	@docker compose down
	@docker compose up -d
	# verify if postgres container is up
	@until docker compose exec postgres pg_isready; do sleep 1; done
	@sqlx database setup

dev:
	cargo watch -x 'run --release'