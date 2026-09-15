## 📝 Description

<!-- Décrivez clairement les changements apportés par cette PR -->

## 🔗 Issue(s) Liée(s)

<!-- Référencez l'issue associée (ex: Closes #42, Fixes #123) -->

Closes #

## 🎯 Type de Changement

<!-- Cochez les cases appropriées avec [x] -->

- [ ] 🐛 Bug fix (changement non-breaking qui corrige un problème)
- [ ] ✨ New feature (changement non-breaking qui ajoute une fonctionnalité)
- [ ] 💥 Breaking change (correction ou fonctionnalité causant un changement incompatible)
- [ ] 📚 Documentation (mise à jour de la documentation uniquement)
- [ ] 🎨 Style (formatage, lint, sans changement de logique)
- [ ] ♻️ Refactoring (ni correction ni ajout de fonctionnalité)
- [ ] ⚡ Performance (amélioration des performances)
- [ ] ✅ Tests (ajout ou correction de tests)
- [ ] 🔧 Configuration (changements de config, build, CI/CD)

## 🏗️ Architecture Hexagonale

<!-- Indiquez les couches impactées -->

### Backend (Rust)

- [ ] **Domain Layer** (`backend/src/domain/`)
  - [ ] Entities modifiées/ajoutées
  - [ ] Services de domaine modifiés/ajoutés
  - [ ] Validation métier ajoutée

- [ ] **Application Layer** (`backend/src/application/`)
  - [ ] Use Cases modifiés/ajoutés
  - [ ] Ports (traits) modifiés/ajoutés
  - [ ] DTOs modifiés/ajoutés

- [ ] **Infrastructure Layer** (`backend/src/infrastructure/`)
  - [ ] Repositories modifiés/ajoutés
  - [ ] Handlers HTTP modifiés/ajoutés
  - [ ] Routes modifiées/ajoutées
  - [ ] Migrations base de données ajoutées

### Frontend (Astro + Svelte)

- [ ] **Components** (`frontend/src/components/`)
- [ ] **Pages** (`frontend/src/pages/`)
- [ ] **Stores** (`frontend/src/stores/`)
- [ ] **Types** (`frontend/src/lib/types.ts`)
- [ ] **API Client** (`frontend/src/lib/api.ts`)

### Infrastructure

- [ ] **Docker** (Dockerfile, docker-compose.yml)
- [ ] **Kubernetes** (Helm charts, manifests)
- [ ] **Terraform** (modules d'infrastructure)
- [ ] **Ansible** (playbooks, templates)

## ✅ Checklist Qualité

### Tests

- [ ] Tests unitaires ajoutés/mis à jour (Domain layer)
- [ ] Tests d'intégration ajoutés/mis à jour (Repositories)
- [ ] Tests BDD ajoutés/mis à jour (Cucumber `.feature`)
- [ ] Tests E2E ajoutés/mis à jour (Playwright)
- [ ] Tous les tests passent localement (`make test`)

### Code Quality

- [ ] Le code suit les conventions du projet (hexagonal architecture)
- [ ] Code formaté (`make format`)
- [ ] Lint passé sans warnings (`make lint`)
- [ ] Pas de code commenté superflu
- [ ] Noms de variables/fonctions clairs et descriptifs
- [ ] Commentaires ajoutés pour la logique complexe

### Documentation

- [ ] `CHANGELOG.md` mis à jour
- [ ] Documentation API mise à jour (si applicable)
- [ ] `README.md` mis à jour (si nécessaire)
- [ ] Docstrings ajoutées/mises à jour (Rust, TypeScript)
- [ ] Migration guide fourni (si breaking change)

### Base de Données

- [ ] Migration SQLx créée (`backend/migrations/`)
- [ ] Migration testée (up + down)
- [ ] Cache SQLx regénéré (`cargo sqlx prepare`)
- [ ] Indexes appropriés ajoutés
- [ ] Backward compatible OU plan de migration fourni

### Sécurité & GDPR

- [ ] Pas de credentials/secrets hardcodés
- [ ] Validation des entrées utilisateur
- [ ] Authorization checks en place
- [ ] Données sensibles protégées (GDPR)
- [ ] Audit logging ajouté (si applicable)
- [ ] Tests de sécurité effectués

### Gouvernance Agent (Tier 1/Tier 2)

<!-- cf. .claude/rules/CRITICAL.md §11 — à cocher seulement si cette PR inclut
     du travail d'agent IA Tier 2 (lecture, diagnostic, proposition, reporting) -->

- [ ] Cette PR inclut du travail d'agent IA Tier 2
- [ ] Si coché ci-dessus : journal daté ajouté dans `docs/agent-activity/`
      (cf. [`docs/agent-activity/README.md`](../docs/agent-activity/README.md)) —
      sinon, demander la mise à jour avant merge

### CI/CD

- [ ] GitHub Actions CI passe
- [ ] Security audit passe (`make audit`)
- [ ] Build Docker réussit (si modifié)
- [ ] Compatible SQLX_OFFLINE mode

## 🧪 Comment Tester

<!-- Décrivez les étapes pour tester cette PR -->

1. Checkout cette branche: `git checkout <branch-name>`
2. Install dependencies: `make setup`
3. Run migrations: `make migrate`
4. Start backend: `make dev`
5. Start frontend: `make dev-frontend`
6. Testez: ...

**Identifiants de test (après seed):**
```
SuperAdmin: admin@koprogo.com / admin123
Syndic: syndic@grandplace.be / syndic123
```

## 📸 Screenshots/Vidéos

<!-- Si changements UI, ajoutez des captures d'écran ou vidéos -->

| Avant | Après |
|-------|-------|
| [screenshot] | [screenshot] |

## 🌱 Impact Écologique

<!-- Optionnel: si cette PR impacte les performances ou l'empreinte carbone -->

- [ ] Réduit l'empreinte carbone (optimisation)
- [ ] Neutre
- [ ] Augmente l'empreinte (justifiez pourquoi nécessaire)

## 🔄 Breaking Changes

<!-- Si breaking change, décrivez l'impact et le plan de migration -->

**Impact:**
- Quoi: ...
- Qui est affecté: ...
- Migration nécessaire: ...

**Plan de migration:**
1. ...
2. ...

## 📋 Checklist Finale

- [ ] J'ai testé cette PR localement
- [ ] J'ai vérifié qu'elle fonctionne avec les données de seed
- [ ] J'ai lu et suivi le [CONTRIBUTING.md](../CONTRIBUTING.md)
- [ ] Cette PR est prête pour review
- [ ] J'ai assigné un reviewer (optionnel)

## 💬 Notes pour les Reviewers

<!-- Informations additionnelles pour faciliter la review -->

**Points d'attention particuliers:**
- ...

**Questions ouvertes:**
- ...

---

**Merci pour votre contribution à KoproGo! 🌱**
