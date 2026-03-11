=====================================
Matrice des Permissions par Rôle
=====================================

Ce document décrit en détail les permissions accordées à chaque rôle dans l'application KoproGo.

Vue d'ensemble des rôles
=========================

KoproGo implémente un système de contrôle d'accès basé sur les rôles (RBAC) avec 4 rôles principaux:

1. **SuperAdmin** - Administrateur de la plateforme SaaS
2. **Syndic** - Gestionnaire de copropriété
3. **Accountant** (Comptable) - Gestionnaire comptable
4. **Owner** (Copropriétaire) - Propriétaire d'un lot

Règles métier fondamentales
============================

Données structurelles (Immuables après création)
-------------------------------------------------

Les **buildings** et **units** (avec leurs quotités) constituent la structure de base d'une copropriété. Ces données ne changent pratiquement jamais après l'encodage initial. Pour éviter les erreurs:

- ✅ Seul le **SuperAdmin** peut créer/modifier/supprimer buildings et units
- ❌ Le **Syndic** ne peut PAS modifier ces données structurelles
- 💡 **Raison**: Prévenir les erreurs sur des données critiques qui impactent tous les calculs de charges

Principe de transparence
-------------------------

Tous les rôles peuvent **lire** toutes les données de leur organisation pour garantir la transparence de la gestion.

Séparation des responsabilités
-------------------------------

- **SuperAdmin**: Configuration initiale et gestion multi-tenant
- **Syndic**: Gestion quotidienne de la copropriété
- **Accountant**: Saisie comptable et gestion des paiements
- **Owner**: Consultation uniquement

Matrice détaillée des permissions
==================================

Buildings (Immeubles)
---------------------

+-------------------+------------+--------+------------+-------+
| Action            | SuperAdmin | Syndic | Accountant | Owner |
+===================+============+========+============+=======+
| Créer             | ✅         | ❌     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+
| Lire              | ✅         | ✅     | ✅         | ✅    |
+-------------------+------------+--------+------------+-------+
| Modifier          | ✅         | ❌     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+
| Supprimer         | ✅         | ❌     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+

Units (Lots)
------------

+-------------------+------------+--------+------------+-------+
| Action            | SuperAdmin | Syndic | Accountant | Owner |
+===================+============+========+============+=======+
| Créer             | ✅         | ❌     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+
| Lire              | ✅         | ✅     | ✅         | ✅    |
+-------------------+------------+--------+------------+-------+
| Modifier          | ✅         | ❌     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+
| Modifier quotités | ✅         | ❌     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+
| Supprimer         | ✅         | ❌     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+

Owners (Copropriétaires)
------------------------

+-------------------+------------+--------+------------+-------+
| Action            | SuperAdmin | Syndic | Accountant | Owner |
+===================+============+========+============+=======+
| Créer             | ✅         | ✅     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+
| Lire              | ✅         | ✅     | ✅         | ✅    |
+-------------------+------------+--------+------------+-------+
| Modifier          | ✅         | ✅     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+
| Supprimer         | ✅         | ✅     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+
| Lier à un User    | ✅         | ❌     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+

Unit-Owner Relations (Qui possède quoi)
----------------------------------------

+------------------------+------------+--------+------------+-------+
| Action                 | SuperAdmin | Syndic | Accountant | Owner |
+========================+============+========+============+=======+
| Ajouter propriétaire   | ✅         | ✅     | ❌         | ❌    |
+------------------------+------------+--------+------------+-------+
| Retirer propriétaire   | ✅         | ✅     | ❌         | ❌    |
+------------------------+------------+--------+------------+-------+
| Modifier quote-part    | ✅         | ✅     | ❌         | ❌    |
+------------------------+------------+--------+------------+-------+
| Transférer propriété   | ✅         | ✅     | ❌         | ❌    |
+------------------------+------------+--------+------------+-------+
| Voir relations         | ✅         | ✅     | ✅         | ✅    |
+------------------------+------------+--------+------------+-------+
| Voir historique        | ✅         | ✅     | ✅         | ✅    |
+------------------------+------------+--------+------------+-------+

Expenses (Charges)
------------------

+-------------------+------------+--------+------------+-------+
| Action            | SuperAdmin | Syndic | Accountant | Owner |
+===================+============+========+============+=======+
| Créer             | ✅         | ✅     | ✅         | ❌    |
+-------------------+------------+--------+------------+-------+
| Lire              | ✅         | ✅     | ✅         | ✅    |
+-------------------+------------+--------+------------+-------+
| Modifier          | ✅         | ✅     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+
| Marquer payé      | ✅         | ✅     | ✅         | ❌    |
+-------------------+------------+--------+------------+-------+
| Annuler           | ✅         | ✅     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+
| Réactiver         | ✅         | ✅     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+
| Supprimer         | ✅         | ✅     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+

