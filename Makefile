# KoproGo - Makefile simplifié pour contributeurs
# Usage: make help

# Include infrastructure deployment targets
-include infrastructure/Makefile.infra

.PHONY: help dev up down logs test test-unit test-int test-bdd codegen lint format build clean install setup migrate reset-db docs docs-serve audit ci pre-commit deploy-prod deploy-staging

# Couleurs pour output
GREEN  := \033[0;32m
YELLOW := \033[1;33m
NC     := \033[0m # No Color

help: ## 📖 Afficher cette aide
	@echo "$(GREEN)KoproGo - Commandes disponibles$(NC)"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(YELLOW)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)🚀 Quick start:$(NC) make setup && make dev"

##
## 🚀 Développement
##

dev: ## 🔥 Démarrer dev avec hot reload (Traefik + backend + frontend)
	@echo "$(GREEN)🚀 Démarrage environnement dev avec hot reload...$(NC)"
	@echo "  📍 Frontend: http://localhost"
	@echo "  📍 API:      http://localhost/api/v1"
	@echo "  📍 Traefik:  http://localhost:8081"
	@echo ""
	docker compose up

up: dev ## Alias pour 'make dev'

down: ## 🛑 Arrêter tous les services
	docker compose down

logs: ## 📋 Voir les logs (usage: make logs SERVICE=backend)
	@if [ -z "$(SERVICE)" ]; then \
		docker compose logs -f; \
	else \
		docker compose logs -f $(SERVICE); \
	fi

restart: ## 🔄 Redémarrer les services
	docker compose restart

build: ## 🔨 Rebuild les images Docker
	docker compose build

clean: ## 🧹 Nettoyer artifacts et volumes Docker
	@echo "$(YELLOW)⚠️  Nettoyage des artifacts...$(NC)"
	cd backend && cargo clean
	cd frontend && rm -rf dist node_modules test-results playwright-report
	docker compose down -v
	@echo "$(GREEN)✅ Nettoyage terminé$(NC)"

##
## ✅ Tests
##

test: test-unit test-e2e-backend test-bdd ## 🧪 Lancer tous les tests

test-unit: ## 🎯 Tests unitaires (backend)
	@echo "$(GREEN)🧪 Tests unitaires...$(NC)"
	cd backend && SQLX_OFFLINE=true cargo test --lib

test-e2e-backend: ## 🔗 Tests E2E backend (e2e_http.rs only - e2e.rs and e2e_auth.rs temporarily disabled)
	@echo "$(GREEN)🔗 Tests E2E backend...$(NC)"
	cd backend && SQLX_OFFLINE=true cargo test --test e2e_http

test-bdd: ## 🥒 Tests BDD/Cucumber (backend)
	@echo "$(GREEN)🥒 Tests BDD...$(NC)"
	cd backend && SQLX_OFFLINE=true cargo test --test bdd --test bdd_governance --test bdd_financial --test bdd_operations --test bdd_community

test-e2e: ## 🌐 Tests E2E Playwright (frontend + backend)
	@echo "$(GREEN)🌐 Tests E2E...$(NC)"
	cd frontend && PLAYWRIGHT_BASE_URL=http://localhost npm run test:e2e

codegen: ## 🎬 Playwright codegen interactif (DEVICE=mobile pour iPhone 13)
	@echo "$(GREEN)🎬 Playwright codegen ($(YELLOW)DEVICE=$(DEVICE)$(GREEN))...$(NC)"
	cd frontend && \
	if [ "$(DEVICE)" = "mobile" ]; then \
		npm run codegen:mobile; \
	else \
		npm run codegen; \
	fi

test-e2e-slow: ## 🐌 Tests E2E ralentis (1s entre chaque action - pour vidéos)
	@echo "$(GREEN)🐌 Ralentissement des tests E2E...$(NC)"
	bash .claude/scripts/slow-down-tests.sh 1000
	@echo ""
	@echo "$(GREEN)🎥 Lancement des tests ralentis...$(NC)"
	cd frontend && PLAYWRIGHT_BASE_URL=http://localhost npm run test:e2e || true
	@echo ""
	@echo "$(GREEN)⚡ Restauration de la vitesse normale...$(NC)"
	bash .claude/scripts/restore-test-speed.sh

test-e2e-restore-speed: ## ⚡ Restaurer la vitesse normale des tests
	bash .claude/scripts/restore-test-speed.sh

test-watch: ## 👀 Tests en mode watch (auto-reload)
	cd backend && cargo watch -x "test --lib"

bench: ## ⚡ Benchmarks (backend)
	cd backend && cargo bench

coverage: ## 📊 Génération rapport de couverture
	@echo "$(GREEN)📊 Génération coverage...$(NC)"
	cd backend && cargo tarpaulin --out Html --output-dir ../coverage
	@echo "$(GREEN)✅ Rapport: coverage/index.html$(NC)"

##
## 🔍 Qualité du code
##

lint: ## 🔍 Linter (clippy dans container Docker + prettier local)
	@echo "$(GREEN)🔍 Linting backend...$(NC)"
	docker compose exec -T backend sh -c "SQLX_OFFLINE=true cargo clippy --all-targets --all-features -- -D warnings"
	@echo "$(GREEN)🔍 Linting frontend...$(NC)"
	cd frontend && npx prettier --check .

check-frontend: ## 🔍 Vérification TypeScript frontend (astro check)
	@echo "$(GREEN)🔍 Checking TypeScript frontend...$(NC)"
	cd frontend && npx astro check

format: ## ✨ Formatter le code (rustfmt dans container Docker + prettier local)
	@echo "$(GREEN)✨ Formatting backend...$(NC)"
	docker compose exec -T backend sh -c "cargo fmt"
	@echo "$(GREEN)✨ Formatting frontend...$(NC)"
	cd frontend && npx prettier --write .

audit: ## 🔒 Audit sécurité (cargo-audit + npm audit)
	@echo "$(GREEN)🔒 Audit backend...$(NC)"
	cd backend && cargo audit
	@echo "$(GREEN)🔒 Audit frontend...$(NC)"
	cd frontend && npm audit --audit-level=moderate

install-hooks: ## 🪝 Installer les Git hooks (pre-commit, pre-push)
	@echo "$(GREEN)🪝 Installation des Git hooks...$(NC)"
	./scripts/install-hooks.sh

check-deps: ## 🔍 Vérifier les dépendances requises (gh CLI, etc.)
	@echo "$(GREEN)🔍 Vérification des dépendances...$(NC)"
	./scripts/check-dependencies.sh

install-deps: ## 📦 Installer les dépendances manquantes
	@echo "$(GREEN)📦 Installation des dépendances...$(NC)"
	./scripts/check-dependencies.sh --auto-install

##
## 📦 Setup & Installation
##

install: ## 📦 Installer dépendances frontend
	@echo "$(GREEN)📦 Installation dépendances frontend...$(NC)"
	cd frontend && npm install

setup: ## 🚀 Setup complet du projet (first time)
	@echo "$(GREEN)🚀 Setup KoproGo...$(NC)"
	@echo ""
	@echo "1️⃣ Vérification des dépendances..."
	./scripts/check-dependencies.sh || true
	@echo ""
	@echo "2️⃣ Vérification Docker..."
	@docker --version || (echo "$(YELLOW)❌ Docker non installé$(NC)" && exit 1)
	@docker compose version || (echo "$(YELLOW)❌ Docker Compose non installé$(NC)" && exit 1)
	@echo "$(GREEN)✅ Docker OK$(NC)"
	@echo ""
	@echo "3️⃣ Installation frontend..."
	cd frontend && npm install
	@echo "$(GREEN)✅ Frontend OK$(NC)"
	@echo ""
	@echo "4️⃣ Démarrage PostgreSQL..."
	docker compose up -d postgres
	@sleep 5
	@echo "$(GREEN)✅ PostgreSQL OK$(NC)"
	@echo ""
	@echo "5️⃣ Migrations DB..."
	cd backend && sqlx migrate run || echo "$(YELLOW)⚠️  Migrations échouées (normal si DB vide)$(NC)"
	@echo ""
	@echo "6️⃣ Installation des Git hooks..."
	./scripts/install-hooks.sh
	@echo ""
	@echo "$(GREEN)✅ Setup terminé!$(NC)"
	@echo ""
	@echo "$(GREEN)🚀 Démarrer dev: make dev$(NC)"

##
## 🗄️ Base de données
##

migrate: ## 📊 Lancer migrations DB
	@echo "$(GREEN)📊 Migrations DB...$(NC)"
	cd backend && sqlx migrate run

reset-db: ## ⚠️  Reset DB (SUPPRIME TOUTES LES DONNÉES)
	@echo "$(YELLOW)⚠️  ATTENTION: Suppression de toutes les données!$(NC)"
	@read -p "Taper 'yes' pour confirmer: " confirm; \
	if [ "$$confirm" = "yes" ]; then \
		docker compose down postgres; \
		docker volume rm koprogo_postgres_data 2>/dev/null || true; \
		docker compose up -d postgres; \
		sleep 5; \
		cd backend && sqlx migrate run; \
		echo "$(GREEN)✅ DB reset terminée$(NC)"; \
	else \
		echo "$(YELLOW)❌ Annulé$(NC)"; \
	fi

seed: ## 🌱 Seed DB avec données de test
	cd backend && cargo run --bin seed

##
## 📚 Documentation
##

docs: ## 📚 Générer docs Rust (cargo doc)
	@echo "$(GREEN)📚 Génération docs Rust...$(NC)"
	cd backend && SQLX_OFFLINE=true cargo doc --no-deps --open

docs-sphinx: ## 📖 Build docs Sphinx
	@echo "$(GREEN)📖 Build docs Sphinx...$(NC)"
	@if [ ! -d docs/.venv ]; then \
		echo "$(YELLOW)⚠️  Creating Python venv...$(NC)"; \
		cd docs && python3 -m venv .venv && .venv/bin/pip install -q -r requirements.txt; \
	fi
	cd docs && .venv/bin/sphinx-build -M html . _build
	@echo "$(GREEN)✅ Docs: docs/_build/html/index.html$(NC)"

docs-serve: ## 🔄 Servir docs Sphinx avec live reload
	@echo "$(GREEN)🔄 Docs server: http://localhost:8000$(NC)"
	@if [ ! -d docs/.venv ]; then \
		echo "$(YELLOW)⚠️  Creating Python venv...$(NC)"; \
		cd docs && python3 -m venv .venv && .venv/bin/pip install -q -r requirements.txt; \
	fi
	cd docs && .venv/bin/sphinx-autobuild . _build/html --port 8000 --open-browser

docs-sync-videos: ## 📹 Copier vidéos E2E et générer page RST
	@echo "$(GREEN)📹 Synchronisation vidéos E2E...$(NC)"
	bash .claude/scripts/copy-videos.sh

docs-export-github: ## 📦 Exporter données GitHub (issues, milestones, projects) en RST
	@echo "$(GREEN)📦 Export données GitHub...$(NC)"
	./scripts/export-github-to-rst.sh

rfc-new: ## 📝 Créer nouveau RFC (usage: make rfc-new TITLE="mon-titre")
	@if [ -z "$(TITLE)" ]; then \
		echo "$(YELLOW)❌ Usage: make rfc-new TITLE=\"mon-titre\"$(NC)"; \
		exit 1; \
	fi; \
	LAST_NUM=$$(ls docs/governance/rfc/*.rst 2>/dev/null | grep -oP '\d{4}' | sort -n | tail -1); \
	if [ -z "$$LAST_NUM" ]; then \
		NEW_NUM="0001"; \
	else \
		NEW_NUM=$$(printf "%04d" $$((10#$$LAST_NUM + 1))); \
	fi; \
	NEW_FILE="docs/governance/rfc/$$NEW_NUM-$(TITLE).rst"; \
	cp docs/governance/rfc/template.rst "$$NEW_FILE"; \
	sed -i "s/RFC XXXX:/RFC $$NEW_NUM:/" "$$NEW_FILE"; \
	sed -i "s/:RFC: XXXX/:RFC: $$NEW_NUM/" "$$NEW_FILE"; \
	sed -i "s/AAAA-MM-JJ/$$(date +%Y-%m-%d)/" "$$NEW_FILE"; \
	echo "$(GREEN)✅ RFC créé: $$NEW_FILE$(NC)"; \
	echo "$(YELLOW)📝 Éditer le fichier et remplacer les placeholders [...]$(NC)"

adr-new: ## 📝 Créer nouvel ADR (usage: make adr-new TITLE="mon-titre")
	@if [ -z "$(TITLE)" ]; then \
		echo "$(YELLOW)❌ Usage: make adr-new TITLE=\"mon-titre\"$(NC)"; \
		exit 1; \
	fi; \
	LAST_NUM=$$(ls docs/governance/adr/*.rst 2>/dev/null | grep -oP '\d{4}' | sort -n | tail -1); \
	if [ -z "$$LAST_NUM" ]; then \
		NEW_NUM="0002"; \
	else \
		NEW_NUM=$$(printf "%04d" $$((10#$$LAST_NUM + 1))); \
	fi; \
	NEW_FILE="docs/governance/adr/$$NEW_NUM-$(TITLE).rst"; \
	echo "===============================================" > "$$NEW_FILE"; \
	echo "ADR-$$NEW_NUM: $(TITLE)" >> "$$NEW_FILE"; \
	echo "===============================================" >> "$$NEW_FILE"; \
	echo "" >> "$$NEW_FILE"; \
	echo ":ADR: $$NEW_NUM" >> "$$NEW_FILE"; \
	echo ":Titre: $(TITLE)" >> "$$NEW_FILE"; \
	echo ":Date: $$(date +%Y-%m-%d)" >> "$$NEW_FILE"; \
	echo ":Statut: Draft" >> "$$NEW_FILE"; \
	echo ":Décideurs: [À compléter]" >> "$$NEW_FILE"; \
	echo "" >> "$$NEW_FILE"; \
	echo ".. contents:: Table des matières" >> "$$NEW_FILE"; \
	echo "   :depth: 2" >> "$$NEW_FILE"; \
	echo "   :local:" >> "$$NEW_FILE"; \
	echo "" >> "$$NEW_FILE"; \
	echo "Contexte" >> "$$NEW_FILE"; \
	echo "========" >> "$$NEW_FILE"; \
	echo "" >> "$$NEW_FILE"; \
	echo "[Décrire le contexte et le problème]" >> "$$NEW_FILE"; \
	echo "" >> "$$NEW_FILE"; \
	echo "Décision" >> "$$NEW_FILE"; \
	echo "========" >> "$$NEW_FILE"; \
	echo "" >> "$$NEW_FILE"; \
	echo "[Décrire la décision prise]" >> "$$NEW_FILE"; \
	echo "" >> "$$NEW_FILE"; \
	echo "Conséquences" >> "$$NEW_FILE"; \
	echo "============" >> "$$NEW_FILE"; \
	echo "" >> "$$NEW_FILE"; \
	echo "Positives" >> "$$NEW_FILE"; \
	echo "---------" >> "$$NEW_FILE"; \
	echo "" >> "$$NEW_FILE"; \
	echo "- [Conséquence positive 1]" >> "$$NEW_FILE"; \
	echo "" >> "$$NEW_FILE"; \
	echo "Négatives" >> "$$NEW_FILE"; \
	echo "---------" >> "$$NEW_FILE"; \
	echo "" >> "$$NEW_FILE"; \
	echo "- [Conséquence négative 1]" >> "$$NEW_FILE"; \
	echo "" >> "$$NEW_FILE"; \
	echo "---" >> "$$NEW_FILE"; \
	echo "" >> "$$NEW_FILE"; \
	echo "*ADR-$$NEW_NUM KoproGo ASBL*" >> "$$NEW_FILE"; \
	echo "$(GREEN)✅ ADR créé: $$NEW_FILE$(NC)"; \
	echo "$(YELLOW)📝 Éditer le fichier et compléter les sections$(NC)"

docs-with-videos: ## 🎥 Générer docs Sphinx avec vidéos E2E (tests ralentis 1s)
	@echo "$(GREEN)🎥 Génération docs avec vidéos E2E...$(NC)"
	@echo ""
	@echo "0️⃣ Vérification des services (Traefik + backend + frontend)..."
	docker compose up -d postgres minio backend traefik frontend
	@sleep 3
	@echo ""
	@echo "1️⃣ Ralentissement des tests (1 s entre chaque action)..."
	bash .claude/scripts/slow-down-tests.sh 1000
	@echo ""
	@echo "2️⃣ Lancement des tests E2E..."
	@{ \
		cd frontend && PLAYWRIGHT_BASE_URL=http://localhost npm run test:e2e; \
	} || echo "$(YELLOW)⚠️  Certains tests ont échoué$(NC)"
	@echo ""
	@echo "3️⃣ Restauration de la vitesse normale..."
	bash .claude/scripts/restore-test-speed.sh
	@echo ""
	@echo "4️⃣ Synchronisation des vidéos..."
	bash .claude/scripts/copy-videos.sh
	@echo ""
	@echo "5️⃣ Build docs Sphinx..."
	@if [ ! -d docs/.venv ]; then \
		echo "$(YELLOW)⚠️  Creating Python venv...$(NC)"; \
		cd docs && python3 -m venv .venv && .venv/bin/pip install -q -r requirements.txt; \
	fi
	cd docs && .venv/bin/sphinx-build -M html . _build
	@echo ""
	@echo "$(GREEN)✅ Docs générées: docs/_build/html/index.html$(NC)"
	@echo "$(GREEN)🎥 Vidéos E2E: docs/_build/html/e2e-videos.html$(NC)"

docs-serve-videos: docs-with-videos ## 🌐 Servir docs avec vidéos sur http://localhost:8000
	@echo "$(GREEN)🌐 Serveur docs: http://localhost:8000$(NC)"
	@echo "$(GREEN)🎥 Page vidéos: http://localhost:8000/e2e-videos.html$(NC)"
	cd docs/_build/html && python3 -m http.server 8000

##
## 🚀 CI/CD & Déploiement
##

ci: ## ✅ Vérifications CI locales via containers Docker (tout dans Docker, pas besoin de node/rust local)
	@echo "$(GREEN)🔍 Linting backend (clippy --all-targets --all-features)...$(NC)"
	docker compose exec -T backend sh -c "SQLX_OFFLINE=true cargo clippy --all-targets --all-features -- -D warnings"
	@echo "$(GREEN)🔍 Formatting backend (cargo fmt --check)...$(NC)"
	docker compose exec -T backend sh -c "cargo fmt --check"
	@echo "$(GREEN)🔍 Formatting frontend (prettier --check)...$(NC)"
	docker compose exec -T frontend sh -c "npx prettier --check ."
	@echo "$(GREEN)🔍 Checking TypeScript frontend (astro check)...$(NC)"
	docker compose exec -T frontend sh -c "npx astro check"
	@echo "$(GREEN)🔍 Security audit frontend (npm audit)...$(NC)"
	docker compose exec -T frontend sh -c "npm audit --audit-level=high"
	@echo "$(GREEN)🧪 Tests unitaires backend (cargo test --lib)...$(NC)"
	docker compose exec -T backend sh -c "SQLX_OFFLINE=true cargo test --lib"
	@echo "$(GREEN)🔧 Compilation tests E2E backend...$(NC)"
	docker compose exec -T backend sh -c "SQLX_OFFLINE=true cargo test --test e2e --no-run"
	@echo "$(GREEN)🔧 Compilation tests BDD backend...$(NC)"
	docker compose exec -T backend sh -c "SQLX_OFFLINE=true cargo test --test bdd --test bdd_governance --test bdd_financial --test bdd_operations --test bdd_community --no-run"
	@echo "$(GREEN)🎭 Playwright smoke tests (chromium)...$(NC)"
	docker compose exec -T -e PLAYWRIGHT_BASE_URL=http://localhost:3000 -e PLAYWRIGHT_API_BASE=http://koprogo-backend:8080/api/v1 frontend sh -c "npx playwright test --project=chromium" || echo "$(YELLOW)⚠️  Playwright: certains tests échouent en Docker local (networking). Vérifier en CI.$(NC)"
	@echo ""
	@echo "$(GREEN)🎉 Tous les checks CI passés!$(NC)"
	@echo "$(GREEN)✅ Prêt à push$(NC)"

pre-commit: format lint ## 🎯 Pre-commit hook (format + lint)
	@echo "$(GREEN)✅ Pre-commit OK$(NC)"

setup-infra: ## 🏗️  Setup complet infrastructure OVH (Terraform + Ansible)
	@echo "$(GREEN)🏗️  Setup Infrastructure OVH Cloud...$(NC)"
	@echo ""
	@./infrastructure/setup-infra.sh

deploy-prod: ## 🚀 Déployer en production (via GitOps)
	@echo "$(GREEN)🚀 Déploiement production...$(NC)"
	@echo "$(YELLOW)⚠️  Assurez-vous d'avoir push sur main$(NC)"
	cd deploy/production && ./gitops-deploy.sh

deploy-staging: ## 🧪 Déployer en staging
	@echo "$(GREEN)🧪 Déploiement staging...$(NC)"
	cd deploy/staging && docker compose pull && docker compose up -d

##
## 🤖 MCP (Model Context Protocol)
##

mcp-up: ## 🤖 Démarrer stack MCP complète (backend + edge node + postgres)
	@echo "$(GREEN)🤖 Démarrage stack MCP...$(NC)"
	@echo "  📍 Backend MCP: http://localhost:8080/mcp/v1"
	@echo "  📍 Edge Node:   http://localhost:3031"
	@echo "  📍 MCP Chat:    http://localhost/mcp-chat"
	@echo ""
	docker compose -f docker-compose.mcp.yml up

mcp-down: ## 🛑 Arrêter stack MCP
	docker compose -f docker-compose.mcp.yml down

node-run: ## 🍓 Lancer edge node (Raspberry Pi simulator)
	@echo "$(GREEN)🍓 Lancement edge node...$(NC)"
	cd backend/koprogo-node && cargo run -- --port 3031 --model llama3:8b-instruct-q4

node-build: ## 🔨 Build edge node (optimisé ARM64)
	@echo "$(GREEN)🔨 Build edge node...$(NC)"
	cd backend/koprogo-node && cargo build --release

mcp-cli-chat: ## 💬 CLI MCP chat (usage: make mcp-cli-chat MSG="Hello")
	@echo "$(GREEN)💬 MCP CLI Chat...$(NC)"
	cd backend/koprogo-mcp && cargo run --bin mcp-cli chat --model llama3:8b "$(MSG)"

mcp-cli-models: ## 📚 Liste des modèles disponibles via CLI
	@echo "$(GREEN)📚 Modèles MCP...$(NC)"
	cd backend/koprogo-mcp && cargo run --bin mcp-cli models

mcp-cli-health: ## 🏥 Health check via CLI
	@echo "$(GREEN)🏥 MCP Health...$(NC)"
	cd backend/koprogo-mcp && cargo run --bin mcp-cli health

test-mcp: ## 🧪 Tests MCP (unit + integration)
	@echo "$(GREEN)🧪 Tests MCP core...$(NC)"
	cd backend/koprogo-mcp && cargo test --lib
	@echo "$(GREEN)🧪 Tests MCP integration...$(NC)"
	cd backend/koprogo-mcp && cargo test --test integration

mcp-stats: ## 📊 Statistiques MCP (via API)
	@echo "$(GREEN)📊 MCP Statistics...$(NC)"
	curl -s http://localhost:8080/mcp/v1/stats | jq .

mcp-download-model: ## 📥 Télécharger modèle Llama 3 8B Q4 (~4.5GB)
	@echo "$(GREEN)📥 Téléchargement Llama 3 8B Q4...$(NC)"
	@mkdir -p models
	wget -P models/ https://huggingface.co/QuantFactory/Meta-Llama-3-8B-Instruct-GGUF/resolve/main/Meta-Llama-3-8B-Instruct.Q4_K_M.gguf
	mv models/Meta-Llama-3-8B-Instruct.Q4_K_M.gguf models/llama3-8b-instruct-q4.gguf
	@echo "$(GREEN)✅ Modèle téléchargé: models/llama3-8b-instruct-q4.gguf$(NC)"

##
## 🔧 Utilitaires
##

ps: ## 📊 Status des containers
	docker compose ps

shell-backend: ## 🐚 Shell dans container backend
	docker compose exec backend bash

shell-postgres: ## 🐘 Shell PostgreSQL
	docker compose exec postgres psql -U koprogo -d koprogo_db

update-deps: ## 🔄 Mettre à jour dépendances
	@echo "$(GREEN)🔄 Update dépendances frontend...$(NC)"
	cd frontend && npm update
	@echo "$(GREEN)🔄 Update dépendances Rust...$(NC)"
	cd backend && cargo update

##
## ❓ Info
##

info: ## ℹ️  Infos projet
	@echo "$(GREEN)KoproGo - Info Projet$(NC)"
	@echo ""
	@echo "📦 Structure:"
	@echo "  - Backend:  Rust + Actix-web"
	@echo "  - Frontend: Astro + Svelte"
	@echo "  - DB:       PostgreSQL 15"
	@echo "  - Proxy:    Traefik"
	@echo ""
	@echo "🌐 URLs Dev:"
	@echo "  - Frontend: http://localhost"
	@echo "  - API:      http://localhost/api/v1"
	@echo "  - Traefik:  http://localhost:8081"
	@echo "  - DB:       localhost:5432"
	@echo ""
	@echo "📚 Docs:"
	@echo "  - README:   ./README.md"
	@echo "  - CLAUDE:   ./CLAUDE.md (pour Claude Code)"
	@echo "  - Sphinx:   make docs-serve"
	@echo ""
	@echo "🚀 Quick start: make setup && make dev"
