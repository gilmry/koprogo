# Agent activity — 2026-05-18 — Audit GitHub Security (Tier-2)

Persona : security triage. Lecture seule via `gh api` (dependabot/code-scanning/secret-scanning). Tier-2 loggé ici. Aucune mutation (dismiss CodeQL / `gh issue create` / modif seed = Tier-1, non fait).

## Constat

| Source | Open | Détail |
|---|---|---|
| Secret scanning | 0 | ✅ rien |
| Code scanning (CodeQL) | 2 **CRITICAL** | #61 `seed.rs:3322` `"marc123"`, #62 `seed.rs:3353` `"sophie123"` — `rust/hard-coded-cryptographic-value` |
| Dependabot | 6 | #95 high devalue (DoS sparse array) ; #96-99 medium svelte (XSS/ReDoS ×4) ; #9 low rust/lru |

## Analyse

- **CodeQL #61/#62** : littéraux mots de passe **démo** passés à `create_demo_user(... password ...)` qui fait `hash(password, DEFAULT_COST)` (bcrypt) avant insert — **non stockés en clair**. Org de démonstration ; creds volontairement connus (renvoyés dans `ScenarioUserResult` pour login démo). Pattern fixture démo. Risque réel faible (v0.1.0 non-prod, cf. project_koprogo-current-state). Techniquement = credential hardcodé (CRITICAL.md #1) → décision humaine requise.
- **Dependabot #95-99 (5 npm svelte/devalue)** : exactement ce que **#538 / WP-B2** corrige (svelte 5.55.7 + devalue 5.8.1). #538 non mergé sur la branche par défaut (et le merge local des 5 PR sur feature/dev non poussé) → GitHub voit encore les vieilles versions. **Se ferment au merge de #538/feature/dev.** Aucune action neuve.
- **Dependabot #9 (rust/lru, RUSTSEC-2026-0002)** : **déjà accepté + documenté** dans `backend/.cargo/audit.toml` (transitif aws-sdk-s3, pas d'usage direct, soundness théorique, monitoré). Aucune action.

**Conclusion : aucune exposition réelle non triée.** Unique item ouvert nécessitant décision = les 2 CRITICAL CodeQL démo-seed.

## Proposition (draft issue — Tier-1 pour `gh issue create`)

**Titre** : `security(seed): mots de passe démo hardcodés (CodeQL #61/#62 critical) — randomiser ou dismiss annoté`
**Labels** : `security`, `priority:medium`
**Constat** : `seed.rs:3322/3353` (et probablement les autres `create_demo_user`) passent des mots de passe littéraux ; bcrypt-hashés mais credential en clair dans le source → 2 CodeQL critical.
**Options** :
1. **Randomiser** : mot de passe démo généré (env `DEMO_SEED_PASSWORD` ou aléatoire par run), exposé via `ScenarioUserResult`/log de seed ; supprime les littéraux → CodeQL clos proprement. RED-first 4-cat sur `create_demo_user`.
2. **Dismiss annoté** : `gh api ... code-scanning/alerts/{n} -f state=dismissed -f dismissed_reason=used_in_tests` + commentaire « known demo seed, bcrypt-hashed, demo org only ». Rapide, mais garde le pattern.
**Reco** : Option 1 (élimine la cause, conforme CRITICAL.md #1) si la story le justifie ; sinon Option 2 documentée le temps de la bêta.
**Hors-scope** : Dependabot npm (résolu par #538 au merge) ; rust/lru (accepté audit.toml).

## Décision & action (Tier-1, autorisée par l'humain : « option 2 »)

CodeQL #61 (`seed.rs:3322`) et #62 (`seed.rs:3353`) **dismissed** via `gh api PATCH` :
`state=dismissed`, `dismissed_reason="won't fix"`, commentaire d'audit (demo-seed
fixture, bcrypt-hashed, demo org, v0.1.0 non-prod, revisit before prod via
`DEMO_SEED_PASSWORD`, ref ce log). **Code-scanning open = 0.** Reste à
reconsidérer (Option 1, randomisation) avant tout passage en prod — non bloquant bêta.
