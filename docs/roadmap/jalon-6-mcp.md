# Jalon 6 : SYMBIOSE - Intégration Model Context Protocol (MCP)

> **Vision, pas un état du produit.** Ce document décrit une cible. Au
> 03/11/2026, le serveur MCP de KoproGo expose des outils de lecture sur
> l'API ; le grid computing, l'exécution locale et la multiplicité de LLM
> décrits ici ne sont pas implémentés.

**Période** : Novembre 2025
**Objectif** : Écosystème IA décentralisé, éco-responsable et open-source

## 📋 Vue d'ensemble

Le Model Context Protocol (MCP) transforme KoproGo en une plateforme IA décentralisée où chaque copropriété peut:

- ✅ Utiliser n'importe quel LLM (Claude, GPT-4, Llama, Mistral)
- ✅ Exécuter des modèles localement sur Raspberry Pi (0g CO₂)
- ✅ Participer au grid computing distribué
- ✅ Générer des revenus passifs via tokens MCP
- ✅ Contribuer au fonds de solidarité climatique

## 🏗️ Architecture

### Composants implémentés

```
koprogo/
├── backend/
│   ├── koprogo-mcp/              # Crate MCP core
│   │   ├── src/
│   │   │   ├── core/             # Domain (entities, services)
│   │   │   ├── ports/            # Traits (McpService, ModelRegistry)
│   │   │   └── adapters/         # Implementations (PostgreSQL, EdgeClient, Actix)
│   │   ├── Cargo.toml
│   │   └── README.md
│   └── koprogo-node/             # Edge node (Raspberry Pi)
│       ├── src/
│       │   ├── main.rs           # Axum server
│       │   ├── mcp_edge.rs       # llama.cpp integration
│       │   ├── grid_client.rs    # Grid communication
│       │   └── model_manager.rs  # Model loading
│       ├── models/               # GGUF models directory
│       ├── Dockerfile.arm64
│       ├── Cargo.toml
│       └── README.md
├── frontend/
│   ├── src/
│   │   ├── pages/mcp-chat.astro  # Page chatbot PWA
│   │   ├── components/McpChatbot.svelte
│   │   └── lib/api/mcp.ts        # API client
├── infrastructure/
│   ├── docker-compose.mcp.yml    # Stack complète
│   └── ansible/roles/mcp/        # Déploiement
├── migrations/
│   └── 20250202000000_create_mcp_tables.sql
└── Makefile                      # Commandes MCP
```

### Architecture Hexagonale

Le crate `koprogo-mcp` suit strictement l'architecture hexagonale :

**Domain Layer (Core)** :
- `entities.rs` : McpRequest, McpResponse, ModelInfo, McpTask
- `services.rs` : McpRequestService, CarbonFootprintService

**Application Layer (Ports)** :
- `mcp_service.rs` : Trait McpService (chat, execute_task)
- `model_registry.rs` : Trait ModelRegistry (list_models, register_model)
- `mcp_repository.rs` : Trait McpRepository (log_request, get_statistics)

**Infrastructure Layer (Adapters)** :
- `postgres_repository.rs` : Implémentation PostgreSQL
- `edge_client.rs` : Communication avec koprogo-node
- `actix_handlers.rs` : Handlers HTTP (POST /mcp/v1/chat, etc.)

## 🚀 Fonctionnalités

### 1. API MCP (Endpoints)

**Chat Completion**
```bash
POST /mcp/v1/chat
{
  "model": "llama3:8b-instruct-q4",
  "messages": [
    {"role": "user", "content": "Résume ce PV"}
  ],
  "context": "copro:123",
  "temperature": 0.7
}

→ Réponse:
{
  "id": "uuid",
  "model": "llama3:8b-instruct-q4",
  "content": "Résumé: ...",
  "finish_reason": "stop",
  "usage": {
    "prompt_tokens": 100,
    "completion_tokens": 50,
    "total_tokens": 150
  },
  "execution_info": {
    "execution_type": "edge",
    "node_id": "http://localhost:3031",
    "latency_ms": 85,
    "co2_grams": 0.0
  }
}
```

**Liste des modèles**
```bash
GET /mcp/v1/models

→ Réponse:
{
  "models": [
    {
      "id": "llama3:8b-instruct-q4",
      "name": "Llama 3 8B Instruct Q4",
      "provider": "local",
      "context_length": 8192,
      "is_available": true,
      "edge_compatible": true
    },
    {
      "id": "claude-3-opus",
      "name": "Claude 3 Opus",
      "provider": "anthropic",
      "context_length": 200000,
      "is_available": true,
      "edge_compatible": false
    }
  ]
}
```

**Statistiques**
```bash
GET /mcp/v1/stats

→ Réponse:
{
  "total_requests": 1543,
  "total_tokens": 782345,
  "total_co2_grams": 123.45,
  "co2_saved_grams": 456.78,
  "edge_requests": 1200,
  "cloud_requests": 343,
  "grid_requests": 0,
  "avg_latency_ms": 127.5,
  "models_used": ["llama3:8b", "claude-3-opus"]
}
```

