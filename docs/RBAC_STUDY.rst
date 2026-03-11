=======================================================
RBAC Granulaire : Etude d'Opportunite (Issues #72, #71)
=======================================================

.. contents:: Table des matieres
   :depth: 3

Resume Executif
===============

Cette etude analyse l'opportunite d'evoluer le systeme RBAC (Role-Based Access
Control) de KoproGo, actuellement base sur 4 roles fixes, vers un systeme plus
flexible integrant quatre nouveaux roles :

- **Organization Admin** : delegation administrative au niveau organisation
- **Building Manager** : gestion d'un portefeuille d'immeubles specifiques
- **Tenant (Locataire)** : acces aux charges de son lot + modules communautaires
- **Guest (Invite)** : participation communautaire limitee pour externes

**Recommandation** : Adopter l'**Alternative 1 (systeme hybride)** - etendre le
systeme actuel avec les 4 nouveaux roles fixes tout en preparant l'infrastructure
pour une granularite future via un champ ``extra_permissions JSONB``.

Systeme Actuel
==============

Architecture RBAC
-----------------

KoproGo utilise 4 roles predefinies avec permissions fixes :

+------------+-------------------------------+-------------------------------+
| Role       | Responsabilite                | Scope                         |
+============+===============================+===============================+
| SuperAdmin | Administration plateforme     | Multi-tenant, acces complet   |
+------------+-------------------------------+-------------------------------+
| Syndic     | Gestion copropriete           | Organisation propre           |
+------------+-------------------------------+-------------------------------+
| Accountant | Comptabilite, paiements       | Organisation propre           |
+------------+-------------------------------+-------------------------------+
| Owner      | Consultation uniquement       | Organisation propre           |
+------------+-------------------------------+-------------------------------+

Implementation technique :

- Enum ``UserRole`` dans le JWT (claim ``role``)
- Verification dans chaque handler HTTP via ``AuthenticatedUser``
- Scoping automatique par ``organization_id`` pour roles non-SuperAdmin
- Documentation : ``docs/ROLE_PERMISSIONS_MATRIX.rst``

Limites Identifiees
-------------------

1. **Pas de delegation administrative** : Un syndic gerant 20+ immeubles ne
   peut pas deleguer la gestion utilisateurs sans donner acces SuperAdmin.

2. **Pas de portefeuille** : Impossible d'assigner un gestionnaire a un
   sous-ensemble d'immeubles specifiques.

3. **Rigidite** : Ajout de roles = modification code + migration + deploiement.

4. **Pas de role locataire** : Les locataires n'ont aucun acces a la plateforme
   alors qu'ils sont concernes par les charges de leur lot et pourraient
   enrichir l'ecosysteme communautaire (SEL, competences, prets).

5. **Pas d'invite externe** : Impossible pour une ACP d'inviter des personnes
   exterieures (artisans, voisins) a participer au SEL ou aux modules
   communautaires, limitant la richesse des echanges.

6. **Cas non couverts** :

   - Deleguer uniquement la gestion documentaire
   - Donner acces ecriture aux expenses mais pas aux meetings
   - Restreindre un syndic a certains immeubles
   - Locataire : voir les charges de son lot uniquement (pas ceux du bailleur)
   - Invite : participer au SEL sans etre coproprietaire

