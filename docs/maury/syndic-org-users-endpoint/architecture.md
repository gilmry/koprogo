---
feature: syndic-org-users-endpoint
phase: C (Information systems architecture TOGAF)
status: DRAFT — en attente signature @gilmry
date: 2026-08-07
authors: [Claude Sonnet 5 (drafting)]
depends_on: brief.md (SIGNED v1.0), prd.md (DRAFT)
---

# Architecture — Endpoint de listing des users pour le syndic (org-scopé)

## 1. Stack confirmé

Rust + Actix-web 4 + sqlx (backend), Astro + Svelte 5 runes (frontend) — cf. `CLAUDE.md`. Aucune nouvelle dépendance. Le repository `find_by_organization()` existe déjà (`user_repository_impl.rs:126`) — zéro nouvelle logique DB.

## 2. Backend — pattern org-scopé standard (mirror `list_organization_tickets`)

### 2.1. Handler

Fichier : `backend/src/infrastructure/web/handlers/user_handlers.rs`, à côté de `list_users` (`GET /users`, superadmin-only, inchangé).

```rust
#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/users",
    tag = "Users",
    summary = "List users for an organization (syndic/accountant own org, superadmin any org)",
    params(
        ("organization_id" = Uuid, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "List of users", body = Vec<UserResponse>),
        (status = 403, description = "Access denied — resource belongs to another organization"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/organizations/{organization_id}/users")]
pub async fn list_organization_users(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    organization_id: web::Path<Uuid>,
) -> impl Responder {
    if let Err(e) = user.verify_org_access(*organization_id) {
        return HttpResponse::Forbidden().json(serde_json::json!({"error": e}));
    }
    match state.user_use_cases.list_by_organization(*organization_id).await {
        Ok(users) => HttpResponse::Ok().json(serde_json::json!({ "data": users })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch users: {}", e)
        })),
    }
}
```

`verify_org_access` (`middleware/mod.rs:70-77`) : bypass total superadmin, sinon exige `user.organization_id == organization_id` du path — **exactement** le pattern déjà utilisé par `ticket_handlers::list_organization_tickets`, `payment_handlers`, `work_report_handlers`, etc. Aucune nouvelle primitive d'autorisation.

### 2.2. Use case

Fichier : `backend/src/application/use_cases/user_use_cases.rs`, à côté de `list_all()`.

```rust
/// List all users belonging to a given organization.
pub async fn list_by_organization(
    &self,
    organization_id: Uuid,
) -> Result<Vec<UserResponse>, String> {
    let users = self.user_repo.find_by_organization(organization_id).await?;
    let user_ids: Vec<Uuid> = users.iter().map(|u| u.id).collect();
    let mut roles_map = self.role_repo.list_for_users(&user_ids).await?;

    Ok(users
        .into_iter()
        .map(|user| {
            let assignments = roles_map.remove(&user.id).unwrap_or_default();
            Self::build_response(user, assignments)
        })
        .collect())
}
```

Réutilise `Self::build_response` (déjà privé dans le même fichier, utilisé par `list_all()`) — **zéro duplication de logique de mapping**. Seule différence avec `list_all()` : `find_by_organization(id)` au lieu de `find_all()`.

### 2.3. Repository

**Aucun changement.** `UserRepository::find_by_organization(&self, org_id: Uuid) -> Result<Vec<User>, String>` existe déjà (`user_repository_impl.rs:126`), déjà dans le trait `application/ports/user_repository.rs`. Vérifié non testé directement aujourd'hui (aucun appelant en prod) — les tests 4-cat de ce chantier couvrent ce chemin pour la première fois de bout en bout.

### 2.4. Routes + OpenAPI

- `backend/src/infrastructure/web/routes.rs` : `.service(list_organization_users)` à côté de `.service(list_users)` (ligne ~564).
- `backend/src/infrastructure/openapi.rs` : ajouter `crate::infrastructure::web::handlers::user_handlers::list_organization_users` à la liste des paths (pattern ligne 97 pour `list_organization_tickets`).
- `make openapi-check` régénère `openapi.json` ; `make types-sync` régénère `frontend/src/types/api.d.ts`.

## 3. Frontend — nouveau wrapper + migration de 2 pages existantes

### 3.1. Client API

Fichier : `frontend/src/lib/api/organizations.ts` (ou nouvelle fonction dans le fichier existant le plus proche du domaine) :

