================================================
backend/src/domain/entities/organization.rs
================================================

Description
===========

Entité domaine représentant une organisation (syndic de copropriété) dans la plateforme Koprogo. Cette entité implémente un système de gestion multi-tenant avec différents plans d'abonnement et des limites de ressources associées.

Responsabilités
===============

1. **Modélisation organisation**
   - Identité et informations de contact
   - Gestion du plan d'abonnement
   - Limites de ressources (immeubles, utilisateurs)

2. **Validation métier**
   - Validation de l'email de contact
   - Validation du nom (longueur minimale)
   - Normalisation des données (trim, lowercase pour email)

3. **Logique métier**
   - Génération automatique de slug URL-friendly
   - Contrôle des limites de ressources par plan
   - Activation/désactivation de l'organisation
   - Mise à niveau du plan d'abonnement

Énumérations
============

SubscriptionPlan
----------------

**Signature:**

.. code-block:: rust

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub enum SubscriptionPlan {
        Free,
        Starter,
        Professional,
        Enterprise,
    }

**Description:**

Énumération des plans d'abonnement disponibles avec des limites de ressources associées.

**Variantes:**

.. list-table::
   :header-rows: 1
   :widths: 20 15 15 50

   * - Plan
     - Max Immeubles
     - Max Utilisateurs
     - Description
   * - ``Free``
     - 1
     - 3
     - Plan gratuit pour petites copropriétés (essai)
   * - ``Starter``
     - 5
     - 10
     - Plan de démarrage pour syndics indépendants
   * - ``Professional``
     - 20
     - 50
     - Plan professionnel pour syndics établis
   * - ``Enterprise``
     - Illimité
     - Illimité
     - Plan entreprise pour grands groupes immobiliers

**Traits implémentés:**

- ``Display`` - Conversion en chaîne lowercase (ex: "free", "professional")
- ``FromStr`` - Parsing depuis chaîne avec gestion d'erreurs
- ``Serialize``/``Deserialize`` - Sérialisation JSON via Serde
- ``PartialEq``/``Eq`` - Comparaison

**Exemples:**

.. code-block:: rust

    use std::str::FromStr;

    // Conversion en String
    let plan = SubscriptionPlan::Professional;
    assert_eq!(plan.to_string(), "professional");

    // Parsing depuis String
    let plan = SubscriptionPlan::from_str("starter").unwrap();
    assert_eq!(plan, SubscriptionPlan::Starter);

    // Erreur pour plan invalide
    let result = SubscriptionPlan::from_str("invalid");
    assert!(result.is_err());

Structures
==========

Organization
------------

**Signature:**

