.PHONY: help dev test build clean docker-up docker-down

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

dev: ## Start development environment
	docker-compose up -d postgres
	cd backend && cargo watch -x run

dev-all: ## Start all services with Docker Compose
	docker-compose up

test: ## Run all tests
	cd backend && cargo test
	cd backend && cargo test --test '*'

test-unit: ## Run unit tests only
	cd backend && cargo test --lib

test-integration: ## Run integration tests
	cd backend && cargo test --test integration

test-bdd: ## Run BDD tests
	cd backend && cargo test --test bdd

test-e2e: ## Run E2E tests
	cd backend && cargo test --test e2e

bench: ## Run benchmarks
	cd backend && cargo bench

coverage: ## Generate test coverage report
	cd backend && cargo tarpaulin --out Html --output-dir ../coverage

lint: ## Run linters
	cd backend && cargo fmt --check
	cd backend && cargo clippy -- -D warnings
	cd frontend && npm run build

format: ## Format code
	cd backend && cargo fmt
	cd frontend && npm run format

build: ## Build all services
	cd backend && cargo build --release
	cd frontend && npm run build

clean: ## Clean build artifacts
	cd backend && cargo clean
	cd frontend && rm -rf dist node_modules

docker-up: ## Start Docker services
	docker-compose up -d

docker-down: ## Stop Docker services
	docker-compose down

docker-build: ## Build Docker images
	docker-compose build

docker-logs: ## Show Docker logs
	docker-compose logs -f

migrate: ## Run database migrations
	cd backend && sqlx migrate run

seed: ## Seed database with test data
	cd backend && cargo run --bin seed

docs: ## Generate documentation
	cd backend && cargo doc --no-deps --open

audit: ## Security audit
	cd backend && cargo audit
	cd frontend && npm audit
