# CLAUDE.md — Guide d'agent KoproGo

> **Première lecture pour tout nouvel agent** (Claude Code, Cowork, agents distants) :
> [`Maury/README.md`](Maury/README.md) — la méthode + positionnement.
>
> **Garde-fous actifs** : [`.claude/AGENT_GUARDRAILS.md`](.claude/AGENT_GUARDRAILS.md)
> **Règles non négociables** (injectées à chaque prompt) : [`.claude/rules/CRITICAL.md`](.claude/rules/CRITICAL.md)
>
> Issues GitHub clés : [#425](https://github.com/gilmry/koprogo/issues/425) garde-fous build • [#426](https://github.com/gilmry/koprogo/issues/426) cleanup docs • [#427](https://github.com/gilmry/koprogo/issues/427) validation TDD/BDD+Cowork • [#428](https://github.com/gilmry/koprogo/issues/428) simulation org • [#429](https://github.com/gilmry/koprogo/issues/429) runtime ops

---

## Aperçu projet

KoproGo est un SaaS de gestion de copropriété belge construit en **architecture hexagonale + DDD**.

- **Backend** : Rust + Actix-web 4 + sqlx
- **Frontend** : Astro + Svelte 5 (runes)
- **DB** : PostgreSQL 15
- **Tests** : unit + integration (testcontainers) + BDD (cucumber) + E2E (Playwright)
- **i18n** : FR / NL / EN / DE

**État courant** : v0.1.0 pre-release, aucun système en production. Phase de stabilisation via les recettes d'industrialisation IA (#425-#429).

---

## Quick start (dev)

```bash
make setup              # premier setup
make dev                # docker compose : Postgres + backend (cargo-watch hot reload) + frontend (Astro hot reload) + Traefik
make test               # tous les tests
make ci                 # CI complet (lint + check + test + secret-scan)
make claude-check       # valider la config guardrails IA
```

### URLs dev (mode localhost)

- Frontend : `http://localhost`
- Backend API : `http://localhost/api/v1`
- Traefik UI : `http://localhost:8081`
- Postgres : `localhost:5432` (user `koprogo`, db `koprogo_db`)

### Logs backend

```bash
docker compose logs -f backend
```

`cargo-watch` tourne dans le container backend (rebuild automatique). **Ne pas** lancer `cargo build` ou `cargo check` à la main — c'est déjà fait en boucle par cargo-watch.

---

## Architecture hexagonale (règles invariables)

3 couches avec dépendances descendantes uniquement :

```
Domain (entités + invariants)         ← zéro dépendance infra
   ↑
Application (ports = traits + use cases + DTOs)
   ↑
Infrastructure (adapters PostgreSQL + Actix handlers)
```

**Règle de pureté** : `backend/src/domain/` ne doit JAMAIS importer `sqlx`, `actix_web`, ni aucun crate infra. Toute violation = rejet en review.

### Pattern d'ajout de feature (ordre Maury)

1. **Brief / PRD / Architecture / Stories** signés *avant* de coder (cf. méthode Maury).
2. Entité dans `backend/src/domain/entities/<x>.rs` avec invariants validés en `::new()`.
3. Trait port dans `backend/src/application/ports/<x>_repository.rs`.
4. Use case dans `backend/src/application/use_cases/<x>_use_cases.rs`.
5. Repository PostgreSQL dans `backend/src/infrastructure/database/repositories/<x>_repository_impl.rs`.
6. Handler Actix dans `backend/src/infrastructure/web/handlers/<x>_handlers.rs`.
7. **Tests 4 catégories obligatoires** (cf. ci-dessous).
8. Frontend Svelte 5 (runes) si user-facing : composant + tests Vitest + scénario E2E.

---

## Discipline tests (TDD/BDD 4 catégories)

**Pour TOUT élément public livrable** (handler API, use case, entité de domaine, composant Svelte interactif) :

| Catégorie | Couvre |
|---|---|
| `@happy` | chemin nominal end-to-end |
| `@edge` | bornes (max/min/empty/0/1/N, dates limites) |
| `@security` | RBAC, auth, injection, rate limit, escalade |
| `@negative` | défaillance correcte (pas de panic, erreur typée, message utilisateur correct) |

**RED-first** : écrire le test échouant AVANT le code (hook PreToolUse à venir, cf. #427).

**`Result<E, AppError>` typé**, jamais `Result<_, String>`. `unwrap()` / `expect()` interdits hors tests — utiliser `?` + `AppError`. Hook PostToolUse warn sur introduction.

**Bibliothèque baseline** (à venir, cf. #427 §A.2) : `backend/tests/features/_security_baseline.feature` — scénarios génériques d'erreur réutilisables (401, 403, 422, 429, 503, etc.).

### Commandes de test

```bash
cargo test --lib                             # unit (in-module #[cfg(test)])
cargo test --test integration                # integration (testcontainers)
cargo test --test bdd                        # BDD/Cucumber (backend/tests/features/)
cargo test --test bdd -- --tags @security    # filtrer par catégorie
cd frontend && npm run test                  # Vitest composants
cd frontend && npx playwright test           # E2E
make coverage                                # tarpaulin → coverage/index.html
```

---

## Multi-rôles E2E (acteurs métier corrects)

Scénarios E2E avec **les bons rôles** :

| Workflow | Acteurs |
|---|---|
| Ticket | Owner crée → Syndic assigne → Owner valide |
| Vote AG | Syndic crée résolution → Owner vote → Syndic clôture |
| SEL (échanges) | Owner A offre ↔ Owner B demande (jamais syndic) |
| Convocations | Syndic envoie → Owner confirme |
| Devis | Syndic invite → Contractor soumet → Syndic accepte |
| Paiements | Owner configure son moyen de paiement |

**Pattern multi-rôle dans un même test** :
```typescript
await humanLogin(page, syndicEmail, syndicPassword);
// ... actions syndic ...
await stepPause(page);
await page.goto("/login");
await humanLogin(page, ownerEmail, ownerPassword);
// ... actions owner ...
await finalPause(page);
```

Pas un seul login pour tout le scénario. Voir [`docs/E2E_TESTING_GUIDE.rst`](docs/E2E_TESTING_GUIDE.rst).

---

## Où trouver quoi

| Sujet | Fichier |
|---|---|
| Méthode complète (CTO + armée d'agents) | [`Maury/README.md`](Maury/README.md) → `Maury/Méthode Maury.md` |
| Garde-fous IA actifs | [`.claude/AGENT_GUARDRAILS.md`](.claude/AGENT_GUARDRAILS.md) |
| Règles top-11 (injectées à chaque prompt) | [`.claude/rules/CRITICAL.md`](.claude/rules/CRITICAL.md) |
| Roadmap par capacités (jalons) | [`docs/ROADMAP_PAR_CAPACITES.rst`](docs/ROADMAP_PAR_CAPACITES.rst) |
| Sécurité runtime (LUKS, Suricata, etc.) | [`infrastructure/SECURITY.md`](infrastructure/SECURITY.md) |
| Endpoints API (généré) | [`docs/api/`](docs/api/) |
| Multi-owner (relation many-to-many) | [`docs/MULTI_OWNER_SUPPORT.md`](docs/MULTI_OWNER_SUPPORT.md) |
| Multi-rôle (assignments) | [`docs/MULTI_ROLE_SUPPORT.md`](docs/MULTI_ROLE_SUPPORT.md) |
| Comptabilité PCMN belge | [`docs/BELGIAN_ACCOUNTING_PCMN.rst`](docs/BELGIAN_ACCOUNTING_PCMN.rst) |
| Convocations AG (Art. 3.87 §3 CC) | [`docs/CONVOCATIONS_AG.rst`](docs/CONVOCATIONS_AG.rst) |
| Notifications multi-canal | [`docs/NOTIFICATIONS_SYSTEM.rst`](docs/NOTIFICATIONS_SYSTEM.rst) |
| GDPR | [`docs/GDPR_COMPLIANCE.rst`](docs/GDPR_COMPLIANCE.rst) |
| Workflow Git hooks locaux | [`docs/GIT_HOOKS.rst`](docs/GIT_HOOKS.rst) |
| Test-driven emergence (BDD↔E2E) | [`docs/E2E_TESTING_GUIDE.rst`](docs/E2E_TESTING_GUIDE.rst) |
| Snapshots historiques | [`docs/archive/`](docs/archive/) |

---

## DB / Environnement

- **PostgreSQL 15** via Docker (dev), Vault/SealedSecrets en prod (cf. #429).
- Connection dev : `postgresql://koprogo:koprogo123@localhost:5432/koprogo_db`.
- Migrations : `cd backend && sqlx migrate run` ou `make migrate`.
- SQLX offline (compile sans DB live) : `export SQLX_OFFLINE=true` (auto avec `make lint` / `make docs`).

---

## Performance (cibles ajustées)

- **Latence P99 backend** : ≤ 500ms en prod sur endpoints critiques.
  *(La cible historique 5ms était irréaliste — cf. #429 §6 SLO.)*
- **Throughput** : ≥ 1k req/s sur instance unique.
- **Mémoire** : ≤ 256MB par instance backend.
- **Pool PostgreSQL** : 10 connexions max par instance.
- **Build release** : LTO + opt-level 3 + codegen-units 1 (cf. `Cargo.toml`).

---

## Branches & commits

- `main` : protégée, intégration vers prod (jamais autonome — cf. #429 deploy gate).
- `feature/<X>` ou `story/<STORY-ID>` : développement (Maury impose `story/<ID>` avec story signée).
- `release/vX.Y.Z` : préparation release (passe par Cowork+humain review #427 + doc auto-refresh #428 §8bis avant tag).

**Pas de force-push, pas de `--no-verify`** (deny dans `.claude/settings.json`).

### Commit message convention

```
type(scope): titre court (≤ 70 chars)

Description plus longue si besoin.

Refs: #issue
Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

Types : `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `security`, `perf`.

---

## Pour aller plus loin

- **Architecture & ADRs** : `docs/architecture/`, `docs/adr/` (à venir cf. #428).
- **Roadmap par capacités** : `docs/ROADMAP_PAR_CAPACITES.rst`.
- **Recettes d'industrialisation IA** : suivre les issues #425-#429 et `Maury/CHANGELOG.md`.
- **Audit qualité 2026-04-29** : voir issues #425/#426 (constats + plans).

---

*Ce fichier est volontairement **minimal** (cible ≤ 5 000 tokens). Roadmap, listing API exhaustif, journal "✅ NOUVEAU", entités domaine listing → vivent dans `docs/` (auto-régénérés à chaque release cf. #428 §8bis), pas ici. Chaque byte chargé à chaque session est une taxe — voir #428 §2 stratégie consolidation tokens.*
