# KoproGo Grid Computing - Makefile
# Commands for building, testing, and running the grid computing system

.PHONY: help build run test clean docker-build docker-up docker-down edge install

# Default target
help:
	@echo "KoproGo Grid Computing - Available Commands"
	@echo ""
	@echo "Development:"
	@echo "  make build        - Build all binaries (server + node CLI)"
	@echo "  make run          - Run the grid server locally"
	@echo "  make test         - Run all tests"
	@echo "  make clean        - Clean build artifacts"
	@echo ""
	@echo "Docker:"
	@echo "  make docker-build - Build Docker images (server + node)"
	@echo "  make docker-up    - Start services with docker-compose"
	@echo "  make docker-down  - Stop services"
	@echo ""
	@echo "Edge Node:"
	@echo "  make edge         - Build edge node binary (optimized for Raspberry Pi)"
	@echo "  make install      - Install binaries to /usr/local/bin"
	@echo ""
	@echo "Database:"
	@echo "  make migrate      - Run database migrations"
	@echo "  make db-up        - Start PostgreSQL only"
	@echo ""

# Build all binaries
build:
	@echo "🔨 Building KoproGo Grid binaries..."
	cargo build --release

# Build edge node (optimized for size)
edge:
	@echo "🔨 Building edge node binary (optimized for Raspberry Pi)..."
	cargo build --profile edge --bin koprogo-grid-node
	@echo "✅ Edge binary: target/edge/koprogo-grid-node"
	@ls -lh target/edge/koprogo-grid-node

# Run the grid server locally
run:
	@echo "🚀 Starting KoproGo Grid server..."
	cargo run --bin koprogo-grid-server

# Run all tests
test:
	@echo "🧪 Running tests..."
	cargo test --lib
	cargo test --test integration_test

# Run tests with coverage
coverage:
	@echo "📊 Running tests with coverage..."
	cargo tarpaulin --out Html --output-dir coverage

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# Build Docker images
docker-build:
	@echo "🐳 Building Docker images..."
	docker build -f Dockerfile.server -t koprogo-grid-server:latest .
	docker build -f Dockerfile.node -t koprogo-grid-node:latest .

# Start services with docker-compose
docker-up:
	@echo "🐳 Starting services with docker-compose..."
	docker-compose up -d
	@echo "✅ Services started!"
	@echo "   Grid API: http://localhost:8081"
	@echo "   PostgreSQL: localhost:5432"

# Stop services
docker-down:
	@echo "🐳 Stopping services..."
	docker-compose down

# Start PostgreSQL only
db-up:
	@echo "🐘 Starting PostgreSQL..."
	docker-compose up -d postgres

# Run database migrations
migrate: db-up
	@echo "📦 Running database migrations..."
	@sleep 2  # Wait for PostgreSQL to be ready
	cargo sqlx migrate run

# Install binaries to system
install: build
	@echo "📦 Installing binaries to /usr/local/bin..."
	sudo cp target/release/koprogo-grid-server /usr/local/bin/
	sudo cp target/release/koprogo-grid-node /usr/local/bin/
	@echo "✅ Binaries installed!"

# Format code
fmt:
	@echo "🎨 Formatting code..."
	cargo fmt

# Lint code
lint:
	@echo "🔍 Linting code..."
	cargo clippy -- -D warnings

# Check code without building
check:
	@echo "🔍 Checking code..."
	cargo check

# Development: watch for changes and rebuild
watch:
	@echo "👀 Watching for changes..."
	cargo watch -x run

# Example: Register a node
example-register:
	@echo "📝 Registering example node..."
	cargo run --bin koprogo-grid-node -- register \
		--server http://localhost:8081 \
		--name "Example-Node" \
		--cores 4 \
		--solar \
		--location "Brussels"

# Example: Run a node worker
example-worker:
	@echo "⚡ Running example worker (use your own node ID)..."
	@echo "Usage: make example-worker NODE_ID=<uuid>"
	@if [ -z "$(NODE_ID)" ]; then \
		echo "❌ Error: NODE_ID not set"; \
		echo "Example: make example-worker NODE_ID=123e4567-e89b-12d3-a456-426614174000"; \
		exit 1; \
	fi
	cargo run --bin koprogo-grid-node -- run \
		--server http://localhost:8081 \
		--node-id $(NODE_ID) \
		--solar-watts 500
