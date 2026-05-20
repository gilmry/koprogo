# Agent activity — 2026-05-19 — Audit docs/ 3 phases (post-merges)

**Persona :** doc/architecture agent (Tier 2 — édits ciblés sur docs
publiques. Pas de mutation prod, pas de force-push, pas de `--no-verify`.
Acceptation finale = merge humain Tier-1, CRITICAL.md #11).

**Branche :** `story/wbs-docs-audit-3phases` (depuis `origin/feature/dev`).
Demande explicite humain : « on passe en revue tout le dossier docs, on
enlève ce qui est faux en phase 1, on met à jour ce qui est obsolète en
phase 2 et complète l'existant en documentant ce qui n'est pas documenté
et ce qui a été fait depuis la dernière version. »

**Inventaire initial** : 250 fichiers `.md`+`.rst` dans `docs/`+`Maury/`,
135 datés du 2026-03-30 (batch initial massif). Triage par 3 sous-agents
`Explore` en parallèle.

## Phase 1 — Enlever ce qui est faux (commit `23379a5`)

**Approche** : pas de `git rm` — bandeaux d'archive en tête (les fichiers
restent comme trace historique, leur statut « supersédé / périmé » devient
visible **dans** le fichier).

| Fichier                               | Bandeau                                                                                                                                             |
| ------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `docs/HUMAN_REVIEW_REPORT_v0.1.0.md`  | Bugs WF1-1/2/3 / WF2-1 / WF7-1 / WF14-2 / NaN% **ALREADY-FIXED** vs `feature/dev` (re-vérifié WP-B1 PR #536 + WP-FE2 PR #547). Renvoie WBS_GO_LIVE. |
| `docs/WBS_BUGFIX_UI_v0.1.0.md`        | Supersédé par WBS_GO_LIVE.                                                                                                                          |
| `docs/WBS_CORRECTIONS_v0.1.0.md`      | Idem.                                                                                                                                               |
| `docs/WBS_RELEASE_0_1_0.md`           | État 2026-03-30 supersédé ; umbrella #433 fermée, FE1 livré.                                                                                        |
| `docs/release/REVUE_HUMAINE_0_1_0.md` | Plan revue manuel pointant l'état pré-FE1 ; futur revue = critères GO WBS_GO_LIVE.                                                                  |

## Phase 2 — Mettre à jour l'obsolète (commit `996ee4e`)

| Fichier                          | Action                                                                                                                                                                                   |
| -------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `docs/ROADMAP_PAR_CAPACITES.rst` | Jalon 1 : 83%→100% (#271/#272/#273 ✅, ajout #343 FE1 + #339 rotate + #525 Governance Decimal). Jalon 4 : 50%→54% (ajout #339 dans fermées). §Force de Frappe : "Nov 2025" → "Mai 2026". |
| `docs/MISSION.rst`               | §"Prix Fixe" : note conditionnelle 5€/mois (cible Jalon 2+, gratuit en bêta fermée v0.1.0).                                                                                              |
| `docs/RELEASE_PROCESS.md`        | Ajout section "v0.1.0 Bêta Fermée — cas spécial" pointant les critères GO du WBS_GO_LIVE + gate humain Tier-1.                                                                           |
| `docs/JALONS_MIGRATION.rst`      | **NO ACTION** (statut "Migration Complétée ✅" cohérent, doc historique).                                                                                                                |

**Important** : #274..#280 (Jalon 3 Features Différenciantes) **laissés
⏳** — non livrés dans les 6 PR de la session récente. Ne pas mentir
(CRITICAL.md « ne pas deviner »).

## Phase 3 — Documenter ce qui n'est pas documenté (commit `<HASH-A-VENIR>`)

5 EXTEND sur fichiers existants (CRITICAL.md « préférer édit ciblé,
préférer extend > create ») :

| Fichier                                   | Section ajoutée                                                                                                                                                                                                                                                                |
| ----------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `docs/adr/0002-hexagonal-architecture.md` | Amendment 2026-05-19 — **Domain-typed error pattern** : enum pur + Display/Error + `From<_> for String` (pont) + `From<_> for AppError` (Validation 400). Exemple `ChargeDistributionError`. Renvoie ADR-0008 §carve-outs f64.                                                 |
| `docs/JWT_REFRESH_TOKENS.md`              | Amendment 2026-05-19 — **FE1 HttpOnly Cookie + Single-flight Refresh** : spec cookie, handlers backend, `accessToken.ts` mémoire, `auth.ts` réécrit, single-flight `inflightRefresh` (#550), tests 4-cat backend + Playwright `AuthCookie.spec.ts`, topologie SameSite=Strict. |
| `docs/RD_PUBLIC_API_V2.rst`               | §"API Key Lifecycle — Rotation (2026-05 #339)" : endpoint, gate SYNDIC/SUPERADMIN, transaction, isolation cross-org 404, audit, secret une fois, sqlx::query non-macro, tests 4-cat.                                                                                           |
| `docs/BELGIAN_ACCOUNTING_PCMN.rst`        | §"Annulations & contre-écritures (politique #526)" : `CHECK > 0` conservé, annulations en contre-écritures journal PCMN (pas de relâche schéma). Renvoie ADR-0008 §C.                                                                                                          |
| `docs/E2E_TESTING_GUIDE.rst`              | §"Multi-rôles E2E (post-FE1 cookie HttpOnly)" + §"Pattern silent-refresh single-flight (frontend)" + spec Playwright de référence `AuthCookie.spec.ts`.                                                                                                                        |

**Carve-outs ADR-0008** : déjà documenté dans le fichier ADR lui-même (§Amendment 2026-05-19) → NO ACTION supplémentaire.

## Hors-scope explicite

- **NON touchés** : `docs/agent-activity/**` (traces immuables Tier-2),
  `docs/archive/**`, `docs/archives/**`, `docs/blog/**`, `docs/cowork/**`,
  `Maury/**`, `docs/RD_*.rst` (sauf `RD_PUBLIC_API_V2.rst`),
  `docs/specs/**`, `docs/legal/**`, `docs/energie/**`,
  `docs/user-guides/**` (chantier séparé #426).
- **NON traités** : `docs/NOUVELLES_FONCTIONNALITES_2025.rst` (auto-régénéré
  per #428 §8bis), `docs/PERFORMANCE_REPORT.rst` (non flagé), refonte
  CLAUDE.md (plan précédent rejeté).

## Workflow

3 commits successifs sur la branche dédiée, 1 PR draft. Aucun code
applicatif touché ⇒ pas de gate CI sur le contenu (markdown/rst). Le
gate humain final est le merge de la PR (Tier-1, doc publique).

## Critère qualité

Édits ciblés (pas de réécriture) ; chaque ajout cite la source (PR /
issue / fichier source) ; pas de nouveau fichier doc créé (per CRITICAL.md).
