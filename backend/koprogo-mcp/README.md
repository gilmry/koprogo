# KoproGo MCP - Model Context Protocol

Decentralized AI ecosystem for KoproGo property management platform.

## Overview

KoproGo MCP enables copros to use any AI model (Claude, GPT-4, local Llama) through a standardized interface, with automatic routing to the most eco-friendly execution method:

- **Edge (0g CO₂)**: Local Raspberry Pi inference
- **Cloud (0.3g CO₂/1k tokens)**: API calls to Claude, GPT-4
- **Grid (distributed)**: Heavy tasks across multiple nodes

## Architecture

Follows hexagonal architecture (ports & adapters):

```
core/          - Domain entities (McpRequest, McpResponse, ModelInfo)
ports/         - Trait definitions (McpService, ModelRegistry, McpRepository)
adapters/      - Implementations (PostgreSQL, EdgeClient, Actix handlers)
```

## Features

- ✅ Multi-model support (Llama 3, Claude, GPT-4, Mistral)
- ✅ Edge computing (Raspberry Pi local inference)
- ✅ Grid computing (distributed tasks)
- ✅ CO₂ tracking (0g for edge, calculated for cloud)
- ✅ Request/response logging (PostgreSQL)
- ✅ Token usage statistics
- ✅ Priority queuing

## Usage

### As a library

```rust
use koprogo_mcp::*;

// Create request
let request = McpRequest::new(
    "llama3:8b".to_string(),
    vec![Message::user("Summarize meeting notes".to_string())],
    Some("copro:123".to_string()),
)?;

// Execute on edge
let edge_client = EdgeClient::new(vec!["http://localhost:3031".to_string()]);
let response = edge_client.execute_on_edge(&request).await?;

println!("Response: {}", response.content);
println!("CO₂ saved: {:.4}g", response.calculate_co2_grams());
```

### CLI

```bash
# Chat with a model
cargo run --bin mcp-cli chat --model llama3:8b "Explain GDPR compliance"

# List models
cargo run --bin mcp-cli models

# Check health
cargo run --bin mcp-cli health
```

## API Endpoints

When integrated into Actix backend:

- `POST /mcp/v1/chat` - Chat completion
- `GET /mcp/v1/models` - List models
- `POST /mcp/v1/execute` - Grid task execution
- `GET /mcp/v1/tasks/{id}` - Task status
- `GET /mcp/v1/stats` - Usage statistics
- `GET /mcp/v1/health` - Health check

## Database Schema

Requires PostgreSQL tables:
- `mcp_models` - Model registry
- `mcp_requests` - Request logs
- `mcp_responses` - Response logs
- `mcp_tasks` - Grid tasks

Run migration:
```bash
sqlx migrate run
```

## Testing

```bash
# Unit tests (100% domain coverage)
cargo test --lib

# Integration tests (testcontainers)
cargo test --test integration

# All tests
cargo test
```

## Environment Variables

None required (library crate). Edge node URLs configured at runtime.

## License

AGPL-3.0 - See LICENSE
