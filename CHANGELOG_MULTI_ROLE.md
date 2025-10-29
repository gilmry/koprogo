# Changelog - Multi-role Support (feat/multi-roles-users)

**Date**: 2025-10-29
**Branch**: `feat/multi-roles-users`
**Base**: `main`
**Issue**: Closes #28

---

## 🎯 Vue d'ensemble

Implémentation complète du support multi-rôles pour les utilisateurs, permettant à un seul compte d'avoir plusieurs rôles (syndic, comptable, superadmin) avec changement de rôle actif instantané.

---

## 🔧 Backend - Core Changes

### Domain Layer

**Nouvelle entité** : `UserRoleAssignment` (`backend/src/domain/entities/user_role_assignment.rs`)
- Représente l'association user ↔ rôle ↔ organisation
- Attributs : `id`, `user_id`, `role`, `organization_id`, `is_primary`, `created_at`

### Database

**Migration** : `backend/migrations/20250130000000_add_user_roles.sql`
- Nouvelle table `user_roles` (user_id, role, organization_id, is_primary, timestamps)
- Index unique sur `(user_id, role, organization_id)` pour éviter les doublons
- Contrainte : un seul rôle `is_primary = true` par utilisateur

### Application Layer

**Nouveau repository** : `backend/src/application/ports/user_role_repository.rs`
- Trait `UserRoleRepository` avec méthodes :
  - `create` : ajouter un nouveau rôle à un utilisateur
  - `find_by_id` : récupérer un rôle spécifique
  - `list_for_user` : lister tous les rôles d'un utilisateur
  - `set_primary_role` : définir le rôle actif

**Implémentation** : `backend/src/infrastructure/database/repositories/user_role_repository_impl.rs`
- Implémentation PostgreSQL du repository avec gestion transactionnelle

**DTOs enrichis** (`backend/src/application/dto/auth_dto.rs`) :
- `UserRoleSummary` : résumé d'un rôle (id, role, organization_id, is_primary)
- `UserResponse` : ajout de `roles: Vec<UserRoleSummary>` et `active_role: Option<UserRoleSummary>`
- `SwitchRoleRequest` : payload pour changer de rôle
- `Claims` (JWT) : ajout de `role_id: Option<Uuid>`

**Use cases refactorisés** (`backend/src/application/use_cases/auth_use_cases.rs`) :
- `login` : retourne les rôles et le rôle actif dans la réponse
- `register` : crée automatiquement un `UserRoleAssignment` primaire
- `switch_active_role` : **nouveau** - permet de changer le rôle actif (génère nouveau JWT)
- `refresh_token` : préserve le rôle actif lors du rafraîchissement
- `get_user_by_id` : retourne le profil enrichi avec tous les rôles
- Méthodes privées :
  - `ensure_role_assignments` : garantit que chaque utilisateur a au moins un rôle
  - `apply_active_role_metadata` : synchronise les métadonnées du rôle actif
  - `build_user_response` : construit la réponse standardisée
  - `summarize_role` : convertit `UserRoleAssignment` → `UserRoleSummary`

### Infrastructure - Web

**Handler updates** (`backend/src/infrastructure/web/handlers/auth_handlers.rs`) :
- `switch_role_handler` : **nouveau endpoint** - `POST /api/v1/auth/switch-role`
- Gestion des erreurs enrichie

**Middleware** (`backend/src/infrastructure/web/middleware.rs`) :
- `AuthenticatedUser` : ajout de `role_id: Option<Uuid>` extrait du JWT

**Routes** (`backend/src/infrastructure/web/routes.rs`) :
- Ajout de `/auth/switch-role` (POST)

**Seed data** (`backend/src/infrastructure/database/seed.rs`) :
- Création automatique de `UserRoleAssignment` pour les utilisateurs de test
- Support multi-rôles dans les fixtures

**Main** (`backend/src/main.rs`) :
- Injection du `user_role_repo` dans `AuthUseCases`

---

## 🎨 Frontend - UI Changes

### Stores

**Auth store** (`frontend/src/stores/auth.ts`) :
- `authStore.user` : type enrichi avec `roles` et `active_role`
- `authStore.switchRole(role_id)` : **nouvelle méthode** - appelle `/auth/switch-role`
- Mise à jour automatique du JWT dans localStorage après switch

### Components

**Navigation** (`frontend/src/components/Navigation.svelte`) :
- **Nouveau** : sélecteur de rôle (dropdown) affichant tous les rôles disponibles
- Badge visuel du rôle actif
- Gestion des erreurs de switch

**Formulaires** (`LoginForm.svelte`, `RegisterForm.svelte`) :
- Affichage des rôles disponibles après login/register
- Toast de confirmation après switch

**Admin** (`UserListAdmin.svelte`, `UserForm.svelte`) :
- Ajustements pour supporter les nouveaux champs `roles[]` et `active_role`

### Types

**TypeScript** (`frontend/src/lib/types.ts`) :
- `UserRoleSummary` : interface miroir du backend
- `User` : ajout de `roles` et `active_role`

---

## 🧪 Tests

### Integration Tests

**Nouveaux fichiers** :
- `backend/tests/e2e_auth.rs` : scénarios complets multi-rôles
  - Création d'utilisateurs avec plusieurs rôles
  - Switch entre rôles
  - Validation du JWT après switch
  - Tests de permissions basées sur le rôle actif

