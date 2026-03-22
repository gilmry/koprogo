==============================================================================
Issue #71: Étudier l'ajout des rôles Organization Admin et Building Manager
==============================================================================

:State: **CLOSED**
:Milestone: Jalon 4: Automation & Intégrations 📅
:Labels: enhancement,release:v0.7.0
:Assignees: Unassigned
:Created: 2025-10-31
:Updated: 2026-03-10
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/71>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## 🎯 Objectif
   
   Étudier la faisabilité et concevoir l'implémentation de deux nouveaux rôles intermédiaires pour améliorer la gestion multi-niveaux des organisations.
   
   ## 📋 Contexte
   
   Actuellement, KoproGo dispose de 4 rôles :
   - **SuperAdmin** : Administration plateforme SaaS (multi-tenant)
   - **Syndic** : Gestion quotidienne d'une copropriété
   - **Accountant** : Saisie comptable et paiements
   - **Owner** : Consultation uniquement
   
   ### Limitation identifiée
   
   Dans une organisation de syndic gérant plusieurs immeubles, il n'existe pas de rôles intermédiaires permettant :
   1. Une délégation administrative au niveau organisation (sans accès SuperAdmin)
   2. Une gestion ciblée d'un portefeuille spécifique d'immeubles
   
   ## 🆕 Nouveaux rôles proposés
   
   ### 1. **Organization Admin** (Admin Organisation)
   
   **Responsabilité** : Administration déléguée d'une organisation de syndic
   
   **Permissions envisagées** :
   - ✅ Gestion complète des utilisateurs de son organisation (créer, modifier, supprimer)
   - ✅ Gestion des paramètres de l'organisation
   - ✅ Accès à tous les immeubles de son organisation
   - ✅ Gestion des owners et unit-owners
   - ✅ Toutes les permissions du Syndic
   - ❌ Pas d'accès aux autres organisations (scope limité)
   - ❌ Ne peut pas modifier buildings/units (structural data = SuperAdmin only)
   
   **Cas d'usage** :
   - Directeur d'un syndic gérant plusieurs gestionnaires
   - Responsable administratif d'une organisation
   - Délégation de la gestion utilisateurs sans donner accès SuperAdmin
   
   ### 2. **Building Manager** (Gestionnaire d'Immeubles)
   
   **Responsabilité** : Gestion d'un portefeuille spécifique d'immeubles
   
   **Permissions envisagées** :
   - ✅ Gestion complète des immeubles assignés à son portefeuille
   - ✅ Gestion des owners/unit-owners dans ses immeubles
   - ✅ Gestion des expenses, meetings, documents pour ses immeubles
   - ✅ Lecture seule des autres immeubles de l'organisation (transparence)
   - ❌ Pas d'accès aux immeubles hors portefeuille (modification)
   - ❌ Ne peut pas créer/supprimer buildings/units (structural data)
   - ❌ Ne peut pas gérer les utilisateurs
   
   **Cas d'usage** :
   - Gestionnaire terrain responsable de 5 immeubles sur 20
   - Séparation des responsabilités par portefeuille géographique
   - Délégation opérationnelle sans accès complet organisation
   
   ## 🔍 Points à étudier
   
   ### 1. Modèle de données
   
   - [ ] Ajouter les rôles `organization_admin` et `building_manager` dans l'enum `user_role`
   - [ ] Créer table `building_manager_assignments` (many-to-many: user ↔ buildings)
   - [ ] Vérifier impact sur `user_roles` table (multi-rôles)
   
   ### 2. Logique métier
   
   - [ ] Définir la matrice de permissions précise pour chaque rôle
   - [ ] Hiérarchie des rôles : SuperAdmin > Organization Admin > Building Manager > Syndic
   - [ ] Règles de délégation : qui peut assigner ces rôles ?
   
   ### 3. Implémentation backend
   
   - [ ] Migration PostgreSQL (nouvelle table + enum update)
   - [ ] Domain entity : `BuildingManagerAssignment`
   - [ ] Repository + Use cases
   - [ ] Middleware : étendre `AuthenticatedUser` pour vérifier scope buildings
   - [ ] Handlers : adapter tous les handlers pour vérifier permissions granulaires
   
   ### 4. Implémentation frontend
   
   - [ ] UI pour Organization Admin : gestion utilisateurs, assignation portefeuilles
   - [ ] UI pour Building Manager : vue filtrée sur ses immeubles seulement
   - [ ] Sélecteur de rôle : intégrer les nouveaux rôles dans `Navigation.svelte`
   
   ### 5. Tests & Documentation
   
   - [ ] Tests unitaires, intégration, BDD, E2E
   - [ ] Mettre à jour `docs/ROLE_PERMISSIONS_MATRIX.rst`
   
   ## 📚 Références
   
   - Documentation : `docs/ROLE_PERMISSIONS_MATRIX.rst`
   - Multi-role : `docs/MULTI_ROLE_SUPPORT.md`

.. raw:: html

   </div>

