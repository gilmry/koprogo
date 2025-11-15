===============================================
ADR-0001: Intégration MCP pour Edge Computing
===============================================

:ADR: 0001
:Titre: Intégration Model Context Protocol (MCP) pour Réseau Décentralisé Edge Computing
:Date: 2025-01-15
:Statut: Accepted
:Décideurs: PO (ASBL), Tech Lead Backend, Tech Lead IA
:Impacté: Équipes Backend, IA/Grid, Infra

.. contents:: Table des matières
   :depth: 2
   :local:

Métadonnées
===========

.. list-table::
   :widths: 30 70
   :header-rows: 0

   * - **ADR**
     - 0001
   * - **Titre**
     - Intégration Model Context Protocol (MCP) pour Réseau Décentralisé Edge Computing
   * - **Date décision**
     - 2025-01-15
   * - **Statut**
     - Accepted (implémentation Jalon 6, 2026+)
   * - **Décideurs**
     - - PO (ASBL)
       - Tech Lead Backend (Rust)
       - Tech Lead IA/Grid
   * - **Contributeurs**
     - Communauté KoproGo (review GitHub Discussions)
   * - **Équipes impactées**
     - - Backend (koprogo-mcp server Rust)
       - IA/Grid (koprogo-node edge clients)
       - Infra (réseau P2P, monitoring grid)
   * - **Jalon**
     - 6 (Autonomie IA, 2026+)
   * - **Lié à**
     - - RFC-0042 (Réseau MCP décentralisé - future)
       - ADR-0001 (Rust backend - existant)

Contexte
========

Problème
--------

**Situation actuelle (Jalons 1-5)** :

KoproGo utilise une infrastructure cloud classique (VPS → K3s → K8s) pour héberger backend, frontend, et bases de données. L'IA (future) serait hébergée cloud (GPU instances), coûteuse et écologiquement problématique :

- **Coût cloud IA** : GPU instances (A100, V100) = 2-5€/heure = 1.440-3.600€/mois
- **Impact CO₂** : GPUs cloud ≈ 500g CO₂/requête (vs 0,12g backend actuel)
- **Scalabilité limitée** : Budget ASBL insuffisant pour GPU cloud à grande échelle (> 5.000 copropriétés)

**Opportunité Jalon 6** :

Les participants KoproGo (copropriétés, contributeurs, sympathisants) disposent de **ressources compute inutilisées** :

- Raspberry Pi (ARM64, 4-8GB RAM)
- Vieux laptops (x86, 8-16GB RAM)
- Mac mini (M1/M2, 16GB RAM)
- Serveurs domestiques (NAS Synology, x86)

**Vision** : Transformer ces ressources en **réseau décentralisé edge computing** pour :

1. **Exécuter IA localement** (LLMs, embeddings, OCR) → 0 CO₂ cloud
2. **Monétiser compute** : Participants rémunérés (€/heure compute) → Revenus distribués
3. **Résilience** : Réseau P2P (pas de SPOF cloud)
4. **Souveraineté données** : Données traitées localement (RGPD-friendly)

**Problème technique** :

Comment orchestrer un réseau distribué de milliers de nœuds edge hétérogènes (ARM, x86, macOS, Windows, Linux) pour exécuter des tâches IA de manière fiable, sécurisée, et monétisable ?

Contraintes
-----------

1. **Technique** :

   - Hétérogénéité hardware (ARM64, x86_64, macOS M1/M2)
   - Réseau instable (participants déconnectent, NAT, firewalls)
   - Sécurité (code malveillant, DDoS, vol données)
   - Latence variable (edge ≠ datacenter)

2. **Économique** :

   - Coût infrastructure ASBL minimal (budget serré)
   - Monétisation équitable (participants = co-owners revenus)
   - Comptabilité compute (tracking précis €/heure)

3. **Écologique** :

   - 0 CO₂ cloud (objectif Jalon 6)
   - Réutilisation hardware existant (anti-gaspillage)

4. **Réglementaire** :

   - RGPD (données traitées localement, consentement explicite)
   - Fiscalité (revenus participants = revenus ASBL distribués ?)

Décision
========

Solution choisie
----------------

**Adopter Model Context Protocol (MCP)** comme protocole standard pour orchestrer le réseau edge computing KoproGo.

**Architecture** :

