# Multi-role Support Guide

Ce guide explique comment le module multi-rôles permet d'associer plusieurs responsabilités à un utilisateur KoproGo.

> **Modèle d'accès (état au 03/11/2026)** : l'ACP est titulaire des données ;
> le cabinet syndic (`organization_id`) y reçoit un mandat révocable. L'accès
> est contrôlé dans l'application en comparant l'organisation de l'utilisateur
> à celle de l'ACP. Il n'y a pas de RLS actif, et l'accès par mandat actif
> (ADR 0053) n'est pas encore implémenté.

---

## Objectifs

- Autoriser plusieurs rôles actifs pour un même utilisateur (ex. syndic & comptable).
- Permettre de choisir le rôle actif et de l'inclure dans le JWT.
- Documenter l'API, la base de données et les parcours BDD associées à l'issue #28.

---

## Modèle de données

```
users
└── user_roles (jonction)
    • user_id (FK users.id)
    • role (ENUM: superadmin, syndic, accountant, owner)
    • organization_id (FK organizations.id, nullable)
    • is_primary (bool)
    • created_at / updated_at
```

### Invariants

- Au moins une entrée `user_roles` par utilisateur (migration de rétrocompatibilité).
- Un seul rôle `is_primary = true` par utilisateur (`idx_user_roles_primary_per_org`).
- Les tokens JWT portent `role`, `organization_id`, `role_id`.

---

## API

| Endpoint | Méthode | Description |
|----------|---------|-------------|
| `/auth/login` | POST | Retourne `roles[]`, `active_role` et tokens. |
| `/auth/switch-role` | POST | Sélectionne un rôle secondaire (JWT mis à jour). |
| `/auth/me` | GET | Renvoie l'utilisateur courant avec la liste des rôles. |

### Exemple de réponse `login`

```jsonc
{
  "token": "…",
  "refresh_token": "…",
  "user": {
    "id": "…",
    "email": "alice@example.com",
    "role": "syndic",
    "organization_id": "…",
    "roles": [
      { "id": "…", "role": "syndic", "organization_id": "…", "is_primary": true },
      { "id": "…", "role": "accountant", "organization_id": "…", "is_primary": false }
    ],
    "active_role": { "id": "…", "role": "syndic", "organization_id": "…", "is_primary": true }
  }
}
```

---

## Tests et couverture

- **Unitaires** : validations `UserRoleAssignment`, conversions DTO, `AuthUseCases`.
- **Intégration** : `PostgresUserRoleRepository` (création, switch primary).
- **E2E Backend** : `tests/e2e_auth.rs` (ajout de second rôle, switch via use-case).
- **BDD** : `auth.feature` (scénario multi-rôles) aligne l'issue #28.
- **Frontend** : store `auth.ts` + composant `Navigation.svelte` (sélecteur de rôle).

---

## Flux métier type (Gherkin)

```
Given a coproperty management system
And a user with multiple roles
When I switch to the secondary role
Then my active role should be "accountant"
And the user response should list multiple roles
And the JWT claims should use role "accountant"
And the JWT claims should reference the selected role
```

---

## Migrations et seeds

- Migration `20250130000000_add_user_roles.sql` :
  - crée la table `user_roles`
  - backfill les utilisateurs existants avec `is_primary = true`
- `DatabaseSeeder` :
  - garantit l'assignation du rôle superadmin
  - les utilisateurs de démonstration reçoivent un enregistrement `user_roles`

---

## Frontend

- `authStore.switchRole` appelle `/auth/switch-role` avec JWT.
- `Navigation.svelte` rend le sélecteur (badge de rôle + redirection).
- Les informations `roles` / `activeRole` sont persistées offline.

---

## Migration

1. Déployer la migration SQL.
2. Redémarrer l'API (reconstruit AppState avec `PostgresUserRoleRepository`).
3. Vidanger les sessions (les nouveaux tokens incluent `role_id`).
4. Vérifier la nouvelle BDD BDD (`make test` inclut le scénario multi-rôles).

---

🎯 Alignement Issue #28

- ✅ Table `user_roles`
- ✅ `AuthenticatedUser` expose `role_id`
- ✅ Use case `switch_active_role`
- ✅ UI rôle-switcher
- ✅ Tests BDD + e2e démontrant le parcours métier
