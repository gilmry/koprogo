===========================================================================
Issue #75: feat: Complete Meeting Management API (AG assemblées générales)
===========================================================================

:State: **OPEN**
:Milestone: Phase 1: VPS MVP + Legal Compliance
:Labels: enhancement,phase:vps track:software,priority:critical
:Assignees: Unassigned
:Created: 2025-11-01
:Updated: 2025-11-08
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/75>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Issue #001 - Gestion des Assemblées Générales (API complète)
   
   **Priorité**: 🔴 CRITIQUE  
   **Estimation**: 6-8 heures  
   **Phase**: VPS MVP (Nov 2025 - Mar 2026)  
   
   ## 📋 Description
   
   Implémenter la couche Application et Infrastructure pour la gestion complète des assemblées générales. L'entité de domaine `Meeting` et le schéma de base de données existent déjà, mais aucune API n'est exposée.
   
   **Contexte métier**: Les assemblées générales sont une obligation légale pour toute copropriété. Le système doit permettre de planifier, gérer et archiver les AG (ordinaires et extraordinaires).
   
   ## 🎯 Objectifs
   
   - [ ] Créer les Use Cases pour les opérations métier sur les meetings
   - [ ] Implémenter les handlers HTTP (Actix-web)
   - [ ] Exposer les endpoints REST API (10 endpoints)
   - [ ] Ajouter les tests E2E + BDD Gherkin
   - [ ] Documenter l'API (commentaires OpenAPI-ready)
   
   ## 📐 Spécifications Techniques
   
   ### Endpoints à implémenter
   
   | Méthode | Endpoint | Description | Auth |
   |---------|----------|-------------|------|
   | `POST` | `/api/v1/meetings` | Créer une assemblée | Syndic+ |
   | `GET` | `/api/v1/meetings` | Lister toutes les assemblées | Owner+ |
   | `GET` | `/api/v1/meetings/:id` | Détails d'une assemblée | Owner+ |
   | `PUT` | `/api/v1/meetings/:id` | Mettre à jour une assemblée | Syndic+ |
   | `DELETE` | `/api/v1/meetings/:id` | Supprimer une assemblée | SuperAdmin |
   | `GET` | `/api/v1/buildings/:id/meetings` | Assemblées d'un immeuble | Owner+ |
   | `PUT` | `/api/v1/meetings/:id/publish` | Publier l'agenda | Syndic |
   | `POST` | `/api/v1/meetings/:id/minutes` | Ajouter un PV | Syndic |
   | `GET` | `/api/v1/meetings/:id/minutes` | Récupérer le PV | Owner+ |
   | `PUT` | `/api/v1/meetings/:id/close` | Clôturer l'AG | Syndic |
   
   ## 🔗 Dépendances
   
   **Bloque**: #019 (Convocations AG), #022 (Conseil Copropriété vote élections)
   
   ## 📚 Fichiers à Créer/Modifier
   
   ```
   backend/src/application/use_cases/meeting_use_cases.rs
   backend/src/application/dto/meeting_dto.rs
   backend/src/infrastructure/web/handlers/meeting_handlers.rs
   backend/tests/e2e_meetings.rs
   backend/tests/features/meetings.feature (BDD)
   ```
   
   ## ✅ Critères d'Acceptation
   
   - Toutes les routes retournent les codes HTTP appropriés (200, 201, 404, 403)
   - Tests E2E couvrent tous les scénarios (création, modification, suppression, publication, clôture)
   - Tests BDD Gherkin pour scénarios métier (AG ordinaire, extraordinaire, vote)
   - Validation des permissions (seul Syndic+ peut créer/modifier)
   - Audit logs pour toutes les opérations critiques
   
   ---
   
   **Voir**: `issues/critical/001-meeting-management-api.md` pour détails complets

.. raw:: html

   </div>

