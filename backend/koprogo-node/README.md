# KoproGo Node - Edge AI Server

Raspberry Pi-compatible AI inference server for decentralized copro AI.

## Overview

KoproGo Node runs on Raspberry Pi (4/5 with 8GB RAM recommended) and provides:

- 🤖 Local AI inference (llama.cpp)
- 🌍 Grid computing participation
- 📊 Resource monitoring
- 🌱 0g CO₂ emissions (solar-powered)
- 💰 Passive income (MCP tokens)

## Quick Start

### 1. Download Model

```bash
# Create models directory
mkdir -p models

# Download Llama 3 8B Q4 (~4.5GB)
wget -P models/ https://huggingface.co/QuantFactory/Meta-Llama-3-8B-Instruct-GGUF/resolve/main/Meta-Llama-3-8B-Instruct.Q4_K_M.gguf

# Rename for convenience
mv models/Meta-Llama-3-8B-Instruct.Q4_K_M.gguf models/llama3-8b-instruct-q4.gguf
```

### 2. Build & Run

```bash
# Build (optimized for Pi)
cargo build --release

# Run
./target/release/koprogo-node --model llama3:8b-instruct-q4
```

### 3. Test

```bash
curl -X POST http://localhost:3031/mcp/v1/chat \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama3:8b-instruct-q4",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

## Supported Models

Tested on Raspberry Pi 4/5 (8GB):

| Model | Size | RAM | Quality |
|-------|------|-----|---------|
| **llama3:8b-instruct-q4** | 4.5GB | ~6GB | ⭐⭐⭐⭐ |
| **mistral:7b-instruct-q4** | 4GB | ~5.5GB | ⭐⭐⭐⭐ |
| **phi-2:2.7b-q4** | 1.6GB | ~3GB | ⭐⭐⭐ |

For Pi 4GB, use phi-2 only.

## Command-Line Options

```bash
koprogo-node [OPTIONS]

Options:
  -p, --port <PORT>              Port to listen on [default: 3031]
  -m, --model <MODEL>            Model to load [default: llama3:8b-instruct-q4]
      --models-dir <MODELS_DIR>  Models directory [default: ./models]
  -g, --grid-url <GRID_URL>      Grid server URL (optional)
      --mcp                      Enable MCP server [default: true]
  -h, --help                     Print help
```

## API Endpoints

- `POST /mcp/v1/chat` - Chat completion
- `GET /mcp/v1/models` - List loaded models
- `GET /mcp/v1/health` - Health check (memory, CPU, models)
- `GET /health` - Simple health check

## Grid Computing

Join the grid network to earn MCP tokens:

```bash
koprogo-node --model llama3:8b --grid-url https://grid.koprogo.coop
```

Tasks you can execute:
- OCR on invoices
- Document translation
- Meeting minutes summarization
- Expense predictions

Rewards: Passive income via MCP tokens + solidarity fund contributions.

## Performance

On Raspberry Pi 5 (8GB):
- **Latency**: ~50-100ms first token, ~20-30ms/token
- **Throughput**: ~30-40 tokens/second
- **Memory**: ~6GB for llama3:8b-q4
- **Power**: ~5-8W (0g CO₂ with solar)

## Docker Deployment

```bash
# Build for ARM64
docker build -f Dockerfile.arm64 -t koprogo-node:latest .

# Run
docker run -p 3031:3031 -v $(pwd)/models:/app/models koprogo-node:latest
```

## Systemd Service

Install as system service:

```bash
sudo cp koprogo-node.service /etc/systemd/system/
sudo systemctl enable koprogo-node
sudo systemctl start koprogo-node
```

## Monitoring

Health endpoint includes:
- Models loaded
- Memory usage (total/used)
- Active requests
- Uptime

Example:
```json
{
  "node_url": "http://localhost:3031",
  "is_healthy": true,
  "models_loaded": ["llama3:8b-instruct-q4"],
  "active_requests": 0,
  "total_memory_mb": 8192,
  "used_memory_mb": 6144
}
```

## Production Notes

1. **Security**: Run behind Traefik with HTTPS
2. **Updates**: Auto-update models via cron
3. **Backup**: No persistent state (stateless node)
4. **Scaling**: Add more Pis to increase capacity
5. **Solar**: Use solar panel + battery for true 0g CO₂

## Troubleshooting

**Model not loading:**
```bash
# Check models directory
ls -lh models/

# Verify file permissions
chmod 644 models/*.gguf
```

**Out of memory:**
- Use smaller model (phi-2)
- Reduce context length
- Enable swap (not recommended for SD cards)

**Slow inference:**
- Check CPU throttling: `vcgencmd measure_temp`
- Add heatsink/fan
- Reduce max_tokens

## License

AGPL-3.0 - See LICENSE
