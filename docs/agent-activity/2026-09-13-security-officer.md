---
date: "2026-09-13"
persona: security-officer
session: dependabot-432-closure-triage
tier: 2 # diagnostic + doc only; no merge, no branch deletion, no issue close
branch: story/432
status: RECOMMENDATION — awaiting human Tier-1 actions (close #432, delete 2 zombie branches, confirm live dashboard)
---

# Agent activity — 2026-09-13 — security-officer — Clôture proposée #432 (Dependabot `main`)

## Contexte

Issue #432 (ouverte 2026-04-30) signale "14 vulnérabilités Dependabot (5 high/3
moderate/6 low)" observées lors d'un `git push` vers `feature/dev`, et demande :
classer les 14, confronter aux 28 branches `dependabot/*` zombies (#425), décider
une règle d'auto-merge patch/mineur vs manuel majeur.

Ce chantier a déjà été largement instruit avant cette session :

- **WP-B2** (`docs/agent-activity/2026-05-17-wbs-b2.md`, branche
  `story/432-dependabot-security`) : `cargo audit` clean modulo les résidus déjà
  documentés dans `backend/.cargo/audit.toml` ; côté npm, le vrai #432 était
  **1 HIGH `devalue` <5.8.1 (DoS sparse-array) + 4 MEDIUM `svelte` <5.55.7
  (XSS ×3, ReDoS)** — le "5H/3M/6L" du ticket initial était déjà signalé comme
  stale à ce moment. Corrigé par `npm update svelte devalue` → 0 vulnérabilité
  en prod (`npm audit --omit=dev`), build vert. 1 résidu accepté :
  `@babel/plugin-transform-modules-systemjs` 7.12.0–7.29.0 (devDependency,
  build-time, jamais servi au navigateur).
- **Triage stale du 2026-07-26** (`2026-07-26-triage-stale-restantes.md`,
  catégorie D1) : doute soulevé — "`main` très en retard sur `feature/dev`,
  vérifier si ces vulns sont déjà résolues côté `feature/dev` avant de
  retraiter côté `main`".

## Limitation d'environnement (à signaler, pas à contourner)

Dans cette session, `gh auth status`, `git ls-remote --heads origin` et
`docker compose version` ont tous été bloqués ("this command requires
approval") — aucun accès réseau GitHub API ni Docker disponible. Impossible
donc de :
- Lister les alertes Dependabot live (`gh api .../dependabot/alerts`).
- Ré-exécuter `cargo audit` / `npm audit` dans les conteneurs (cf. CRITICAL.md
  §12) pour confirmer l'état "aujourd'hui".

**Ce qui suit est donc une analyse statique** (git show / git diff sur les
refs déjà présentes localement, sans nouveau fetch), pas une ré-exécution live.
Elle est cohérente et vérifiable, mais un humain avec accès au dashboard
Dependabot doit confirmer le compte final avant fermeture définitive.

## Constat

### 1. Le fix npm du 2026-05-17 est bien sur `main`, pas seulement `feature/dev`

`git show origin/main:frontend/package-lock.json` :
- `svelte` résolu **5.57.0** (patch visé par WP-B2 : ≥5.55.7 — dépassé)
- `devalue` résolu **5.8.1** (patch visé : ≥5.8.1 — atteint)
- `@babel/plugin-transform-modules-systemjs` résolu **7.29.8** (plage
  vulnérable acceptée par WP-B2 : 7.12.0–7.29.0 — **dépassée**, résolu par
  dérive normale des dépendances, sans action dédiée)

→ Le doute du 07-26 est levé : `main` a bien rattrapé `feature/dev` sur ce
point. `npm audit --omit=dev` devrait revenir à 0 (non ré-exécuté, cf.
limitation ci-dessus).

### 2. `backend/.cargo/audit.toml` — identique sur `main` et cette branche

`git diff origin/main -- backend/.cargo/audit.toml` : vide. Les 12 résidus
Rust documentés (RUSTSEC-2023-0071, 2025-0111, 2025-0134, 2026-0049/0098/0099/
0104/0258, 2026-0002, 2026-0112/0113/0145, 2026-0187, 2026-0235) sont donc déjà
sur `main`, avec justification écrite. **Manque constaté et corrigé cette
session** : `SECURITY.md` ne couvrait que 3 de ces IDs et n'avait aucune date
de revue distincte de la date de publication de l'avis — voir section
"Actions" plus bas.

### 3. Confrontation avec les 28 branches `dependabot/*` zombies (#425)

Cette copie locale ne référence que **2** branches `dependabot/*` restantes
(`dependabot/github_actions/feature/dev/actions/cache-6`,
`dependabot/npm_and_yarn/frontend/npm_and_yarn-ce53002fcb`) — la majorité des
28 signalées par l'audit #425 a donc déjà été fusionnée ou supprimée avant
cette session (`scripts/cleanup-dependabot-branches.sh` existe déjà dans le
repo pour ce faire). Vérification des 2 restantes par `git diff` :

| Branche | Propose | État réel sur `origin/main` | Verdict |
|---|---|---|---|
| `.../actions/cache-6` | `actions/cache@v5→v6` sur `ci.yml`/`characterization-gate.yml` | déjà `actions/cache@v6` partout (confirmé par grep direct sur `origin/main`) | **Zombie pure** — le diff ne montre que l'écart avec un vieux merge-base, rien à apporter |
| `.../npm_and_yarn-ce53002fcb` | `astro ^6.3.1→^6.4.6`, `svelte ^5.55.9` | déjà `astro ^7.3.1`, `svelte ^5.57.0` | **Zombie pure** — dépassée par des mises à jour ultérieures |

→ **Aucune PR "prête à fusionner" n'attend en réalité.** Refaire à la main ce
qui attend en revue (la crainte énoncée par la story) ne s'applique pas ici :
il n'y a rien en attente, seulement des doublons obsolètes à nettoyer.

### 4. Règle de fusion automatique

Déjà écrite et datée : `.github/workflows/dependabot-auto-merge.yml`, décision
humaine du **2026-05-11** (gilmry) — auto-merge **tous** les bumps Dependabot
(patch/mineur/majeur/sécurité) vers `feature/dev` dès CI verte, avec gate
explicite sur la conclusion des checks (post-mortem #659 documenté dans le
fichier lui-même). C'est plus large que "patch+mineur auto, majeur manuel"
proposé par la story, mais c'est une **décision humaine déjà tracée** — je ne
la remplace pas sans mandat explicite.

## Actions (Tier 2 — fichiers modifiés)

- **`SECURITY.md`** : synchronisé avec les 12 résidus de `backend/.cargo/audit.toml`
  (5 manquaient entièrement : RUSTSEC-2025-0134, 2026-0098/0099/0104/0258,
  2026-0002, 2026-0112/0113/0145, 2026-0187, 2026-0235 — regroupés par famille
  causale, même rationale que `audit.toml`). Ajout d'un champ `**Last-Reviewed**`
  daté 2026-09-13 sur chaque entrée. Ajout de deux entrées "RESOLVED" (npm
  `devalue`/`svelte`, npm babel systemjs) pour que la clôture soit tracée et non
  silencieuse. Ajout d'une cadence de revue explicite dans "Monitoring".
  "Last updated" mis à jour.
- **`backend/.cargo/audit.toml`** : **non modifié** — la tentative d'ajouter un
  commentaire `# Reviewed: 2026-09-13` par entrée a été bloquée par le garde-fou
  de cette session ("fichier sensible"), probablement à cause du chemin
  `.cargo/`. Pas de contournement tenté. `SECURITY.md` porte donc seule la date
  de revue pour l'instant ; un futur agent avec les permissions adéquates
  pourra reporter la même date en commentaire dans `audit.toml` pour que les
  deux fichiers restent alignés.
- **Ce fichier** : journal Tier-2.

Aucune dépendance, aucun workflow CI, aucune branche n'a été touché(e).

## Décision proposée (nécessite un humain — Tier 1)

1. **Clore #432** en référençant ce log + `2026-05-17-wbs-b2.md` :
   - Les alertes réelles (npm `devalue`/`svelte`) sont corrigées sur `main`.
   - Le résidu babel s'est résolu tout seul (dérive de version).
   - Les résidus Rust sont documentés avec justification + date de revue dans
     `SECURITY.md` (`audit.toml` inchangé, cf. limitation ci-dessus).
   - La règle d'auto-merge existe et est datée (2026-05-11).
   - **Avant de clore** : un humain avec accès au dashboard
     (https://github.com/gilmry/koprogo/security/dependabot) devrait confirmer
     que le compte live correspond à cette analyse statique — je n'ai pas pu
     le vérifier moi-même dans cette session (cf. limitation d'environnement).
2. **Supprimer les 2 branches zombies restantes** — suppression de branche
   distante = Tier 1, non exécutée ici :
   - `dependabot/github_actions/feature/dev/actions/cache-6`
   - `dependabot/npm_and_yarn/frontend/npm_and_yarn-ce53002fcb`
3. **Hors mandat de cette session** : si le dashboard live montre des alertes
   non couvertes par ce qui précède (ex. apparues après le dernier fetch local
   de ce dépôt), rouvrir le triage sur ces alertes spécifiques plutôt que de
   fermer #432 aveuglément.

## Pourquoi pas de nouveaux tests BDD 4-catégories

Cette story ne produit ni entité de domaine, ni use case, ni handler, ni
composant Svelte — c'est un audit de dépendances/config CI. Le précédent
chantier (WP-B2) a validé ses changements via les outils existants
(`cargo audit`, `cargo check --all-targets`, `npm audit`, `npm run build`), pas
via de nouveaux scénarios Cucumber : il n'y a pas de logique de domaine à
tester ici. Cette session n'a touché aucun code exécutable (seulement de la
documentation), donc aucune suite de tests n'a été exécutée ni n'était
nécessaire.