**Tâches Grid**
```bash
POST /mcp/v1/execute
{
  "task_type": "ocr_invoice",
  "input_data": {
    "document_url": "s3://invoices/2025/invoice-001.pdf"
  },
  "copro_id": "uuid"
}

→ Réponse:
{
  "id": "task-uuid",
  "status": "pending",
  "result": null
}

GET /mcp/v1/tasks/{id}
→ Statut + résultat
```

### 2. Edge Node (Raspberry Pi)

**Installation**
```bash
# 1. Télécharger modèle
make mcp-download-model

# 2. Lancer node
make node-run

# ou directement:
cd backend/koprogo-node
cargo run -- --port 3031 --model llama3:8b-instruct-q4
```

**Configuration**
```bash
koprogo-node [OPTIONS]

Options:
  -p, --port <PORT>              Port (default: 3031)
  -m, --model <MODEL>            Modèle à charger
      --models-dir <DIR>         Répertoire modèles (default: ./models)
  -g, --grid-url <URL>           URL serveur grid (optionnel)
      --mcp                      Activer serveur MCP (default: true)
```

**Performance** (Raspberry Pi 5, 8GB) :
- Latency first token : ~50-100ms
- Throughput : ~30-40 tokens/s
- Memory : ~6GB pour llama3:8b-q4
- Power : ~5-8W (0g CO₂ avec panneau solaire)

**Modèles supportés** :
| Modèle | Taille | RAM | Qualité |
|--------|--------|-----|---------|
| llama3:8b-instruct-q4 | 4.5GB | ~6GB | ⭐⭐⭐⭐ |
| mistral:7b-instruct-q4 | 4GB | ~5.5GB | ⭐⭐⭐⭐ |
| phi-2:2.7b-q4 | 1.6GB | ~3GB | ⭐⭐⭐ |

### 3. Frontend Chatbot PWA

**Page** : `/mcp-chat`

**Fonctionnalités** :
- ✅ Chat interactif avec historique
- ✅ Sélection du modèle (local/cloud)
- ✅ Actions rapides (Résumer PV, Traduire, OCR, Calculer)
- ✅ Stockage local (IndexedDB) pour mode offline
- ✅ Affichage statistiques (tokens, latence, CO₂)
- ✅ Indicateur edge (🍓) vs cloud (☁️)

**Technologies** :
- Astro + Svelte
- TypeScript
- IndexedDB pour offline
- Tailwind CSS

### 4. Grid Computing

**Architecture** :
```
Grid Server (coordinateur)
    ↓
  Tasks Queue
    ↓
Edge Nodes (Raspberry Pi) ← poll tasks
    ↓
  Results
    ↓
Validation (Proof of Green)
    ↓
MCP Tokens + Solidarity Fund
```

**Types de tâches** :
- OCR factures (invoices PDF → JSON)
- Traduction documents (FR ↔ EN, NL)
- Résumé PV (meeting minutes → key points)
- Prédiction charges (expense forecasting)

**Récompenses** :
- Tokens MCP pour tâches complétées
- CO₂ économisé → fonds solidarité
- Revenus passifs pour membres exécutant des nodes

## 🗄️ Base de Données

Migration : `20250202000000_create_mcp_tables.sql`

**Tables** :
- `mcp_models` : Registre des modèles disponibles
- `mcp_requests` : Log des requêtes (user_id, model, messages, context)
- `mcp_responses` : Log des réponses (tokens, latency, execution_info)
- `mcp_tasks` : Tâches grid (task_type, status, result, assigned_node)

**Seed data** :
- 9 modèles pré-configurés (llama3, mistral, claude, gpt-4)
- Providers: local, anthropic, openai, mistral

**Indexes** :
- user_id, context, model, created_at
- execution_type (edge/cloud/grid)
- task status, copro_id

## 🔧 Commandes Make

```bash
# Stack MCP complète
make mcp-up              # Démarrer backend + edge node + postgres
make mcp-down            # Arrêter

# Edge node
make node-run            # Lancer Raspberry Pi simulator
make node-build          # Build optimisé ARM64

# CLI
make mcp-cli-chat MSG="Hello"   # Chat via CLI
make mcp-cli-models             # Liste modèles
make mcp-cli-health             # Health check

# Tests
make test-mcp            # Tests MCP (unit + integration)

# Stats
make mcp-stats           # GET /mcp/v1/stats (via curl | jq)

# Modèles
make mcp-download-model  # Télécharge llama3:8b-q4 (4.5GB)
```

## 🧪 Tests

**Pyramide de tests** :

```
         E2E (chatbot UI)
         /             \
    Integration     BDD (Gherkin)
   /                               \
Unit (domain logic - 100% coverage)
```

**Commandes** :
```bash
# Unit tests (domain entities + services)
cd backend/koprogo-mcp && cargo test --lib

# Integration tests (PostgreSQL via testcontainers)
cd backend/koprogo-mcp && cargo test --test integration

# E2E (Playwright)
cd frontend && npm run test:e2e -- mcp-chat.spec.ts
```

