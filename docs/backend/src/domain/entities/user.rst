========================================
backend/src/domain/entities/user.rs
========================================

Description
===========

Entité domaine représentant un utilisateur de la plateforme Koprogo. Cette entité constitue le cœur du système d'authentification et de gestion des permissions multi-rôles et multi-tenant.

Responsabilités
===============

1. **Modélisation utilisateur**
   - Identité et informations personnelles
   - Gestion des rôles et permissions
   - Association avec une organisation

2. **Validation métier**
   - Validation de l'email (format valide)
   - Validation des noms (longueur minimale)
   - Normalisation des données (trim, lowercase pour email)

3. **Logique métier**
   - Activation/désactivation du compte
   - Mise à jour du profil
   - Contrôle d'accès multi-tenant

Énumérations
============

UserRole
--------

**Signature:**

.. code-block:: rust

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub enum UserRole {
        SuperAdmin,
        Syndic,
        Accountant,
        Owner,
    }

**Description:**

Énumération des rôles disponibles dans le système.

**Variantes:**

.. list-table::
   :header-rows: 1
   :widths: 20 80

   * - Rôle
     - Description
   * - ``SuperAdmin``
     - Administrateur plateforme avec accès illimité à toutes les organisations
   * - ``Syndic``
     - Gestionnaire de copropriété avec accès complet aux immeubles de son organisation
   * - ``Accountant``
     - Comptable avec accès aux données financières de son organisation
   * - ``Owner``
     - Copropriétaire avec accès limité à ses propres lots et informations

**Traits implémentés:**

- ``Display`` - Conversion en chaîne lowercase (ex: "superadmin", "syndic")
- ``FromStr`` - Parsing depuis chaîne avec gestion d'erreurs
- ``Serialize``/``Deserialize`` - Sérialisation JSON via Serde
- ``PartialEq``/``Eq`` - Comparaison

**Exemples:**

.. code-block:: rust

    use std::str::FromStr;

    // Conversion en String
    let role = UserRole::Syndic;
    assert_eq!(role.to_string(), "syndic");

    // Parsing depuis String
    let role = UserRole::from_str("accountant").unwrap();
    assert_eq!(role, UserRole::Accountant);

    // Erreur pour rôle invalide
    let result = UserRole::from_str("invalid");
    assert!(result.is_err());

Structures
==========

User
----

**Signature:**

