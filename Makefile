.PHONY: help dev dev-all dev-frontend test test-unit test-integration test-bdd test-e2e-backend test-e2e-install test-e2e-full test-e2e-ui test-e2e-headed test-e2e-debug test-e2e-report bench coverage lint format build clean install install-all setup docker-up docker-down docker-build docker-logs migrate reset-db seed docs docs-build docs-serve docs-clean audit ci-local ci-check

help: ## Show this help
	@grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

dev: ## Start development environment
	docker-compose up -d postgres
	cd backend && cargo watch -x run

dev-all: ## Start all services with Docker Compose (development mode with hot-reload)
	@echo "🚀 Starting all services in development mode..."
	@echo "📝 Note: docker-compose.override.yml enables hot-reload"
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

reset-db: ## Reset database (WARNING: deletes all data)
	@echo "⚠️  WARNING: This will delete ALL database data!"
	@read -p "Are you sure? (type 'yes' to confirm): " confirm; \
	if [ "$$confirm" = "yes" ]; then \
		echo "🗑️  Stopping PostgreSQL..."; \
		docker-compose down postgres; \
		echo "🧹 Removing database volume..."; \
		docker volume rm koprogo_postgres_data 2>/dev/null || true; \
		echo "🚀 Starting fresh PostgreSQL..."; \
		docker-compose up -d postgres; \
		sleep 5; \
		echo "📊 Running migrations..."; \
		cd backend && sqlx migrate run; \
		echo "✅ Database reset complete! Restart server to create superadmin."; \
	else \
		echo "❌ Cancelled."; \
	fi

seed: ## Seed database with test data
	cd backend && cargo run --bin seed

docs: ## Generate Rust API documentation
	cd backend && cargo doc --no-deps --open

docs-build: ## Build Sphinx documentation
	@echo "📚 Building Sphinx documentation..."
	@if [ ! -d "docs/_build" ]; then \
		echo "Installing Sphinx dependencies..."; \
		pip3 install -r docs/requirements.txt; \
	fi
	cd docs && sphinx-build -b html . _build/html
	@echo "✅ Documentation built in docs/_build/html/index.html"

docs-serve: ## Serve documentation with live reload
	@echo "🔄 Starting documentation server with auto-reload..."
	@if ! command -v sphinx-autobuild &> /dev/null; then \
		echo "Installing sphinx-autobuild..."; \
		pip3 install sphinx-autobuild; \
	fi
	cd docs && sphinx-autobuild -b html . _build/html --port 8000 --open-browser

docs-clean: ## Clean generated documentation
	rm -rf docs/_build

audit: ## Security audit
	cd backend && cargo audit --ignore RUSTSEC-2023-0071
	cd frontend && npm audit --audit-level=moderate

ci-local: ## Test GitHub Actions workflows locally using act
	@echo "🎬 Testing CI workflows locally..."
	@if ! command -v act &> /dev/null; then \
		echo "❌ act is not installed. Install it from: https://github.com/nektos/act"; \
		echo "   macOS: brew install act"; \
		echo "   Linux: curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash"; \
		exit 1; \
	fi
	act -l
	@echo ""
	@echo "Run specific workflow:"
	@echo "  act -j lint                # Lint & Format Check"
	@echo "  act -j test-unit           # Unit Tests"
	@echo "  act -j test-bdd            # BDD Tests"
	@echo "  act -j test-e2e            # E2E Tests"
	@echo "  act -j frontend-check      # Frontend Check"
	@echo "  act -j cargo-audit         # Security Audit"

ci-check: ## Run all CI checks locally
	@echo "🔍 Running all CI checks locally..."
	@echo ""
	@echo "1️⃣ Formatting..."
	cd backend && cargo fmt --check || (echo "❌ Format check failed" && exit 1)
	@echo "✅ Formatting OK"
	@echo ""
	@echo "2️⃣ Linting..."
	cd backend && cargo clippy --all-targets --all-features -- -D warnings || (echo "❌ Clippy failed" && exit 1)
	@echo "✅ Clippy OK"
	@echo ""
	@echo "3️⃣ Unit tests..."
	cd backend && cargo test --lib || (echo "❌ Unit tests failed" && exit 1)
	@echo "✅ Unit tests OK"
	@echo ""
	@echo "4️⃣ BDD tests..."
	cd backend && cargo test --test bdd || (echo "❌ BDD tests failed" && exit 1)
	@echo "✅ BDD tests OK"
	@echo ""
	@echo "5️⃣ E2E tests..."
	cd backend && cargo test --test e2e || (echo "❌ E2E tests failed" && exit 1)
	@echo "✅ E2E tests OK"
	@echo ""
	@echo "6️⃣ Security audit..."
	cd backend && cargo audit --ignore RUSTSEC-2023-0071 || (echo "❌ Security audit failed" && exit 1)
	@echo "✅ Security audit OK"
	@echo ""
	@echo "7️⃣ Frontend build..."
	cd frontend && npm run build || (echo "❌ Frontend build failed" && exit 1)
	@echo "✅ Frontend build OK"
	@echo ""
	@echo "8️⃣ Frontend formatting..."
	cd frontend && npx prettier --check . || (echo "❌ Prettier check failed" && exit 1)
	@echo "✅ Prettier OK"
	@echo ""
	@echo "9️⃣ Frontend security..."
	cd frontend && npm audit --audit-level=moderate || (echo "❌ NPM audit failed" && exit 1)
	@echo "✅ NPM audit OK"
	@echo ""
	@echo "🎉 All CI checks passed! Ready to push."
