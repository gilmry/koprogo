# Installation Guide - KoproGo Grid

## Quick Start (Docker - Recommended)

The easiest way to get started is using Docker, which handles all dependencies:

```bash
# 1. Clone and navigate
cd koprogo-grid

# 2. Build Docker images
make docker-build

# 3. Start all services (PostgreSQL + Grid Server)
make docker-up

# 4. Check logs
docker-compose logs -f grid-server
```

The grid server will be available at `http://localhost:8081`.

---

## Local Development Setup

### Prerequisites

- **Rust** 1.83+ ([install](https://rustup.rs/))
- **PostgreSQL** 15+
- **SQLx CLI** (for migrations)

### Step 1: Install Dependencies

```bash
# Install SQLx CLI
cargo install sqlx-cli --no-default-features --features postgres
```

### Step 2: Database Setup

```bash
# Start PostgreSQL (Docker)
docker run -d \
  --name koprogo-grid-postgres \
  -e POSTGRES_DB=koprogo_grid \
  -e POSTGRES_USER=koprogo \
  -e POSTGRES_PASSWORD=koprogo123 \
  -p 5432:5432 \
  postgres:15-alpine

# OR install PostgreSQL locally
# Ubuntu/Debian:
sudo apt install postgresql-15

# macOS:
brew install postgresql@15
```

### Step 3: Configure Environment

```bash
# Copy environment template
cp .env.example .env

# Edit .env and set:
# DATABASE_URL=postgresql://koprogo:koprogo123@localhost:5432/koprogo_grid
```

### Step 4: Run Migrations

```bash
# Prepare SQLx for offline compilation (optional)
cargo sqlx prepare

# Run migrations
sqlx migrate run
# OR: make migrate
```

### Step 5: Build

```bash
# Build all binaries
cargo build --release

# Binaries will be in:
# - target/release/koprogo-grid-server
# - target/release/koprogo-grid-node
```

### Step 6: Run

```bash
# Start the server
cargo run --bin koprogo-grid-server

# OR use the Makefile
make run
```

---

## Compilation Without Database (Offline Mode)

If you encounter SQLx compilation errors and don't have a database running:

### Option 1: Use Docker Build

```bash
# Docker handles everything
make docker-build
```

### Option 2: Generate SQLx Cache

```bash
# 1. Start PostgreSQL
make db-up

# 2. Run migrations
make migrate

# 3. Generate offline cache
cargo sqlx prepare

# 4. Now you can build without DATABASE_URL
cargo build --release
```

The `sqlx-data.json` file will be created and cached for future builds.

---

## Edge Node (Raspberry Pi) Setup

### Cross-Compilation for ARM64 (Raspberry Pi)

```bash
# Install cross-compilation target
rustup target add aarch64-unknown-linux-gnu

# Build for ARM64
cargo build --profile edge --target aarch64-unknown-linux-gnu

# Binary: target/aarch64-unknown-linux-gnu/edge/koprogo-grid-node
```

### Direct Build on Raspberry Pi

```bash
# Install Rust on Raspberry Pi
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/gilmry/koprogo.git
cd koprogo/koprogo-grid

# Build (optimized for size)
make edge

# Install binary
sudo make install
```

### Docker on Raspberry Pi

```bash
# Pull pre-built image
docker pull koprogo/grid-node:latest

# Run node
docker run -d \
  --name grid-node \
  -e RUST_LOG=info \
  koprogo/grid-node:latest run \
    --server http://YOUR_SERVER_IP:8081 \
    --node-id YOUR_NODE_ID \
    --solar-watts 500
```

---

## Troubleshooting

### SQLx Compilation Errors

**Error**: `set DATABASE_URL to use query macros online`

**Solution**:
```bash
# Option 1: Set DATABASE_URL
export DATABASE_URL=postgresql://koprogo:koprogo123@localhost:5432/koprogo_grid

# Option 2: Use offline mode (after initial cargo sqlx prepare)
export SQLX_OFFLINE=true

# Option 3: Use Docker
make docker-build
```

### Database Connection Refused

**Error**: `Connection refused (os error 111)`

**Solution**:
```bash
# Check if PostgreSQL is running
docker ps | grep postgres

# Start if not running
make db-up

# Check logs
docker logs koprogo-grid-postgres
```

### Port Already in Use

**Error**: `Address already in use (os error 98)`

**Solution**:
```bash
# Find process using port 8081
lsof -i :8081

# Kill the process
kill -9 <PID>

# OR change the port in .env
SERVER_PORT=8082
```

---

## Verification

Test that everything is working:

```bash
# Health check
curl http://localhost:8081/grid/stats

# Expected response:
# {"nodes":{"total_nodes":0,"active_nodes":0,...},"tasks":{...}}
```

---

## Next Steps

1. **Register a node**: See [README.md](README.md#running-an-edge-node-raspberry-pi)
2. **Create tasks**: `POST /grid/task`
3. **Monitor stats**: `GET /grid/stats`
4. **Deploy production**: See [DEPLOYMENT.md](DEPLOYMENT.md) (coming soon)

---

## Support

- **Issues**: [github.com/gilmry/koprogo/issues](https://github.com/gilmry/koprogo/issues)
- **Docs**: [README.md](README.md)
- **Email**: grid@koprogo.coop
