# Agent activity — 2026-08-06 — Resync marqueurs WBS (A1/A3/B1)

**Persona :** diagnostic + correction documentaire (Tier 2, doc-only, aucun code touché).

**Contexte :** demande utilisateur « je veux avancer dans le WBS ». Avant de choisir un chantier, vérification systématique des marqueurs `FAIT` du `docs/WBS_GO_LIVE_v0.1.0.md` — la dérive documentaire est un risque connu du projet (cf. correction 2026-07-26 sur #618, log `2026-07-26-sync-audit-feature-dev.md`).

## Constat

Trois work packages étaient livrés en code mais sans marqueur `FAIT` dans le WBS :

| WP | Preuve | Détail |
| --- | --- | --- |
| WP-A1 (clôture C1 gouvernance) | PR #535, commits `98c16518`/`7fb172e0`/`a8459218` | Ferme #525 (confirmé CLOSED sur GitHub) |
| WP-A3 (EXP-006 journal_entry) | commits `1a59d0eb`/`61e5e757` | `JournalEntryError` + pont `AppError` + 4-cat livrés |
| WP-B1 (re-vérif bugs revue humaine) | log `docs/agent-activity/2026-05-17-wbs-b1.md` | 4 bugs (WF14-2, WF2-1, WF7-1, NaN%) déjà ALREADY-FIXED, vérifié 2026-05-17 |

Cause probable : les 3 WPs ont été traités début du chantier (2026-05-16/17/19) et le marqueur n'a jamais été rétro-ajouté, contrairement aux WPs suivants où la convention `**FAIT** (…)` a été systématisée.

## Action prise

`docs/WBS_GO_LIVE_v0.1.0.md` : ajout du marqueur `**FAIT**` + preuve (SHA/PR/log) sur les 3 lignes WP-A1/A3/B1, + mise à jour de la classe `done` du graphe mermaid (`classDef done`) pour inclure A1/A3/B1. Aucun autre contenu modifié.

## Ce qui reste réellement ouvert (post-resync)

- **#617** (WP-I9 Phase C) — 8 specs Playwright multi-rôle cassées, bloque le gate G1 (dépendance directe dans le graphe de dépendances).
- **#634** — frontend cassé par 4 bumps Dependabot majeurs (astro 7/svelte 9), listé comme blocage go-live dans le WBS.
- **WP-D2** — couverture vitest composants critiques (convocation/réunion) à étoffer ; auth store déjà couvert par `auth.test.ts`.
- **Track F/G** — 100% Tier 1 (VPS provisioning, TLS, poller, revue humaine, tag), hors périmètre agent.
- **ADR-0008 amendement** — acceptation humaine (@gilmry) pendante au merge (WP-A7).

Aucune régression, aucun code applicatif touché.
