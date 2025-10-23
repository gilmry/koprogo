============================================
backend/src/domain/entities/building.rs
============================================

Description
===========

Entité domaine représentant un immeuble en copropriété dans le système Koprogo. Cette entité encapsule toutes les informations géographiques et structurelles d'un immeuble géré par un syndic.

Responsabilités
===============

1. **Modélisation d'immeuble**
   - Informations d'identification (nom, adresse)
   - Localisation géographique complète
   - Caractéristiques structurelles (nombre de lots, année de construction)

2. **Validation métier**
   - Nom d'immeuble non vide
   - Nombre de lots strictement positif
   - Intégrité des données

3. **Gestion du cycle de vie**
   - Création avec validation
   - Mise à jour des informations
   - Tracking des modifications (timestamps)

Structures
==========

Building
--------

**Signature:**

.. code-block:: rust

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct Building {
        pub id: Uuid,
        pub name: String,
        pub address: String,
        pub city: String,
        pub postal_code: String,
        pub country: String,
        pub total_units: i32,
        pub construction_year: Option<i32>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

**Description:**

Représente un immeuble en copropriété avec sa localisation complète et ses caractéristiques.

**Champs:**

.. list-table::
   :header-rows: 1
   :widths: 25 20 55

   * - Champ
     - Type
     - Description
   * - ``id``
     - ``Uuid``
     - Identifiant unique UUID v4
   * - ``name``
     - ``String``
     - Nom de l'immeuble ou de la résidence (non vide)
   * - ``address``
     - ``String``
     - Adresse complète (numéro et rue)
   * - ``city``
     - ``String``
     - Ville de localisation
   * - ``postal_code``
     - ``String``
     - Code postal (format libre pour support international)
   * - ``country``
     - ``String``
     - Pays de localisation
   * - ``total_units``
     - ``i32``
     - Nombre total de lots dans l'immeuble (> 0)
   * - ``construction_year``
     - ``Option<i32>``
     - Année de construction (optionnelle si inconnue)
   * - ``created_at``
     - ``DateTime<Utc>``
     - Date de création de l'enregistrement (UTC)
   * - ``updated_at``
     - ``DateTime<Utc>``
     - Date de dernière modification (UTC)

**Contraintes métier:**

- ``name`` ne peut pas être vide
- ``total_units`` doit être > 0 (au moins 1 lot)
- ``construction_year`` peut être ``None`` si inconnu
- Les timestamps sont automatiquement gérés

Méthodes
========

new()
-----

**Signature:**

.. code-block:: rust

    pub fn new(
        name: String,
        address: String,
        city: String,
        postal_code: String,
        country: String,
        total_units: i32,
        construction_year: Option<i32>,
    ) -> Result<Self, String>

**Description:**

Constructeur qui crée un nouvel immeuble avec validation des règles métier.

**Comportement:**

1. Valide que ``name`` n'est pas vide
2. Valide que ``total_units`` est strictement positif
3. Génère un UUID v4 unique
4. Initialise ``created_at`` et ``updated_at`` à ``Utc::now()``

**Paramètres:**

- ``name`` - Nom de l'immeuble (ex: "Résidence Les Jardins")
- ``address`` - Adresse complète (ex: "123 Rue de la Paix")
- ``city`` - Ville (ex: "Paris")
- ``postal_code`` - Code postal (ex: "75001")
- ``country`` - Pays (ex: "France")
- ``total_units`` - Nombre de lots (doit être > 0)
- ``construction_year`` - Année de construction (optionnelle)

**Retour:**

- ``Ok(Building)`` - Immeuble créé avec succès
- ``Err(String)`` - Message d'erreur de validation

**Erreurs possibles:**

.. list-table::
   :header-rows: 1
   :widths: 60 40

   * - Condition
     - Message d'erreur
   * - ``name.is_empty()``
     - "Building name cannot be empty"
   * - ``total_units <= 0``
     - "Total units must be greater than 0"

**Exemples:**

.. code-block:: rust

    use uuid::Uuid;
    use chrono::Utc;

    // ✅ Création réussie
    let building = Building::new(
        "Résidence Les Champs".to_string(),
        "15 Avenue des Champs-Élysées".to_string(),
        "Paris".to_string(),
        "75008".to_string(),
        "France".to_string(),
        50,  // 50 lots
        Some(1985),
    );

    assert!(building.is_ok());
    let building = building.unwrap();
    assert_eq!(building.name, "Résidence Les Champs");
    assert_eq!(building.total_units, 50);
    assert!(building.construction_year == Some(1985));

    // ✅ Sans année de construction
    let building = Building::new(
        "Résidence Moderne".to_string(),
        "10 Rue de la République".to_string(),
        "Lyon".to_string(),
        "69001".to_string(),
        "France".to_string(),
        30,
        None,  // Année inconnue
    );
    assert!(building.is_ok());

    // ❌ Nom vide
    let building = Building::new(
        "".to_string(),  // Invalide
        "123 Rue Test".to_string(),
        "Paris".to_string(),
        "75001".to_string(),
        "France".to_string(),
        10,
        None,
    );
    assert!(building.is_err());
    assert_eq!(building.unwrap_err(), "Building name cannot be empty");

    // ❌ Nombre de lots invalide
    let building = Building::new(
        "Résidence Test".to_string(),
        "123 Rue Test".to_string(),
        "Paris".to_string(),
        "75001".to_string(),
        "France".to_string(),
        0,  // Invalide
        None,
    );
    assert!(building.is_err());
    assert_eq!(building.unwrap_err(), "Total units must be greater than 0");

update_info()
-------------

**Signature:**

.. code-block:: rust

    pub fn update_info(
        &mut self,
        name: String,
        address: String,
        city: String,
        postal_code: String,
    )

**Description:**

Met à jour les informations principales de l'immeuble.

**Comportement:**

1. Met à jour les champs ``name``, ``address``, ``city``, ``postal_code``
2. Met à jour ``updated_at`` à ``Utc::now()``
3. Ne modifie PAS ``country``, ``total_units``, ``construction_year`` (données structurelles)

**Paramètres:**

- ``name`` - Nouveau nom de l'immeuble
- ``address`` - Nouvelle adresse
- ``city`` - Nouvelle ville
- ``postal_code`` - Nouveau code postal

**Note:**

Cette méthode ne valide pas les données (pas de ``Result``). La validation doit être faite au niveau supérieur (use case) si nécessaire.

**Exemple:**

.. code-block:: rust

    let mut building = Building::new(
        "Ancien Nom".to_string(),
        "Ancienne Adresse".to_string(),
        "Ancienne Ville".to_string(),
        "00000".to_string(),
        "France".to_string(),
        20,
        None,
    ).unwrap();

    let old_updated_at = building.updated_at;

    // Mise à jour
    building.update_info(
        "Nouveau Nom".to_string(),
        "Nouvelle Adresse".to_string(),
        "Nouvelle Ville".to_string(),
        "75001".to_string(),
    );

    // Vérifications
    assert_eq!(building.name, "Nouveau Nom");
    assert_eq!(building.address, "Nouvelle Adresse");
    assert_eq!(building.city, "Nouvelle Ville");
    assert_eq!(building.postal_code, "75001");

    // Le timestamp est mis à jour
    assert!(building.updated_at > old_updated_at);

    // Les données structurelles ne changent pas
    assert_eq!(building.total_units, 20);
    assert_eq!(building.country, "France");

Cas d'usage typiques
=====================

Création d'un immeuble
----------------------

.. code-block:: rust

    // Dans un use case ou handler
    let building = Building::new(
        "Résidence Les Jardins".to_string(),
        "123 Rue de la Paix".to_string(),
        "Paris".to_string(),
        "75001".to_string(),
        "France".to_string(),
        45,
        Some(1990),
    )?;

    // Sauvegarder via repository
    let saved_building = building_repository.create(building).await?;

Recherche d'immeubles
---------------------

.. code-block:: rust

    // Par ville
    let buildings = building_repository
        .find_by_city("Paris")
        .await?;

    // Par ID
    let building = building_repository
        .find_by_id(building_id)
        .await?;

Mise à jour d'informations
---------------------------

.. code-block:: rust

    // Récupérer l'immeuble
    let mut building = building_repository
        .find_by_id(building_id)
        .await?;

    // Mettre à jour
    building.update_info(
        "Nouveau Nom".to_string(),
        "Nouvelle Adresse".to_string(),
        "Paris".to_string(),
        "75002".to_string(),
    );

    // Sauvegarder
    building_repository.update(building).await?;

Tests unitaires
===============

Le fichier contient 4 tests unitaires couvrant:

.. list-table::
   :header-rows: 1
   :widths: 50 50

   * - Test
     - Scénario couvert
   * - ``test_create_building_success``
     - Création réussie avec toutes les données
   * - ``test_create_building_empty_name_fails``
     - Rejet nom vide
   * - ``test_create_building_zero_units_fails``
     - Rejet nombre de lots = 0
   * - ``test_update_building_info``
     - Mise à jour informations + timestamp

**Exécuter les tests:**

.. code-block:: bash

    cd backend
    cargo test domain::entities::building

Modèle de données
=================

**Schéma de base de données PostgreSQL:**

.. code-block:: sql

    CREATE TABLE buildings (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        name VARCHAR(255) NOT NULL CHECK (length(name) > 0),
        address VARCHAR(255) NOT NULL,
        city VARCHAR(100) NOT NULL,
        postal_code VARCHAR(20) NOT NULL,
        country VARCHAR(100) NOT NULL,
        total_units INTEGER NOT NULL CHECK (total_units > 0),
        construction_year INTEGER,
        organization_id UUID REFERENCES organizations(id),
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
    );

    CREATE INDEX idx_buildings_city ON buildings(city);
    CREATE INDEX idx_buildings_org ON buildings(organization_id);

**Mapping Rust ↔ PostgreSQL:**

.. list-table::
   :header-rows: 1
   :widths: 30 35 35

   * - Rust
     - PostgreSQL
     - Notes
   * - ``Uuid``
     - ``UUID``
     - Génération côté Rust (v4)
   * - ``String``
     - ``VARCHAR``/``TEXT``
     - Validation longueur côté Rust
   * - ``i32``
     - ``INTEGER``
     - Contrainte CHECK en base
   * - ``Option<i32>``
     - ``INTEGER NULL``
     - Nullable en base
   * - ``DateTime<Utc>``
     - ``TIMESTAMPTZ``
     - Toujours UTC

Relations avec autres entités
==============================

Un immeuble (Building) est lié à:

.. code-block:: text

    Building
        │
        ├──> Organization (1:1) - Appartient à une organisation
        │
        ├──> Units (1:N) - Contient plusieurs lots
        │
        ├──> Owners (N:M via Units) - Propriétaires via les lots
        │
        ├──> Expenses (1:N) - Charges de l'immeuble
        │
        ├──> Meetings (1:N) - Assemblées générales
        │
        └──> Documents (1:N) - Documents liés à l'immeuble

**Exemple relationnel:**

.. code-block:: rust

    // Récupérer immeuble avec tous ses lots
    let building = building_repository.find_by_id(id).await?;
    let units = unit_repository.find_by_building_id(building.id).await?;

    // Calculer occupation
    let occupied_units = units.iter().filter(|u| u.owner_id.is_some()).count();
    let occupancy_rate = (occupied_units as f64 / building.total_units as f64) * 100.0;

Intégration Multi-tenant
=========================

L'entité Building est multi-tenant via le champ ``organization_id`` (dans la base de données, pas exposé dans cette struct simplifiée).

.. code-block:: text

    ┌─────────────────────────────────────────────────────────┐
    │           Organization A (Syndic Paris)                 │
    │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
    │   │ Building 1  │  │ Building 2  │  │ Building 3  │    │
    │   │ 75001       │  │ 75002       │  │ 75003       │    │
    │   └─────────────┘  └─────────────┘  └─────────────┘    │
    └─────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────┐
    │           Organization B (Syndic Lyon)                  │
    │   ┌─────────────┐  ┌─────────────┐                      │
    │   │ Building 4  │  │ Building 5  │                      │
    │   │ 69001       │  │ 69002       │                      │
    │   └─────────────┘  └─────────────┘                      │
    └─────────────────────────────────────────────────────────┘

Dépendances
===========

Crates externes:

- ``uuid`` - Identifiants uniques
- ``chrono`` - Gestion des dates et timestamps
- ``serde`` - Sérialisation/désérialisation JSON

Modules internes:

- Aucun (entité auto-suffisante sans dépendances internes)

Évolutions possibles
====================

1. **Validation améliorée**

   .. code-block:: rust

       use validator::Validate;

       #[derive(Validate)]
       pub struct Building {
           #[validate(length(min = 3, max = 255))]
           pub name: String,

           #[validate(range(min = 1, max = 10000))]
           pub total_units: i32,

           #[validate(range(min = 1800, max = 2100))]
           pub construction_year: Option<i32>,

           // ...
       }

2. **Adresse structurée**

   .. code-block:: rust

       pub struct Address {
           pub street_number: String,
           pub street_name: String,
           pub city: String,
           pub postal_code: String,
           pub country: Country,  // Enum
       }

       pub struct Building {
           // ...
           pub address: Address,
           // ...
       }

3. **Géolocalisation**

   .. code-block:: rust

       pub struct GeoCoordinates {
           pub latitude: f64,
           pub longitude: f64,
       }

       pub struct Building {
           // ...
           pub coordinates: Option<GeoCoordinates>,
           // ...
       }

4. **Métadonnées étendues**

   .. code-block:: rust

       pub struct Building {
           // ...
           pub building_type: BuildingType,  // Enum: Residential, Commercial, Mixed
           pub floors_count: Option<i32>,
           pub has_elevator: bool,
           pub has_parking: bool,
           pub cadastral_reference: Option<String>,
           // ...
       }

Fichiers associés
=================

- ``backend/src/domain/entities/unit.rs`` - Entité Lot (Unit)
- ``backend/src/domain/entities/owner.rs`` - Entité Propriétaire
- ``backend/src/domain/entities/expense.rs`` - Entité Charge
- ``backend/src/application/ports/building_repository.rs`` - Trait repository
- ``backend/src/infrastructure/database/repositories/building_repository_impl.rs`` - Implémentation PostgreSQL
- ``backend/src/application/use_cases/building_use_cases.rs`` - Cas d'usage métier
- ``backend/src/application/dto/building_dto.rs`` - DTOs pour API
- ``backend/src/infrastructure/web/handlers/building_handlers.rs`` - Handlers HTTP
