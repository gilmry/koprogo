# KoproGo Grid Computing

🌱 **Decentralized Green Grid Computing for the KoproGo Cooperative**

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.83%2B-orange.svg)](https://www.rust-lang.org)
[![Architecture](https://img.shields.io/badge/architecture-hexagonal-green.svg)](docs/ARCHITECTURE.md)

## 📖 Overview

KoproGo Grid is an open-source, decentralized grid computing system that enables communities to pool computational resources while prioritizing **green energy** and **ecological sustainability**. Built with Rust and hexagonal architecture, it's designed to run on edge devices like Raspberry Pi with minimal carbon footprint.

### Key Features

- ✅ **Decentralized Computing**: Distribute tasks across community-owned nodes
- 🌱 **Proof of Green Blockchain**: Lightweight blockchain validating green energy contributions
- ⚡ **Solar-Powered Nodes**: Prioritize nodes with solar energy surplus
- 💰 **Carbon Credits**: Automatic calculation and fair distribution (70% node, 30% cooperative)
- 🔒 **Hexagonal Architecture**: Clean separation of domain, application, and infrastructure
- 🚀 **Edge-Optimized**: Binary size < 10MB, memory usage < 50MB
- 📊 **PostgreSQL Storage**: All data persisted with full ACID guarantees
- 🔐 **Secure by Design**: Non-root containers, minimal dependencies

### Carbon Impact

- **Target**: < 0.01g CO₂ per task
- **Belgian Grid Offset**: 0.18 kg CO₂/kWh avoided when using solar
- **Cooperative Fund**: 30% of carbon credits fund community initiatives

---

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.83+ ([install](https://rustup.rs/))
- **PostgreSQL** 15+ (or use Docker)
- **Docker** (optional, for containerized deployment)

### Installation

```bash
# Clone the repository
git clone https://github.com/gilmry/koprogo.git
cd koprogo/koprogo-grid

# Copy environment file
cp .env.example .env

# Start PostgreSQL (Docker)
make db-up

# Run migrations
make migrate

# Build binaries
make build
```

### Running the Server

```bash
# Start the grid server
make run

# Or with Docker
make docker-up
```

The server will be available at `http://localhost:8081`.

---

## 🖥️ Running an Edge Node (Raspberry Pi)

### 1. Register Your Node

```bash
cargo run --bin koprogo-grid-node -- register \
  --server http://localhost:8081 \
  --name "MyRaspberryPi" \
  --cores 4 \
  --solar \
  --location "Brussels"
```

This will output a **Node ID** (UUID). Save it!

### 2. Start the Worker

```bash
cargo run --bin koprogo-grid-node -- run \
  --server http://localhost:8081 \
  --node-id YOUR_NODE_ID \
  --solar-watts 500 \
  --interval 30
```

### 3. Monitor Your Contributions

```bash
# Check grid stats
curl http://localhost:8081/grid/stats | jq
```

### Docker Deployment (Raspberry Pi)

```bash
# Build edge-optimized image
docker build -f Dockerfile.node -t koprogo-grid-node:edge .

# Run node
docker run -e RUST_LOG=info koprogo-grid-node:edge run \
  --server http://your-grid-server:8081 \
  --node-id YOUR_NODE_ID \
  --solar-watts 500
```

---

## 📡 API Documentation

### Base URL

```
http://localhost:8081/grid
```

### Endpoints

#### `POST /grid/register`

Register a new node with the grid.

**Request:**
```json
{
  "name": "RaspberryPi-001",
  "cpu_cores": 4,
  "has_solar": true,
  "location": "Brussels"
}
```

**Response:** `201 Created`
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "RaspberryPi-001",
  "eco_score": 0.0,
  "status": "active"
}
```

---

#### `POST /grid/heartbeat`

Send node heartbeat with CPU and solar data.

**Request:**
```json
{
  "node_id": "550e8400-e29b-41d4-a716-446655440000",
  "cpu_usage": 25.5,
  "solar_watts": 450
}
```

**Response:** `200 OK`
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "eco_score": 0.65
}
```

---

#### `GET /grid/task?node_id=<uuid>`

Fetch the next available task for a node.

**Response:** `200 OK`
```json
{
  "id": "task-uuid",
  "task_type": "ml_train",
  "data_url": "s3://bucket/data.csv",
  "deadline": "2025-11-15T14:30:00Z",
  "estimated_reward": 0.05
}
```

**Response (no tasks):** `204 No Content`

---

#### `POST /grid/report`

Report task completion with results.

**Request:**
```json
{
  "task_id": "task-uuid",
  "result_hash": "abc123def456",
  "energy_used_wh": 12.5,
  "solar_contribution_wh": 8.0
}
```

**Response:** `200 OK`
```json
{
  "message": "Task completed successfully",
  "carbon_credits": 0.00144,
  "node_share_eur": 0.0000252,
  "cooperative_share_eur": 0.0000108
}
```

---

#### `GET /grid/stats`

Get overall grid statistics.

**Response:** `200 OK`
```json
{
  "nodes": {
    "total_nodes": 42,
    "active_nodes": 38,
    "total_cpu_cores": 168,
    "nodes_with_solar": 29,
    "total_energy_saved_wh": 12500.5,
    "total_carbon_credits": 2.25
  },
  "tasks": {
    "total_tasks": 1523,
    "pending_tasks": 12,
    "completed_tasks": 1489,
    "failed_tasks": 22
  },
  "cooperative_fund_eur": 0.016875
}
```

---

## 🏗️ Architecture

KoproGo Grid follows **Hexagonal Architecture** (Ports & Adapters):

```
┌─────────────────────────────────────────┐
│           Core Domain                   │
│  ┌─────────────────────────────────┐   │
│  │  Node, Task, GreenProof         │   │
│  │  CarbonCredit                    │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
                   ▲
                   │
┌─────────────────────────────────────────┐
│        Application Layer (Ports)        │
│  ┌─────────────────────────────────┐   │
│  │  NodeRepository                  │   │
│  │  TaskDistributor                 │   │
│  │  GreenProofRepository            │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
                   ▲
                   │
┌─────────────────────────────────────────┐
│    Infrastructure (Adapters)            │
│  ┌──────────┐  ┌──────────┐            │
│  │ Actix-Web│  │PostgreSQL│            │
│  │   API    │  │   SQLx   │            │
│  └──────────┘  └──────────┘            │
└─────────────────────────────────────────┘
```

### Technology Stack

- **Language**: Rust 1.83+
- **Web Framework**: Actix-Web 4.9
- **Database**: PostgreSQL 15 via SQLx 0.8
- **Blockchain**: Lightweight SHA-256 Proof of Work
- **Testing**: Unit, Integration, Testcontainers
- **Deployment**: Docker, Kubernetes-ready

---

## 🧪 Testing

```bash
# Run all tests
make test

# Run with coverage
make coverage

# Lint
make lint

# Format
make fmt
```

### Test Coverage

- ✅ **Domain Logic**: 100% coverage (enforced)
- ✅ **Integration**: PostgreSQL testcontainers
- ✅ **E2E**: Full API workflow tests

---

## 🌍 Carbon Credits & Economics

### How It Works

1. **Task Execution**: Node completes a computational task
2. **Energy Measurement**: Records total energy (Wh) and solar contribution
3. **Carbon Calculation**: `carbon_saved_kg = (solar_wh / 1000) * 0.18`
   - Belgian grid: 0.18 kg CO₂/kWh
4. **Credit Valuation**: `euro_value = carbon_saved_kg * €0.025`
   - Market rate: ~€25/ton CO₂
5. **Distribution**:
   - **70%** to node owner
   - **30%** to cooperative solidarity fund

### Example Calculation

```
Task: ML Training (100 Wh total, 60 Wh solar)

Carbon Saved:
  (60 / 1000) * 0.18 = 0.0108 kg CO₂

Euro Value:
  0.0108 * 0.025 = €0.00027

Distribution:
  Node:        €0.000189 (70%)
  Cooperative: €0.000081 (30%)
```

### Cooperative Fund Use

The 30% cooperative share funds:
- 🌱 **Green infrastructure** (solar panels for members)
- 💡 **Community projects** (open-source development)
- 🤝 **Solidarity initiatives** (digital inclusion)

---

## 🛠️ Development

### Project Structure

```
koprogo-grid/
├── src/
│   ├── core/                   # Domain entities
│   │   ├── node.rs
│   │   ├── task.rs
│   │   ├── green_proof.rs
│   │   └── carbon_credit.rs
│   ├── ports/                  # Trait definitions
│   │   ├── node_repository.rs
│   │   ├── task_distributor.rs
│   │   └── ...
│   ├── adapters/
│   │   ├── postgres/           # PostgreSQL implementations
│   │   └── actix/              # HTTP API handlers
│   ├── bin/
│   │   └── node_cli.rs         # Edge node CLI
│   ├── lib.rs
│   └── main.rs                 # Server entrypoint
├── tests/
│   └── integration_test.rs
├── migrations/
│   └── 20250115000000_create_grid_tables.sql
├── Dockerfile.server
├── Dockerfile.node
├── docker-compose.yml
├── Makefile
└── README.md
```

### Makefile Commands

```bash
make help           # Show all commands
make build          # Build release binaries
make run            # Run server
make test           # Run tests
make docker-build   # Build Docker images
make docker-up      # Start with docker-compose
make edge           # Build edge-optimized binary
make install        # Install to /usr/local/bin
```

---

## 🔐 Security

- **Non-root containers**: All services run as unprivileged user
- **Input validation**: Domain entities enforce business invariants
- **SQL injection prevention**: SQLx compile-time query verification
- **Minimal dependencies**: < 30 direct dependencies
- **HTTPS**: Use reverse proxy (Traefik, Nginx) in production

### Production Checklist

- [ ] Use TLS/HTTPS (reverse proxy)
- [ ] Set strong `POSTGRES_PASSWORD`
- [ ] Enable PostgreSQL SSL mode
- [ ] Configure firewall (only expose port 8081)
- [ ] Regular security audits (`cargo audit`)
- [ ] Monitor logs (Loki, ELK)

---

## 📊 Performance

### Benchmarks (Raspberry Pi 4B)

| Metric                  | Value        |
|-------------------------|--------------|
| Binary Size (server)    | ~8.5 MB      |
| Binary Size (node)      | ~6.2 MB      |
| Memory (server)         | ~35 MB       |
| Memory (node)           | ~18 MB       |
| Latency (P99)           | < 5ms        |
| Tasks/sec (single node) | ~10          |

### Optimization Profiles

```toml
# Release (server)
[profile.release]
opt-level = 3
lto = true
codegen-units = 1

# Edge (Raspberry Pi)
[profile.edge]
opt-level = "z"      # Optimize for size
lto = "fat"
panic = "abort"
strip = true
```

---

## 🤝 Contributing

We welcome contributions! KoproGo Grid is part of the **KoproGo Cooperative** ecosystem.

### How to Contribute

1. **Fork** the repository
2. Create a **feature branch** (`git checkout -b feature/amazing-feature`)
3. **Commit** your changes (`git commit -m 'Add amazing feature'`)
4. **Push** to the branch (`git push origin feature/amazing-feature`)
5. Open a **Pull Request**

### Contribution Areas

- 🔧 Core features (task scheduling, blockchain optimization)
- 🧪 Testing (BDD scenarios, stress tests)
- 📚 Documentation (API docs, tutorials)
- 🌍 Translations (i18n support)
- 🐛 Bug reports and fixes

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

---

## 📜 License

This project is licensed under the **AGPL-3.0-or-later** license.

**Key points:**
- ✅ **Freedom to use** for any purpose
- ✅ **Freedom to study** and modify
- ✅ **Freedom to distribute** copies
- ✅ **Copyleft**: Modifications must remain AGPL
- ⚠️ **Network use = distribution**: SaaS deployments must share source

See [LICENSE](../LICENSE) for full text.

---

## 🙏 Acknowledgments

- **Rust Community**: For an incredible language and ecosystem
- **KoproGo Members**: For supporting cooperative innovation
- **Open Source Contributors**: For building the tools we depend on
- **Renewable Energy Advocates**: For inspiring green computing

---

## 📬 Contact & Support

- **Website**: [koprogo.coop](https://koprogo.coop)
- **GitHub Issues**: [github.com/gilmry/koprogo/issues](https://github.com/gilmry/koprogo/issues)
- **Email**: grid@koprogo.coop
- **Matrix**: `#koprogo-grid:matrix.org`

---

## 🗺️ Roadmap

### Phase 1: MVP (Q1 2025) ✅
- [x] Core domain entities
- [x] PostgreSQL repositories
- [x] Proof of Green blockchain
- [x] REST API
- [x] Node CLI
- [x] Docker deployment

### Phase 2: Production (Q2 2025)
- [ ] JWT authentication
- [ ] Task prioritization algorithm
- [ ] WebSocket real-time updates
- [ ] Grafana dashboard
- [ ] Kubernetes Helm charts
- [ ] Multi-region support

### Phase 3: Scale (Q3-Q4 2025)
- [ ] ScyllaDB migration (multi-datacenter)
- [ ] Task result caching (Redis/DragonflyDB)
- [ ] Mobile app (Flutter)
- [ ] Advanced ML task types
- [ ] Inter-grid federation
- [ ] Carbon credit marketplace

---

**Built with 💚 by the KoproGo Cooperative**

*Empowering communities through sustainable, cooperative technology.*