### BDD Features

**Cucumber** (`backend/tests/features/auth.feature`) :
- **Nouveau scenario** : "Un utilisateur peut basculer entre plusieurs rôles" (issue #28)
  ```gherkin
  Given un utilisateur avec 2 rôles (syndic et comptable)
  When il se connecte et change de rôle
  Then son profil reflète le nouveau rôle actif
  ```

**BDD runner** (`backend/tests/bdd.rs`) :
- Steps implémentés pour tester le multi-rôle

---

## 📚 Documentation

### CLAUDE.md
- Section **User roles** ajoutée dans "API Endpoints"
- Section **Multi-role support** ajoutée dans "Domain Entities"
- Détails sur les endpoints `/auth/login`, `/auth/switch-role`, `/auth/me`

### README.md
- Ajout de la feature "Multi-rôles utilisateurs" dans la section Features
- Lien vers `docs/MULTI_ROLE_SUPPORT.md`

### Nouvelle documentation produit
- `docs/MULTI_ROLE_SUPPORT.md` : **nouveau fichier** - guide complet du support multi-rôle
  - Architecture (domain, use cases, repository)
  - Flow de login et switch
  - API endpoints détaillés
  - Exemples d'intégration frontend
  - Tests

---

## 🗃️ Database - SQLx Metadata

**Fichiers supprimés** (anciens queries obsolètes) :
- `query-2b053874...json` (ancien UPDATE users)
- `query-3600312...json` (ancien deactivate user)
- `query-38944562...json` (ancien activate user)
- `query-a16ef5e4...json` (ancien INSERT users)

**Nouveaux fichiers** (queries multi-rôles) :
- `query-1c327776...json` (INSERT user_roles)
- `query-2b28d108...json` (UPDATE user_roles primary)
- `query-4e12da7e...json` (SELECT user_roles by ID)
- `query-5cfc0197...json` (SELECT user_roles for user)
- `query-6e963862...json` (UPDATE users with role metadata)
- `query-829e2757...json` (SELECT users by email with roles)
- `query-99ef329a...json` (SELECT users by ID with roles)
- `query-9e56e5c5...json` (SELECT all user_roles)
- `query-a48b74cc...json` (DELETE user_roles)
- `query-e5be99ee...json` (INSERT users with roles)

---

## 🔒 Security & Permissions

- **JWT Claims** : enrichi avec `role_id` pour tracer le rôle actif
- **Middleware** : validation du `role_id` dans les requêtes protégées
- **Repository** : contraintes DB garantissent l'unicité du rôle primaire

---

## 🚀 API Endpoints (Nouveau/Modifié)

| Méthode | Endpoint | Description | Changement |
|---------|----------|-------------|-----------|
| POST | `/auth/login` | Connexion | ✏️ Retourne `roles[]` et `active_role` |
| POST | `/auth/register` | Inscription | ✏️ Retourne `roles[]` et `active_role` |
| POST | `/auth/switch-role` | Changer rôle actif | ✨ **NOUVEAU** |
| GET | `/auth/me` | Profil utilisateur | ✏️ Retourne `roles[]` et `active_role` |
| POST | `/auth/refresh` | Rafraîchir token | ✏️ Préserve le rôle actif |

---

## 🎯 Issue Tracking

- Résout : **#28** - Support multi-rôles utilisateurs
- Dépendances : Aucune
- Impact : Compatible avec les données existantes (migration automatique)

---

## ✅ Checklist de déploiement

- [x] Migration database (`20250130000000_add_user_roles.sql`)
- [x] Tests unitaires (domain layer)
- [x] Tests d'intégration (PostgreSQL)
- [x] Tests E2E (auth flow)
- [x] Tests BDD (Cucumber)
- [x] Documentation technique (CLAUDE.md)
- [x] Documentation produit (MULTI_ROLE_SUPPORT.md)
- [x] Frontend UI (sélecteur de rôle)
- [x] Seed data compatible

---

## 📝 Notes techniques

### Backward compatibility
✅ Les utilisateurs existants sans `user_roles` sont automatiquement migrés lors du premier login via `ensure_role_assignments()`

### Performance
- Requêtes optimisées avec index sur `(user_id, role, organization_id)`
- JOIN minimal dans `list_for_user` (1 requête pour récupérer tous les rôles)

### Sécurité
- Validation que le `role_id` appartient bien au `user_id` dans `switch_role`
- Refresh tokens révoqués lors du switch pour forcer nouvelle session

---

## 📊 Files Changed

- **Backend**: 47 files (13 new, 34 modified)
  - Domain: 1 new entity
  - Application: 1 new port + 1 new repository + DTOs/use cases refactored
  - Infrastructure: 1 new handler + middleware/routes/seed updates
  - Tests: 2 new test files (E2E + BDD)
- **Frontend**: 8 files (5 modified components, types, stores)
- **Documentation**: 3 files (CLAUDE.md, README.md, MULTI_ROLE_SUPPORT.md)
- **Database**: 1 migration + 14 SQLx metadata files (4 deleted, 10 added)

**Total**: 58 files changed

---

**Status** : ✅ Prêt pour review et merge
