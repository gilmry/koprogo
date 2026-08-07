# Agent activity — 2026-08-06 — #617 Phase C, C4 investigué : 1 vrai fix + même blocage que C2/C3

**Persona :** diagnostic + fix root cause (Tier 2 — code non-prod). Un vrai bug corrigé et vérifié, mais C4 lui-même reste rouge (bloqué par le même trou que C2/C3).

**Contexte :** suite de C3. Sous-tâche **C4** (`role-delegation.spec.ts`, Story B4).

---

## Root cause #1 (fixé) — login sensible à la casse email

`AuthUseCases::login` (`auth_use_cases.rs:38-40`) passait `request.email` brut à `find_by_email`, alors que `User::new` normalise TOUJOURS l'email en lowercase au stockage (`user.rs:208`). Résultat : un login avec une casse différente de celle utilisée à l'inscription échoue en `401 Invalid credentials`, même avec le bon mot de passe — un **vrai bug utilisateur**, pas spécifique aux tests (n'importe qui tape son email avec une majuscule au login échoue silencieusement).

Trouvé car le test C4 construit ses emails avec un prénom capitalisé (`syndic-Sophie-<ts>@example.com`) puis les réutilise tels quels pour le login UI, alors que le backend les avait stockés en lowercase. Reproduit à l'identique en pur `curl` (register puis login, casse mixte → 401), hors de tout contexte Playwright — confirme que ce n'est pas un artefact de harnais.

**Fix** : normalisation `.trim().to_lowercase()` ajoutée à `login()` (recherche) ET `register()` (check doublon — même bug latent : un doublon de casse échappait au check applicatif et aurait remonté une erreur DB brute au lieu d'un « Email already exists » propre, la contrainte UNIQUE `users.email` restant sensible à la casse).

**Vérifié** : `cargo check --lib` propre. Repro curl avant/après (401 → 200) sur `Sophie-Case-<ts>@example.com`.

## Root cause #2 (PAS fixé — même blocage que C2/C3)

Une fois le login corrigé, le test `@happy` progresse jusqu'à `role-delegate-target-input` (select destinataire), qui reste **sans options** — `RoleDelegationsPage.svelte:65` appelle le même `GET /users?per_page=1000` superadmin-only déjà identifié dans le brief `docs/maury/syndic-org-users-endpoint/brief.md` (PR #691). **4e page confirmée avec le même trou** (après RoleAssignmentForm — légitime, MandatesPage, ContractorEvaluationsPage).

Le test `@security` échoue différemment (banner `role-delegate-non-transitive-banner` absent) — pas creusé en détail, probablement une conséquence en cascade du même blocage ou un problème distinct sur le endpoint `/role-delegations` lui-même ; à revoir une fois le brief signé.

## Actions prises

- **Fixé et vérifiable indépendamment de #617** : `backend/src/application/use_cases/auth_use_cases.rs` (login + register, normalisation email).
- Pas de fix sur `role-delegation.spec.ts` ni sur `playwright.config.ts` — C4 reste rouge, bloqué par le brief.

## Ce qui reste

Le fix de casse email mérite sa propre PR (bug réel, indépendant de #617, impact large). C4 lui-même attend le même brief que C2/C3.