.. code-block:: rust

    #[derive(Debug, Clone, Serialize, Deserialize, Validate)]
    pub struct Organization {
        pub id: Uuid,

        #[validate(length(min = 2, message = "Name must be at least 2 characters"))]
        pub name: String,

        pub slug: String,

        #[validate(email(message = "Contact email must be valid"))]
        pub contact_email: String,

        pub contact_phone: Option<String>,

        pub subscription_plan: SubscriptionPlan,

        pub max_buildings: i32,

        pub max_users: i32,

        pub is_active: bool,

        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

**Description:**

Représente une organisation (syndic de copropriété) avec ses informations de contact, son plan d'abonnement et ses limites de ressources.

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
   * - ``name``
     - ``String``
     - Nom de l'organisation (minimum 2 caractères)
   * - ``slug``
     - ``String``
     - Identifiant URL-friendly (généré automatiquement depuis le nom)
   * - ``contact_email``
     - ``String``
     - Email de contact (validé, normalisé en lowercase)
   * - ``contact_phone``
     - ``Option<String>``
     - Numéro de téléphone optionnel
   * - ``subscription_plan``
     - ``SubscriptionPlan``
     - Plan d'abonnement actuel
   * - ``max_buildings``
     - ``i32``
     - Nombre maximum d'immeubles autorisés (déterminé par le plan)
   * - ``max_users``
     - ``i32``
     - Nombre maximum d'utilisateurs autorisés (déterminé par le plan)
   * - ``is_active``
     - ``bool``
     - Indicateur d'activation de l'organisation
   * - ``created_at``
     - ``DateTime<Utc>``
     - Date de création (UTC)
   * - ``updated_at``
     - ``DateTime<Utc>``
     - Date de dernière modification (UTC)

**Validations automatiques:**

- Email: Format RFC 5322
- Nom: Longueur >= 2 caractères
- Email normalisé: trim() + to_lowercase()
- Nom normalisé: trim()
- Slug: Généré automatiquement (alphanumeric + tirets)

Méthodes
========

new()
-----

**Signature:**

.. code-block:: rust

    pub fn new(
        name: String,
        contact_email: String,
        contact_phone: Option<String>,
        subscription_plan: SubscriptionPlan,
    ) -> Result<Self, String>

**Description:**

Constructeur qui crée une nouvelle organisation avec validation automatique et configuration du plan d'abonnement.

**Comportement:**

1. Génère un UUID v4 unique
2. Normalise le nom (trim)
3. Génère un slug URL-friendly depuis le nom
4. Normalise l'email (lowercase + trim)
5. Détermine les limites (max_buildings, max_users) selon le plan
6. Active l'organisation par défaut (``is_active = true``)
7. Initialise les timestamps à ``Utc::now()``
8. Exécute les validations (email format, longueur nom)

**Paramètres:**

- ``name`` - Nom de l'organisation
- ``contact_email`` - Email de contact (sera normalisé)
- ``contact_phone`` - Numéro de téléphone optionnel
- ``subscription_plan`` - Plan d'abonnement initial

**Retour:**

- ``Ok(Organization)`` - Organisation créée avec succès
- ``Err(String)`` - Message d'erreur de validation

**Exemples:**

.. code-block:: rust

    // ✅ Création réussie avec plan Professional
    let org = Organization::new(
        "Syndic Immobilier Paris".to_string(),
        "contact@syndic-paris.fr".to_string(),
        Some("+33123456789".to_string()),
        SubscriptionPlan::Professional,
    );
    assert!(org.is_ok());
    let org = org.unwrap();
    assert_eq!(org.name, "Syndic Immobilier Paris");
    assert_eq!(org.slug, "syndic-immobilier-paris");
    assert_eq!(org.max_buildings, 20);
    assert_eq!(org.max_users, 50);

    // ✅ Slug généré avec caractères spéciaux
    let org = Organization::new(
        "My Super Company!!!".to_string(),
        "contact@example.com".to_string(),
        None,
        SubscriptionPlan::Free,
    ).unwrap();
    assert_eq!(org.slug, "my-super-company");

    // ❌ Email invalide
    let result = Organization::new(
        "Test Company".to_string(),
        "invalid-email".to_string(),
        None,
        SubscriptionPlan::Starter,
    );
    assert!(result.is_err());

generate_slug() (privée)
------------------------

**Signature:**

.. code-block:: rust

    fn generate_slug(name: &str) -> String

**Description:**

Génère un slug URL-friendly à partir du nom de l'organisation.

**Algorithme:**

1. Convertit en lowercase
2. Remplace les caractères non-alphanumériques par ``-``
3. Supprime les tirets consécutifs
4. Supprime les tirets en début/fin

**Exemples:**

.. code-block:: rust

    // "Company Name" → "company-name"
    // "Café-Bar & Restaurant!" → "caf-bar-restaurant"
    // "123 Main Street" → "123-main-street"

get_limits_for_plan() (privée)
------------------------------

**Signature:**

.. code-block:: rust

    fn get_limits_for_plan(plan: &SubscriptionPlan) -> (i32, i32)

**Description:**

Retourne les limites (max_buildings, max_users) pour un plan donné.

**Retour:**

Tuple ``(max_buildings, max_users)``

**Limites par plan:**

.. code-block:: rust

    Free         → (1, 3)
    Starter      → (5, 10)
    Professional → (20, 50)
    Enterprise   → (i32::MAX, i32::MAX)

upgrade_plan()
--------------

**Signature:**

.. code-block:: rust

    pub fn upgrade_plan(&mut self, new_plan: SubscriptionPlan)

**Description:**

Met à niveau (ou rétrograde) le plan d'abonnement de l'organisation et ajuste automatiquement les limites de ressources.

**Comportement:**

1. Modifie ``subscription_plan``
2. Recalcule ``max_buildings`` et ``max_users``
3. Met à jour ``updated_at``

**Paramètres:**

- ``new_plan`` - Nouveau plan d'abonnement

**Exemple:**

.. code-block:: rust

    let mut org = Organization::new(
        "Test Org".to_string(),
        "test@test.com".to_string(),
        None,
        SubscriptionPlan::Free,
    ).unwrap();

    assert_eq!(org.max_buildings, 1);
    assert_eq!(org.max_users, 3);

    org.upgrade_plan(SubscriptionPlan::Professional);
    assert_eq!(org.subscription_plan, SubscriptionPlan::Professional);
    assert_eq!(org.max_buildings, 20);
    assert_eq!(org.max_users, 50);

update_contact()
----------------

**Signature:**

.. code-block:: rust

    pub fn update_contact(&mut self, email: String, phone: Option<String>) -> Result<(), String>

**Description:**

Met à jour les informations de contact de l'organisation avec validation.

**Comportement:**

1. Normalise le nouvel email (lowercase + trim)
2. Met à jour ``contact_email`` et ``contact_phone``
3. Met à jour ``updated_at``
4. Valide les nouvelles valeurs

**Paramètres:**

- ``email`` - Nouveau email de contact
- ``phone`` - Nouveau numéro de téléphone (optionnel)

**Retour:**

- ``Ok(())`` - Mise à jour réussie
- ``Err(String)`` - Erreur de validation (email invalide)

**Exemple:**

.. code-block:: rust

    let mut org = Organization::new(/* ... */).unwrap();

    let result = org.update_contact(
        "new-contact@example.com".to_string(),
        Some("+33987654321".to_string()),
    );
    assert!(result.is_ok());
    assert_eq!(org.contact_email, "new-contact@example.com");

deactivate()
------------

**Signature:**

.. code-block:: rust

    pub fn deactivate(&mut self)

**Description:**

Désactive l'organisation. Une organisation désactivée ne peut plus ajouter d'immeubles ou d'utilisateurs.

**Comportement:**

1. Définit ``is_active`` à ``false``
2. Met à jour ``updated_at``

**Exemple:**

.. code-block:: rust

    let mut org = Organization::new(/* ... */).unwrap();
    assert!(org.is_active);

    org.deactivate();
    assert!(!org.is_active);
    assert!(!org.can_add_building(0));
    assert!(!org.can_add_user(0));

activate()
----------

**Signature:**

.. code-block:: rust

    pub fn activate(&mut self)

**Description:**

Réactive une organisation précédemment désactivée.

**Comportement:**

1. Définit ``is_active`` à ``true``
2. Met à jour ``updated_at``

**Exemple:**

.. code-block:: rust

    let mut org = Organization::new(/* ... */).unwrap();
    org.deactivate();

    org.activate();
    assert!(org.is_active);

can_add_building()
------------------

**Signature:**

.. code-block:: rust

    pub fn can_add_building(&self, current_count: i32) -> bool

**Description:**

Vérifie si l'organisation peut ajouter un nouvel immeuble selon son plan d'abonnement et son statut.

**Logique:**

.. code-block:: text

    ┌─────────────────────────────────────────────────┐
    │ Organisation active ?                            │
    │   ├─ Non → ❌ Refusé                            │
    │   └─ Oui → Vérifier limites                     │
    │       └─ current_count < max_buildings ?        │
    │           ├─ Oui → ✅ Autorisé                  │
    │           └─ Non → ❌ Limite atteinte           │
    └─────────────────────────────────────────────────┘

**Paramètres:**

- ``current_count`` - Nombre actuel d'immeubles de l'organisation

**Retour:**

- ``true`` - Ajout autorisé
- ``false`` - Limite atteinte ou organisation désactivée

**Exemples:**

.. code-block:: rust

    // Plan Starter: max 5 immeubles
    let org = Organization::new(
        "Test Org".to_string(),
        "test@test.com".to_string(),
        None,
        SubscriptionPlan::Starter,
    ).unwrap();

    assert!(org.can_add_building(0));  // ✅ 0 < 5
    assert!(org.can_add_building(4));  // ✅ 4 < 5
    assert!(!org.can_add_building(5)); // ❌ 5 >= 5

    // Organisation désactivée
    let mut org = org;
    org.deactivate();
    assert!(!org.can_add_building(0)); // ❌ Inactif

can_add_user()
--------------

**Signature:**

.. code-block:: rust

    pub fn can_add_user(&self, current_count: i32) -> bool

**Description:**

Vérifie si l'organisation peut ajouter un nouvel utilisateur selon son plan d'abonnement et son statut.

**Logique:**

Identique à ``can_add_building()`` mais compare avec ``max_users``.

**Paramètres:**

- ``current_count`` - Nombre actuel d'utilisateurs de l'organisation

**Retour:**

- ``true`` - Ajout autorisé
- ``false`` - Limite atteinte ou organisation désactivée

**Exemple:**

.. code-block:: rust

    // Plan Free: max 3 utilisateurs
    let org = Organization::new(
        "Test Org".to_string(),
        "test@test.com".to_string(),
        None,
        SubscriptionPlan::Free,
    ).unwrap();

    assert!(org.can_add_user(0));  // ✅ 0 < 3
    assert!(org.can_add_user(2));  // ✅ 2 < 3
    assert!(!org.can_add_user(3)); // ❌ 3 >= 3

Tests unitaires
===============

Le fichier contient 6 tests unitaires couvrant:

.. list-table::
   :header-rows: 1
   :widths: 50 50

   * - Test
     - Scénario couvert
   * - ``test_create_organization_success``
     - Création réussie avec données valides
   * - ``test_generate_slug``
     - Génération de slug avec caractères spéciaux
   * - ``test_subscription_limits``
     - Vérification des limites par plan
   * - ``test_upgrade_plan``
     - Mise à niveau de plan
   * - ``test_can_add_building``
     - Vérification des limites d'immeubles
   * - ``test_deactivate_prevents_adding``
     - Organisation désactivée ne peut rien ajouter

**Exécuter les tests:**

.. code-block:: bash

    cd backend
    cargo test domain::entities::organization

Architecture Multi-tenant
==========================

L'entité Organization est au cœur du système multi-tenant de Koprogo:

.. code-block:: text

    ┌─────────────────────────────────────────────────┐
    │            Organization 1 (Free)                │
    │  ┌──────────────┐                               │
    │  │ Building A   │                               │
    │  └──────────────┘                               │
    │  Users: Syndic, Owner1, Owner2 (max 3)         │
    └─────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────┐
    │        Organization 2 (Professional)            │
    │  ┌──────────────┐  ┌──────────────┐            │
    │  │ Building A   │  │ Building B   │            │
    │  └──────────────┘  └──────────────┘            │
    │  ...jusqu'à 20 immeubles...                    │
    │  Users: jusqu'à 50 utilisateurs                │
    └─────────────────────────────────────────────────┘

Matrice des plans d'abonnement
================================

.. list-table::
   :header-rows: 1
   :widths: 20 20 20 40

   * - Fonctionnalité
     - Free
     - Starter
     - Professional
   * - Immeubles
     - 1
     - 5
     - 20
   * - Utilisateurs
     - 3
     - 10
     - 50
   * - Cas d'usage
     - Petite copropriété
     - Syndic indépendant
     - Cabinet immobilier
   * - Prix suggéré
     - 0€/mois
     - 49€/mois
     - 199€/mois

.. note::

   Le plan **Enterprise** offre des ressources illimitées (``i32::MAX``) et est destiné aux grands groupes immobiliers avec des centaines d'immeubles.

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

**Création d'une organisation (use case):**

.. code-block:: rust

    // Enregistrement d'un nouveau syndic
    let organization = Organization::new(
        "Cabinet Syndic Paris 15".to_string(),
        "contact@syndic-paris15.fr".to_string(),
        Some("+33145678901".to_string()),
        SubscriptionPlan::Starter,
    )?;

    // Sauvegarde via repository
    organization_repository.create(organization).await?;

**Vérification des limites avant ajout:**

.. code-block:: rust

    // Dans un use case d'ajout d'immeuble
    let org = organization_repository.find_by_id(org_id).await?;
    let building_count = building_repository.count_by_org(org_id).await?;

    if !org.can_add_building(building_count) {
        return Err(Error::SubscriptionLimitReached {
            resource: "buildings",
            current: building_count,
            max: org.max_buildings,
        });
    }

    // Créer l'immeuble...

**Mise à niveau de plan:**

.. code-block:: rust

    // Quand l'utilisateur upgrade son abonnement
    let mut org = organization_repository.find_by_id(org_id).await?;
    org.upgrade_plan(SubscriptionPlan::Professional);
    organization_repository.update(org).await?;

Notes de conception
===================

.. note::

   **Slug unique:**

   Le slug est généré automatiquement depuis le nom mais n'est pas garanti unique. Pour un système de production, vous pourriez vouloir:

   - Ajouter une contrainte UNIQUE en base de données
   - Implémenter un système de suffixe (``company-name-2``)
   - Utiliser le slug pour des URL public-facing

.. warning::

   **Limites de ressources:**

   Les méthodes ``can_add_building()`` et ``can_add_user()`` vérifient uniquement les limites. Il est de la responsabilité du code appelant de:

   - Compter correctement les ressources actuelles
   - Appliquer ces vérifications avant création
   - Gérer les cas de courses (race conditions) en base

.. tip::

   **Soft delete recommandé:**

   Utilisez ``deactivate()`` plutôt que de supprimer les organisations pour:

   - Préserver l'intégrité référentielle (users, buildings liés)
   - Garder l'historique pour audit
   - Possibilité de réactivation avec données intactes

Fichiers associés
=================

- ``backend/src/domain/entities/user.rs`` - Entité User (liée via organization_id)
- ``backend/src/domain/entities/building.rs`` - Entité Building (liée via organization_id)
- ``backend/src/application/ports/organization_repository.rs`` - Trait repository
- ``backend/src/infrastructure/database/repositories/organization_repository_impl.rs`` - Implémentation PostgreSQL
- ``backend/src/application/use_cases/organization_use_cases.rs`` - Cas d'usage (si existe)
