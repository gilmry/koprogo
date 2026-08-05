==============================================================================
Issue #439: EXP-004: Migrate EtatDate entity (7 monetary f64 fields → Decimal)
==============================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: track:software,priority:high rust,finance legal-compliance
:Assignees: Unassigned
:Created: 2026-04-30
:Updated: 2026-04-30
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/439>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Constat
   
   L'entité `EtatDate` (`backend/src/domain/entities/etat_date.rs`) contient
   encore **7 champs monétaires en `f64`** alors que ADR-0007 forbid f64 monetary
   et que le reste du module financier a été migré vers `Decimal` en #437
   (EXP-003 complete).
   
   ```rust
   // État actuel etat_date.rs
   pub struct EtatDate {
       // ...
       pub unit_area: Option<f64>,           // physique - OK ADR-0009
       pub ordinary_charges_quota: f64,      // ❌ adjacent monétaire
       pub extraordinary_charges_quota: f64, // ❌ adjacent monétaire
       pub owner_balance: f64,               // ❌ MONETARY
       pub arrears_amount: f64,              // ❌ MONETARY
       pub monthly_provision_amount: f64,    // ❌ MONETARY
       pub total_balance: f64,               // ❌ MONETARY
       pub approved_works_unpaid: f64,       // ❌ MONETARY
   }
   ```
   
   Workaround actuel dans `etat_date_use_cases.rs:73` :
   ```rust
   let ordinary_charges_quota = (total_quota * dec!(100)).to_f64().unwrap_or(0.0);
   ```
   
   C'est une conversion boundary qui contredit ADR-0007 et doit être supprimée.
   
   ## Cause
   
   Sprint pilote W18 a découpé la migration en stories digestibles (S/M).
   EXP-003 a couvert les entités domain de premier ordre (expense, invoice,
   journal_entry, charge_distribution, unit, unit_owner, call_for_funds,
   owner_contribution). EtatDate, qui dépend de ces entités, est traité
   en story dédiée (M).
   
   État daté est un **document légal** (Art. 577-11 §1 Code Civil belge —
   information du futur acquéreur d'un lot en copropriété). La cohérence
   monétaire est critique : un état daté avec drift IEEE 754 sur balance
   ou arrears peut générer un préjudice juridique (notaire informé d'un
   montant inexact à 0.01€ près lors de la signature acte authentique).
   
   ## Recette proposée
   
   ### 1. Entity `etat_date.rs`
   
   Migrer les 6 champs monétaires + 2 champs quote-parts vers `Decimal`.
   Garder `unit_area: Option<f64>` (mesure physique m², ADR-0009).
   
   ```rust
   pub struct EtatDate {
       // ...
       pub unit_area: Option<f64>, // physique m² — ADR-0009
       pub ordinary_charges_quota: Decimal,
       pub extraordinary_charges_quota: Decimal,
       pub owner_balance: Decimal,
       pub arrears_amount: Decimal,
       pub monthly_provision_amount: Decimal,
       pub total_balance: Decimal,
       pub approved_works_unpaid: Decimal,
   }
   ```
   
   ### 2. Use case `etat_date_use_cases.rs`
   
   - Supprimer le `.to_f64()` au boundary (ligne 73)
   - Passer `total_quota * dec!(100)` directement à `EtatDate::new`
   - Migrer mocks tests (Result<Decimal>) — déjà fait dans EXP-003
   
   ### 3. Repository `etat_date_repository_impl.rs`
   
   Migrer les 6 colonnes SQL DOUBLE PRECISION → NUMERIC(15,2) (champs montants)
   ou NUMERIC(7,4) (quotas %).
   
   Migration SQL `20260502000000_etat_date_decimal.sql` à créer.
   Régénération offline cache requise.
   
   ### 4. Handlers `etat_date_handlers.rs`
   
   Mettre à jour les DTO request/response (champs `Decimal`).
   
   ### 5. Tests RED-first 4-cat
   
   - `@happy` : génération état daté nominal avec balance positive
   - `@edge` : balance exactement 0, arrears = 0.01€ (centime)
   - `@security` : RBAC notaire only, pas de syndic d'autre org
   - `@negative` : montant négatif rejeté pour `monthly_provision_amount`
   
   ## Critères de succès
   
   - [ ] EtatDate entity 0 occurrence `f64` sur champs monétaires
   - [ ] Migration SQL appliquée + sqlx prepare régénéré
   - [ ] Use case `etat_date_use_cases:73` n'a plus de `.to_f64()` boundary
   - [ ] 4 nouveaux tests RED-first (@happy/@edge/@security/@negative)
   - [ ] `cargo check --lib` clean, `cargo test --lib` clean
   - [ ] Génération PDF état daté validée manuellement (Cowork-Chrome)
     avec montants ronds et avec montants centimes (drift visible si f64)
   
   ## Effort estimé
   
   **M** (4-6h) — entity + use case + 1 SQL migration + handlers + 4 tests
   + regenerate offline cache.
   
   ## Liens
   
   - PR mère EXP-003 : #437
   - Audit f64 : `docs/audit/2026-04-30-f64-monetary-audit.md` ligne 110-130
     (cluster MEDIUM — état daté)
   - ADR-0007 : `docs/adr/0007-decimal-vs-f64-for-money.md`
   - Umbrella : #433
   - Loi : Art. 577-11 §1 Code Civil belge
   
   🤖 Auto-drafted by Claude Opus 4.7 sous Claude Desktop primary runtime.

.. raw:: html

   </div>

