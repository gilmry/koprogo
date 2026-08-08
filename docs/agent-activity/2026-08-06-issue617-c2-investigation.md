# Agent activity — 2026-08-06 — #617 Phase C, C2 investigué (bloqué, pas de fix)

**Persona :** diagnostic (Tier 2). Aucun code produit modifié — le seul changement (test) ne suffit pas à faire passer C2 et n'est **pas commité**.

**Contexte :** suite de C1 (cf. `2026-08-06-issue617-c1-role-assignment.md`), sous-tâche **C2** (`magic-link-issue.spec.ts`, Story B2).

---

## Root cause #1 (test, fixé) — auth de test stale

`injectSyndicAuth()` posait un token dans une clé localStorage `koprogo_auth` que l'architecture courante ne lit plus (WP-FE1 : JWT hors localStorage). En prime, le cookie de session hérité du `register` syndic est de toute façon écrasé par le `register` contractor qui suit dans `seedSyndicWithTicketAndContractor`. Remplacé par un vrai login UI (`humanLogin`, pattern C1) — corrige la redirection intempestive vers `/login`, mais **ne suffit pas** : les 2 tests concernés échouent encore.

## Root cause #2 (produit, PAS fixé — nécessite décision humaine)

`MagicLinkIssueForm.svelte` est un composant pur : `users`/`scopeIdsByKind` sont des **props**, jamais fetchées en interne (« injectable en test. Si non fournie, autocomplete vide »). Or `pages/syndic/magic-links.astro` monte `<MagicLinkIssueForm client:load />` **sans aucune prop** — le formulaire est vide en usage réel, pour n'importe quel syndic, pas seulement dans le test. Contrairement à Story B3 (Mandates), aucun wrapper `MagicLinksPage.svelte` n'a été écrit pour orchestrer le fetch + les props.

En creusant la source de données manquante : le seul endpoint listant les users est `GET /users`, **strictement superadmin-only** (`user_handlers.rs:63`). Il n'existe **aucun** endpoint org-scopé pour qu'un syndic liste les membres de son organisation (contrairement aux tickets/paiements/etc. qui ont tous un `/organizations/{id}/...` gaté par `verify_org_access`). Le repository a pourtant déjà `find_by_organization()` (`user_repository_impl.rs:126`) — jamais exposé en REST.

**Portée du vrai fix** = nouvel endpoint public (`GET /organizations/{id}/users` ou équivalent) + décision d'autorisation (qui peut lister qui) + schema OpenAPI + 4-cat tests + wrapper `MagicLinksPage.svelte`. C'est une **décision produit/sécurité** (exposition de PII cross-user), pas un rewiring mécanique — hors du mandat Tier 2 (CRITICAL.md #5 : Brief → PRD → Architecture → Story signés avant code ; #11 : dans le doute, Tier 1).

## Découverte annexe — le même trou touche potentiellement C3 et C8

```
grep -rn '"/users"' frontend/src --include=*.svelte --include=*.ts
```
révèle 2 autres pages syndic-facing avec le même appel cassé :
- `MandatesPage.svelte:50` (`pages/syndic/mandates.astro`, Story B3) → probablement la cause de C3 (`mandate-issue.spec.ts`).
- `ContractorEvaluationsPage.svelte:71` (`pages/syndic/contractor-evaluations.astro`, Story B8) → probablement la cause de C8 (`contractor-eval.spec.ts`).

`RoleAssignmentForm.svelte` (C1, `/admin/role-assignments`) appelle aussi `/users` mais c'est légitime : cette page est superadmin-only.

## Note annexe (perf, non bloquant)

`list_role_assignments_admin` (`role_assignment_handlers.rs:277-298`) itère tous les users puis appelle `list_assignments_for_user` par user — O(n) round-trips DB par requête. Vu en passant, pas un blocage go-live, tracé ici pour mémoire.

## Actions prises

- `frontend/tests/e2e/refonte-ux/phase-b-fe/magic-link-issue.spec.ts` : fix auth de test (non commité, C2 reste rouge malgré ça — voir root cause #2).
- `frontend/playwright.config.ts` : `magic-link-issue.spec.ts` remis dans `testIgnore` (C2 non stabilisé, pas de régression du gate `chromium`).

## Ce qui reste

Décision @gilmry nécessaire : ouvrir une story Maury (ou un commentaire #617) pour l'endpoint `GET /organizations/{id}/users` avant de pouvoir clôturer C2 (et probablement C3/C8 par la même occasion).
