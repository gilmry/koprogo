# Agent activity — 2026-08-08 — Resync marqueurs WBS + fermeture issues (#634/#617/#697/#698)

**Persona :** diagnostic + correction documentaire (Tier 2, doc-only sur le WBS ; fermeture d'issues GitHub).

**Contexte :** demande utilisateur « regarde la dette du wbs ». Vérification des 6 blocages go-live tracés en ligne 214 de `WBS_GO_LIVE_v0.1.0.md` contre l'état réel du code et de GitHub — la dérive documentaire est un risque connu du projet (cf. resync précédent `2026-08-06-wbs-status-resync.md`).

## Constat

| Issue | GitHub avant | Code | Action |
| --- | --- | --- | --- |
| #634 (frontend Dependabot) | OPEN | ✅ fixé `3291f1b1`, vérifié vert en CI réelle | Fermé |
| #636 (Rust audit) | CLOSED | ✅ | Aucune |
| #604 (trigger organization_id) | CLOSED | ✅ | Aucune |
| #603 (verify_org_access skip) | CLOSED | ✅ | Aucune |
| #617 (Phase C Doc Vivante) | OPEN | ✅ 8/8 fermé Story S3 `387128c4` | Fermé |
| #605 (Gdpr flaky, distinct de #617) | OPEN | ❌ non traité | Reste ouvert |

Nouveau depuis le dernier resync : #697 et #698, trouvés et corrigés le 2026-08-08 en testant l'admin au navigateur (clic réel, pas visite passive) — `docs/maury/fix-admin-buttons-acp/`. Commits `7bfb3b56` (Story 1, #697) et `a046de40` (Story 2, #698).

## Action prise

- Fermé #634, #617, #697, #698 sur GitHub avec commentaire référençant le commit de fix et la preuve de vérification (CI réelle / suite Playwright locale).
- `docs/WBS_GO_LIVE_v0.1.0.md` : ligne 214 réécrite (état par item + commit refs), `WP-I9` marqueur `PARTIEL`→`FAIT`, statut Track I global mis à jour (2026-06-15→2026-08-08), graphe mermaid `classDef done` étendu à `I0,I1,I2,I3,I4,I6,I8,I9` (I5/I7 gardent leur classe `critical` existante, pas de double-classe).

## Ce qui reste réellement ouvert (post-resync)

- **#605** — 3 tests Gdpr Playwright flaky (timing/race), seul bloquant go-live encore non traité.
- **WP-D2** — couverture vitest composants critiques (convocation/réunion) à étoffer.
- **Track F/G** — 100% Tier 1 (VPS provisioning, TLS, poller, revue humaine, tag), hors périmètre agent.
- **ADR-0008 amendement** — acceptation humaine (@gilmry) pendante au merge (WP-A7).

Aucune régression, aucun code applicatif touché dans ce resync (uniquement doc + fermetures d'issues).