Meetings (Assemblées)
---------------------

+-------------------+------------+--------+------------+-------+
| Action            | SuperAdmin | Syndic | Accountant | Owner |
+===================+============+========+============+=======+
| Créer             | ✅         | ✅     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+
| Lire              | ✅         | ✅     | ✅         | ✅    |
+-------------------+------------+--------+------------+-------+
| Modifier          | ✅         | ✅     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+
| Compléter         | ✅         | ✅     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+
| Annuler           | ✅         | ✅     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+

Documents
---------

+--------------------+------------+--------+------------+-------+
| Action             | SuperAdmin | Syndic | Accountant | Owner |
+====================+============+========+============+=======+
| Upload             | ✅         | ✅     | ✅         | ❌    |
+--------------------+------------+--------+------------+-------+
| Lire/Télécharger   | ✅         | ✅     | ✅         | ✅    |
+--------------------+------------+--------+------------+-------+
| Supprimer          | ✅         | ✅     | ❌         | ❌    |
+--------------------+------------+--------+------------+-------+

Organizations
-------------

+-------------------+------------+--------+------------+-------+
| Action            | SuperAdmin | Syndic | Accountant | Owner |
+===================+============+========+============+=======+
| Créer             | ✅         | ❌     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+
| Lire              | ✅         | ✅\*   | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+
| Modifier          | ✅         | ❌     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+
| Supprimer         | ✅         | ❌     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+

\* Syndic peut voir uniquement les données de sa propre organisation

Users
-----

+-------------------+------------+--------+------------+-------+
| Action            | SuperAdmin | Syndic | Accountant | Owner |
+===================+============+========+============+=======+
| Créer             | ✅         | ✅\*\* | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+
| Lire              | ✅         | ✅\*   | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+
| Modifier          | ✅         | ✅\*\* | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+
| Supprimer         | ✅         | ❌     | ❌         | ❌    |
+-------------------+------------+--------+------------+-------+

\*\* Syndic peut créer/modifier des users uniquement dans sa propre organisation

Détails par rôle
================

SuperAdmin
----------

**Responsabilité**: Administration de la plateforme multi-tenant

**Permissions**:

- ✅ Accès complet à toutes les organisations
- ✅ Création et configuration initiale des buildings et units
- ✅ Spécification libre de l'organization_id et building_id lors de la création
- ✅ Gestion des organizations
- ✅ Liaison des comptes User aux entités Owner
- ✅ Peut opérer au niveau de n'importe quelle organisation

**Cas d'usage typiques**:

1. Créer une nouvelle organisation (syndic)
2. Configurer la structure initiale d'un immeuble (lots, quotités)
3. Lier un compte utilisateur owner à l'entité propriétaire correspondante
4. Support et dépannage multi-tenant

**Endpoints spécifiques**:

- ``PUT /owners/{id}/link-user`` - Lier un user à un owner

Syndic
------

**Responsabilité**: Gestion quotidienne d'une copropriété

**Permissions**:

- ✅ Gestion des owners (créer, modifier, supprimer)
- ✅ Gestion des attributions (qui possède quel lot)
- ✅ Gestion des expenses (créer, marquer payé, annuler)
- ✅ Gestion des meetings
- ✅ Upload et gestion des documents
- ✅ Lecture de toutes les données de son organisation
- ❌ Ne peut PAS modifier buildings ni units (données structurelles)

**Cas d'usage typiques**:

1. Ajouter un nouveau copropriétaire
2. Attribuer un lot à un propriétaire lors d'une vente
3. Créer et gérer les charges
4. Organiser les assemblées générales
5. Uploader les procès-verbaux et documents officiels

**Restrictions importantes**:

- Scope limité à sa propre organization_id
- Ne peut pas modifier la structure (buildings/units) pour éviter les erreurs
- Ne peut pas lier des users aux owners (réservé au SuperAdmin)

Accountant (Comptable)
----------------------

**Responsabilité**: Saisie comptable et gestion des paiements

**Permissions**:

- ✅ Créer des expenses
- ✅ Marquer les expenses comme payés
- ✅ Upload de documents (factures, justificatifs)
- ✅ Lecture complète de toutes les données (transparence)
- ❌ Ne peut PAS modifier buildings, units, owners
- ❌ Ne peut PAS gérer les attributions de propriété
- ❌ Ne peut PAS annuler ou modifier des expenses

**Cas d'usage typiques**:

1. Encoder les factures reçues
2. Pointer les paiements effectués
3. Uploader les justificatifs comptables
4. Consulter les données pour préparer les comptes

**Restrictions importantes**:

- Rôle strictement limité à la comptabilité
- Pas d'accès aux données structurelles (lecture seule)
- Pas de gestion des propriétaires

Owner (Copropriétaire)
----------------------

**Responsabilité**: Consultation de ses données

**Permissions**:

