# Agent activity — 2026-08-21 — Fix #695 : seed BDD `bdd_operations.rs` post-H15

**Persona :** correction de bug (Tier 2, code non-prod).

**Contexte :** suite au resync WBS du jour (`docs/agent-activity/2026-08-21-wbs-status-resync.md`), demande utilisateur « regarde le WBS et avance ». Le seul blocage go-live réellement actionnable et sur le chemin critique (`make ci` VERT) est **#695** — #699 (npm audit) nécessite un arbitrage bump/overrides hors scope immédiat, #696 est explicitement documenté non-bloquant par l'issue elle-même.

## Root cause (rappel #695)

La migration H15 (WP-CL6, commit `b9fa9d6`) a supprimé `units.organization_id` au profit de `units.acp_id` (NOT NULL). Les helpers de seed dans `backend/tests/bdd_operations.rs` construisaient encore `INSERT INTO units (..., organization_id, ...)`.

## Constat sur le code courant (post-pull `feature/dev` du jour, HEAD `cf7326d`)

Seuls **4 sites** restaient concernés dans `backend/tests/bdd_operations.rs` (les variantes citées dans l'issue d'origine — « insert unit for age request owner », « insert poll unit », « insert etat date unit », « create unit for legal holds », « create Alice unit » — n'existent plus sous ces noms dans le fichier actuel, probablement déjà nettoyées par du travail antérieur non recoupé avec #695) :

1. `given_building_with_units` (step `a building "..." with N units exists`)
2. `given_2_uploads` (step `I have uploaded 2 energy bills`)
3. `given_unverified_upload` (step `an unverified upload exists`)
4. `given_n_participants` (step `N participants have uploaded energy data`)

## Fix appliqué

Pour chacun des 4 sites : colonne `organization_id` → `acp_id` dans le `INSERT INTO units`, valeur bindée = `ensure_default_acp_for_org(pool, org_id).await` (helper déjà présent dans le fichier, importé de `tests/common/acp_test_helper.rs`, déjà utilisé pour la création du `building` principal du monde de test à la ligne ~320). Idempotent : réutilise le même ACP que celui déjà associé au `building_id` du scénario (cohérent avec l'invariant de la migration : « un lot appartient toujours à la même ACP que son building »).

Aucune migration touchée, aucun fichier `.feature` à modifier (les steps eux-mêmes ne référencent pas `organization_id`).

## Vérification

- `grep -n "INSERT INTO units" backend/tests/bdd_operations.rs` : 0 occurrence résiduelle de `organization_id`, 4/4 sites utilisent désormais `acp_id`.
- `rustfmt --check --edition 2021 backend/tests/bdd_operations.rs` : propre (aucune sortie).
- **Limitation d'environnement** : ni `docker compose run --rm backend cargo test --test bdd_operations` (daemon Docker indisponible dans cette session distante — `service docker start` échoue avec `ulimit: Operation not permitted`), ni le fallback hôte `cargo check --test bdd_operations` (bloqué par la politique réseau du proxy de session : le build script de `utoipa-swagger-ui` télécharge un zip depuis `github.com/swagger-api/swagger-ui`, hors du scope repo autorisé pour cette session, `403`) n'ont pu être exécutés localement. **CI GitHub Actions (accès réseau complet) validera à la place** — PR suivie (`subscribe_pr_activity`) pour confirmer `BDD Tests` vert et agir sur tout résidu.

## Ce qui reste ouvert

- **#699** (npm audit dev-only) — nécessite bump amont ou `overrides` npm ciblé, hors scope de cette passe.
- **#696** (instabilité smoke Playwright pré-existante) — explicitement non-bloquant, à garder sous les yeux avant tag.
