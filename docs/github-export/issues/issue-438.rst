===============================================================================================
Issue #438: SQL-MIGRATION-001: unit_owners.ownership_percentage DOUBLE PRECISION → NUMERIC(6,5)
===============================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: track:software,priority:medium rust,finance
:Assignees: Unassigned
:Created: 2026-04-30
:Updated: 2026-04-30
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/438>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Constat
   
   Suite à #436 / #437 (EXP-003 Decimal cascade), le repository
   `unit_owner_repository_impl.rs` utilise encore des helpers de conversion
   boundary `decimal_to_f64` / `f64_to_decimal` parce que la colonne SQL
   `unit_owners.ownership_percentage` est typée `DOUBLE PRECISION`
   (migration `20250127000000_refactor_owners_multitenancy.sql` ligne 24).
   
   ```sql
   -- Actuel (DOUBLE PRECISION)
   ownership_percentage DOUBLE PRECISION NOT NULL DEFAULT 1.0
     CHECK (ownership_percentage > 0 AND ownership_percentage <= 1.0)
   ```
   
   ADR-0008 (NUMERIC vs DOUBLE PRECISION) prescrit `NUMERIC` pour tout champ
   financier ou monétairement-adjacent. Les quote-parts pilotent les
   distributions de charges → exactness Decimal critique.
   
   ## Cause
   
   Migration historique (Phase pre-PCMN) avait choisi `DOUBLE PRECISION`
   par défaut. ADR-0008 publié dans #435 (ADR-decimal-policy) le 2026-04-30
   formalise la politique NUMERIC. La cascade EXP-003 a respecté la politique
   côté Rust (Decimal partout) mais le SQL legacy reste à migrer.
   
   ## Impact
   
   - **Précision** : drift IEEE 754 cumulatif possible sur quote-parts
     (ex : `0.1 + 0.1 + 0.1 != 0.3` en f64). En pratique limité car sommes
     rarement > 10 propriétaires par lot, mais théoriquement non-conforme ADR.
   - **Cohérence** : helpers `decimal_to_f64`/`f64_to_decimal` au boundary
     ajoutent une couche de complexité non nécessaire après migration SQL.
   - **Audit** : la divergence Rust/SQL rend les query plans moins lisibles
     (CAST implicit DOUBLE → NUMERIC dans certaines projections).
   
   ## Recette proposée
   
   ### 1. Migration SQL `20260501000000_unit_owners_ownership_percentage_numeric.sql`
   
   ```sql
   -- Migrate unit_owners.ownership_percentage : DOUBLE PRECISION → NUMERIC(6,5)
   -- Cf. ADR-0008.
   -- Borne 0 < x <= 1 préservée. NUMERIC(6,5) = 1.00000 maximum.
   
   -- Préparer la nouvelle colonne
   ALTER TABLE unit_owners
     ALTER COLUMN ownership_percentage TYPE NUMERIC(6, 5) USING ownership_percentage::NUMERIC(6, 5);
   
   -- CHECK constraint (idempotent)
   ALTER TABLE unit_owners DROP CONSTRAINT IF EXISTS unit_owners_ownership_percentage_check;
   ALTER TABLE unit_owners ADD CONSTRAINT unit_owners_ownership_percentage_check
     CHECK (ownership_percentage > 0 AND ownership_percentage <= 1);
   
   -- Note : views unit_ownership_summary, unit_owners_full automatiques car PG
   -- propage ALTER COLUMN aux views non-MATERIALIZED.
   
   -- Trigger validate_unit_ownership_total reste valide (SUM auto-adapt en NUMERIC).
   ```
   
   ### 2. Régénérer offline cache sqlx
   
   ```bash
   docker compose up -d postgres
   docker compose run --rm backend bash -c "
     sqlx migrate run &&
     cargo sqlx prepare -- --lib --tests
   "
   git add backend/.sqlx/
   ```
   
   ### 3. Supprimer helpers `decimal_to_f64`/`f64_to_decimal`
   
   Dans `backend/src/infrastructure/database/repositories/unit_owner_repository_impl.rs` :
   - Supprimer les 2 fonctions helpers (lignes 18-28)
   - Remplacer `decimal_to_f64(unit_owner.ownership_percentage)` → `unit_owner.ownership_percentage`
   - Remplacer `f64_to_decimal(row.ownership_percentage)` → `row.ownership_percentage`
   - Idem pour `find_active_by_building`, `get_total_ownership_percentage`
   
   ### 4. Test RED-first
   
   ```rust
   // @edge — quote-part 0.33333 doit être stockée et relue exact
   #[tokio::test]
   async fn edge_ownership_percentage_decimal_exactness_pg() {
       let pool = setup_test_pool().await;
       let repo = PostgresUnitOwnerRepository::new(pool);
       let uo = UnitOwner::new(unit_id, owner_id, dec!(0.33333), false).unwrap();
       repo.create(&uo).await.unwrap();
       let total = repo.get_total_ownership_percentage(unit_id).await.unwrap();
       assert_eq!(total, dec!(0.33333));
   }
   ```
   
   ## Critères de succès
   
   - [ ] Migration SQL `20260501*_unit_owners_ownership_percentage_numeric.sql` créée
   - [ ] `cargo sqlx prepare` régénéré et committé
   - [ ] Helpers `decimal_to_f64`/`f64_to_decimal` supprimés
     (`unit_owner_repository_impl.rs` ne contient plus de `f64`)
   - [ ] Test `@edge` PG-backed prouve exactness round-trip Decimal
   - [ ] `cargo check --lib` clean + `cargo test --lib` clean
   - [ ] PR commentaire CSI : drift accumulé pré-migration sur fixtures de test
     (mesure pour rapport CSI mensuel — si drift négligeable, simple cleanup
     d'ergonomie ; si drift mesurable, alimente narrative Maury)
   
   ## Effort estimé
   
   **S** (1-2h) — purement mécanique, le pattern Decimal est déjà en place.
   
   ## Liens
   
   - PR mère : #437 (EXP-003 cascade complete)
   - ADR : `docs/adr/0008-numeric-vs-double-precision-postgresql.md` (#435)
   - Audit : `docs/audit/2026-04-30-f64-monetary-audit.md` (#434)
   - Umbrella : #433 (f64 migration umbrella)
   
   🤖 Auto-drafted by Claude Opus 4.7 sous Claude Desktop primary runtime.

.. raw:: html

   </div>