**Couverture** :
- Domain layer : 100%
- Ports : Mockés via mockall
- Adapters : Testcontainers PostgreSQL

## 🐳 Docker

**Multi-arch** :
- `Dockerfile.arm64` : Raspberry Pi (ARM64)
- `docker-compose.mcp.yml` : Stack complète

**Déploiement** :
```bash
# Build ARM64 pour Pi
docker buildx build --platform linux/arm64 \
  -f backend/koprogo-node/Dockerfile.arm64 \
  -t koprogo-node:latest .

# Lancer stack
docker compose -f docker-compose.mcp.yml up
```

**Services** :
- `postgres` : PostgreSQL 15 (shared)
- `backend` : KoproGo + MCP API
- `edge-node` : Raspberry Pi simulator
- `frontend` : Astro + Svelte (optional)

## 📊 Métriques & KPIs

**Suivi** :
- Nombre de requêtes MCP (total, edge, cloud, grid)
- Tokens consommés
- CO₂ émis vs économisé
- Latence moyenne (edge < 100ms, cloud ~200ms)
- Modèles utilisés (distribution)
- Revenus MCP tokens (nodes grid)

**Dashboard** : `/mcp/v1/stats` (API) → intégration Grafana

## 🌱 Impact Écologique

**Calcul CO₂** :
- Edge (Raspberry Pi solaire) : **0g CO₂**
- Cloud API (GPT-4, Claude) : **~0.3g CO₂ / 1000 tokens**

**Exemple** :
- 1000 requêtes/mois à 500 tokens = 500k tokens
- Cloud : 500 × 0.3 = **150g CO₂**
- Edge : **0g CO₂**
- **Économie : 150g CO₂/mois**

Pour 100 copros : **15kg CO₂/mois** = **180kg CO₂/an**

**Fonds Solidarité** :
- Crédits carbone → financement panneaux solaires pour copros
- Tokens MCP → revenus passifs membres grid
- Open-source → réplication par autres coops

## 🔐 Sécurité

**Authentication** :
- JWT tokens (contexte copro)
- Scopes : `mcp:read`, `mcp:write`, `mcp:admin`

**Rate Limiting** :
- 100 req/min par utilisateur
- 1000 req/min par copro

**Data Privacy** :
- Logs chiffrés (context = copro UUID, pas de PII)
- Rétention : 30 jours
- GDPR compliant

**Edge Security** :
- Nodes derrière Traefik HTTPS
- Authentification grid (signed tasks)
- Anti-fraud : verification multi-nodes

## 📚 Documentation

**Fichiers** :
- `backend/koprogo-mcp/README.md` : Usage crate MCP
- `backend/koprogo-node/README.md` : Guide Raspberry Pi
- `docs/roadmap/jalon-6-mcp.md` : Ce document
- API : Swagger/OpenAPI (TODO)

**Exemples** :
```bash
# Chat simple
curl -X POST http://localhost:8080/mcp/v1/chat \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama3:8b",
    "messages": [{"role": "user", "content": "Hello"}]
  }'

# Avec contexte copro
curl -X POST http://localhost:8080/mcp/v1/chat \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama3:8b",
    "messages": [{"role": "user", "content": "Résume PV AG"}],
    "context": "copro:550e8400-e29b-41d4-a716-446655440000"
  }'

# Stats
curl http://localhost:8080/mcp/v1/stats | jq .

# Modèles
curl http://localhost:8080/mcp/v1/models | jq '.models[] | select(.edge_compatible == true)'
```

## 🎯 Roadmap Technique

**Phase 1 - MVP (actuel)** :
- ✅ API MCP core
- ✅ Edge node (demo mode)
- ✅ Frontend chatbot
- ✅ Migrations DB
- ✅ Docker multi-arch

**Phase 2 - Production** :
- ⏳ Intégration llama.cpp réelle (llm crate)
- ⏳ Grid computing serveur
- ⏳ Proof of Green
- ⏳ MCP tokens (blockchain)

**Phase 3 - Scale** :
- ⏳ Streaming responses (SSE)
- ⏳ Multi-tenancy (isolation copros)
- ⏳ Fine-tuning modèles (copro-specific)
- ⏳ Federated learning

## 🤝 Contribution

**Open-Source** :
- Licence : AGPL-3.0
- Repo : github.com/gilmry/koprogo
- Issues : GitHub Projects

**Comment contribuer** :
1. Fork + branch `feature/mcp-xxx`
2. Tests obligatoires (unit + integration)
3. Format : `make format`
4. Lint : `make lint`
5. PR → review + merge

**Focus areas** :
- Intégration llama.cpp production
- Optimisation Pi (quantization, mmap)
- Grid server implémentation
- Modèles fine-tuned copro

## 📞 Support

**Documentation** : `make docs-serve` → http://localhost:8000
**Issues** : GitHub Issues
**Chat** : Discord KoproGo
**Email** : contact@koprogo.coop

---

**Auteurs** : KoproGo Team
**Date** : Février 2025
**Version** : 0.1.0 (MVP)
**Licence** : AGPL-3.0