- ✅ Lecture complète de toutes les données de son organisation (transparence)
- ❌ Aucune modification (lecture seule complète)

**Cas d'usage typiques**:

1. Consulter ses lots
2. Voir les charges et leur état de paiement
3. Télécharger les procès-verbaux
4. Consulter les autres copropriétaires (transparence)

**Restrictions importantes**:

- Aucune action de modification possible
- Rôle strictement consultatif

Implémentation technique
========================

Vérification des permissions
-----------------------------

Les permissions sont vérifiées au niveau des **handlers HTTP** via le middleware ``AuthenticatedUser`` qui extrait le rôle du JWT.

**Exemple de vérification** (building_handlers.rs)::

    #[post("/buildings")]
    pub async fn create_building(
        state: web::Data<AppState>,
        user: AuthenticatedUser,
        dto: web::Json<CreateBuildingDto>,
    ) -> impl Responder {
        // Only SuperAdmin can create buildings (structural data)
        if user.role != "superadmin" {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Only SuperAdmin can create buildings"
            }));
        }
        // ... rest of the handler
    }

Fonctions helper
----------------

Pour éviter la duplication, des fonctions helper sont utilisées::

    // expense_handlers.rs
    fn check_owner_readonly(user: &AuthenticatedUser) -> Option<HttpResponse> {
        if user.role == "owner" {
            Some(HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Owner role has read-only access"
            })))
        } else {
            None
        }
    }

    // unit_owner_handlers.rs
    fn check_unit_ownership_permission(user: &AuthenticatedUser) -> Option<HttpResponse> {
        if user.role == "owner" || user.role == "accountant" {
            Some(HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Only SuperAdmin and Syndic can modify unit ownership"
            })))
        } else {
            None
        }
    }

Organisation Scoping
--------------------

Pour les rôles non-SuperAdmin, les requêtes sont automatiquement scopées à leur ``organization_id`` via le JWT::

    let organization_id = user.require_organization()?;
    // Filters all queries by this organization_id

Messages d'erreur
=================

Lorsqu'un utilisateur tente une action non autorisée, il reçoit:

- **Code HTTP**: ``403 Forbidden``
- **Message**: Description claire de la restriction

**Exemples de messages**:

+------------+------------------+--------------------------------------------------------------------+
| Rôle       | Action           | Message                                                            |
+============+==================+====================================================================+
| Syndic     | Créer building   | "Only SuperAdmin can create buildings (structural data)"           |
+------------+------------------+--------------------------------------------------------------------+
| Accountant | Modifier unit    | "Only SuperAdmin can update units (structural data)"               |
+------------+------------------+--------------------------------------------------------------------+
| Owner      | Créer expense    | "Owner role has read-only access"                                  |
+------------+------------------+--------------------------------------------------------------------+
| Syndic     | Lier user-owner  | "Only SuperAdmin can link users to owners"                         |
+------------+------------------+--------------------------------------------------------------------+

Évolution et maintenance
========================

Ajouter un nouveau endpoint
----------------------------

1. Déterminer quel(s) rôle(s) peut/peuvent y accéder
2. Ajouter la vérification de permission en début de handler
3. Mettre à jour cette documentation
4. Ajouter des tests pour vérifier les restrictions

Modifier les permissions existantes
------------------------------------

1. Discuter et valider le changement métier
2. Modifier les handlers concernés
3. Mettre à jour cette documentation
4. Vérifier les tests existants
5. Communiquer le changement aux utilisateurs

Références
==========

**Code source**:

- Building handlers: ``backend/src/infrastructure/web/handlers/building_handlers.rs``
- Unit handlers: ``backend/src/infrastructure/web/handlers/unit_handlers.rs``
- Owner handlers: ``backend/src/infrastructure/web/handlers/owner_handlers.rs``
- Unit-Owner handlers: ``backend/src/infrastructure/web/handlers/unit_owner_handlers.rs``
- Expense handlers: ``backend/src/infrastructure/web/handlers/expense_handlers.rs``

**Middleware**: ``backend/src/infrastructure/web/mod.rs`` (AuthenticatedUser)

**Tests**:

- ``backend/tests/e2e_auth.rs`` - Tests E2E d'authentification et permissions
- ``backend/tests/features/auth.feature`` - Tests BDD multi-rôles

Notes de sécurité
=================

1. **Jamais de confiance côté client**: Toutes les permissions sont vérifiées côté serveur
2. **JWT sécurisé**: Le rôle est extrait du JWT signé et ne peut être falsifié
3. **Organization scoping**: Les utilisateurs ne peuvent accéder qu'aux données de leur organisation
4. **Audit logging**: Toutes les actions de modification sont loggées pour audit
5. **Principe du moindre privilège**: Chaque rôle a le minimum de permissions nécessaires

----

**Dernière mise à jour**: 31 octobre 2025

**Version**: 1.0
