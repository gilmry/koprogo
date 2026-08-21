# Agent activity — 2026-08-21 — Resync WBS ↔ issues GitHub (#687/#695/#696)

**Persona :** diagnostic + correction documentaire (Tier 2, doc-only sur le WBS ; fermeture d'issue GitHub).

**Contexte :** demande utilisateur « regarde que le WBS soit reflété dans les issues » puis « vérifie que les issues sur GH soient dans le dépôt ». Vérification croisée dans les deux sens entre `docs/WBS_GO_LIVE_v0.1.0.md` (section blocages go-live, ligne 214) et l'état réel de GitHub — dérive documentaire déjà connue du projet (cf. resyncs précédents `2026-08-06` et `2026-08-08-wbs-status-resync.md`).

## Constat

**Sens 1 — WBS → GitHub** (les 7 issues tracées au resync du 2026-08-09 sont-elles toujours dans l'état documenté ?) : aucune dérive. #634/#636/#604/#603/#617/#605 toujours closes, #699 toujours ouverte, exactement comme documenté. Aucune activité GitHub sur ces issues depuis le 2026-08-09.

**Sens 2 — GitHub → WBS** (les 71 issues ouvertes sont-elles toutes rattachées à un doc du repo ?) : recherche `grep -rlE "(#<n>\b|issues/<n>\b)"` sur `*.md`/`*.rst` pour chaque issue ouverte. 68/71 référencées (WBS, `docs/agent-activity/`, ou EPIC parent). 3 non trouvées :

| Issue | État avant | Constat | Action |
| --- | --- | --- | --- |
| #687 (babel plugin-transform-modules-systemjs) | OPEN, jamais tracée | Doublon de #699 (créée 4 j après, même vuln + une deuxième) | Fermée avec commentaire renvoyant vers #699 |
| #695 (BDD cassés post-H15) | OPEN, jamais tracée | Réel : régression de WP-CL6/H15 (marqué ✅ dans le WBS), 137/592 scénarios BDD rouges, trouvée le 2026-08-08 en vérifiant la CI de #617/#634, jamais reportée | Ajoutée au WBS (blocages go-live + note sur WP-CL6) |
| #696 (109 échecs smoke Playwright) | OPEN, jamais tracée | Réel : instabilité pré-existante, explicitement non liée à #617/#634, trouvée le même jour, jamais reportée | Ajoutée au WBS (blocages go-live) |

## Action prise

- Fermé **#687** sur GitHub (doublon de #699, sans action code).
- `docs/WBS_GO_LIVE_v0.1.0.md` : ligne 214 (compteur blocages) mis à jour `resync 2026-08-09 : 6/7 FAIT, 1 ouvert` → `resync 2026-08-21 : 6/9 FAIT, 3 ouverts (#699, #695, #696)` ; ajout de deux puces #695/#696 dans la liste des blocages ; note d'avertissement ajoutée sur la puce WP-CL6 (H15) pointant vers #695.

## Ce qui reste réellement ouvert (post-resync)

- **#699** — npm audit 3 vulns high, dev-only, non traité (inchangé depuis le 2026-08-09).
- **#695** — régression BDD H15, bloquant réel si non résolu avant tag v0.1.0 (WP-CL6 était marqué FAIT sur la seule base de testcontainers, pas de la suite BDD complète).
- **#696** — instabilité smoke Playwright, pré-existante et documentée comme non-bloquante par l'issue elle-même, mais à garder sous les yeux avant tag.
- Track F/G, ADR-0008 amendement : inchangés, hors périmètre de ce resync.

Aucune régression, aucun code applicatif touché dans ce resync (uniquement doc + fermeture d'une issue doublon).
