---
feature: refonte-ux-multi-role-acp/phase-b-fe
phase: brief
phase_togaf: A (Vision)
agent_bmad: Mary (Analyste TOGAF)
authors: [Claude Opus 4.7 (drafting), @gilmry (signature pending)]
date: 2026-06-09
version: 0.2
status: Draft 0.2 — Maury-grade rewrite (v0.1 jugé insuffisant 2026-06-09)
parent_brief: docs/maury/refonte-ux-multi-role-acp/brief.md (v1.0, signed 2026-05-20 by @gilmry)
parent_phase: Phase A (Backend BMAD slice 3 — Stories 3.1→3.9 mergées 2026-06-09, commits 9598298→cf41ef4, 1556+ tests lib, zero régression)
trigger_event: "Documentation Vivante e2e a cassé silencieusement post-slice 3 BE car les UIs syndic/owner pour les nouvelles features n'existaient pas. Smoke tests verts mais signal video cassé. Workflow CI tweaké avec continue-on-error (a698f6d) en attendant Phase B."
issues_source: [#553, #554, #555, Documentation Vivante drift observed 2026-06-09 CI run 27210791347]
memories_applied:
  - maury-fullstack-first
  - data-testid-systematic
  - a11y-wcag-aa-baseline
  - fe-refactor-test-driven
  - multirole-narrative-scenarios
  - validate-before-compute
  - world-model-seed
  - no-f64-in-money
  - docker-parallelism-bottleneck
  - subagent-worktree-git-salvage
changelog:
  - "0.2 (2026-06-09) — Maury-grade rewrite : enrichissement personas (4 → 8 avec usage concret), capacités CB1-CB10 (8 → 10), invariants INV-FE1 à INV-FE9 (5 → 9), critères succès SCB1-SCB9 (5 → 9), section Risques détaillée + mitigations, section Coût Phase B (token budget ~2,1 M)."
  - "0.1 (2026-06-09) — Initial draft (jugé insuffisant, format non-Maury-grade)."
---

# Brief — Phase B FE catch-up (refonte UX multi-rôle ACP)

## Méthode Maury — Phase TOGAF A (Vision) — BMAD FE-centric

**GATE de signature humaine** : à signer par @gilmry avant ouverture PRD FE.

> **Évolution méthode Maury** observée durant la session 2026-06-09 (cf. mémoire `feedback_maury-fullstack-first`) : pour une app full-stack découplée FE/BE, la méthode Maury doit penser FE+BE dès le brief, pas découper BE-d'abord puis Phase B FE post-mortem. Ce brief est l'application directe de cette leçon : on rattrape la dette FE de Phase A par un BMAD complet FE-centric, intégré au WBS go-live (Track I) plutôt qu'en chantier annexe.

---

## 1. Contexte et déclenchement

Phase A (BE, 9 stories) a été livrée en ~6 h de session le 2026-06-09 :

| Story | Commit | Tests | Statut |
|---|---|---|---|
| 3.1 Sous-rôles (encodeur/émetteur/community/mandataires) | `9598298` | 39 | Mergé |
| 3.2 MagicLink | `d08407c` | 26 | Mergé |
| 3.3 PWA Contractor | `af820bd` | 5 vitest | Mergé |
| 3.4 Mandate | `237c81e` | 45 | Mergé |
| 3.5 Role delegation | `edf171f` | 21 | Mergé |
| 3.6 Ticket complaint + severity + evidence + witnesses | `2142019` | 59 | Mergé |
| 3.7 SyndicResponse + SLA | `62570fb` | 24 | Mergé |
| 3.8 TechnicalSpec versionnable + signatures | `d820c39` | 44 | Mergé |
| 3.9 ContractorEvaluation gated by TechSpec | `c53a7e1` | 31 | Mergé |
| + 5 commits fix CI (astro static, Pixel 7, MagicLink invariant, lint, openapi, types) | `fed175d→cf41ef4` | — | Mergé |

**Total : 1556+ lib tests GREEN, zéro régression. CI fully GREEN sur `cf41ef4`. Chaîne d'audit complète back-end : `Ticket complaint → SyndicResponse SLA → TechnicalSpec signed → ContractorEvaluation`.**

**Découplage observé** (déclencheur Phase B) :
- Le step CI Playwright `Run Documentation Vivante scenarios` cassait régulièrement car le DOM ciblé par les vidéos n'existait pas (UIs syndic/owner manquantes).
- Solution Phase A : `continue-on-error: true` ajouté au workflow (commit `a698f6d`) — pragmatique pour ne pas bloquer la CI BE, mais c'est de la **dette UX**.
- Solution Phase B (ce brief) : reconstruire les UIs avec qualité Maury-grade et retirer le bypass.

## 2. Vision Phase B

Livrer 8 features UI (= 8 user journeys multi-rôles) qui exposent les capacités back-end Phase A à leurs **personas réels**, avec :

1. **Test-id systématique** : `data-testid="<entity>-<action>"` stable, i18n-safe, refactor-safe (mémoire `data-testid-systematic`).
2. **WCAG 2.1 AA** : focus visible, labels associés, aria-live pour erreurs, tap targets ≥ 44 × 44 px, contraste 4,5:1, axe-core en CI gate (mémoire `a11y-wcag-aa-baseline`).
3. **Documentation Vivante refresh** : chaque scénario Playwright `--project=scenarios` re-tourne vert sans `continue-on-error`.
4. **Multi-rôles narratifs** : pas un seul login pour tout un test ; ≥ 2 acteurs distincts par flow critique (mémoire `multirole-narrative-scenarios`).
5. **TDD 3 niveaux** : caractérisation (régression safety net) + Vitest 4-cat unit + Playwright multi-rôle e2e (mémoire `fe-refactor-test-driven`).
6. **Pas de stockage JWT en `localStorage`** : cohérent WP-FE1 mergé Phase 1 (cookie HttpOnly).
7. **Bundle budget** : ≤ +50 KB gzip sur la baseline mesurée 4,3 MB total / 0,6 MB gzip JS (cf. mesure 2026-06-07).

## 3. Personas Phase B (sous-ensemble Phase A — usage concret)

| Persona | Rôle BE | Pages/composants Phase B | Cas concret d'usage |
|---|---|---|---|
| **Admin superadmin** | `superadmin` | `/admin/role-assignments` | Bootstrap initial : assigner le premier syndic d'une ACP. |
| **Syndic** | `syndic` | `/syndic/{magic-links,mandates,role-delegations,technical-specs,contractor-evaluations}` | Délègue ses droits 7j avant vacances, émet mandat notaire pour vente Lot A2, évalue contractor X après travaux. |
| **Owner / Copropriétaire** | `owner` | `/tickets/new` (refacto Complaint), `/c?t=<token>` (PWA déjà livrée) | Dépose plainte tapage nocturne avec photos + 2 témoins. |
| **Board member (CdC)** | `board_member` | Read-only sur tickets complaint dans `/board` (existant) | Reçoit notification escalade SLA dépassé → ouvre le dossier preuve. |
| **Contractor externe** | `contractor` | PWA `/c?t=<token>` (Story 3.3, déjà livrée) | Reçoit lien magique → consulte ticket → soumet devis. |
| **Mandataire AMO / Avocat / Notaire / Architecte / BET** | `amo`/`lawyer`/`notary`/`architect`/`bet` | `/dashboard` section "Specs en attente de signature" | Signe TechnicalSpec en utilisant son mandate actif (Story 3.4) comme justification de pouvoir. |
| **Concierge / Gardien** | `warden` | (lecture seule pour bêta) | Voit le dashboard tickets ouverts du bâtiment. |
| **Délégué temporaire** | rôle = `syndic` reçu par délégation (Story 3.5) | Voit menu syndic + **banner persistant "rôle délégué, re-délégation interdite"** | Owner Pierre remplace le Syndic pendant 7j de vacances. |

## 4. Capacités CB Phase B (10 capacités)

Phase B ne crée **pas** de nouvelles capacités produit — elle expose celles déjà livrées en BE Phase A via UI :

- **CB1** : assigner / révoquer un sous-rôle comptable (encodeur ou émetteur) → UI Story B1.
- **CB2** : émettre un MagicLink (single-use, token signé HMAC, expirable 60s-30j) → UI Story B2.
- **CB3** : émettre / lister / révoquer un Mandate juridique avec validité 5 ans max → UI Story B3.
- **CB4** : déléguer temporairement un rôle (max 90j, non-transitive) → UI Story B4.
- **CB5** : créer un Ticket Complaint enrichi (severity + incident_date + evidence + witnesses, max 10 + 10) → UI Story B5.
- **CB6** : poster une SyndicResponse append-only avec badge SLA color-coded → UI Story B6.
- **CB7** : créer + faire signer une TechnicalSpec versionnée (semver strict, signatures multi-parties, bump major invalide signatures) → UI Story B7.
- **CB8** : évaluer un Contractor (5 scores + comment, gated by TechnicalSpec Approved) → UI Story B8.
- **CB9** : consulter la reputation moyennée d'un Contractor (read-only) → UI Story B8 bis.
- **CB10** : retrouver un signal Documentation Vivante CI sans `continue-on-error` (gouvernance) → Story B9.

## 5. Invariants UI Phase B (INV-FE)

- **INV-FE1** : tout composant interactif expose `data-testid="<entity>-<action>"` stable.
- **INV-FE2** : tout form a `<label for>` + `aria-describedby` pour erreurs + `aria-live="polite"` pour feedback succès, focus visible, tap target ≥ 44 px.
- **INV-FE3** : champs monétaires utilisent `<input type="number" step="0.01">` (alignement Decimal BE, mémoire `no-f64-in-money`).
- **INV-FE4** : tap target ≥ 44 × 44 px (WCAG 2.5.5).
- **INV-FE5** : aucun token JWT en `localStorage` / `sessionStorage` (cohérent WP-FE1 cookie HttpOnly).
- **INV-FE6** : listes paginées affichent "X sur N" + boutons précédent/suivant désactivés aux bornes.
- **INV-FE7** : Phase B respecte la **non-transitivité de délégation** (Story 3.5) en MASQUANT le bouton "Re-déléguer" côté DOM si current user a hérité son rôle (pas juste disabled — absent).
- **INV-FE8** : append-only entities (`SyndicResponse`, `TechnicalSpecSignature`, `ContractorEvaluation`) ne montrent **AUCUN** bouton "Edit" / "Delete" (cohérent INV-23/INV-24 BE).
- **INV-FE9** : composants atomiques `ExpirationBadge`/`SlaBadge` n'utilisent **pas que la couleur** comme indicateur (texte + icône — accessibilité daltoniens).

## 6. Critères succès Phase B (SCB1-SCB9)

- **SCB1** : Documentation Vivante CI passe verte sans `continue-on-error: true`.
- **SCB2** : `@axe-core/playwright` violations = 0 sur chaque composant Phase B.
- **SCB3** : Vitest 4-cat (`@happy/@edge/@security/@negative`) sur chaque composant Svelte (≥ 30 nouveaux tests).
- **SCB4** : Playwright multi-rôle e2e sur chaque flow CB1 → CB8 (8 scénarios, chacun ≥ 2 acteurs distincts).
- **SCB5** : `svelte-check 0/0` erreur/warning sur tout le repo `frontend/`.
- **SCB6** : bundle Phase B cumulé ≤ +50 KB gzip (mesure post-merge via comparaison `dist/_astro/` avant/après).
- **SCB7** : 9 stories (B0-B9) mergées sur `feature/dev` avec CI verte par commit (pas cumulatif).
- **SCB8** : utoipa::path BE complète (Story B0) → `frontend/src/types/api.d.ts` régénéré + CI Contract Types Check vert.
- **SCB9** : leçon "fullstack-first" formalisée comme évolution méthode Maury (mémoire `feedback_maury-fullstack-first` + intégration WBS_GO_LIVE Track I plutôt que document séparé Phase B).

## 7. Hors-scope Phase B (explicite)

Pour éviter le scope creep, ne sont **PAS** dans la Phase B :

- ❌ Pas de refonte des composants existants hors Story 3.6 (qui mandate refacto `TicketCreate.svelte`).
- ❌ Pas de PWA Owner (déjà couvert par Story 3.3 PWA Contractor).
- ❌ Pas de redesign UX global (tokens design, palettes, typography) — on étend l'UI existante avec ses patterns Tailwind/Layout existants.
- ❌ Pas de nouvelles features produit (CB ne sont QUE des exposes BE).
- ❌ Pas d'i18n NL/EN/DE des nouveaux composants Phase B (FR suffit pour bêta privée fermée — i18n complète = Phase C).
- ❌ Pas d'optimisation bundle au-delà du budget +50 KB (lazy-loading i18n = mentionné dans audit 2026-06-07 mais réservé à Phase C).
- ❌ Pas de mobile-native (iOS/Android wrap) — la PWA Story 3.3 suffit.
- ❌ Pas d'e-signature cryptographique réelle (signatures Story 3.8 sont audit-grade DB simple ; eIDAS qualifié = follow-up dédié).

## 8. Dépendances bloquantes

- **DEP-B1** : Phase A BE complete ✅ (commit `cf41ef4`).
- **DEP-B2** : Story B0 (utoipa::path registrations + api.d.ts regen) DOIT être mergée AVANT B1-B8 (sinon les `frontend/src/lib/api/*.ts` doivent caster manuellement, anti-pattern).
- **DEP-B3** : Docker Desktop stable (mémoire session 2026-06-09 : crashes récurrents — ALLOUER 12-16 GB RAM dans Docker Desktop > Settings > Resources).
- **DEP-B4** : signature humaine `@gilmry` du brief / PRD / architecture / stories avant agents.

## 9. Risques + mitigations

| ID | Risque | Probabilité | Impact | Mitigation |
|---|---|---|---|---|
| **RB1** | Docker instable casse les passes agents (observé session 2026-06-09 : 4 crashes) | Haute | Modéré | Restart périodique Docker Desktop + allouer 12-16 GB RAM ; fallback partiel cargo host si Mingw dlltool disponible. |
| **RB2** | utoipa::path manquants empêchent `api.d.ts` auto-typage → cast manuel anti-pattern | Moyenne | Élevé | Story B0 OBLIGATOIRE en V1 préalable. Pas de raccourci. |
| **RB3** | Documentation Vivante e2e flaky (Pixel 7 viewport, top-level config) | Moyenne | Modéré | Pattern `test.use(devices['Pixel 7'])` top-level (cf. fix `4de8f0d`) + seed contractor distinct (cf. fix `709f649`) systématiques dans tous les `.spec.ts` Phase B. |
| **RB4** | Subagent en isolation worktree pollue le main checkout / commit sur la mauvaise branche (cf. mémoire `subagent-worktree-git-salvage`) | Moyenne | Élevé | Brief explicite "commit sur main checkout (pas worktree) si stale base" ; orchestrateur salvage post-agent + cherry-pick. |
| **RB5** | Bundle FE explose au-delà +50 KB | Faible | Modéré | Mesure bundle avant/après chaque story (script CI) ; lazy-loading composants admin/syndic via dynamic imports si dépassement. |
| **RB6** | axe-core en CI ralentit job Playwright significativement | Faible | Faible | Lancer axe-core uniquement sur les pages avec composants Phase B (whitelist). |
| **RB7** | Multi-rôle e2e prend > 5 min wall-clock par scénario (3 logins + actions) | Moyenne | Faible | Réutiliser le `humanLogin` helper + `stepPause` existants (cf. memory `playwright-scenarios-method`). |

## 10. Budget estimé Phase B

| Item | Valeur | Source |
|---|---|---|
| Wall-clock critical path | 4,5 j (B0 → B7 → B8 → B9) | Cf. stories.md Gantt RGRR |
| Wall-clock optimiste avec parallélisme V1-V4 | 3 j | 4 agents // V1, 2 agents // V2-V4 |
| Wall-clock pessimiste (Docker incidents) | 5-6 j | Session 2026-06-09 a perdu ~1h sur crashes Docker |
| Tokens estimés (modèle Opus 4.7) | ~2,1 M tokens | Baseline Phase A slice 3 = ~1,8 M mesurés |
| Nouvelles vidéos Documentation Vivante | 8 (1 par CB) | 1 par flow user journey multi-rôle |
| Nouveaux composants Svelte 5 | ~24 (15 forms/lists + 5 atomiques + 4 conteneurs) | Cf. architecture.md component tree |
| Nouveaux tests Vitest | ~40 (5 par composant en moyenne) | 4-cat × 10 composants principaux |
| Nouveaux e2e Playwright | 9 spec files (≥ 4 scénarios chacun = 36 scénarios) | 1 par CB |

## 11. Gate signature

```
SIGNED-BY:  @____________
DATE:       2026-__-__
HASH:       sha256(this file) = TBD post-signature
NEXT-PHASE: PRD FE (prd.md) — débloquée par signature de ce brief
WBS_REF:    docs/WBS_GO_LIVE_v0.1.0.md Track I (intégration confirmée)
MEMORY_REF: feedback_maury-fullstack-first (évolution méthode capturée)
```