.. code-block:: text

   ┌─────────────────────────────────────────────────┐
   │  KoproGo Cloud (K8s)                            │
   │  ┌───────────────────────────────────┐          │
   │  │  koprogo-mcp (Server Rust)        │          │
   │  │  - Task orchestration             │          │
   │  │  - Node registry                  │          │
   │  │  - Compute accounting             │          │
   │  └───────────────┬───────────────────┘          │
   └──────────────────┼──────────────────────────────┘
                      │ MCP Protocol (JSONRPC)
        ┌─────────────┼─────────────┬─────────────┐
        │             │             │             │
   ┌────▼─────┐ ┌────▼─────┐ ┌─────▼────┐ ┌─────▼────┐
   │ Edge 1   │ │ Edge 2   │ │ Edge 3   │ │ Edge N   │
   │ RPi 4    │ │ Laptop   │ │ Mac M1   │ │ NAS x86  │
   │ (ARM64)  │ │ (x86_64) │ │ (ARM64)  │ │ (x86_64) │
   └──────────┘ └──────────┘ └──────────┘ └──────────┘
   koprogo-node (Client)
   - llama.cpp (LLM inference)
   - Tesseract (OCR)
   - Embedding models

**Composants** :

1. **koprogo-mcp (Server Rust)** :

   - Orchestrateur central (K8s cluster)
   - API MCP (JSONRPC over WebSocket)
   - Registry nœuds edge (capabilities, uptime, rewards)
   - Task queue (Redis backend)
   - Compute accounting (PostgreSQL)

2. **koprogo-node (Client Rust)** :

   - Agent edge (Raspberry Pi, laptops, etc.)
   - MCP client (connect to koprogo-mcp server)
   - Task executor (llama.cpp, Tesseract, etc.)
   - Monitoring (CPU, RAM, uptime)
   - Auto-update (binary releases GitHub)

