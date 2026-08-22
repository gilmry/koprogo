# Agent activity — 2026-08-06 — #617 Phase C, C8 stabilisé (contractor-eval.spec.ts) — dernier de la liste C1-C8

**Persona :** diagnostic + fix (Tier 2 — code test uniquement, aucun code produit touché). Fix appliqué et vérifié 3x sans flake — avec une réserve honnête (voir ci-dessous).

**Contexte :** suite de C7. Sous-tâche **C8** (`contractor-eval.spec.ts`, Story B8) — dernière du lot #617 Phase C.

---

## Root cause #1 (test) — admin au lieu de 2 vrais syndics

Même classe de bug que C3/C7 : le test utilisait admin pour incarner à la fois « Syndic A » et « Syndic B », alors que `/syndic/contractor-evaluations` est gated côté FE au seul rôle SYNDIC (`guards.ts`). Fix : deux vrais syndics seedés (`syndicA`, `syndicB`), conforme à l'intention narrative du docstring et à la règle CRITICAL.md #9.

## Root cause #2 (test) — même bug de casse status que C7

`spec.status === "Approved"` (le check qui gate la branche de création d'évaluation) comparait contre le mauvais format — le backend renvoie `"approved"` en snake_case (cf. C7). Corrigé.

## Ce qui reste **non résolu** (honnêteté du log) — branche @happy vacuously skip

Le test avait été écrit avec une tolérance explicite pour le cas où `/users` ne renvoie pas les contractors (« si le contractor n'apparaît pas dans le select... on skip le scenario @happy » — l'auteur savait déjà). Confirmé en direct dans les logs backend pendant le run :

```
GET /api/v1/users → 403   (syndic, page contractor-evaluations)
```

Donc **la branche de création d'évaluation ne s'exécute jamais actuellement** — le test passe, mais uniquement grâce à sa propre tolérance intégrée, pas parce que le flow @happy complet a été exercé. Ça reste bloqué par le même trou que C2/C3 (brief `docs/maury/syndic-org-users-endpoint/brief.md`, PR #691). Une fois cet endpoint livré, il faudra revérifier que la branche `if (newBtnEnabled && spec.status === "approved")` s'exécute réellement (pas juste que le test reste vert).

## Découverte annexe — `GET /users/{id}` inexistant

Repéré dans les logs backend pendant le run :
```
GET /api/v1/users/{id} → 404
```
`ContractorReputation.svelte` attend un `contractorName` résolu par le parent via `/users/{id}` (cf. commentaire du composant) — **cet endpoint n'existe pas du tout** côté backend (aucun handler `#[get("/users/{id}")]`). La page reputation affiche donc probablement l'UUID brut ou un placeholder à la place du nom du contractor. Cosmétique (l'assertion du test ne vérifie que la visibilité, pas le contenu), non bloquant, mais même classe de trou que le brief #691 (accès en lecture aux users hors superadmin) — **pas fixé ici**, à ajouter au brief ou traiter séparément si @gilmry le juge utile.

## Vérifications

- E2E `contractor-eval.spec.ts` : 3 runs consécutifs, zéro flake.
- Pas de code produit touché — uniquement le spec de test.

## Actions prises

- `frontend/tests/e2e/refonte-ux/phase-b-fe/contractor-eval.spec.ts` — 2 vrais syndics + fix casse status.
- `frontend/playwright.config.ts` — `contractor-eval.spec.ts` retiré du `testIgnore` du projet `chromium`.

## Bilan #617 Phase C — C1 à C8, tous investigués

| # | Spec | Statut | Nature |
|---|---|---|---|
| C1 | role-assignment.spec.ts | ✅ (PR #690) | bug backend `valid_until` + fix test |
| C2 | magic-link-issue.spec.ts | ⏸️ bloqué | endpoint `/users` manquant — brief PR #691 |
| C3 | mandate-issue.spec.ts | ⏸️ bloqué | même cause que C2 |
| C4 | role-delegation.spec.ts | ⏸️ bloqué* | *bug email casse fixé (indépendant), reste bloqué par C2 |
| C5 | ticket-complaint.spec.ts | ✅ | bug de test (ordre capture URL) |
| C6 | syndic-response-sla.spec.ts | ✅ | gratuit (fix C4) |
| C7 | technical-spec-flow.spec.ts | ✅ | **bug produit réel** (status casing) |
| C8 | contractor-eval.spec.ts | ✅ (partiel) | bug de test + branche @happy vacuously skip (bloquée par C2) |

**Prochaine étape logique** : signature du brief `docs/maury/syndic-org-users-endpoint/brief.md` par @gilmry, qui débloquerait C2/C3/C4 complètement et compléterait réellement C8.
