===========================================================
Issue #28: feat: Support multi-rôles pour les utilisateurs
===========================================================

:State: **CLOSED**
:Milestone: Jalon 0: Fondations Techniques ✅
:Labels: enhancement,phase:vps track:software,priority:high
:Assignees: Unassigned
:Created: 2025-10-27
:Updated: 2025-11-13
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/28>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   Dans le contexte associatif belge, une même personne peut avoir plusieurs rôles dans une copropriété. Par exemple :
   - Un propriétaire qui est aussi syndic bénévole
   - Un propriétaire qui est aussi comptable
   - Un syndic qui gère plusieurs organisations
   
   ## Problème actuel
   - La table `users` a un champ `role` unique (VARCHAR)
   - Un utilisateur ne peut avoir qu'un seul rôle à la fois
   - Impossible de modéliser les cas multi-rôles courants en associatif
   
   ## Solutions possibles
   
   ### Option 1: Table de jonction (recommandée)
   - Créer une table `user_roles` avec `user_id`, `role`, `organization_id`
   - Permet plusieurs rôles par organisation
   - Nécessite refactoring de l'authentification et des permissions
   
   ### Option 2: Array de rôles
   - Ajouter `secondary_roles` (TEXT[] ou JSONB) à la table users
   - Garder `role` principal pour compatibilité
   - Plus simple mais moins flexible
   
   ### Option 3: Email aliases
   - Créer des utilisateurs séparés avec +suffixe (pierre+syndic@email.be)
   - Solution temporaire, pas idéale UX
   
   ## Tâches
   - [ ] Choisir l'approche (probablement Option 1)
   - [ ] Créer migration pour table `user_roles`
   - [ ] Refactorer `AuthenticatedUser` middleware
   - [ ] Adapter les handlers de permissions
   - [ ] Ajouter sélecteur de rôle dans l'UI (si multi-rôles dans même org)
   - [ ] Mettre à jour le seed avec cas multi-rôles
   - [ ] Tests d'intégration
   
   ## Labels
   enhancement, authentication, database

.. raw:: html

   </div>