3. **MCP Protocol** :

   - JSONRPC 2.0 over WebSocket (bi-directionnel)
   - Standard Anthropic (https://modelcontextprotocol.io)
   - Extensible (custom capabilities)

**Flux exemple (génération résumé PV assemblée)** :

.. code-block:: text

   1. Syndic upload PV PDF (frontend → backend)
   2. Backend crée task "extract_text_from_pdf" (koprogo-mcp)
   3. koprogo-mcp sélectionne nœud edge (capability: OCR, uptime > 99%)
   4. Edge node télécharge PDF, exécute Tesseract OCR, retourne texte
   5. Backend crée task "summarize_text" (LLM)
   6. koprogo-mcp sélectionne nœud edge (capability: LLM, RAM > 8GB)
   7. Edge node exécute llama.cpp (Mistral-7B), retourne résumé
   8. Backend stocke résumé, notifie syndic
   9. koprogo-mcp comptabilise compute (2 min OCR + 5 min LLM = 0,12€)
   10. Revenus distribués : 80% ASBL (développement), 20% fonds solidarité

Justification MCP
-----------------

**Pourquoi MCP vs alternatives ?**

.. list-table::
   :header-rows: 1
   :widths: 25 25 25 25

   * - Critère
     - MCP ✅
     - IPFS/Filecoin
     - Kubernetes (edge)
   * - **Protocole standardisé**
     - Oui (Anthropic)
     - Oui (Protocol Labs)
     - Non (custom)
   * - **Edge-first**
     - Oui (design goal)
     - Partiel (storage-first)
     - Non (cloud-first)
   * - **Hétérogénéité hardware**
     - Excellent (ARM, x86, macOS)
     - Bon (binaries multiples)
     - Moyen (K3s lourd)
   * - **Latence**
     - Faible (local compute)
     - Moyenne (fetch data)
     - Élevée (orchestration)
   * - **Compute accounting**
     - Custom (implement)
     - Intégré (Filecoin)
     - Aucun
   * - **Communauté**
     - Petite (récent 2024)
     - Grande (mature)
     - Énorme
   * - **Complexité**
     - Faible (JSONRPC)
     - Élevée (blockchain)
     - Élevée (K8s)
   * - **Coût**
     - Gratuit (MIT license)
     - Coût Filecoin (crypto)
     - Gratuit (Apache 2.0)

**Décision** : MCP (standard, edge-first, simplicité) > IPFS/Filecoin (complexité blockchain) > Kubernetes edge (trop lourd)

Alternatives Rejetées
=====================

Alternative 1 : IPFS + Filecoin
--------------------------------

**Description** :

Utiliser IPFS (stockage distribué) + Filecoin (compute blockchain) pour orchestrer edge computing.

**Avantages** :

- ➕ Protocole mature (Protocol Labs, 2014)
- ➕ Compute accounting natif (Filecoin smart contracts)
- ➕ Grande communauté (docs, libs)

**Inconvénients** :

- ➖ Complexité blockchain (smart contracts, gas fees)
- ➖ Dépendance crypto (volatilité, fiscalité complexe)
- ➖ Storage-first (compute = feature secondaire)
- ➖ Latence élevée (consensus blockchain)
- ➖ RGPD : Stockage immutable IPFS (droit à l'oubli impossible)

**Décision** : ❌ Rejetée (complexité + RGPD)

Alternative 2 : Kubernetes Edge (K3s/KubeEdge)
----------------------------------------------

**Description** :

Déployer K3s (Kubernetes lightweight) sur chaque nœud edge, orchestrer via Kubernetes API.

**Avantages** :

- ➕ Technologie connue (K8s)
- ➕ Tooling mature (kubectl, Helm)
- ➕ Auto-scaling natif

**Inconvénients** :

- ➖ Trop lourd pour edge (K3s = 512MB RAM min, trop pour Raspberry Pi)
- ➖ Complexité opérationnelle (certificats, etcd, networking)
- ➖ Latence orchestration élevée (API calls)
- ➖ Pas de compute accounting natif (custom implementation)

**Décision** : ❌ Rejetée (trop lourd pour edge)

Alternative 3 : Custom Protocol (JSONRPC ad-hoc)
------------------------------------------------

**Description** :

Développer protocole custom (JSONRPC over WebSocket) sans dépendre de MCP.

**Avantages** :

- ➕ Contrôle total (no dependencies)
- ➕ Optimisé KoproGo (pas de features inutiles)

**Inconvénients** :

- ➖ Réinventer la roue (MCP existe déjà)
- ➖ Pas de standard (interopérabilité impossible)
- ➖ Maintenance long-terme (burden ASBL)
- ➖ Pas de communauté (0 contributeurs externes)

**Décision** : ❌ Rejetée (réinvention inutile)

Alternative 4 : Ne rien faire (Cloud GPU)
------------------------------------------

**Description** :

Héberger IA sur cloud GPU (AWS, GCP, OVH).

**Avantages** :

- ➕ Simple (pas de réseau edge à gérer)
- ➕ Fiable (SLA cloud 99.9%)

**Inconvénients** :

- ➖ Coût prohibitif (2-5€/h GPU = 1.440-3.600€/mois)
- ➖ Impact CO₂ énorme (500g/requête vs 0,12g cible)
- ➖ Pas scalable (budget ASBL insuffisant)
- ➖ Pas de monétisation participants (revenus 0)

**Décision** : ❌ Rejetée (coût + écologie)

Conséquences
============

Positives
---------

1. **Écologie** :

   - 0 CO₂ cloud (compute local) ✅ Objectif Jalon 6
   - Réutilisation hardware existant (anti-gaspillage)
   - Impact : -100% CO₂ IA (vs cloud GPU)

2. **Économique** :

   - Coût infra ASBL minimal (pas de GPU cloud)
   - Revenus distribués participants (80% ASBL, 20% fonds solidarité)
   - Scalabilité illimitée (plus de participants = plus de compute)

3. **Souveraineté** :

   - Données traitées localement (pas de cloud tiers)
   - RGPD-friendly (consentement explicite, droit effacement)

4. **Communauté** :

   - Participants = co-owners (revenus partagés)
   - Engagement communautaire (contribution hardware)

5. **Résilience** :

   - Réseau P2P (pas de SPOF cloud)
   - Redondance (milliers de nœuds)

Négatives
---------

1. **Complexité technique** :

   - Orchestration réseau distribué (nouveau pour équipe)
   - Sécurité edge (sandboxing, malware, DDoS)
   - Monitoring grid (uptime, rewards tracking)

2. **Fiabilité** :

   - Nœuds edge instables (participants déconnectent)
   - Latence variable (réseau domestique)
   - Pas de SLA (vs cloud 99.9%)

3. **Adoption** :

   - Participants doivent installer koprogo-node (friction)
   - Hardware hétérogène (support multi-arch)
   - Onboarding complexe (config réseau, NAT, firewalls)

4. **Légal/Fiscal** :

   - Revenus participants = fiscalité complexe (ASBL distribue revenus ?)
   - RGPD : Responsabilité traitement données (ASBL = data controller)

5. **Maintenance** :

   - Nouveau protocole MCP (risque abandonnement Anthropic ?)
   - Support multi-platform (ARM, x86, macOS, Windows)
   - Auto-update nœuds (sécurité, compatibilité)

Risques
-------

.. list-table::
   :header-rows: 1
   :widths: 30 30 30 10

   * - Risque
     - Impact
     - Mitigation
     - Probabilité
   * - MCP protocol abandonné (Anthropic)
     - Bloque Jalon 6
     - Fork protocol, maintain in-house
     - Faible
   * - Adoption faible (< 100 nœuds)
     - Grid non viable
     - Incentives (€/h compute), gamification
     - Moyenne
   * - Sécurité edge (malware, DDoS)
     - Réseau compromis
     - Sandboxing (containers), rate limiting
     - Moyenne
   * - Fiscalité revenus participants
     - Problème légal ASBL
     - Conseil fiscal expert, statut coopérative
     - Moyenne
   * - Performance insuffisante (P99 > 10s)
     - UX dégradée
     - Sélection nœuds (uptime, latency), fallback cloud
     - Faible

Plan Implémentation
===================

Phases
------

**Phase 1 : Prototype (Sprint 1-2, 4 semaines)** :

- Minimal koprogo-mcp server (Rust, JSONRPC)
- Minimal koprogo-node client (Rust, llama.cpp wrapper)
- Task simple (echo "hello" → LLM completion)
- Tests locaux (2 nœuds edge, 1 laptop + 1 RPi)

**Phase 2 : Alpha (Sprint 3-4, 4 semaines)** :

- Registry nœuds (PostgreSQL, capabilities, uptime)
- Task queue (Redis)
- Compute accounting (€/heure tracking)
- Tests alpha (10 participants beta, 10 nœuds)

**Phase 3 : Beta (Sprint 5-8, 8 semaines)** :

- Sécurité (sandboxing, rate limiting)
- Monitoring (Prometheus, Grafana)
- Auto-update (binary releases GitHub)
- Dashboard participants (revenus, uptime)
- Tests beta (100 participants, 100 nœuds)

**Phase 4 : Production (Sprint 9+, ongoing)** :

- Déploiement production (1.000+ nœuds)
- Support multi-tasks (OCR, embeddings, classification)
- Optimisations (load balancing, latency)

Stack Technique
---------------

**koprogo-mcp (Server Rust)** :

- Framework: Actix-web (async, WebSocket)
- MCP SDK: anthropic/mcp-rust-sdk (si dispo, sinon custom)
- Database: PostgreSQL (nodes, tasks, rewards)
- Queue: Redis (task queue)
- Monitoring: Prometheus + Grafana

**koprogo-node (Client Rust)** :

- MCP client: anthropic/mcp-rust-sdk
- LLM: llama.cpp (C++ bindings Rust)
- OCR: tesseract-rs (Tesseract bindings)
- Monitoring: prometheus-client (metrics export)

**Protocol** :

- MCP v1 (JSONRPC 2.0 over WebSocket)
- Custom capabilities: ``koprogo/llm``, ``koprogo/ocr``, ``koprogo/embedding``

Tests
-----

1. **Unit** : Domain logic (task selection, rewards calculation)
2. **Integration** : PostgreSQL (node registry), Redis (task queue)
3. **E2E** : Full workflow (submit task → edge compute → return result)
4. **Edge** : Multi-arch (ARM64 RPi, x86_64 laptop, macOS M1)
5. **Load** : 1.000 tasks/min, 100 nœuds simultanés

Monitoring
----------

**Métriques Prometheus** :

- ``koprogo_mcp_nodes_total`` (nœuds actifs)
- ``koprogo_mcp_tasks_queued`` (tasks en attente)
- ``koprogo_mcp_tasks_completed`` (tasks terminées)
- ``koprogo_mcp_compute_hours`` (heures compute totales)
- ``koprogo_mcp_rewards_distributed`` (€ distribués)

**Alertes Grafana** :

- Nodes < 50 → Alerte (grid non viable)
- Tasks queued > 100 (5 min) → Alerte (nœuds surchargés)
- Compute hours/day < 10h → Alerte (adoption faible)

Implémentation Détaillée
=========================

Architecture koprogo-mcp (Server)
----------------------------------

**Modules Rust** :

.. code-block:: rust

   // src/mcp/server/mod.rs
   pub struct McpServer {
       node_registry: Arc<NodeRegistry>,
       task_queue: Arc<TaskQueue>,
       reward_tracker: Arc<RewardTracker>,
   }

   // src/mcp/server/node_registry.rs
   pub struct NodeRegistry {
       db: PgPool, // PostgreSQL
   }
   impl NodeRegistry {
       pub async fn register_node(&self, node: Node) -> Result<(), Error>;
       pub async fn heartbeat(&self, node_id: Uuid) -> Result<(), Error>;
       pub async fn get_available_nodes(&self, capability: &str) -> Result<Vec<Node>, Error>;
   }

   // src/mcp/server/task_queue.rs
   pub struct TaskQueue {
       redis: RedisPool,
   }
   impl TaskQueue {
       pub async fn enqueue(&self, task: Task) -> Result<Uuid, Error>;
       pub async fn dequeue(&self, node_id: Uuid) -> Result<Option<Task>, Error>;
       pub async fn complete(&self, task_id: Uuid, result: TaskResult) -> Result<(), Error>;
   }

   // src/mcp/server/reward_tracker.rs
   pub struct RewardTracker {
       db: PgPool,
   }
   impl RewardTracker {
       pub async fn track_compute(&self, node_id: Uuid, hours: f64) -> Result<(), Error>;
       pub async fn calculate_rewards(&self, node_id: Uuid) -> Result<f64, Error>; // €
   }

**Database Schema** :

.. code-block:: sql

   -- Nodes registry
   CREATE TABLE mcp_nodes (
       id UUID PRIMARY KEY,
       owner_id UUID REFERENCES users(id),
       name TEXT NOT NULL,
       capabilities JSONB, -- ['koprogo/llm', 'koprogo/ocr']
       hardware JSONB, -- {arch: 'arm64', ram_gb: 8, cpu_cores: 4}
       uptime_percentage FLOAT,
       last_heartbeat TIMESTAMPTZ,
       created_at TIMESTAMPTZ
   );

   -- Tasks
   CREATE TABLE mcp_tasks (
       id UUID PRIMARY KEY,
       task_type TEXT NOT NULL, -- 'koprogo/llm', 'koprogo/ocr'
       payload JSONB,
       status TEXT, -- 'queued', 'running', 'completed', 'failed'
       assigned_node_id UUID REFERENCES mcp_nodes(id),
       result JSONB,
       created_at TIMESTAMPTZ,
       completed_at TIMESTAMPTZ
   );

   -- Rewards
   CREATE TABLE mcp_rewards (
       id UUID PRIMARY KEY,
       node_id UUID REFERENCES mcp_nodes(id),
       compute_hours FLOAT,
       amount_eur FLOAT, -- €/heure compute
       period_start TIMESTAMPTZ,
       period_end TIMESTAMPTZ,
       paid BOOLEAN,
       created_at TIMESTAMPTZ
   );

Architecture koprogo-node (Client)
-----------------------------------

**Modules Rust** :

.. code-block:: rust

   // src/mcp/client/mod.rs
   pub struct McpClient {
       server_url: String, // wss://koprogo.app/mcp
       node_id: Uuid,
       capabilities: Vec<String>,
       executor: Arc<TaskExecutor>,
   }

   impl McpClient {
       pub async fn connect(&mut self) -> Result<(), Error>;
       pub async fn heartbeat_loop(&self) -> Result<(), Error>; // Every 60s
       pub async fn poll_tasks(&self) -> Result<(), Error>; // Every 10s
   }

   // src/mcp/client/task_executor.rs
   pub struct TaskExecutor {
       llm: Option<LlamaRunner>, // llama.cpp
       ocr: Option<TesseractRunner>, // tesseract-rs
   }

   impl TaskExecutor {
       pub async fn execute(&self, task: Task) -> Result<TaskResult, Error> {
           match task.task_type.as_str() {
               "koprogo/llm" => self.llm.run(task.payload),
               "koprogo/ocr" => self.ocr.run(task.payload),
               _ => Err(Error::UnsupportedTask),
           }
       }
   }

**Config TOML** :

.. code-block:: toml

   # koprogo-node.toml
   [node]
   id = "550e8400-e29b-41d4-a716-446655440000"
   name = "alice-rpi4"
   owner_email = "alice@example.com"

   [server]
   url = "wss://koprogo.app/mcp"
   auth_token = "..." # JWT

   [capabilities]
   llm = true
   ocr = true

   [llm]
   model_path = "/models/mistral-7b-q4.gguf"
   max_tokens = 2048
   threads = 4

   [ocr]
   tesseract_path = "/usr/bin/tesseract"
   languages = ["fra", "eng"]

MCP Protocol Messages
---------------------

**Register Node** :

.. code-block:: json

   // Client → Server
   {
     "jsonrpc": "2.0",
     "method": "mcp/register_node",
     "params": {
       "node_id": "550e8400-e29b-41d4-a716-446655440000",
       "name": "alice-rpi4",
       "capabilities": ["koprogo/llm", "koprogo/ocr"],
       "hardware": {"arch": "arm64", "ram_gb": 8, "cpu_cores": 4}
     },
     "id": 1
   }

   // Server → Client
   {
     "jsonrpc": "2.0",
     "result": {"status": "registered"},
     "id": 1
   }

**Heartbeat** :

.. code-block:: json

   // Client → Server (every 60s)
   {
     "jsonrpc": "2.0",
     "method": "mcp/heartbeat",
     "params": {"node_id": "550e8400-e29b-41d4-a716-446655440000"},
     "id": 2
   }

**Poll Task** :

.. code-block:: json

   // Client → Server (every 10s)
   {
     "jsonrpc": "2.0",
     "method": "mcp/poll_task",
     "params": {"node_id": "550e8400-e29b-41d4-a716-446655440000"},
     "id": 3
   }

   // Server → Client
   {
     "jsonrpc": "2.0",
     "result": {
       "task_id": "660f9511-f30c-52e5-b827-557766551111",
       "task_type": "koprogo/llm",
       "payload": {"prompt": "Résume ce texte...", "max_tokens": 200}
     },
     "id": 3
   }

**Complete Task** :

.. code-block:: json

   // Client → Server
   {
     "jsonrpc": "2.0",
     "method": "mcp/complete_task",
     "params": {
       "task_id": "660f9511-f30c-52e5-b827-557766551111",
       "result": {"completion": "Le texte résumé est...", "tokens": 42},
       "compute_time_ms": 5240
     },
     "id": 4
   }

Critères Acceptation
====================

Fonctionnels
------------

1. ✅ Node edge peut s'enregistrer (koprogo-mcp)
2. ✅ Node edge reçoit tasks (polling 10s)
3. ✅ Node exécute task LLM (llama.cpp) et retourne résultat
4. ✅ Server track compute hours (PostgreSQL)
5. ✅ Dashboard participants (revenus €, uptime %)

Techniques
----------

1. ✅ Tests unit coverage > 90% (domain, use cases)
2. ✅ Tests integration PostgreSQL + Redis (testcontainers)
3. ✅ Tests E2E (submit task → edge compute → result)
4. ✅ Tests multi-arch (ARM64 RPi, x86_64 laptop, macOS M1)
5. ✅ Performance P99 < 10s (task LLM 200 tokens)
6. ✅ Monitoring Prometheus (metrics koprogo_mcp_*)

Non-fonctionnels
----------------

1. **Scalabilité** : Support 1.000 nœuds simultanés
2. **Fiabilité** : Retry tasks si node timeout (3x max)
3. **Sécurité** : Sandboxing tasks (containers), rate limiting (10 tasks/min/node)
4. **Accessibilité** : Dashboard WCAG 2.1 AA
5. **Documentation** : User guide participants (install, config, troubleshooting)

Statut ADR
==========

**Accepted** (2025-01-15) :

- Approuvé PO (ASBL) : Vision alignée Jalon 6
- Approuvé Tech Lead Backend : Stack Rust compatible
- Approuvé Tech Lead IA : MCP protocol suitable

**Implémentation** : Jalon 6 (2026+)

**Review** : Trimestrielle (évaluer alternatives si MCP abandonné)

Références
==========

- **MCP Official** : https://modelcontextprotocol.io (Anthropic)
- **llama.cpp** : https://github.com/ggerganov/llama.cpp
- **Rust MCP SDK** : https://github.com/anthropics/mcp-rust-sdk (future)
- :doc:`/governance/togaf/adm` : TOGAF ADM (Jalon 6 = Phase E Opportunités)
- :doc:`/ROADMAP_PAR_CAPACITES` : Roadmap Jalon 6 détaillé

**ADRs liées** :

- :doc:`/adr/0001-rust-actix-web-backend` : Stack backend Rust

**RFCs liées** (futures) :

- RFC-0042 : Réseau MCP décentralisé (détails protocole, incentives)

---

*ADR-0001 KoproGo ASBL - Décision immutable, review trimestrielle*