.. code-block:: rust

    #[derive(Debug, Clone, Serialize, Deserialize, Validate)]
    pub struct User {
        pub id: Uuid,

        #[validate(email(message = "Email must be valid"))]
        pub email: String,

        #[serde(skip_serializing)]
        pub password_hash: String,

        #[validate(length(min = 2, message = "First name must be at least 2 characters"))]
        pub first_name: String,

        #[validate(length(min = 2, message = "Last name must be at least 2 characters"))]
        pub last_name: String,

        pub role: UserRole,
        pub organization_id: Option<Uuid>,
        pub is_active: bool,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

**Description:**

Représente un utilisateur avec ses informations personnelles, son rôle et son affiliation organisationnelle.

**Champs:**

.. list-table::
   :header-rows: 1
   :widths: 25 15 60

   * - Champ
     - Type
     - Description
   * - ``id``
     - ``Uuid``
     - Identifiant unique UUID v4
   * - ``email``
     - ``String``
     - Email (unique, validé, normalisé en lowercase)
   * - ``password_hash``
     - ``String``
     - Hash bcrypt du mot de passe (non sérialisé dans JSON)
   * - ``first_name``
     - ``String``
     - Prénom (minimum 2 caractères)
   * - ``last_name``
     - ``String``
     - Nom de famille (minimum 2 caractères)
   * - ``role``
     - ``UserRole``
     - Rôle déterminant les permissions
   * - ``organization_id``
     - ``Option<Uuid>``
     - ID organisation (None pour SuperAdmin)
   * - ``is_active``
     - ``bool``
     - Indicateur d'activation du compte
   * - ``created_at``
     - ``DateTime<Utc>``
     - Date de création (UTC)
   * - ``updated_at``
     - ``DateTime<Utc>``
     - Date de dernière modification (UTC)

**Validations automatiques:**

- Email: Format RFC 5322
- Prénom/Nom: Longueur >= 2 caractères
- Email normalisé: trim() + to_lowercase()
- Noms normalisés: trim()

Méthodes
========

new()
-----

**Signature:**

.. code-block:: rust

    pub fn new(
        email: String,
        password_hash: String,
        first_name: String,
        last_name: String,
        role: UserRole,
        organization_id: Option<Uuid>,
    ) -> Result<Self, String>

**Description:**

Constructeur qui crée un nouvel utilisateur avec validation automatique.

**Comportement:**

1. Génère un UUID v4 unique
2. Normalise l'email (lowercase + trim)
3. Normalise les noms (trim)
4. Active le compte par défaut (``is_active = true``)
5. Initialise les timestamps à ``Utc::now()``
6. Exécute les validations (email format, longueur noms)

**Paramètres:**

- ``email`` - Adresse email (sera normalisée)
- ``password_hash`` - Hash bcrypt du mot de passe
- ``first_name`` - Prénom de l'utilisateur
- ``last_name`` - Nom de famille
- ``role`` - Rôle dans le système
- ``organization_id`` - ID organisation (``None`` pour SuperAdmin)

**Retour:**

- ``Ok(User)`` - Utilisateur créé avec succès
- ``Err(String)`` - Message d'erreur de validation

**Exemples:**

.. code-block:: rust

    use bcrypt::{hash, DEFAULT_COST};

    // ✅ Création réussie
    let password_hash = hash("password123", DEFAULT_COST).unwrap();
    let user = User::new(
        "  JOHN.DOE@EXAMPLE.COM  ".to_string(), // Sera normalisé
        password_hash,
        "  John  ".to_string(), // Sera trim
        "Doe".to_string(),
        UserRole::Syndic,
        Some(Uuid::new_v4()),
    );
    assert!(user.is_ok());
    assert_eq!(user.unwrap().email, "john.doe@example.com");

    // ❌ Email invalide
    let result = User::new(
        "invalid-email".to_string(),
        password_hash,
        "John".to_string(),
        "Doe".to_string(),
        UserRole::Syndic,
        None,
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Email must be valid"));

    // ❌ Prénom trop court
    let result = User::new(
        "valid@example.com".to_string(),
        password_hash,
        "J".to_string(),
        "Doe".to_string(),
        UserRole::Syndic,
        None,
    );
    assert!(result.is_err());

full_name()
-----------

**Signature:**

.. code-block:: rust

    pub fn full_name(&self) -> String

**Description:**

Retourne le nom complet de l'utilisateur (prénom + nom).

**Retour:**

Chaîne formatée: ``"{first_name} {last_name}"``

**Exemple:**

.. code-block:: rust

    let user = User::new(
        "john@example.com".to_string(),
        "hash".to_string(),
        "John".to_string(),
        "Doe".to_string(),
        UserRole::Syndic,
        None,
    ).unwrap();

    assert_eq!(user.full_name(), "John Doe");

update_profile()
----------------

**Signature:**

.. code-block:: rust

    pub fn update_profile(&mut self, first_name: String, last_name: String) -> Result<(), String>

**Description:**

Met à jour le prénom et le nom de l'utilisateur avec validation.

**Comportement:**

1. Normalise les nouveaux noms (trim)
2. Met à jour ``first_name`` et ``last_name``
3. Met à jour ``updated_at`` à ``Utc::now()``
4. Valide les nouvelles valeurs

**Paramètres:**

- ``first_name`` - Nouveau prénom
- ``last_name`` - Nouveau nom

**Retour:**

- ``Ok(())`` - Mise à jour réussie
- ``Err(String)`` - Erreur de validation

**Exemple:**

.. code-block:: rust

    let mut user = User::new(/* ... */).unwrap();

    let result = user.update_profile("Jane".to_string(), "Smith".to_string());
    assert!(result.is_ok());
    assert_eq!(user.full_name(), "Jane Smith");

    // Le timestamp est mis à jour
    // assert!(user.updated_at > old_timestamp);

deactivate()
------------

**Signature:**

.. code-block:: rust

    pub fn deactivate(&mut self)

**Description:**

Désactive le compte utilisateur. Un compte désactivé ne peut plus se connecter.

**Comportement:**

1. Définit ``is_active`` à ``false``
2. Met à jour ``updated_at``

**Exemple:**

.. code-block:: rust

    let mut user = User::new(/* ... */).unwrap();
    assert!(user.is_active);

    user.deactivate();
    assert!(!user.is_active);

activate()
----------

**Signature:**

.. code-block:: rust

    pub fn activate(&mut self)

**Description:**

Réactive un compte utilisateur précédemment désactivé.

**Comportement:**

1. Définit ``is_active`` à ``true``
2. Met à jour ``updated_at``

**Exemple:**

.. code-block:: rust

    let mut user = User::new(/* ... */).unwrap();
    user.deactivate();

    user.activate();
    assert!(user.is_active);

can_access_building()
---------------------

**Signature:**

.. code-block:: rust

    pub fn can_access_building(&self, building_org_id: Option<Uuid>) -> bool

**Description:**

Vérifie si l'utilisateur peut accéder à un immeuble donné selon son rôle et son organisation.

**Logique d'accès:**

.. code-block:: text

    ┌─────────────────────────────────────────────────────────┐
    │ Utilisateur SuperAdmin ?                                 │
    │   ├─ Oui → ✅ Accès autorisé (accès universel)          │
    │   └─ Non → Vérifier organization_id                     │
    │       └─ self.organization_id == building_org_id ?      │
    │           ├─ Oui → ✅ Accès autorisé (même org)         │
    │           └─ Non → ❌ Accès refusé (org différente)     │
    └─────────────────────────────────────────────────────────┘

**Paramètres:**

- ``building_org_id`` - ID de l'organisation propriétaire de l'immeuble

**Retour:**

- ``true`` - L'utilisateur peut accéder à l'immeuble
- ``false`` - Accès refusé

**Exemples:**

.. code-block:: rust

    // SuperAdmin: accès universel
    let superadmin = User::new(
        "admin@example.com".to_string(),
        "hash".to_string(),
        "Admin".to_string(),
        "User".to_string(),
        UserRole::SuperAdmin,
        None,
    ).unwrap();

    assert!(superadmin.can_access_building(Some(Uuid::new_v4())));
    assert!(superadmin.can_access_building(None));

    // Utilisateur régulier: accès limité à son organisation
    let org_id = Uuid::new_v4();
    let syndic = User::new(
        "syndic@example.com".to_string(),
        "hash".to_string(),
        "John".to_string(),
        "Syndic".to_string(),
        UserRole::Syndic,
        Some(org_id),
    ).unwrap();

    assert!(syndic.can_access_building(Some(org_id))); // ✅ Même org
    assert!(!syndic.can_access_building(Some(Uuid::new_v4()))); // ❌ Autre org
    assert!(!syndic.can_access_building(None)); // ❌ Pas d'org

Tests unitaires
===============

Le fichier contient 7 tests unitaires couvrant:

.. list-table::
   :header-rows: 1
   :widths: 50 50

   * - Test
     - Scénario couvert
   * - ``test_create_user_success``
     - Création réussie avec données valides
   * - ``test_create_user_invalid_email``
     - Rejet email invalide
   * - ``test_update_profile``
     - Mise à jour prénom/nom
   * - ``test_deactivate_user``
     - Désactivation compte
   * - ``test_superadmin_can_access_all_buildings``
     - SuperAdmin: accès universel
   * - ``test_regular_user_access_control``
     - Utilisateur régulier: accès limité

**Exécuter les tests:**

.. code-block:: bash

    cd backend
    cargo test domain::entities::user

Architecture Multi-tenant
==========================

La structure User implémente le pattern **Multi-tenancy** via ``organization_id``:

.. code-block:: text

    ┌───────────────────────────────────────────────────────┐
    │                    Organization A                     │
    │  ┌────────────┐  ┌────────────┐  ┌────────────┐      │
    │  │ Syndic     │  │ Accountant │  │ Owner 1    │      │
    │  │ User       │  │ User       │  │ User       │      │
    │  └────────────┘  └────────────┘  └────────────┘      │
    └───────────────────────────────────────────────────────┘

    ┌───────────────────────────────────────────────────────┐
    │                    Organization B                     │
    │  ┌────────────┐  ┌────────────┐                       │
    │  │ Syndic     │  │ Owner 2    │                       │
    │  │ User       │  │ User       │                       │
    │  └────────────┘  └────────────┘                       │
    └───────────────────────────────────────────────────────┘

    ┌───────────────────────────────────────────────────────┐
    │                     SuperAdmin                        │
    │  (organization_id = None)                             │
    │  ├─ Accès Organization A                              │
    │  ├─ Accès Organization B                              │
    │  └─ Accès toutes les organisations                    │
    └───────────────────────────────────────────────────────┘

Hiérarchie des permissions
===========================

.. code-block:: text

    SuperAdmin (plateforme)
        ↓
    ┌───────────────────────────────────────┐
    │         Organisation                  │
    │                                       │
    │   Syndic (gestion complète)          │
    │      ↓                                │
    │   Accountant (finance uniquement)    │
    │      ↓                                │
    │   Owner (consultation limitée)       │
    └───────────────────────────────────────┘

Dépendances
===========

Crates externes:

- ``uuid`` - Génération d'identifiants uniques
- ``chrono`` - Gestion des timestamps UTC
- ``serde`` - Sérialisation JSON
- ``validator`` - Validation déclarative (email, longueur)

Modules internes:

- Aucun (entité auto-suffisante)

Utilisation dans l'application
===============================

**Création d'un utilisateur (use case):**

.. code-block:: rust

    use bcrypt::{hash, DEFAULT_COST};

    // Hash du mot de passe
    let password_hash = hash("user_password", DEFAULT_COST)?;

    // Création de l'entité
    let user = User::new(
        "user@example.com".to_string(),
        password_hash,
        "John".to_string(),
        "Doe".to_string(),
        UserRole::Syndic,
        Some(organization_id),
    )?;

    // Sauvegarde via repository
    user_repository.create(user).await?;

**Authentification (JWT):**

.. code-block:: rust

    // Vérification du mot de passe
    let user = user_repository.find_by_email(email).await?;
    let valid = bcrypt::verify(password, &user.password_hash)?;

    if valid && user.is_active {
        // Générer token JWT
        let claims = Claims {
            sub: user.id.to_string(),
            role: user.role.to_string(),
            org: user.organization_id.map(|id| id.to_string()),
            exp: /* ... */,
        };
        let token = encode(&Header::default(), &claims, &encoding_key)?;
    }

**Contrôle d'accès:**

.. code-block:: rust

    // Dans un handler
    let building = building_repository.find_by_id(building_id).await?;

    if !current_user.can_access_building(building.organization_id) {
        return Err(Error::Forbidden);
    }

Notes de sécurité
=================

.. warning::

   **Password Hash:**

   Le champ ``password_hash`` utilise ``#[serde(skip_serializing)]`` pour éviter de l'exposer dans les réponses JSON. Assurez-vous de:

   - Utiliser bcrypt avec cost >= 12
   - Ne JAMAIS logger le password_hash
   - Ne JAMAIS l'inclure dans les réponses API

.. warning::

   **Désactivation vs Suppression:**

   Utilisez ``deactivate()`` plutôt que de supprimer les utilisateurs pour:

   - Préserver l'intégrité référentielle
   - Garder l'historique des actions
   - Possibilité de réactivation

Fichiers associés
=================

- ``backend/src/domain/entities/organization.rs`` - Entité Organisation
- ``backend/src/application/ports/user_repository.rs`` - Trait repository
- ``backend/src/infrastructure/database/repositories/user_repository_impl.rs`` - Implémentation PostgreSQL
- ``backend/src/application/use_cases/auth_use_cases.rs`` - Cas d'usage authentification
- ``backend/src/application/dto/auth_dto.rs`` - DTOs pour authentification
