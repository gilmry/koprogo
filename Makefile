.PHONY: help dev dev-all dev-frontend test test-unit test-integration test-bdd test-e2e-backend test-e2e-install test-e2e-full test-e2e-ui test-e2e-headed test-e2e-debug test-e2e-report bench coverage lint format build clean install install-all setup docker-up docker-down docker-build docker-logs migrate seed docs audit

help: ## Show this help
	@grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

dev: ## Start development environment
	docker-compose up -d postgres
	cd backend && cargo watch -x run

dev-all: ## Start all services with Docker Compose
	docker-compose up

dev-frontend: ## Start frontend development server
	cd frontend && npm run dev

test: ## Run all tests (backend + frontend E2E)
	cd backend && cargo test --lib
	cd backend && cargo test --tests
	$(MAKE) test-e2e-full

test-unit: ## Run unit tests only
	cd backend && cargo test --lib

test-integration: ## Run integration tests
	cd backend && cargo test --tests integration

test-bdd: ## Run BDD tests
	cd backend && cargo test --test bdd

test-e2e-backend: ## Run backend E2E tests (Rust only)
	cd backend && cargo test --test e2e

# Frontend E2E Tests with Playwright (with video documentation!)
test-e2e-install: ## Install Playwright browsers (run once)
	cd frontend && npm run test:install

test-e2e-full: ## Run full E2E tests (Frontend + Backend) - Generates videos!
	@echo "🎬 Running E2E tests with video documentation..."
	cd frontend && npm run test:e2e

test-e2e-ui: ## Run E2E tests in UI mode (interactive)
	@echo "🎨 Opening Playwright UI..."
	cd frontend && npm run test:e2e:ui

test-e2e-headed: ## Run E2E tests in headed mode (see browser)
	@echo "👀 Running tests with visible browser..."
	cd frontend && npm run test:e2e:headed

test-e2e-debug: ## Run E2E tests in debug mode (step by step)
	@echo "🐛 Starting debug mode..."
	cd frontend && npm run test:e2e:debug

test-e2e-report: ## Open E2E test report with videos
	@echo "📊 Opening test report with videos..."
	cd frontend && npm run test:e2e:report

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
	cd frontend && rm -rf dist node_modules test-results playwright-report

install: ## Install all dependencies
	cd frontend && npm install

install-all: ## Install all dependencies including Playwright
	$(MAKE) install
	$(MAKE) test-e2e-install

setup: ## Complete project setup (dependencies + migrations + playwright)
	@echo "🚀 Setting up KoproGo..."
	@echo "📦 Installing frontend dependencies..."
	cd frontend && npm install
	@echo "🎭 Installing Playwright browsers..."
	cd frontend && npm run test:install
	@echo "🐘 Starting PostgreSQL..."
	docker-compose up -d postgres
	sleep 5
	@echo "🗄️  Running database migrations..."
	cd backend && sqlx migrate run
	@echo "✅ Setup complete! Run 'make dev' to start development."

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
