==============================================================================================
Issue #658: Build backend cassé : lopdf 0.44 ⊥ time 0.3.47 (bump Dependabot printpdf 0.7→0.11)
==============================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: None
:Assignees: Unassigned
:Created: 2026-07-25
:Updated: 2026-07-25
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/658>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Constat
   
   `docker compose build backend` échoue (et `cargo build` à froid également) :
   
   ```
   error[E0599]: no variant, associated function, or constant named `StringLiteral`
                 found for enum `BorrowedFormatItem<'a>`
   error: could not compile `lopdf` (lib) due to 1 previous error
   ```
   
   Versions verrouillées : `lopdf 0.44.0` + `time 0.3.47`, tirées par **`printpdf 0.11.1`**.
   
   ## Cause
   
   Bump Dependabot **`printpdf 0.7 → 0.11.1`** (#642 et suivants) → `lopdf 0.44` incompatible avec le `time 0.3.47` du lockfile.
   
   **Pré-existant sur `feature/dev`, pas une régression locale** : `Cargo.lock` identique à `origin/feature/dev`. Le bump est passé sans vérification (cf. issue soeur « auto-merge Dependabot sans gate CI »), et l'image dev n'avait pas été rebuildée depuis.
   
   ### ⚠️ Piège de diagnostic (important)
   
   Les tests **passent quand même** si le volume `target_cache` contient `lopdf` compilé **avant** le bump — c'est ce qui a permis de valider Track H / Story H15 sans voir le problème. Un `docker volume rm koprogo_target_cache` (ou un prune de l'image dev) rend le blocage visible.
   
   ➡️ **Ne pas conclure « env OK » depuis un cache chaud.**
   
   ## Recette proposée
   
   Au choix, à trancher :
   
   1. **Revenir à `printpdf = "0.7"`** — version qui fonctionnait ; simple, mais réintroduit la version visée par le bump.
   2. **Bumper `time`** vers une version compatible avec `lopdf 0.44` (vérifier l'API `BorrowedFormatItem`).
   3. **Attendre un `lopdf` corrigé** et geler `printpdf` en attendant.
   
   **Recoupe #636** (advisory `lopdf` RUSTSEC-2026-0187) — probablement à traiter conjointement, puisque les deux portent sur la version de `lopdf`.
   
   ## Critères de sortie
   
   - [ ] `docker compose build backend` vert **sans cache** (`--no-cache` ou après `docker volume rm koprogo_target_cache`).
   - [ ] `docker compose run --rm backend bash -c "SQLX_OFFLINE=true cargo check --lib --tests"` vert.
   - [ ] Choix de version documenté (et articulé avec #636 si résolution commune).
   
   ## Contexte
   
   - Découvert le 2026-07-25 en rebuildant l'image dev après le pin du toolchain (`ea251da`).
   - Le pin nightly (`ea251da`) corrige un **autre** blocage (ICE codegen `nightly-2026-07-24` sur `tokio`) — indépendant de celui-ci.

.. raw:: html

   </div>