Nouveaux Roles Proposes (Issue #71)
===================================

Organization Admin
------------------

**Cas d'usage** : Directeur d'un cabinet de syndic gerant plusieurs gestionnaires.

+----------------------------------+-------------------------------------------+
| Permission                       | Details                                   |
+==================================+===========================================+
| Gestion utilisateurs             | CRUD dans son organisation                |
+----------------------------------+-------------------------------------------+
| Parametres organisation          | Modifier nom, contact, abonnement         |
+----------------------------------+-------------------------------------------+
| Toutes permissions Syndic        | Owners, expenses, meetings, documents     |
+----------------------------------+-------------------------------------------+
| Acces tous immeubles org         | Lecture + ecriture sur tous les buildings |
+----------------------------------+-------------------------------------------+
| Donnees structurelles            | Lecture seule (buildings, units)          |
+----------------------------------+-------------------------------------------+

**Restrictions** :

- Scope limite a sa propre organisation
- Ne peut pas creer/modifier buildings/units (SuperAdmin only)
- Ne peut pas lier users a owners (SuperAdmin only)

Building Manager
----------------

**Cas d'usage** : Gestionnaire terrain responsable de 5 immeubles sur 20.

+----------------------------------+-------------------------------------------+
| Permission                       | Details                                   |
+==================================+===========================================+
| Immeubles assignes               | CRUD complet (owners, expenses, meetings) |
+----------------------------------+-------------------------------------------+
| Autres immeubles org             | Lecture seule (transparence)              |
+----------------------------------+-------------------------------------------+
| Documents                        | Upload/download pour ses immeubles        |
+----------------------------------+-------------------------------------------+
| Gestion utilisateurs             | Non autorisee                             |
+----------------------------------+-------------------------------------------+
| Donnees structurelles            | Lecture seule                             |
+----------------------------------+-------------------------------------------+

**Restrictions** :

- Ne peut modifier que les immeubles de son portefeuille
- Portefeuille defini via table ``building_manager_assignments``
- Ne peut pas gerer les utilisateurs

Tenant (Locataire)
------------------

**Cas d'usage** : Locataire d'un lot, lie par un bail avec un Owner. Doit
acceder aux charges de son lot et participer a la vie communautaire sans
avoir de droits de copropriete.

+----------------------------------+-------------------------------------------+
| Permission                       | Details                                   |
+==================================+===========================================+
| Charges du lot (bail)            | Lecture seule, limite au(x) lot(s) du     |
|                                  | bail actif uniquement                     |
+----------------------------------+-------------------------------------------+
| Modules communautaires           | SEL, tableau d'affichage, competences,    |
|                                  | prets d'objets, reservations              |
+----------------------------------+-------------------------------------------+
| Documents                        | Lecture des documents de l'immeuble       |
+----------------------------------+-------------------------------------------+
| Notifications                    | Recevoir notifications liees a son lot    |
+----------------------------------+-------------------------------------------+
| Assemblees generales             | Pas de vote (sauf delegation explicite)   |
+----------------------------------+-------------------------------------------+
| Donnees structurelles            | Lecture seule (immeuble, lot du bail)     |
+----------------------------------+-------------------------------------------+

**Restrictions** :

- Acces limite au(x) lot(s) pour le(s)quel(s) il a un bail actif
- Si le Owner possede 5 appartements, le locataire ne voit que le sien
- Bail avec dates de debut/fin : acces automatiquement revoque a l'expiration
- Pas de modification des donnees structurelles ni financieres
- Pas d'acces aux donnees des autres lots

**Modele de donnees** ::

    CREATE TABLE tenants (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
        unit_id UUID NOT NULL REFERENCES units(id),
        owner_id UUID NOT NULL REFERENCES owners(id),  -- bailleur
        organization_id UUID NOT NULL REFERENCES organizations(id),
        building_id UUID NOT NULL REFERENCES buildings(id),
        lease_start DATE NOT NULL,
        lease_end DATE,                    -- NULL = bail indetermine
        monthly_rent_cents INTEGER,        -- montant loyer mensuel
        can_participate_sel BOOLEAN DEFAULT TRUE,
        can_participate_community BOOLEAN DEFAULT TRUE,
        can_access_charges BOOLEAN DEFAULT TRUE,
        is_active BOOLEAN DEFAULT TRUE,
        created_at TIMESTAMPTZ DEFAULT NOW(),
        UNIQUE(user_id, unit_id, lease_start)
    );

    CREATE INDEX idx_tenants_active ON tenants(building_id)
        WHERE is_active = TRUE;
    CREATE INDEX idx_tenants_user ON tenants(user_id)
        WHERE is_active = TRUE;

**Scope dans le JWT** ::

    {
        "role": "tenant",
        "organization_id": "uuid",
        "tenant_unit_ids": ["unit-uuid-1"],
        "building_id": "uuid"
    }

**Impact sur le SEL** :

Le systeme SEL actuel utilise ``owner_id`` comme identifiant de participant.
Pour integrer les locataires, deux approches :

1. **Approche recommandee** : Ajouter ``tenant_id`` nullable dans
   ``local_exchanges`` et ``owner_credit_balances``, avec contrainte
   CHECK ``(owner_id IS NOT NULL) OR (tenant_id IS NOT NULL)``

2. **Alternative** : Creer une abstraction ``participant_id`` qui peut
   designer un Owner ou un Tenant

Le credit balance des tenants est separe de celui des owners (un locataire
ne cumule pas de credits pour le compte de son bailleur).

Guest (Invite Externe)
----------------------

**Cas d'usage** : Personne exterieure a la copropriete, invitee par l'ACP
(Association des Coproprietaires) pour participer a certains modules
communautaires. Exemples : un artisan local participant au SEL, un voisin
invite a une bourse d'echanges, un prestataire participant aux competences.

+----------------------------------+-------------------------------------------+
| Permission                       | Details                                   |
+==================================+===========================================+
| Modules communautaires           | SEL, competences, prets d'objets          |
|                                  | (configurable par invitation)             |
+----------------------------------+-------------------------------------------+
| Tableau d'affichage              | Lecture seule                             ||
+----------------------------------+-------------------------------------------+
| Reservations                     | Non autorise (espace reserve residents)   |
+----------------------------------+-------------------------------------------+
| Charges / Finances               | Aucun acces                               |
+----------------------------------+-------------------------------------------+
| Documents                        | Aucun acces                               |
+----------------------------------+-------------------------------------------+
| Assemblees generales             | Aucun acces                               |
+----------------------------------+-------------------------------------------+

**Restrictions** :

- Invitation avec date d'expiration obligatoire (defaut : 6 mois)
- Acces limite aux modules specifies dans l'invitation
- Aucun acces aux donnees financieres, documents, ou AG
- Peut etre revoque a tout moment par le Syndic ou Org Admin
- Credits SEL isoles (pas de transfert vers comptes Owner)

**Modele de donnees** ::

    CREATE TABLE guests (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
        building_id UUID NOT NULL REFERENCES buildings(id),
        organization_id UUID NOT NULL REFERENCES organizations(id),
        invited_by UUID NOT NULL REFERENCES users(id),
        invitation_reason TEXT,
        valid_from DATE NOT NULL DEFAULT CURRENT_DATE,
        valid_until DATE NOT NULL,         -- date d'expiration obligatoire
        -- Permissions granulaires
        can_participate_sel BOOLEAN DEFAULT FALSE,
        can_access_notices BOOLEAN DEFAULT TRUE,
        can_access_skills BOOLEAN DEFAULT FALSE,
        can_access_shared_objects BOOLEAN DEFAULT FALSE,
        can_access_bookings BOOLEAN DEFAULT FALSE,
        is_active BOOLEAN DEFAULT TRUE,
        created_at TIMESTAMPTZ DEFAULT NOW(),
        revoked_at TIMESTAMPTZ,
        revoked_by UUID REFERENCES users(id),
        CONSTRAINT valid_dates CHECK (valid_until > valid_from)
    );

    CREATE INDEX idx_guests_active ON guests(building_id)
        WHERE is_active = TRUE AND valid_until >= CURRENT_DATE;

**Scope dans le JWT** ::

    {
        "role": "guest",
        "organization_id": "uuid",
        "building_id": "uuid",
        "guest_permissions": ["sel", "notices", "skills"],
        "guest_valid_until": "2026-09-07"
    }

**Scenarios d'utilisation concrets** :

1. Un plombier du quartier s'inscrit au SEL pour proposer ses services
   aux residents en echange de credits temps
2. Un voisin d'un immeuble adjacent est invite a participer a une bourse
   d'echanges d'objets
3. Un professeur de yoga propose des cours via le module competences
4. Un jardinier communal offre ses services d'entretien des espaces communs

Hierarchie des Roles (Etendue)
-------------------------------

::

    SuperAdmin
        |
    Organization Admin
        |
    +-------+--------+-----------+
    |       |        |           |
    Syndic  Building  Accountant  BoardMember
            Manager
                |
    +-----------+----------+
    |           |          |
    Owner     Tenant     Guest

**Regles de hierarchie** :

- SuperAdmin > Organization Admin > Syndic = Building Manager = Accountant
- Owner > Tenant (le owner a tous les droits, le tenant un sous-ensemble)
- Guest est isole (pas de hierarchie, permissions explicites par invitation)

Matrice des Permissions Etendue (Tous Roles)
=============================================

+-------------------+--------+------+-------+-------+------+-------+--------+-------+
| Fonctionnalite    | Super  | Org  | Synd  | BM    | Comp | Owner | Tenant | Guest |
|                   | Admin  | Adm  |       |       |      |       |        |       |
+===================+========+======+=======+=======+======+=======+========+=======+
| Buildings CRUD    | CRUD   | R    | R     | R     | R    | R     | R*     | -     |
+-------------------+--------+------+-------+-------+------+-------+--------+-------+
| Units CRUD        | CRUD   | R    | R     | R     | R    | R     | R*     | -     |
+-------------------+--------+------+-------+-------+------+-------+--------+-------+
| Owners CRUD       | CRUD   | CRUD | CRUD  | CRUD  | R    | R     | -      | -     |
+-------------------+--------+------+-------+-------+------+-------+--------+-------+
| Expenses          | CRUD   | CRUD | CRUD  | CRUD  | CR+  | R     | R*     | -     |
+-------------------+--------+------+-------+-------+------+-------+--------+-------+
| Meetings/AG       | CRUD   | CRUD | CRUD  | CRUD  | R    | R     | -      | -     |
+-------------------+--------+------+-------+-------+------+-------+--------+-------+
| Documents         | CRUD   | CRUD | CRUD  | CRUD  | CR   | R     | R      | -     |
+-------------------+--------+------+-------+-------+------+-------+--------+-------+
| SEL Echanges      | CRUD   | CRUD | CRUD  | CRUD  | R    | CRUD  | CRUD   | CRUD  |
+-------------------+--------+------+-------+-------+------+-------+--------+-------+
| Notices           | CRUD   | CRUD | CRUD  | CRUD  | R    | CRUD  | CRUD   | R     |
+-------------------+--------+------+-------+-------+------+-------+--------+-------+
| Competences       | CRUD   | CRUD | CRUD  | CRUD  | R    | CRUD  | CRUD   | CRUD  |
+-------------------+--------+------+-------+-------+------+-------+--------+-------+
| Prets d'objets    | CRUD   | CRUD | CRUD  | CRUD  | R    | CRUD  | CRUD   | CRUD  |
+-------------------+--------+------+-------+-------+------+-------+--------+-------+
| Reservations      | CRUD   | CRUD | CRUD  | CRUD  | R    | CRUD  | CRUD   | -     |
+-------------------+--------+------+-------+-------+------+-------+--------+-------+
| Gamification      | CRUD   | CRUD | R     | R     | R    | R     | R      | R     |
+-------------------+--------+------+-------+-------+------+-------+--------+-------+
| Votes/Resolutions | CRUD   | CRUD | CRUD  | CRUD  | R    | Vote  | -      | -     |
+-------------------+--------+------+-------+-------+------+-------+--------+-------+
| Users org         | CRUD   | CRUD | CRU   | R     | R    | R     | -      | -     |
+-------------------+--------+------+-------+-------+------+-------+--------+-------+
| GDPR (self)       | Oui    | Oui  | Oui   | Oui   | Oui  | Oui   | Oui    | Oui   |
+-------------------+--------+------+-------+-------+------+-------+--------+-------+

``*`` Tenant : lecture limitee au(x) lot(s) du bail actif uniquement.
``BM`` = Building Manager, limite aux immeubles assignes.
``CR+`` = Creer + Marquer paye.

Analyse des Alternatives (Issue #72)
=====================================

Alternative 1 : Systeme Hybride (RECOMMANDEE)
----------------------------------------------

Garder les roles fixes et ajouter ``Organization Admin`` + ``Building Manager``
comme nouveaux roles dans l'enum existante. Ajouter un champ
``extra_permissions JSONB`` pour des permissions additionnelles futures.

**Schema** ::

    -- Ajouter les nouveaux roles a l'enum
    ALTER TYPE user_role ADD VALUE 'organization_admin';
    ALTER TYPE user_role ADD VALUE 'building_manager';

    -- Table d'assignation portefeuille
    CREATE TABLE building_manager_assignments (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
        building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
        organization_id UUID NOT NULL REFERENCES organizations(id),
        assigned_at TIMESTAMPTZ DEFAULT NOW(),
        assigned_by UUID REFERENCES users(id),
        UNIQUE(user_id, building_id)
    );

    -- Extension future : permissions additionnelles
    ALTER TABLE user_roles ADD COLUMN extra_permissions JSONB DEFAULT '{}';

**Avantages** :

- Migration progressive, retrocompatible
- Performance maintenue (pas de JOINs complexes)
- Simplicite d'implementation (~1 semaine)
- Roles fixes = comportement previsible et testable

**Inconvenients** :

- Flexibilite limitee pour cas non prevus
- Pas de roles completement custom par organisation

**Effort estime** : 5-7 jours

Alternative 2 : Matrice Dynamique Complete
------------------------------------------

Remplacer le systeme de roles fixes par des permissions granulaires assignables
dynamiquement via 4 nouvelles tables (``permissions``, ``roles``,
``role_permissions``, ``user_assignments``).

**Schema** ::

    CREATE TABLE permissions (
        id UUID PRIMARY KEY,
        resource VARCHAR(50) NOT NULL,   -- "buildings", "expenses"
        action VARCHAR(20) NOT NULL,     -- "create", "read", "update", "delete"
        scope VARCHAR(20) NOT NULL,      -- "all", "organization", "assigned"
        UNIQUE(resource, action, scope)
    );

    CREATE TABLE roles (
        id UUID PRIMARY KEY,
        name VARCHAR(50) NOT NULL UNIQUE,
        organization_id UUID,            -- NULL = global, UUID = org-specific
        is_system BOOLEAN DEFAULT FALSE
    );

    CREATE TABLE role_permissions (
        role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
        permission_id UUID REFERENCES permissions(id) ON DELETE CASCADE,
        PRIMARY KEY(role_id, permission_id)
    );

    CREATE TABLE user_assignments (
        user_id UUID NOT NULL REFERENCES users(id),
        role_id UUID NOT NULL REFERENCES roles(id),
        organization_id UUID,
        constraints JSONB,               -- {"building_ids": [...]}
        UNIQUE(user_id, role_id, organization_id)
    );

**Avantages** :

- Flexibilite maximale
- Roles custom par organisation
- Interface admin pour configurer sans deploiement

**Inconvenients** :

- Complexite accrue (JOINs multiples a chaque requete)
- Risque sur performance P99 < 5ms (necessite cache Redis)
- Effort consequent (~3-4 semaines)
- Explosion du nombre de cas de tests
- UX complexe pour la configuration

**Effort estime** : 15-20 jours

Alternative 3 : Librairie Tierce (Casbin/Oso)
----------------------------------------------

Integrer une solution RBAC/ABAC existante comme ``casbin-rs`` ou Oso.

**Avantages** :

- Solution eprouvee, robuste
- Gain de temps sur la logique de permissions

**Inconvenients** :

- Dependance externe critique
- Courbe d'apprentissage
- Integration complexe avec Actix-web
- casbin-rs : maintenance limitee en 2026
- Oso : fermeture du service cloud en 2025

**Effort estime** : 10-15 jours

Matrice de Decision
===================

+-------------------------+----------+----------+----------+
| Critere                 | Hybride  | Dynamique| Tierce   |
+=========================+==========+==========+==========+
| Effort implementation   | 5-7j     | 15-20j   | 10-15j   |
+-------------------------+----------+----------+----------+
| Performance P99 < 5ms   | Oui      | Risque   | Variable |
+-------------------------+----------+----------+----------+
| Retrocompatibilite      | Totale   | Migration| Partielle|
+-------------------------+----------+----------+----------+
| Flexibilite             | Moyenne  | Haute    | Haute    |
+-------------------------+----------+----------+----------+
| Maintenabilite          | Simple   | Complexe | Externe  |
+-------------------------+----------+----------+----------+
| Testabilite             | Simple   | Complexe | Moyenne  |
+-------------------------+----------+----------+----------+
| Risque                  | Faible   | Moyen    | Moyen    |
+-------------------------+----------+----------+----------+

Plan d'Implementation (Alternative 1)
======================================

Phase 1 : Backend (3 jours)
---------------------------

1. Migration PostgreSQL :

   - Ajouter ``organization_admin`` et ``building_manager`` a l'enum ``user_role``
   - Creer table ``building_manager_assignments``
   - Ajouter colonne ``extra_permissions JSONB`` sur ``user_roles``

2. Domain entities :

   - ``BuildingManagerAssignment`` (building_id, user_id, organization_id)
   - Methodes de validation (user doit etre dans la meme organisation)

3. Repository + Use Cases :

   - ``BuildingManagerAssignmentRepository`` (CRUD + find_by_user)
   - ``BuildingManagerUseCases`` (assign, unassign, list_buildings)

4. Middleware :

   - Etendre ``AuthenticatedUser`` pour verifier scope buildings
   - Helper ``check_building_access(user, building_id)``

Phase 2 : Adaptation Handlers (2 jours)
----------------------------------------

1. Adapter tous les handlers pour :

   - ``organization_admin`` : memes permissions que Syndic + gestion users
   - ``building_manager`` : permissions Syndic mais limitees au portefeuille

2. Pattern de verification ::

       fn check_building_manager_access(
           user: &AuthenticatedUser,
           building_id: Uuid,
           assignments: &[BuildingManagerAssignment],
       ) -> Result<(), HttpResponse> {
           match user.role.as_str() {
               "superadmin" | "syndic" | "organization_admin" => Ok(()),
               "building_manager" => {
                   if assignments.iter().any(|a| a.building_id == building_id) {
                       Ok(())
                   } else {
                       Err(HttpResponse::Forbidden().json(json!({
                           "error": "Access limited to assigned buildings"
                       })))
                   }
               }
               _ => Err(HttpResponse::Forbidden().json(json!({
                   "error": "Insufficient permissions"
               })))
           }
       }

Phase 3 : Frontend + Tests (2 jours)
--------------------------------------

1. Navigation : integrer les nouveaux roles dans le selecteur
2. Building Manager : vue filtree sur ses immeubles assignes
3. Organization Admin : page de gestion utilisateurs
4. Tests : unitaires, integration, BDD pour les nouveaux roles
5. Mettre a jour ``docs/ROLE_PERMISSIONS_MATRIX.rst``

Plan d'Implementation Tenant/Guest
===================================

Phase T1 : Tenant - Backend (3 jours)
--------------------------------------

1. Migration : table ``tenants`` avec contraintes et index
2. Domain entity : ``Tenant`` (validation bail, dates, permissions)
3. Repository : ``TenantRepository`` (CRUD, find_by_user, find_by_unit)
4. Use cases : ``TenantUseCases`` (create, update, expire, list_charges)
5. Middleware : etendre JWT pour inclure ``tenant_unit_ids``
6. Handlers adaptes : filtrer charges par ``unit_id`` du bail actif

Phase T2 : Tenant - Integration SEL (2 jours)
-----------------------------------------------

1. Ajouter ``tenant_id`` nullable dans ``local_exchanges``
2. Creer ``TenantCreditBalance`` (separe de ``OwnerCreditBalance``)
3. Adapter ``LocalExchangeUseCases`` pour accepter Owner OU Tenant
4. Validation : tenant doit avoir ``can_participate_sel = TRUE``

Phase G1 : Guest - Backend (2 jours)
--------------------------------------

1. Migration : table ``guests`` avec contraintes et expiration
2. Domain entity : ``Guest`` (validation dates, permissions granulaires)
3. Repository : ``GuestRepository`` (CRUD, find_active, revoke)
4. Use cases : ``GuestUseCases`` (invite, revoke, check_permission)
5. Middleware : JWT avec ``guest_permissions`` et ``guest_valid_until``
6. Background job : expiration automatique des invitations

Phase G2 : Guest - Integration Communautaire (1 jour)
------------------------------------------------------

1. Adapter les modules communautaires pour verifier ``guest_permissions``
2. Filtrage lecture seule pour notices et gamification
3. Participation SEL/competences/objets si permission accordee

Considerations GDPR pour Tenant/Guest
--------------------------------------

- **Article 15** : Tenants et Guests ont droit a l'export de leurs donnees
- **Article 17** : Erasure doit inclure les donnees tenant/guest
- **Consentement** : Accord explicite pour le traitement des donnees
- **Minimisation** : Ne stocker que les donnees necessaires au bail/invitation
- **Expiration Guest** : Suppression automatique des donnees apres expiration

Criteres de Succes
==================

- Performance P99 < 5ms maintenue apres ajout des nouveaux roles
- 0 regression sur les permissions existantes (tests BDD)
- Documentation mise a jour (matrice des permissions)
- Integration transparente dans le frontend (selecteur de roles)

Conclusion
==========

Le systeme hybride (Alternative 1) offre le meilleur rapport cout/benefice :

- Repond aux besoins metier identifies (delegation, portefeuilles)
- Integre locataires et invites pour une ouverture communautaire
- Maintient la simplicite et la performance du systeme actuel
- Prepare l'infrastructure pour une evolution future (``extra_permissions``)
- Effort total raisonnable (~2 semaines pour les 4 nouveaux roles)

**Priorite d'implementation recommandee** :

1. **Tenant** (valeur metier haute) : les locataires representent une part
   significative des residents et leur inclusion dans le SEL et les modules
   communautaires renforce l'ecosysteme
2. **Organization Admin + Building Manager** (valeur operationnelle) : delegation
   et portefeuilles pour les syndics gerant >10 immeubles
3. **Guest** (valeur communautaire) : ouverture de l'ecosysteme a l'exterieur
   pour enrichir les echanges SEL et les competences

La matrice dynamique complete (Alternative 2) pourra etre envisagee
ulterieurement si le nombre de cas d'usage non couverts augmente significativement
(> 5 demandes distinctes de roles custom).

References
==========

- Matrice actuelle : ``docs/ROLE_PERMISSIONS_MATRIX.rst``
- Multi-role support : ``docs/MULTI_ROLE_SUPPORT.md``
- Middleware auth : ``backend/src/infrastructure/web/mod.rs``
- User roles : ``backend/src/domain/entities/user_role_assignment.rs``
- Migration roles : ``backend/migrations/20250130000000_add_user_roles.sql``
