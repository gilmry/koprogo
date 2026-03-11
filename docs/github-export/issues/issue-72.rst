======================================================================================
Issue #72: Étudier l'opportunité d'une matrice des droits dynamique (RBAC granulaire)
======================================================================================

:State: **OPEN**
:Milestone: Jalon 4: Automation & Intégrations 📅
:Labels: enhancement
:Assignees: Unassigned
:Created: 2025-10-31
:Updated: 2025-11-13
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/72>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## 🎯 Objectif
   
   Étudier la faisabilité et l'opportunité de remplacer le système RBAC actuel basé sur des rôles fixes par une matrice de permissions dynamique et granulaire.
   
   ## 📋 Contexte actuel
   
   ### Système RBAC actuel (Role-Based Access Control)
   
   KoproGo utilise actuellement **4 rôles prédéfinis** avec des permissions fixes :
   
   | Rôle | Permissions | Limitations |
   |------|-------------|-------------|
   | **SuperAdmin** | Accès complet multi-tenant | Aucune limitation |
   | **Syndic** | Gestion copropriété (owners, expenses, meetings) | Pas de modif structural data |
   | **Accountant** | Saisie comptable, pointer paiements | Lecture seule sur reste |
   | **Owner** | Lecture seule complète | Aucune modification |
   
   **Documentation** : `docs/ROLE_PERMISSIONS_MATRIX.rst`
   
   ### Limites du système actuel
   
   1. **Rigidité** : Impossible de créer des rôles personnalisés ou d'ajuster finement les permissions
   2. **Scalabilité** : Ajout de nouveaux rôles = modification du code + migration base de données
   3. **Besoins métier évolutifs** : Demandes pour Organization Admin, Building Manager (issue #71)
   4. **Cas d'usage complexes** : 
      - Déléguer uniquement la gestion documentaire à un utilisateur
      - Donner accès lecture/écriture aux expenses mais pas aux meetings
      - Restreindre un syndic à certains immeubles uniquement
   
   ## 🆕 Proposition : Matrice des droits dynamique
   
   ### Concept
   
   Remplacer le système de rôles fixes par un système de **permissions granulaires** assignables dynamiquement.
   
   ### Architecture proposée
   
   #### 1. Table `permissions` (catalogue des permissions disponibles)
   
   ```sql
   CREATE TABLE permissions (
       id UUID PRIMARY KEY,
       resource VARCHAR(50) NOT NULL,      -- Ex: "buildings", "units", "owners"
       action VARCHAR(20) NOT NULL,        -- Ex: "create", "read", "update", "delete"
       scope VARCHAR(20) NOT NULL,         -- Ex: "all", "organization", "assigned"
       description TEXT,
       created_at TIMESTAMP DEFAULT NOW(),
       UNIQUE(resource, action, scope)
   );
   
   -- Exemples d'entrées :
   -- ('buildings', 'create', 'all', 'Créer des immeubles dans toute la plateforme')
   -- ('buildings', 'read', 'organization', 'Lire les immeubles de son organisation')
   -- ('buildings', 'update', 'assigned', 'Modifier uniquement les immeubles assignés')
   -- ('expenses', 'create', 'organization', 'Créer des charges dans son organisation')
   -- ('expenses', 'mark_paid', 'organization', 'Marquer des charges comme payées')
   ```
   
   #### 2. Table `roles` (définition dynamique des rôles)
   
   ```sql
   CREATE TABLE roles (
       id UUID PRIMARY KEY,
       name VARCHAR(50) NOT NULL UNIQUE,   -- Ex: "syndic", "accountant", "building_manager"
       display_name VARCHAR(100) NOT NULL, -- Ex: "Gestionnaire de Copropriété"
       description TEXT,
       is_system BOOLEAN DEFAULT FALSE,    -- TRUE = rôle système (non supprimable)
       organization_id UUID,               -- NULL = rôle global, UUID = rôle spécifique org
       created_at TIMESTAMP DEFAULT NOW(),
       updated_at TIMESTAMP DEFAULT NOW()
   );
   ```
   
   #### 3. Table `role_permissions` (assignation permissions → rôles)
   
   ```sql
   CREATE TABLE role_permissions (
       id UUID PRIMARY KEY,
       role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
       permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
       granted_at TIMESTAMP DEFAULT NOW(),
       granted_by UUID REFERENCES users(id),
       UNIQUE(role_id, permission_id)
   );
   ```
   
   #### 4. Table `user_assignments` (utilisateurs + contraintes de scope)
   
   ```sql
   CREATE TABLE user_assignments (
       id UUID PRIMARY KEY,
       user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
       role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
       organization_id UUID REFERENCES organizations(id),  -- Scope organisation
       constraints JSONB,                                   -- Contraintes additionnelles
       -- Ex: {"building_ids": ["uuid1", "uuid2"], "unit_ids": [...]}
       is_active BOOLEAN DEFAULT TRUE,
       assigned_at TIMESTAMP DEFAULT NOW(),
       assigned_by UUID REFERENCES users(id),
       UNIQUE(user_id, role_id, organization_id)
   );
   ```
   
   ### Exemples de configurations
   
   #### Exemple 1 : Building Manager (gestionnaire de portefeuille)
   
   ```json
   {
     "role": "building_manager",
     "permissions": [
       {"resource": "buildings", "action": "read", "scope": "assigned"},
       {"resource": "buildings", "action": "update", "scope": "assigned"},
       {"resource": "owners", "action": "create", "scope": "assigned"},
       {"resource": "owners", "action": "update", "scope": "assigned"},
       {"resource": "expenses", "action": "create", "scope": "assigned"},
       {"resource": "meetings", "action": "create", "scope": "assigned"}
     ],
     "constraints": {
       "building_ids": ["building-uuid-1", "building-uuid-2", "building-uuid-3"]
     }
   }
   ```
   
   #### Exemple 2 : Document Manager (gestionnaire documents uniquement)
   
   ```json
   {
     "role": "document_manager",
     "permissions": [
       {"resource": "buildings", "action": "read", "scope": "organization"},
       {"resource": "documents", "action": "create", "scope": "organization"},
       {"resource": "documents", "action": "delete", "scope": "organization"}
     ]
   }
   ```
   
   ## 🔍 Avantages et Inconvénients
   
   ### ✅ Avantages
   
   1. **Flexibilité maximale** : Création de rôles personnalisés sans modification du code
   2. **Granularité** : Permissions ajustables au niveau action + ressource + scope
   3. **Scalabilité** : Nouveaux besoins = ajout de permissions en base, pas de déploiement
   4. **Multi-tenant amélioré** : Chaque organisation peut définir ses propres rôles
   5. **Audit trail** : Traçabilité de qui a donné quelles permissions à qui
   6. **Interface admin** : UI pour configurer les rôles sans toucher au code
   
   ### ❌ Inconvénients
   
   1. **Complexité accrue** :
      - Logique de vérification des permissions plus complexe
      - Plus de requêtes SQL (JOINs multiples)
      - Risque de bugs dans la vérification des permissions
   2. **Performance** :
      - Latence accrue : chaque requête doit vérifier permissions dynamiquement
      - Cache nécessaire pour maintenir P99 < 5ms
   3. **Migration** :
      - Migration des rôles actuels vers le nouveau système
      - Rétrocompatibilité pendant la transition
   4. **UX** :
      - Interface de configuration complexe
      - Risque de confusion pour les utilisateurs
   5. **Tests** :
      - Explosion du nombre de cas de tests
      - Tests de non-régression critiques
   
   ## 🏗️ Implémentation envisagée
   
   ### Phase 1 : Étude et validation (2-3 jours)
   
   - [ ] Valider le besoin métier avec utilisateurs pilotes
   - [ ] Benchmark performance : comparer latence système actuel vs dynamique
   - [ ] Définir périmètre des permissions (liste exhaustive)
   - [ ] Concevoir l'interface admin de configuration des rôles
   
   ### Phase 2 : Backend core (1 semaine)
   
   - [ ] Migrations PostgreSQL (4 nouvelles tables)
   - [ ] Domain entities : `Permission`, `Role`, `RolePermission`, `UserAssignment`
   - [ ] Repositories (ports + impls)
   - [ ] Use cases : 
     - `create_role`, `update_role_permissions`
     - `assign_role_to_user`, `check_user_permission`
   - [ ] Middleware : remplacer `AuthenticatedUser.role` par `AuthenticatedUser.check_permission()`
   - [ ] Cache : Redis/DragonflyDB pour cache des permissions par user
   
   ### Phase 3 : Migration et handlers (1 semaine)
   
   - [ ] Script de migration : mapper rôles actuels → nouvelles permissions
   - [ ] Adapter tous les handlers pour utiliser `check_permission()`
   - [ ] Rétrocompatibilité : garder l'ancien système en parallèle (feature flag)
   
   ### Phase 4 : Frontend admin UI (1 semaine)
   
   - [ ] Page "Gestion des Rôles" : CRUD rôles personnalisés
   - [ ] Interface glisser-déposer : assigner permissions aux rôles
   - [ ] Page "Assignation Utilisateurs" : assigner rôles + contraintes
   - [ ] Prévisualisation : "Ce rôle peut faire X, Y mais pas Z"
   
   ### Phase 5 : Tests et documentation (3 jours)
   
   - [ ] Tests unitaires : validation des permissions
   - [ ] Tests d'intégration : vérification end-to-end
   - [ ] Tests de performance : vérifier P99 < 5ms maintenu
   - [ ] Documentation complète : guide admin + guide développeur
   
   ## 📊 Alternatives
   
   ### Alternative 1 : Système hybride (rôles fixes + permissions additionnelles)
   
   - Garder les 4 rôles actuels comme base
   - Ajouter une colonne JSONB `extra_permissions` sur `user_roles`
   - Permissions additionnelles = extension du rôle de base
   
   **Avantages** : Migration progressive, simplicité relative
   **Inconvénients** : Moins flexible, pas de rôles complètement custom
   
   ### Alternative 2 : Étendre le système actuel avec plus de rôles fixes
   
   - Ajouter Organization Admin, Building Manager, Document Manager, etc.
   - Rester sur le système de rôles fixes mais en augmenter le nombre
   
   **Avantages** : Simplicité, performance maintenue
   **Inconvénients** : Rigidité, scalabilité limitée
   
   ### Alternative 3 : Utiliser une librairie existante (Casbin, Oso)
   
   - Intégrer une solution RBAC/ABAC tierce (ex: Casbin pour Rust)
   - Déléguer la logique de permissions à une librairie éprouvée
   
   **Avantages** : Robustesse, gain de temps
   **Inconvénients** : Dépendance externe, courbe d'apprentissage
   
   ## ✅ Critères de décision
   
   - [ ] **Besoin métier validé** : Les utilisateurs ont-ils réellement besoin de cette flexibilité ?
   - [ ] **Performance acceptable** : P99 < 5ms maintenu même avec système dynamique ?
   - [ ] **Complexité justifiée** : Le gain de flexibilité vaut-il l'effort d'implémentation ?
   - [ ] **Maintenabilité** : L'équipe peut-elle maintenir ce système à long terme ?
   - [ ] **ROI** : Valeur ajoutée vs coût de développement (estimation 3-4 semaines)
   
   ## 📈 Métriques de succès
   
   Si implémentation validée :
   
   - **Performance** : P99 latency < 5ms maintenu
   - **Flexibilité** : 5+ rôles personnalisés créés par utilisateurs pilotes
   - **Adoption** : 80%+ des organisations utilisent au moins 1 rôle custom
   - **Stabilité** : 0 bug critique lié aux permissions sur 3 mois post-release
   
   ## 📚 Références
   
   - Documentation actuelle : `docs/ROLE_PERMISSIONS_MATRIX.rst`
   - Issue connexe : #71 (Organization Admin et Building Manager)
   - Multi-role support : `docs/MULTI_ROLE_SUPPORT.md`
   - Casbin (Rust) : https://github.com/casbin/casbin-rs
   - Oso (Authorization library) : https://www.osohq.com/
   
   ## 📅 Priorité
   
   **Faible à Moyen** : 
   - Pas bloquant pour MVP
   - Valeur ajoutée importante si besoin métier confirmé
   - Effort conséquent (3-4 semaines)
   - Décision à prendre après validation utilisateurs
   
   ---
   
   **Next Steps** :
   1. Validation métier : interviews utilisateurs pilotes (syndics gérant >10 immeubles)
   2. Benchmark performance : POC avec système dynamique
   3. Décision go/no-go basée sur critères ci-dessus
   4. Si GO → planifier sprint dédié (4 semaines)

.. raw:: html

   </div>

