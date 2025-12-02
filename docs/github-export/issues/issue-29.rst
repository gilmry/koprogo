======================================================================
Issue #29: feat: Validation des quotes-parts unitaires (total = 100%)
======================================================================

:State: **CLOSED**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: enhancement,phase:vps track:software,priority:high
:Assignees: Unassigned
:Created: 2025-10-27
:Updated: 2025-11-17
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/29>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   Dans une copropriété, la somme des quotes-parts (`ownership_percentage`) de tous les copropriétaires actifs d'un même lot doit toujours égaler 100% (1.0 en décimal).
   
   ## Problème actuel
   - Aucune validation côté backend lors de la création/modification de `unit_owners`
   - Possible de créer des situations incohérentes (total ≠ 100%)
   - L'UI affiche un warning mais n'empêche pas la sauvegarde
   
   ## Solution proposée
   
   ### Backend validation
   1. Créer une fonction `validate_unit_ownership_total(unit_id)`
   2. Vérifier dans les handlers:
      - `add_owner_to_unit`
      - `update_unit_owner`
      - `remove_owner_from_unit`
   3. Retourner erreur 400 si total ≠ 1.0 (avec tolérance de 0.001 pour les arrondis)
   
   ### Contrainte database
   Ajouter un CHECK constraint via trigger PostgreSQL:
   ```sql
   CREATE OR REPLACE FUNCTION check_unit_ownership_total()
   RETURNS TRIGGER AS $$
   DECLARE
       total DECIMAL(5,4);
   BEGIN
       SELECT COALESCE(SUM(ownership_percentage), 0) INTO total
       FROM unit_owners
       WHERE unit_id = NEW.unit_id AND end_date IS NULL;
       
       IF total > 1.0001 OR total < 0.9999 THEN
           RAISE EXCEPTION 'Total ownership for unit % must equal 100%% (got %)', NEW.unit_id, total * 100;
       END IF;
       
       RETURN NEW;
   END;
   $$ LANGUAGE plpgsql;
   
   CREATE TRIGGER validate_unit_ownership
       AFTER INSERT OR UPDATE ON unit_owners
       FOR EACH ROW EXECUTE FUNCTION check_unit_ownership_total();
   ```
   
   ### Frontend improvements
   - Bloquer la sauvegarde si total ≠ 100%
   - Afficher calcul en temps réel lors de l'édition
   - Suggérer ajustements automatiques
   
   ## Tâches
   - [ ] Créer migration avec trigger PostgreSQL
   - [ ] Ajouter validation dans `unit_owner_handlers.rs`
   - [ ] Tests unitaires pour cas limites (99.99%, 100.01%, etc.)
   - [ ] Améliorer UX frontend (bloquer sauvegarde + suggestions)
   - [ ] Documenter règle métier dans CLAUDE.md
   
   ## Cas particuliers
   - Tolérance de ±0.01% pour arrondis
   - Validation uniquement pour `end_date IS NULL` (actifs)
   - Permettre états transitoires (ajout séquentiel avec total temporaire ≠ 100%)

.. raw:: html

   </div>

