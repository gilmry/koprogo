=========================================================================================================
Issue #273: fix(legal): Réduction de vote mandataire (Art. 3.87 §7 CC) — limite procurations ✅ done
=========================================================================================================

:State: **CLOSED**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: bug,priority:medium legal-compliance,governance release:0.1.0
:Assignees: Unassigned
:Created: 2026-03-11
:Updated: 2026-03-21
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/273>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Problème ~~CRITIQUE~~ → Partiellement résolu
   
   ~~Lacune CRITIQUE : aucune limite sur le nombre de procurations qu'un mandataire peut détenir.~~
   
   **Mise à jour 2026-03-14** : La validation des procurations (max 3 + exception 10%) est **déjà implémentée** dans `resolution_use_cases.rs::validate_proxy_limit()`. Il reste la **réduction de vote** à implémenter.
   
   ## Règles légales (Art. 3.87 §7 CC)
   
   ### ✅ Règle 1 : Limite de 3 procurations — IMPLÉMENTÉ
   > *"Nul ne peut accepter plus de trois procurations de vote."*
   
   - Implémenté dans `resolution_use_cases.rs::validate_proxy_limit()`
   - Test unitaire : `test_proxy_limit_max_3_enforced`
   
   ### ✅ Règle 2 : Exception 10% — IMPLÉMENTÉ
   > *"Toutefois, un mandataire peut recevoir plus de trois procurations de vote si le total des voix dont il dispose lui-même et de celles de ses mandants n'excède pas 10% du total des voix affectées à l'ensemble des lots de la copropriété."*
   
   - Implémenté dans `resolution_use_cases.rs::validate_proxy_limit()`
   - Test unitaire : `test_proxy_limit_10_percent_exception_allows_more`
   
   ### ❌ Règle 3 : Réduction de vote — NON IMPLÉMENTÉ
   > *"Nul ne peut voter, pas même comme mandataire, pour un nombre de voix supérieur au nombre total de voix dont disposent les autres copropriétaires présents ou représentés."*
   
   Un mandataire ne peut jamais peser plus que la somme de tous les autres présents/représentés. Mécanisme anti-majorité-capture.
   
   ## Fichiers existants (règles 1 & 2)
   - `backend/src/domain/entities/vote.rs` — self-proxy interdit (`CHECK owner_id != proxy_owner_id`)
   - `backend/src/application/use_cases/resolution_use_cases.rs` — `validate_proxy_limit()` avec exception 10%
   - `backend/src/application/ports/vote_repository.rs` — `count_proxy_votes_for_mandataire()`
   - `backend/src/infrastructure/database/repositories/vote_repository_impl.rs` — SQL aggregation
   - `backend/migrations/20251115120000_create_resolutions_and_votes.sql` — DB constraints
   
   ## Reste à faire (règle 3 : réduction de vote)
   
   ### Fichiers à modifier
   - `backend/src/application/use_cases/resolution_use_cases.rs` — ajouter `validate_vote_reduction()` dans `cast_vote()`
   - `backend/src/application/ports/vote_repository.rs` — méthode pour calculer total voix autres présents
   
   ### Algorithme
   ```
   total_mandataire = voix_propres + somme(voix_procurations)
   total_autres = somme(voix de tous les autres présents/représentés)
   si total_mandataire > total_autres → réduire à total_autres
   ```
   
   ## Definition of Done
   - [x] Validation proxy_count ≤ 3 avec exception 10%
   - [x] Erreur métier claire avec référence légale
   - [x] Test unitaire : 4e procuration refusée, exception 10% acceptée
   - [x] Tests existants toujours verts
   - [ ] Réduction de vote : mandataire ne peut peser plus que tous les autres réunis
   - [ ] Test unitaire : réduction de vote appliquée quand seuil dépassé

.. raw:: html

   </div>

