======================================================================================================
Issue #526: question(domain): expenses_amount_check rejects amount = 0 — intentionnel ou trop strict ?
======================================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: question
:Assignees: Unassigned
:Created: 2026-05-14
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/526>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ### Constat
   
   Lors de l'écriture des BDD @edge pour #521 Story A, le scenario test "Zero amount = 0.0000" déclenche :
   
   ```
   violates check constraint "expenses_amount_check"
   detail: "Failing row contains (..., maintenance, Zero, 0.00, ...)"
   ```
   
   Le constraint est défini quelque part dans `backend/migrations/*.sql` (probablement `amount > 0`).
   
   ### Question domain
   
   Est-ce qu'une charge à 0 € a un sens métier ? Cas réels possibles :
   
   - Devis prévisionnel à 0 (placeholder) — probablement non, on n'inscrirait pas l'expense
   - Régulation comptable avec ligne à 0 (rare)
   - Charge offerte par un fournisseur (gratuité ponctuelle)
   
   Si la contrainte est intentionnelle (`amount > 0`), alors le test @edge devrait utiliser une valeur minimale légale (e.g. `0.01`) au lieu de `0`. Si la contrainte est trop stricte (devrait être `amount >= 0`), il faut la relaxer.
   
   ### Recette
   
   - Décision produit / domain expert
   - Si garder `> 0` : ajuster le test #521 @edge `Zero` → utiliser `0.01`
   - Si relaxer à `>= 0` : migration SQL + test BDD passe tel quel
   
   ### Refs
   
   - Découvert via BDD #521 Story A scenario @edge
   
   ### Hors scope #521
   
   Cette question est tangente à #521 (qui concerne le decode f64/NUMERIC). Traitée séparément pour ne pas polluer le scope Story A.

.. raw:: html

   </div>