```typescript
export async function listOrganizationUsers(
  organizationId: string,
): Promise<{ data: UserLike[] }> {
  return api.get(`/organizations/${encodeURIComponent(organizationId)}/users`);
}
```

Type `UserLike` déjà défini localement dans `MandatesPage.svelte` / `ContractorEvaluationsPage.svelte` — à factoriser dans un seul endroit partagé si l'occasion se présente (pas un blocker de ce chantier).

### 3.2. `MagicLinksPage.svelte` — nouveau wrapper (pattern `MandatesPage.svelte`)

`MagicLinkIssueForm.svelte` est déjà un composant pur (`users`/`scopeIdsByKind` en props, cf. brief §1) — **aucun changement dans ce composant**. Le wrapper :

- `onMount` → fetch en parallèle : `listOrganizationUsers(currentUserOrgId)` (filtré côté client sur `role === "contractor"`, pattern `ELIGIBLE_ROLES` de `MandatesPage.svelte`), `ticketsApi.listByOrganization(currentUserOrgId)` pour peupler `scopeIdsByKind.ticket` (les 3 autres scope kinds — quote/invoice/contractor_evaluation — **restent non câblés**, cf. brief §6 : seul `ticket` est exercé par les tests, pas de scope creep).
- `currentUserId` via `authStore` (pattern déjà utilisé partout : `get(authStore).user?.id`).
- `magic-links.astro` monte `<MagicLinksPage client:load />` au lieu de `<MagicLinkIssueForm client:load />` directement.

### 3.3. `MandatesPage.svelte` — migration

Ligne 49-51 : `api.get<{ data: UserLike[] }>("/users")` → `listOrganizationUsers(organizationId)`. `organizationId` = `get(authStore).user?.organization_id` (déjà lu ailleurs dans le même fichier pour `currentUserId`, pattern identique).

### 3.4. `ContractorEvaluationsPage.svelte` — migration

Ligne 69-71 : même changement mécanique que 3.3.

## 4. Data architecture

Aucun changement de schéma. `users.organization_id` (existant) est la seule donnée consultée. Pas de migration SQL.

## 5. Tests architecture

### 5.1. Backend — 4-cat sur `list_organization_users`

Pattern `cargo test --lib` (unit sur use-case avec mock repo) + `cargo test --test bdd` (intégration handler→use-case→repo réel via testcontainers) :

- `@happy` — syndic org A liste org A → 200 + liste correcte.
- `@edge` — org sans aucun user → 200 + `[]`.
- `@security` — syndic org A liste org B → 403 ; superadmin liste n'importe quelle org → 200.
- `@negative` — organization_id malformé (non-UUID) → 400 (géré par l'extracteur `web::Path<Uuid>` d'Actix, pas de code applicatif à écrire).

### 5.2. Frontend

- Pas de nouveau test Vitest nécessaire pour les composants purs (déjà couverts).
- E2E : `magic-link-issue.spec.ts`, `mandate-issue.spec.ts`, `contractor-eval.spec.ts` réexécutés — la branche de création doit désormais s'exécuter réellement (plus de skip silencieux faute de contractor sélectionnable), 3 runs sans flake chacun (cf. PRD AC-2.5).

## 6. Risques techniques + mitigations

| Risque | Probabilité | Impact | Mitigation |
|---|---|---|---|
| `find_by_organization` (jamais exercé en prod aujourd'hui) révèle un bug latent (ex. tri, pagination implicite) | Faible | Faible | 4-cat neufs couvrent le chemin complet avant merge. |
| Régression sur `list_all()` par erreur de refactor du `build_response` partagé | Faible | Moyen | `build_response` n'est pas modifié, seulement appelé depuis un 2e endroit — tests existants de `list_all()` non touchés servent de garde-fou. |
| `MagicLinksPage.svelte` mal câblé casse le flow @happy déjà partiellement testé | Faible | Moyen | Réutilise le pattern `MandatesPage.svelte` à l'identique, pas d'invention. |

## 7. Signature

```
Mary (Brief)         : SIGNED v1.0 par @gilmry 2026-08-07
John (PRD)            : DRAFT — en attente signature @gilmry
Winston (Architecture) : DRAFT — en attente signature @gilmry
```

→ Une fois signé, Stories débloquées (`stories.md`).
