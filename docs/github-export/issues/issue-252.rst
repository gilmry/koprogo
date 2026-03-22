===============================================
Issue #252: MCP: Serveur SSE + JSON-RPC handler
===============================================

:State: **OPEN**
:Milestone: Jalon 4: Automation & Intégrations 📅
:Labels: enhancement,track:mcp release:0.2.0
:Assignees: Unassigned
:Created: 2026-03-10
:Updated: 2026-03-15
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/252>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   
   Implémenter le serveur MCP standard (JSON-RPC 2.0 over Server-Sent Events) comme nouvel adaptateur dans l'architecture hexagonale existante.
   
   ## Architecture
   
   ```
   ┌─────────────────┐     MCP/SSE      ┌──────────────────────┐
   │   Claude / AI   │ ◄──────────────► │  KoproGo MCP Server  │
   │   (claude.ai)   │                  │  (Rust / Actix-web)  │
   └─────────────────┘                  └──────────┬───────────┘
                                                   │
                                        ┌──────────▼───────────┐
                                        │   KoproGo Backend     │
                                        │   (hexagonal arch.)   │
                                        │   PostgreSQL          │
                                        └──────────────────────┘
   ```
   
   ## Spécification complète
   
   Voir `backend/koprogo-mcp/README.md` pour l'architecture détaillée et la structure cible.
   
   ## Tâches
   
   - [ ] Créer le module `src/mcp/server.rs` avec endpoint SSE (`/mcp/sse`)
   - [ ] Implémenter le handler JSON-RPC 2.0 (parse request, dispatch tool, format response)
   - [ ] Gérer le lifecycle MCP : `initialize`, `tools/list`, `tools/call`
   - [ ] Support streaming via SSE chunks
   - [ ] Intégrer avec le `McpService` trait existant (Phase 0)
   - [ ] Tests unitaires + intégration
   
   ## Contexte technique
   
   - Protocole : [MCP Specification](https://modelcontextprotocol.io/specification)
   - Transport : SSE (Server-Sent Events) sur Actix-web
   - Le code Phase 0 (REST `/mcp/v1/*`) reste actif en parallèle
   - Endpoint cible : `https://app.koprogo.be/mcp/sse`
   - Auth : JWT Bearer token (même auth que l'API KoproGo)
   
   ## Priorité
   
   Bloquant pour toutes les autres issues MCP tools (#253-#261, #265).

.. raw:: html

   </div>

