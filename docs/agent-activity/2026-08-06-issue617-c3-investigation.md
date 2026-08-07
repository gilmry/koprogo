# Agent activity — 2026-08-06 — #617 Phase C, C3 investigué (même blocage que C2)

**Persona :** diagnostic (Tier 2). Aucun code modifié, aucun commit — investigation seule.

**Contexte :** suite de C2 (cf. `2026-08-06-issue617-c2-investigation.md` + brief `docs/maury/syndic-org-users-endpoint/brief.md`, PR #691 en attente de signature). Sous-tâche **C3** (`mandate-issue.spec.ts`, Story B3).

---

## Constat

Le test `@happy` échoue dès `mandate-new-button` introuvable (`page.goto('/syndic/mandates')` puis timeout). Screenshot : redirigé vers `/admin` (dashboard superadmin), pas vers le formulaire.

**Cause immédiate** : le test se logue en tant qu'**admin** (superadmin), pas en tant que syndic — son propre commentaire l'assume : « login admin (= role syndic-like superadmin in-context) ». Or `guards.ts` restreint `/syndic/*` au seul rôle `SYNDIC` (pas de carve-out superadmin) → `RouteGuard` redirige tout superadmin visitant `/syndic/mandates` vers `/admin`.

**Pourquoi l'auteur a utilisé admin plutôt qu'un vrai syndic** : `require_syndic_or_superadmin` côté backend (`mandate_handlers.rs:112`) autorise bien les deux rôles à émettre un mandat — donc admin *semblait* un raccourci valide. Mais ce choix viole la règle CRITICAL.md #9 (« Scénarios E2E ont les bons acteurs ») — un syndic aurait dû être seedé, comme le fait `role-assignment.spec.ts` (C1) pour son test `@security`.

## Vérification empirique — même root cause que C2

Utiliser le VRAI acteur (syndic, conforme #9) ne résout rien : `MandatesPage.svelte:50` appelle le même `GET /users` superadmin-only déjà identifié dans le brief C2. Confirmé en direct :

```
curl GET /api/v1/users -H "Authorization: Bearer <token syndic>"
→ 403
```

**C3 est donc bloqué par exactement la même cause que C2** (`docs/maury/syndic-org-users-endpoint/brief.md`) — pas une cause indépendante. Le contournement "admin" du test masque le vrai trou en changeant d'acteur, mais se prend le garde-fou FE à la place.

## Ce qu'il faudra faire une fois le brief signé/implémenté

En plus du fix produit (endpoint `GET /organizations/{id}/users` + `MandatesPage.svelte` migré) :
- Réécrire `mandate-issue.spec.ts` pour utiliser un **vrai syndic** seedé (pattern `seedOrgWithUser` de C1), pas admin — conformité règle #9. Une fois le vrai syndic utilisé, `/syndic/mandates` passera nativement le `RouteGuard` (déjà correct pour SYNDIC).

## Aucune action prise

Pas de code touché, pas de commit. `playwright.config.ts` inchangé (testIgnore intact).
