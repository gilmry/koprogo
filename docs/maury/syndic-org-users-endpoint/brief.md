---
feature: syndic-org-users-endpoint
phase: A (Vision TOGAF)
status: DRAFT — en attente signature @gilmry
date: 2026-08-06
authors: [Claude Sonnet 5 (drafting)]
related_issues: [617]
parent_maury: none — trouvé en investiguant #617 Phase C (C2/C3/C8)
---

# Brief — Endpoint de listing des users pour le syndic (org-scopé)

## 1. Vision

**Trois formulaires Phase B FE côté syndic sont cassés en production pour n'importe quel syndic réel**, parce qu'ils appellent `GET /users` pour peupler un sélecteur "destinataire", et cet endpoint est **strictement superadmin-only** (`user_handlers.rs:63`). Aucun endpoint org-scopé équivalent n'existe — contrairement à tickets, paiements, work-reports, etc. qui ont tous un `/organizations/{id}/...` gaté par `verify_org_access`. Le repository a pourtant déjà `find_by_organization()` (`user_repository_impl.rs:126`), jamais exposé en REST.

**Trouvé en investiguant** [#617](https://github.com/gilmry/koprogo/issues/617) Phase C :
- C2 `magic-link-issue.spec.ts` (Story B2) — `MagicLinkIssueForm.svelte` n'est même pas câblé du tout (composant pur, aucune prop fournie par `magic-links.astro`).
- C3 `mandate-issue.spec.ts` (Story B3) — `MandatesPage.svelte:50` appelle `GET /users`, 403 pour un vrai syndic.
- C8 `contractor-eval.spec.ts` (Story B8) — `ContractorEvaluationsPage.svelte:71` même appel, même 403.

Un seul fix d'autorisation débloque potentiellement les 3.

## 2. Personas concernés

### 2.1. Syndic (cible)

- **Rôle** : émet un lien magique (contractor), un mandat (avocat/notaire/AMO/architecte/BET/gardien), ou évalue un contractor — dans chaque cas doit choisir un destinataire parmi les users de **sa propre** organisation.
- **Frustration actuelle** : les 3 formulaires échouent silencieusement (403 masqué en toast + liste vide) — impossible d'émettre quoi que ce soit.
- **Besoin** : lister les users de son org (filtrable par rôle côté FE, comme c'est déjà fait pour `RoleAssignmentForm.svelte` avec `ASSIGNABLE_ROLES`).

### 2.2. Superadmin (existant, non affecté)

- Continue d'utiliser `GET /users` (tous users, toutes orgs) — inchangé, aucune régression.

## 3. Capacité business (CB)

| CB | Description |
|---|---|
| **CB-1** | Un syndic authentifié peut lister les users de **sa propre** organisation (celle de son JWT). Un syndic d'org A ne peut PAS lister les users d'org B. |
| **CB-2** | Le superadmin peut lister les users de **n'importe quelle** organisation via le même endpoint (cohérent avec le pattern `verify_org_access` déjà utilisé pour tickets/paiements/work-reports). |
| **CB-3** | `MagicLinksPage.svelte` (nouveau wrapper, pattern `MandatesPage.svelte`), `MandatesPage.svelte`, `ContractorEvaluationsPage.svelte` consomment ce nouvel endpoint au lieu de `GET /users`. |

## 4. Invariants techniques (INV)

| INV | Énoncé |
|---|---|
| **INV-1** | `GET /organizations/{organization_id}/users` — handler gate `user.verify_org_access(*organization_id)` (même pattern exact que `ticket_handlers.rs::list_organization_tickets`), 403 sinon. |
| **INV-2** | Use-case thin wrapper : `UserUseCases::list_by_organization(org_id) -> Result<Vec<UserResponse>, AppError>`, appelle `user_repo.find_by_organization(org_id)` (déjà existant, pas de nouvelle logique DB). |
| **INV-3** | Réponse = même shape `{ data: UserResponse[] }` que `GET /users`, pour compat FE minimale (les composants existants font déjà `.data`). |
| **INV-4** | `utoipa::path` déclaré → `openapi.json` + `api.d.ts` régénérés (gate Contract CI). |
| **INV-5** | Tests 4-cat : `@happy` syndic liste son org ; `@edge` org vide → `[]` ; `@security` syndic org A → org B → 403, superadmin → n'importe quelle org → 200 ; `@negative` org inconnue → 404 ou liste vide (à trancher en Architecture, cohérent avec le comportement de `list_organization_tickets` sur org inconnue). |
| **INV-6** | Aucun champ sensible superflu dans `UserResponse` au-delà de ce qui est déjà exposé par `GET /users` aujourd'hui (pas de nouvelle fuite de PII — même DTO, juste un filtre d'accès différent). |

## 5. Critères de succès (SCB)

| SCB | Mesure |
|---|---|
| **SCB-1** | `cargo test --lib` + `cargo test --test bdd` : 4-cat GREEN sur le nouvel endpoint. |
| **SCB-2** | `MagicLinksPage.svelte` créée (pattern `MandatesPage.svelte`), `magic-links.astro` mis à jour, `MandatesPage.svelte` et `ContractorEvaluationsPage.svelte` migrées vers le nouvel endpoint. |
| **SCB-3** | C2, C3, C8 (`docs/agent-activity/2026-08-06-issue617-c2-investigation.md`) re-testés : GREEN ou nouvelle cause distincte documentée si un des trois a un problème indépendant. |
| **SCB-4** | `make openapi-check` + `make types-sync` verts. |
| **SCB-5** | Aucune régression sur `GET /users` (superadmin) ni sur `RoleAssignmentForm.svelte` qui continue à l'utiliser légitimement (page superadmin-only). |

## 6. Hors-scope explicite

- Filtrage par rôle côté backend (`?role=contractor`) — le FE filtre déjà côté client (pattern `RoleAssignmentForm.svelte` / `MandatesPage.svelte` `ELIGIBLE_ROLES`). Ajouter un filtre serveur = optimisation future, pas nécessaire pour débloquer C2/C3/C8.
- Pagination — les orgs bêta fermée sont petites (5-10 copropriétés, cf. WBS go-live). `per_page` existant sur `GET /users` peut être ignoré/simplifié ici.
- Toute autre page/composant syndic-facing non listée en §1 (audit exhaustif de tous les appels `/users` hors scope de ce brief).

## 7. Risques et mitigations

| Risque | Probabilité | Impact | Mitigation |
|---|---|---|---|
| `find_by_organization` (repo) pas exactement testé pour ce nouveau chemin d'appel | Faible | Faible | 4-cat neufs couvrent le chemin handler→use-case→repo en entier. |
| Régression accidentelle sur pages qui utilisaient `GET /users` en superadmin | Faible | Moyen | `RoleAssignmentForm.svelte` (seul appelant légitime restant) non touché — migration ciblée aux 2 pages syndic identifiées. |
| Scope creep vers un audit complet des endpoints Phase B FE | Moyenne | Moyen | Hors-scope explicite §6 — un ticket séparé si d'autres trous du même type sont découverts. |

## 8. Signature

```
Mary (Brief) : DRAFT — en attente signature @gilmry
```

→ Une fois signé, PRD + Architecture + Story (format court, scope réduit) avant code.
