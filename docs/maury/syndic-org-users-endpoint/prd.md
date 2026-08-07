---
feature: syndic-org-users-endpoint
phase: B (Business architecture TOGAF)
status: SIGNED v1.0 par @gilmry 2026-08-07
date: 2026-08-07
authors: [Claude Sonnet 5 (drafting)]
depends_on: brief.md (SIGNED v1.0 2026-08-07)
related_issues: [617, 691, 694]
---

# PRD — Endpoint de listing des users pour le syndic (org-scopé)

> Phase B TOGAF — Functional Requirements (FR), goals métier, user journeys, AC business, NFR. Architecture technique dans `architecture.md`, stories briefables dans `stories.md`.

## 1. FR-1 — `GET /organizations/{organization_id}/users`

### Goal métier

Un syndic doit pouvoir lister les users de sa propre organisation pour peupler un sélecteur "destinataire" (contractor pour un lien magique, mandataire pour un mandat, contractor pour une évaluation). Aujourd'hui `GET /users` est strictement superadmin-only — un syndic authentifié se prend un 403.

### User journey

1. **Syndic** (org A) ouvre `/syndic/magic-links` (ou `/syndic/mandates`, ou `/syndic/contractor-evaluations`).
2. La page appelle `GET /organizations/{org_A_id}/users`.
3. Backend vérifie `verify_org_access(org_A_id)` — syndic de l'org A : OK. Syndic d'une autre org tentant `org_A_id` : 403.
4. Réponse `{ data: UserResponse[] }` — mêmes champs que `GET /users` aujourd'hui (id, email, first_name, last_name, role, organization_id, is_active, roles, active_role).
5. Le FE filtre côté client par rôle (pattern déjà utilisé : `RoleAssignmentForm.svelte` `ASSIGNABLE_ROLES`, `MandatesPage.svelte` `ELIGIBLE_ROLES`).

### AC business

- AC-1.1 — Un syndic authentifié listant `org_A_id` (= son `organization_id` JWT) reçoit `200` + la liste des users de cette org.
- AC-1.2 — Un syndic de l'org A appelant avec `org_B_id` (org différente) reçoit `403`.
- AC-1.3 — Le superadmin peut lister n'importe quelle organisation (comportement `verify_org_access` déjà standard : bypass total pour superadmin).
- AC-1.4 — Une organisation sans aucun user retourne `200` + `{ data: [] }` (pas d'erreur).
- AC-1.5 — Un `organization_id` qui n'existe pas en base : comportement aligné sur `list_organization_tickets` (pas de vérification d'existence — retourne liste vide, cohérent avec le reste du pattern org-scopé du projet ; à confirmer en Architecture si un comportement différent est préférable).
- AC-1.6 — Aucune régression sur `GET /users` (superadmin) — route et comportement existants inchangés.
- AC-1.7 — `RoleAssignmentForm.svelte` (page `/admin/role-assignments`, superadmin-only) continue d'utiliser `GET /users` — pas de migration, c'est un appel légitime.

### NFR

- Performance : réutilise `user_repo.find_by_organization()` déjà existant, pas de nouvelle requête SQL à concevoir.
- Sécurité : aucun champ supplémentaire exposé au-delà de ce que `GET /users` renvoie déjà aujourd'hui (pas de nouvelle fuite de PII, juste un filtre d'accès différent).

---

## 2. FR-2 — Migration FE des 3 pages syndic cassées

### Goal métier

Les 3 formulaires identifiés en investiguant #617 (C2, C3, C8) doivent effectivement charger leurs listes de destinataires via le nouvel endpoint, pas seulement "pouvoir" le faire.

### User journey

1. **Syndic** ouvre `/syndic/magic-links` → sélecteur "Destinataire" peuplé avec les contractors de son org (au lieu d'un select vide, cf. bug #617 C2 où le composant n'était même pas câblé).
2. **Syndic** ouvre `/syndic/mandates` → sélecteur "Mandataire" peuplé (au lieu d'un 403 masqué en toast, cf. #617 C3).
3. **Syndic** ouvre `/syndic/contractor-evaluations` → sélecteur "Contractor" peuplé (au lieu d'un 403 masqué, cf. #617 C8 — la branche de création du test E2E reste actuellement *vacuously skip*).

### AC business

- AC-2.1 — Nouveau wrapper `MagicLinksPage.svelte` (pattern `MandatesPage.svelte`) créé, orchestre le fetch + passe `users`/`scopeIdsByKind` en props à `MagicLinkIssueForm.svelte` (composant pur existant, jamais câblé — cf. brief §1).
- AC-2.2 — `magic-links.astro` monte `MagicLinksPage` au lieu du form nu directement.
- AC-2.3 — `MandatesPage.svelte:50` migré de `GET /users` vers `GET /organizations/{id}/users`.
- AC-2.4 — `ContractorEvaluationsPage.svelte:71` migré de `GET /users` vers `GET /organizations/{id}/users`.
- AC-2.5 — E2E `magic-link-issue.spec.ts` (C2), `mandate-issue.spec.ts` (C3), `contractor-eval.spec.ts` (C8) re-testés avec un vrai syndic — la branche de création s'exécute réellement (plus de skip silencieux), 3 runs sans flake chacun.
- AC-2.6 — `RoleDelegationsPage.svelte` (C4 — même trou identifié mais pas listé dans le brief §1, découvert plus tard) — **hors scope explicite**, cf. §4.

### NFR

- Pas de régression sur les 4-cat Vitest existants (`MagicLinkIssueForm.test.ts`, `MandateIssueForm.test.ts`, `ContractorEvaluationForm.test.ts`) — composants purs inchangés, seul le wrapper/fetch change.

---

## 3. Matrice de traçabilité

| FR | CB (brief) | INV (brief) | SCB (brief) | Files BE | Files FE |
|---|---|---|---|---|---|
| FR-1 | CB-1, CB-2 | INV-1 à INV-6 | SCB-1, SCB-4 | `user_handlers.rs` (nouveau handler), `user_use_cases.rs` (nouvelle méthode), `routes.rs`, `openapi.rs` | — |
| FR-2 | CB-3 | — | SCB-2, SCB-3, SCB-5 | — | `magic-links.astro`, `MagicLinksPage.svelte` (nouveau), `MandatesPage.svelte`, `ContractorEvaluationsPage.svelte` |

## 4. Hors-scope explicite (hérité du brief + précisions)

- Filtrage par rôle côté backend, pagination — cf. brief §6.
- **`RoleDelegationsPage.svelte` (C4)** — appelle aussi `GET /users` sans org-scope, découvert après signature du brief. Même fix mécanique que FR-2, mais non inclus dans ce PRD pour rester fidèle au scope signé. À couvrir dans une story de suivi si @gilmry le confirme (impact : C4 resterait bloqué par le trou `/users` même après ce PRD, indépendamment de son propre fix de casse email déjà livré).
- **Scoping user↔ACP** — cf. [#694](https://github.com/gilmry/koprogo/issues/694), chantier de fond distinct.
- `GET /users/{id}` (lookup single user, découvert cassé en C8 — `ContractorReputation.svelte` ne peut jamais résoudre le nom d'un contractor) — hors scope, noté pour référence future.

## 5. Signature

```
Mary (Brief) : SIGNED v1.0 par @gilmry 2026-08-07
John (PRD)   : SIGNED v1.0 par @gilmry 2026-08-07
```

→ Une fois signé, Architecture débloquée (`architecture.md`).
