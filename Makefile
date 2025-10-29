# KoproGo - Makefile simplifié pour contributeurs
# Usage: make help

.PHONY: help dev up down logs test test-unit test-int test-bdd lint format build clean install setup migrate reset-db docs docs-serve audit ci pre-commit deploy-prod deploy-staging

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

test-e2e-backend: ## 🔗 Tests E2E backend (e2e.rs, e2e_auth.rs, e2e_http.rs)
	@echo "$(GREEN)🔗 Tests E2E backend...$(NC)"
	cd backend && SQLX_OFFLINE=true cargo test --test e2e --test e2e_auth --test e2e_http

test-bdd: ## 🥒 Tests BDD/Cucumber (backend)
	@echo "$(GREEN)🥒 Tests BDD...$(NC)"
	cd backend && SQLX_OFFLINE=true cargo test --test bdd

test-e2e: ## 🌐 Tests E2E Playwright (frontend + backend)
	@echo "$(GREEN)🌐 Tests E2E...$(NC)"
	cd frontend && npm run test:e2e

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

lint: ## 🔍 Linter (clippy + prettier)
	@echo "$(GREEN)🔍 Linting backend...$(NC)"
	cd backend && SQLX_OFFLINE=true cargo clippy --all-targets --all-features -- -D warnings
	@echo "$(GREEN)🔍 Linting frontend...$(NC)"
	cd frontend && npx prettier --check .

check-frontend: ## 🔍 Vérification TypeScript frontend (astro check)
	@echo "$(GREEN)🔍 Checking TypeScript frontend...$(NC)"
	cd frontend && npx astro check

format: ## ✨ Formatter le code (rustfmt + prettier)
	@echo "$(GREEN)✨ Formatting backend...$(NC)"
	cd backend && cargo fmt
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

##
## 📦 Setup & Installation
##

install: ## 📦 Installer dépendances frontend
	@echo "$(GREEN)📦 Installation dépendances frontend...$(NC)"
	cd frontend && npm install

setup: ## 🚀 Setup complet du projet (first time)
	@echo "$(GREEN)🚀 Setup KoproGo...$(NC)"
	@echo ""
	@echo "1️⃣ Vérification Docker..."
	@docker --version || (echo "$(YELLOW)❌ Docker non installé$(NC)" && exit 1)
	@docker compose version || (echo "$(YELLOW)❌ Docker Compose non installé$(NC)" && exit 1)
	@echo "$(GREEN)✅ Docker OK$(NC)"
	@echo ""
	@echo "2️⃣ Installation frontend..."
	cd frontend && npm install
	@echo "$(GREEN)✅ Frontend OK$(NC)"
	@echo ""
	@echo "3️⃣ Démarrage PostgreSQL..."
	docker compose up -d postgres
	@sleep 5
	@echo "$(GREEN)✅ PostgreSQL OK$(NC)"
	@echo ""
	@echo "4️⃣ Migrations DB..."
	cd backend && sqlx migrate run || echo "$(YELLOW)⚠️  Migrations échouées (normal si DB vide)$(NC)"
	@echo ""
	@echo "5️⃣ Installation des Git hooks..."
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

##
## 🚀 CI/CD & Déploiement
##

ci: lint check-frontend test audit ## ✅ Vérifications CI locales (avant push)
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
