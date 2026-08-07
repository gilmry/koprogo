# Agent activity — 2026-08-06 — #617 Phase C, C7 stabilisé : bug produit significatif

**Persona :** diagnostic + fix root cause (Tier 2 — code non-prod). Fix appliqué et vérifié (Vitest + E2E 3x sans flake).

**Contexte :** suite de C6. Sous-tâche **C7** (`technical-spec-flow.spec.ts`, Story B7).

---

## Root cause #1 (test) — admin au lieu d'un vrai syndic

Même classe de bug que C3 : le test utilisait admin comme « syndic-émulé » pour accéder à `/syndic/technical-spec`, route gated côté FE au seul rôle SYNDIC (`guards.ts`). Contrairement à C3, ce test n'a en réalité que 2 acteurs actifs dans le code (le docstring en annonce 3 dont un AMO jamais exercé) — remplacer admin par un vrai syndic seedé (conforme règle CRITICAL.md #9) suffit, pas besoin de l'endpoint `/users` bloqué.

## Root cause #2 (PRODUIT, réel, impact large) — status snake_case comparé en PascalCase

`TechnicalSpecDetail.svelte` comparait `spec.status` contre des littéraux **PascalCase** (`"Draft"`, `"PendingSignatures"`, `"Approved"`, `"Superseded"`) dans 4 `$derived` + 2 fonctions de badge (classes + label). Le backend sérialise `TechnicalSpecStatus` via son `impl Display` (`technical_spec.rs:141-149`), qui renvoie du **snake_case** (`"draft"`, `"pending_signatures"`, `"approved"`, `"superseded"`) — confirmé par `TechnicalSpecDto::from` (`status: s.status.to_string()`) et vérifié en direct (attribut `data-status` réellement rendu).

**Conséquence en production, pour TOUS les users, TOUTES les fiches techniques** : `isDraft`/`isPendingSignatures`/`isApproved`/`isSuperseded` étaient TOUJOURS `false` → les boutons "Soumettre pour signatures", "Signer" et "Bump" ne s'affichaient **jamais**, quel que soit le statut réel. Même bug répliqué dans `TechnicalSpecVersionTimeline.svelte` (badge + grisage "Superseded" jamais appliqué).

**Fix** : les 4 dérivations + les 2 maps de badge (Detail + Timeline) corrigées en snake_case.

## Root cause #3 (produit, mineur, catché) — `listSpecs()` sans `acp_id` obligatoire

Toast d'erreur parasite visible à chaque ouverture du détail d'une fiche technique : `listSpecs()` (utilisée pour l'historique des versions, résultat non-fatal car catché) appelait `GET /technical-specs` sans aucun paramètre, alors que le backend exige `acp_id: Uuid` (non-Option, `ListTechnicalSpecsQuery`). Fix : `listSpecs(acpId?: string)` — `TechnicalSpecPage.svelte` passe maintenant `s.acp_id` (déjà disponible après `getSpec`).

**Hors scope, non touché** : `TechnicalSpecsPage.svelte` (liste globale) et `ContractorEvaluationsPage.svelte` (Story B8) appellent encore `listSpecs()` sans argument pour lister across-ACP — ce cas reste cassé (même trou), nécessiterait soit un endpoint cross-ACP soit une itération multi-appels côté FE. Signalé pour référence future (possiblement pertinent pour C8).

## Vérifications

- `npx astro check` : 0 erreur, 0 warning (inchangé vs baseline).
- Vitest `TechnicalSpecDetail.test.ts` (8 tests) + `TechnicalSpecVersionTimeline.test.ts` (6 tests) + `TechnicalSpecCreate.test.ts` (7 tests, non touché mais revérifié) : **21/21 verts**. Les fixtures PascalCase des 2 premiers fichiers (testaient un monde auto-cohérent mais faux vs le vrai contrat backend) mises à jour en snake_case.
- E2E `technical-spec-flow.spec.ts` : **3 runs consécutifs, zéro flake**.

## Actions prises

- `backend` : aucun changement (le bug était uniquement FE).
- `frontend/src/lib/components/syndic/TechnicalSpecDetail.svelte` — 4 dérivations + 2 maps badge en snake_case.
- `frontend/src/lib/components/syndic/TechnicalSpecVersionTimeline.svelte` — idem (map badge + comparaison isSuperseded).
- `frontend/src/lib/components/syndic/TechnicalSpecDetail.test.ts` + `TechnicalSpecVersionTimeline.test.ts` — fixtures snake_case.
- `frontend/src/lib/api/technical_specs.ts` — `listSpecs(acpId?: string)`.
- `frontend/src/lib/components/syndic/TechnicalSpecPage.svelte` — passe `s.acp_id` à `listSpecs()`.
- `frontend/tests/e2e/refonte-ux/phase-b-fe/technical-spec-flow.spec.ts` — acteur syndic réel (au lieu d'admin) + assertion status en snake_case.
- `frontend/playwright.config.ts` — `technical-spec-flow.spec.ts` retiré du `testIgnore` du projet `chromium`.

## Restant sur #617

C8 (`contractor-eval.spec.ts`) non investigué — probablement affecté par le même trou `/users` (C2) **et** potentiellement par le même trou `listSpecs()` cross-ACP identifié ci-dessus (Root cause #3, `ContractorEvaluationsPage.svelte` fait partie des appelants non corrigés).
