===========================================================================================
Issue #335: fix: Marketplace handlers 100% stub — #276 fermée mais aucune implémentation DB
===========================================================================================

:State: **CLOSED**
:Milestone: Jalon 3: Features Différenciantes 🎯
:Labels: bug,track:software priority:high
:Assignees: Unassigned
:Created: 2026-03-25
:Updated: 2026-03-29
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/335>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Problème
   
   L'issue #276 (feat: Marketplace corps de métier + enquêtes satisfaction) a été fermée, mais le fichier `backend/src/infrastructure/web/handlers/marketplace_handlers.rs` contient **100% de stubs** sans aucune intégration base de données.
   
   ## Endpoints concernés
   
   | Endpoint | Comportement actuel |
   |----------|-------------------|
   | `GET /marketplace/providers` | Retourne un vec vide (TODO: Implement search) |
   | `GET /marketplace/providers/{slug}` | Retourne toujours 404 NotFound |
   | `POST /service-providers` | Crée l'objet en mémoire mais ne persiste pas |
   | `GET /buildings/{id}/reports/contract-evaluations/annual` | Retourne des zéros hardcodés |
   
   ## TODOs dans le code
   
   ```rust
   // TODO: Implement search with filters (line 20)
   // TODO: Query provider by slug, return public profile (line 33)
   // TODO: Save to database using repository (line 64)
   // TODO: Query contract evaluations for building and year (line 86)
   ```
   
   ## Action requise
   
   - Créer le `MarketplaceRepository` trait + implémentation PostgreSQL
   - Créer la migration pour la table `service_providers`
   - Implémenter les 4 endpoints avec persistance réelle
   - Ajouter tests E2E
   
   ## Contexte
   
   Détecté lors d'un audit croisé code vs GitHub issues (2026-03-25).
   Réf: #276

.. raw:: html

   </div>

