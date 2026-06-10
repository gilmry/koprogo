---
feature: refonte-ux-multi-role-acp/phase-b-fe
phase: index
status: Draft 0.2 — Maury-grade
date: 2026-06-09
authors: [Claude Opus 4.7 (drafting), @gilmry (signature pending)]
---

# Phase B FE — Refonte UX multi-rôle ACP — BMAD index

Cible : livrer les UIs Svelte 5 manquantes pour exposer les capacités back-end Phase A (Stories 3.1→3.9 mergées 2026-06-09) à leurs personas réels (Syndic, Owner, Admin, Mandataire).

## Trigger

- Stories 3.1→3.9 BE mergées avec CI green (`cf41ef4`).
- Documentation Vivante CI cassée silencieusement car UIs manquantes.
- `continue-on-error: true` ajouté workflow (`a698f6d`) — dette UX à amortir.

## Évolution méthode Maury observée

Voir mémoire **`feedback_maury-fullstack-first.md`** : pour une app full-stack découplée FE/BE, BMAD doit penser **FE+BE dès le brief**, pas découper Phase A (BE) puis Phase B (FE) post-mortem. Ce dossier est **l'application** de cette leçon — mais aussi **la dette à payer** parce qu'on a fait l'inverse en Phase A.

## Index des documents (ordre de lecture Maury TOGAF)

1. **`brief.md`** — Phase A (Vision TOGAF). Personas, capacités CB1-CB10, invariants INV-FE1-9, critères succès SCB1-SCB9, hors-scope, risques, budget. Signé par @gilmry → débloque PRD.
2. **`prd.md`** — Phase B (Business architecture). FR-B0-B9 avec Goal métier + User Journey narratif + critères acceptation business + NFR. Matrice FR↔Story↔Persona↔Capacité↔Composant↔Page. Signé → débloque Architecture.
3. **`architecture.md`** — Phase C (Application + Data). Stack confirmé, component tree (Mermaid), data flow type-safe, state management Svelte 5 runes patterns, a11y pattern library, bundle strategy, risques techniques. Signé → débloque Stories.
4. **`stories.md`** — Phase D (Stories). 10 stories (B0-B9) Maury-grade self-contained briefables agent. Chaque story = Goal + parent BE + user journey + AC 4-cat détaillées + data-testid exhaustifs + Files exhaustifs + a11y checklist + wireframe ASCII + notes anti-pattern + cluster coord. Signé → agents Phase B peuvent être lancés selon Gantt.

## Mapping WBS

Track I du **`docs/WBS_GO_LIVE_v0.1.0.md`** (commit `815f98a`) intègre la dette FE dans le plan go-live :

| Story Phase B | WP WBS Track I | Wave Gantt |
|---|---|---|
| B0 utoipa BE | WP-I0 | V1 |
| B1 RoleAssignmentForm/List | WP-I1 | V1 |
| B2 MagicLinkIssueForm | WP-I2 | V1 |
| B3 MandateIssueForm/List/Badge | WP-I3 | V1 |
| B4 RoleDelegationForm/List | WP-I4 | V2 |
| B5 TicketCreate refacto Complaint | WP-I5 | V3 |
| B6 SyndicResponseForm + SlaBadge | WP-I6 | V2 |
| B7 TechnicalSpec full flow | WP-I7 | V3 |
| B8 ContractorEvaluationForm | WP-I8 | V4 |
| B9 Documentation Vivante refresh | WP-I9 | V4 |

**Wall-clock estimé** : 4,5 jours critical path (B0→B7→B8→B9), 3-6j range selon parallélisme docker.

## Gates de signature (workflow Maury)

```
brief.md (Mary)
   ↓ @gilmry sign
prd.md (John)
   ↓ @gilmry sign
architecture.md (Winston)
   ↓ @gilmry sign
stories.md (Bob)
   ↓ @gilmry sign
agents Phase B briefés depuis stories.md
   ↓ DoD-B1..B9 atteints (cf. stories.md)
Documentation Vivante CI verte sans continue-on-error
   ↓
convergence Gate G1 (revue humaine fraîche)
   ↓
Tag v0.1.0
```

## Mémoires Maury appliquées

- `feedback_maury-fullstack-first` — leçon principale.
- `data-testid-systematic` — testids stables i18n-safe.
- `a11y-wcag-aa-baseline` — WCAG 2.1 AA + axe-core CI gate.
- `fe-refactor-test-driven` — TDD 3 niveaux (caractérisation + Vitest + Playwright).
- `multirole-narrative-scenarios` — ≥ 2 acteurs distincts par flow e2e `@happy`.
- `validate-before-compute` — pas de calcul UI sur entités non conformes BE.
- `world-model-seed` — seeds via use-cases, fluent WorldBuilder.
- `no-f64-in-money` — `<input type="number" step="0.01">` pour montants.
- `docker-parallelism-bottleneck` — 4 agents // V1 OK (1 BE + 3 FE), pas plus.
- `subagent-worktree-git-salvage` — orchestrateur stash + salvage si worktree stale.

## Statut de signature

```
brief.md          : SIGNED v0.2 par @gilmry 2026-06-09
prd.md            : SIGNED v0.2 par @gilmry 2026-06-09
architecture.md   : SIGNED v0.2 par @gilmry 2026-06-09
stories.md        : SIGNED v0.2 par @gilmry 2026-06-09
WBS Track I       : intégration validée @gilmry 2026-06-09 (commit 815f98a)
```

**→ Agents Phase B autorisés à être briefés et lancés selon Gantt par passe d'agent (cf. `stories.md` §Gantt et `docs/WBS_GO_LIVE_v0.1.0.md` Track I).**
