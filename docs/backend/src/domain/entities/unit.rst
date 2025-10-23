==================================================
backend/src/domain/entities/unit.rs
==================================================

Description et Responsabilités
==================================================

Le fichier ``unit.rs`` définit l'entité de domaine **Unit** (Lot de copropriété) dans le système KoproGo. Cette entité représente les lots individuels au sein d'une copropriété (appartements, parkings, caves, locaux commerciaux) et gère leurs caractéristiques et leur propriété.

**Responsabilités principales:**

- Représenter un lot avec ses caractéristiques (type, surface, quote-part)
- Valider les données lors de la création (numéro, surface, quota)
- Gérer l'attribution et le retrait de propriétaires
- Maintenir les métadonnées temporelles (création, mise à jour)
- Stocker la quote-part (tantièmes) pour le calcul des charges

**Contexte métier:**

Dans le droit français de la copropriété, un lot (ou tantième) représente une partie privative d'un immeuble. Chaque lot possède une quote-part exprimée en millièmes qui détermine sa contribution aux charges communes. Les lots peuvent être de différents types (appartements, parkings, caves, commerces) et appartiennent à un ou plusieurs copropriétaires.

Énumérations
==================================================

UnitType
--------------------------------------------------

**Signature:**

.. code-block:: rust

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum UnitType {
        Apartment,
        Parking,
        Cellar,
        Commercial,
        Other,
    }

**Description:**

Énumération représentant les différents types de lots dans une copropriété.

**Variantes:**

.. list-table::
   :header-rows: 1
   :widths: 20 80

   * - Variante
     - Description
   * - ``Apartment``
     - Lot d'habitation (appartement, studio, maison individuelle en copropriété horizontale)
   * - ``Parking``
     - Place de stationnement (parking couvert, box, garage)
   * - ``Cellar``
     - Cave ou local de rangement
   * - ``Commercial``
     - Local commercial (boutique, bureau, entrepôt)
   * - ``Other``
     - Autre type de lot (grenier, comble, jardin privatif, etc.)

**Traits dérivés:**

- ``Debug``: Permet l'affichage pour le débogage
- ``Clone``: Permet la copie de l'énumération
- ``Serialize``: Permet la sérialisation JSON (via serde)
- ``Deserialize``: Permet la désérialisation JSON (via serde)
- ``PartialEq``: Permet la comparaison d'égalité

**Utilisation:**

.. code-block:: rust

    let unit_type = UnitType::Apartment;

    // Sérialisation JSON
    let json = serde_json::to_string(&unit_type).unwrap();
    // json = "\"Apartment\""

    // Pattern matching
    match unit_type {
        UnitType::Apartment => println!("Lot d'habitation"),
        UnitType::Parking => println!("Place de parking"),
        UnitType::Cellar => println!("Cave"),
        UnitType::Commercial => println!("Local commercial"),
        UnitType::Other => println!("Autre type"),
    }

**Notes:**

Cette énumération pourrait être étendue avec des variantes supplémentaires comme:

- ``Storage``: Local de stockage
- ``Garden``: Jardin privatif
- ``Terrace``: Terrasse privative
- ``Office``: Bureau
- ``Workshop``: Atelier

Structures et Types
==================================================

Unit
--------------------------------------------------

**Signature:**

.. code-block:: rust

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct Unit {
        pub id: Uuid,
        pub building_id: Uuid,
        pub unit_number: String,
        pub unit_type: UnitType,
        pub floor: Option<i32>,
        pub surface_area: f64,
        pub quota: f64,
        pub owner_id: Option<Uuid>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

**Description:**

Structure représentant un lot de copropriété avec toutes ses caractéristiques et son propriétaire.

**Champs:**

.. list-table::
   :header-rows: 1
   :widths: 20 20 60

   * - Champ
     - Type
     - Description
   * - ``id``
     - ``Uuid``
     - Identifiant unique généré automatiquement (UUID v4)
   * - ``building_id``
     - ``Uuid``
     - Référence vers l'immeuble auquel appartient le lot
   * - ``unit_number``
     - ``String``
     - Numéro ou identifiant du lot (ex: "A101", "Cave 12", "Parking 5")
   * - ``unit_type``
     - ``UnitType``
     - Type de lot (Apartment, Parking, Cellar, Commercial, Other)
   * - ``floor``
     - ``Option<i32>``
     - Étage du lot (None pour caves, parkings sans étage)
   * - ``surface_area``
     - ``f64``
     - Surface en m² (doit être > 0)
   * - ``quota``
     - ``f64``
     - Quote-part en millièmes (0 < quota ≤ 1000)
   * - ``owner_id``
     - ``Option<Uuid>``
     - Référence vers le propriétaire actuel (None si lot vacant)
   * - ``created_at``
     - ``DateTime<Utc>``
     - Date et heure de création de l'enregistrement
   * - ``updated_at``
     - ``DateTime<Utc>``
     - Date et heure de dernière mise à jour

**Traits dérivés:**

- ``Debug``: Permet l'affichage pour le débogage
- ``Clone``: Permet la copie de l'instance
- ``Serialize``: Permet la sérialisation JSON (via serde)
- ``Deserialize``: Permet la désérialisation JSON (via serde)
- ``PartialEq``: Permet la comparaison d'égalité

**Notes de conception:**

- **Quote-part (quota):** Exprimée en millièmes (‰), elle détermine la part de chaque lot dans les charges communes. La somme des quotes-parts de tous les lots d'un immeuble doit égaler 1000 millièmes.

- **Surface:** Mesurée selon la loi Carrez pour les lots d'habitation en France (surface plancher ≥ 1,80m de hauteur).

- **Numéro de lot:** Format libre permettant diverses conventions de numérotation (ex: "A101" pour Bâtiment A, 1er étage, lot 01).

- **Étage:** Optionnel car certains lots (caves, parkings) ne sont pas situés à un étage spécifique.

- **Propriétaire:** Optionnel car un lot peut être temporairement vacant (entre deux ventes).

Méthodes
==================================================

Unit::new
--------------------------------------------------

**Signature:**

.. code-block:: rust

    pub fn new(
        building_id: Uuid,
        unit_number: String,
        unit_type: UnitType,
        floor: Option<i32>,
        surface_area: f64,
        quota: f64,
    ) -> Result<Self, String>

**Description:**

Constructeur pour créer une nouvelle instance de Unit avec validation des données.

**Comportement:**

1. Valide que ``unit_number`` n'est pas vide
2. Valide que ``surface_area`` est strictement positive (> 0)
3. Valide que ``quota`` est dans la plage ]0, 1000]
4. Génère un nouvel UUID v4 pour ``id``
5. Capture le timestamp actuel UTC pour ``created_at`` et ``updated_at``
6. Initialise ``owner_id`` à ``None`` (lot non attribué)
7. Retourne une instance Unit si toutes les validations passent
8. Retourne une erreur descriptive si une validation échoue

**Paramètres:**

.. list-table::
   :header-rows: 1
   :widths: 20 20 60

   * - Paramètre
     - Type
     - Description
   * - ``building_id``
     - ``Uuid``
     - Identifiant de l'immeuble parent (doit exister)
   * - ``unit_number``
     - ``String``
     - Numéro/identifiant du lot (doit être non vide)
   * - ``unit_type``
     - ``UnitType``
     - Type de lot (Apartment, Parking, etc.)
   * - ``floor``
     - ``Option<i32>``
     - Étage du lot (None si non applicable)
   * - ``surface_area``
     - ``f64``
     - Surface en m² (doit être > 0.0)
   * - ``quota``
     - ``f64``
     - Quote-part en millièmes (0.0 < quota ≤ 1000.0)

**Retour:**

- ``Ok(Unit)``: Instance Unit valide avec ID généré et timestamps
- ``Err(String)``: Message d'erreur descriptif si validation échoue

**Erreurs possibles:**

- ``"Unit number cannot be empty"``: Le numéro de lot est vide
- ``"Surface area must be greater than 0"``: La surface est ≤ 0
- ``"Quota must be between 0 and 1000"``: La quote-part est ≤ 0 ou > 1000

**Exemple d'utilisation:**

.. code-block:: rust

    let building_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    // Création d'un appartement
    let apartment = Unit::new(
        building_id,
        "A101".to_string(),
        UnitType::Apartment,
        Some(1),           // 1er étage
        75.5,              // 75.5 m²
        50.0,              // 50 millièmes
    ).unwrap();

    // Création d'une cave (sans étage)
    let cellar = Unit::new(
        building_id,
        "Cave 12".to_string(),
        UnitType::Cellar,
        None,              // Pas d'étage
        8.0,               // 8 m²
        2.5,               // 2.5 millièmes
    ).unwrap();

    // Création échouée - surface invalide
    let invalid = Unit::new(
        building_id,
        "A102".to_string(),
        UnitType::Apartment,
        Some(1),
        0.0,               // Surface = 0 → ERREUR
        50.0,
    );
    assert!(invalid.is_err());
    assert_eq!(invalid.unwrap_err(), "Surface area must be greater than 0");

    // Création échouée - quota invalide
    let invalid_quota = Unit::new(
        building_id,
        "A103".to_string(),
        UnitType::Apartment,
        Some(1),
        75.0,
        1500.0,            // Quota > 1000 → ERREUR
    );
    assert!(invalid_quota.is_err());
    assert_eq!(invalid_quota.unwrap_err(), "Quota must be between 0 and 1000");

**Cas d'usage métier:**

.. code-block:: rust

    // Immeuble de 10 lots avec répartition des quotes-parts
    let building_id = Uuid::new_v4();

    // Appartements (80% des quotes-parts = 800‰)
    let apt_1 = Unit::new(building_id, "A1".to_string(), UnitType::Apartment, Some(1), 50.0, 100.0).unwrap();
    let apt_2 = Unit::new(building_id, "A2".to_string(), UnitType::Apartment, Some(2), 60.0, 120.0).unwrap();
    let apt_3 = Unit::new(building_id, "A3".to_string(), UnitType::Apartment, Some(3), 70.0, 140.0).unwrap();

    // Parkings (15% = 150‰)
    let parking_1 = Unit::new(building_id, "P1".to_string(), UnitType::Parking, Some(-1), 12.5, 50.0).unwrap();
    let parking_2 = Unit::new(building_id, "P2".to_string(), UnitType::Parking, Some(-1), 12.5, 50.0).unwrap();
    let parking_3 = Unit::new(building_id, "P3".to_string(), UnitType::Parking, Some(-1), 12.5, 50.0).unwrap();

    // Caves (5% = 50‰)
    let cellar_1 = Unit::new(building_id, "C1".to_string(), UnitType::Cellar, None, 5.0, 25.0).unwrap();
    let cellar_2 = Unit::new(building_id, "C2".to_string(), UnitType::Cellar, None, 5.0, 25.0).unwrap();

    // Total des quotes-parts = 800 + 150 + 50 = 1000‰ ✓

Unit::assign_owner
--------------------------------------------------

**Signature:**

.. code-block:: rust

    pub fn assign_owner(&mut self, owner_id: Uuid)

**Description:**

Attribue un propriétaire à ce lot.

**Comportement:**

1. Définit ``owner_id`` avec l'UUID du propriétaire fourni
2. Met à jour ``updated_at`` avec le timestamp actuel UTC

**Paramètres:**

.. list-table::
   :header-rows: 1
   :widths: 20 20 60

   * - Paramètre
     - Type
     - Description
   * - ``owner_id``
     - ``Uuid``
     - Identifiant du propriétaire à attribuer

**Retour:**

Aucun (``()``). Méthode mutatrice.

**Exemple d'utilisation:**

.. code-block:: rust

    let building_id = Uuid::new_v4();
    let mut unit = Unit::new(
        building_id,
        "A101".to_string(),
        UnitType::Apartment,
        Some(1),
        75.5,
        50.0,
    ).unwrap();

    // Initialement, pas de propriétaire
    assert_eq!(unit.owner_id, None);

    // Attribution d'un propriétaire
    let owner_id = Uuid::new_v4();
    unit.assign_owner(owner_id);

    assert_eq!(unit.owner_id, Some(owner_id));

**Cas d'usage métier:**

.. code-block:: rust

    // Scénario: Vente d'un lot
    async fn handle_unit_sale(
        unit_id: Uuid,
        new_owner_id: Uuid,
        unit_repo: &impl UnitRepository,
        transaction_repo: &impl TransactionRepository,
    ) -> Result<(), Error> {
        // 1. Récupérer le lot
        let mut unit = unit_repo.find_by_id(unit_id).await?
            .ok_or(Error::NotFound)?;

        let old_owner_id = unit.owner_id;

        // 2. Attribuer le nouveau propriétaire
        unit.assign_owner(new_owner_id);

        // 3. Sauvegarder
        unit_repo.update(unit).await?;

        // 4. Enregistrer la transaction
        transaction_repo.record_sale(unit_id, old_owner_id, Some(new_owner_id)).await?;

        Ok(())
    }

**Notes:**

- Cette méthode **ne valide pas** l'existence du propriétaire
- La validation doit être effectuée au niveau de l'use case ou du repository
- Le timestamp ``updated_at`` est automatiquement mis à jour pour traçabilité

Unit::remove_owner
--------------------------------------------------

**Signature:**

.. code-block:: rust

    pub fn remove_owner(&mut self)

**Description:**

Retire le propriétaire de ce lot (rend le lot vacant).

**Comportement:**

1. Définit ``owner_id`` à ``None``
2. Met à jour ``updated_at`` avec le timestamp actuel UTC

**Paramètres:**

Aucun

**Retour:**

Aucun (``()``). Méthode mutatrice.

**Exemple d'utilisation:**

.. code-block:: rust

    let building_id = Uuid::new_v4();
    let mut unit = Unit::new(
        building_id,
        "A101".to_string(),
        UnitType::Apartment,
        Some(1),
        75.5,
        50.0,
    ).unwrap();

    // Attribution d'un propriétaire
    let owner_id = Uuid::new_v4();
    unit.assign_owner(owner_id);
    assert_eq!(unit.owner_id, Some(owner_id));

    // Retrait du propriétaire
    unit.remove_owner();
    assert_eq!(unit.owner_id, None);

**Cas d'usage métier:**

.. code-block:: rust

    // Scénario: Succession en cours, lot temporairement sans propriétaire désigné
    async fn handle_owner_deceased(
        owner_id: Uuid,
        unit_repo: &impl UnitRepository,
    ) -> Result<(), Error> {
        // 1. Récupérer tous les lots du propriétaire décédé
        let units = unit_repo.find_by_owner_id(owner_id).await?;

        // 2. Retirer le propriétaire de chaque lot
        for mut unit in units {
            unit.remove_owner();
            unit_repo.update(unit).await?;
        }

        // 3. Les lots sont maintenant vacants en attente de succession
        Ok(())
    }

    // Scénario: Expulsion ou saisie immobilière
    async fn handle_unit_seizure(
        unit_id: Uuid,
        unit_repo: &impl UnitRepository,
    ) -> Result<(), Error> {
        let mut unit = unit_repo.find_by_id(unit_id).await?
            .ok_or(Error::NotFound)?;

        // Retrait du propriétaire suite à saisie
        unit.remove_owner();
        unit_repo.update(unit).await?;

        Ok(())
    }

**Notes:**

- Cette méthode est utile pour les cas de transition (succession, saisie, vente en cours)
- Un lot sans propriétaire peut poser des questions de gestion des charges
- Le système devrait avoir une règle métier pour gérer les lots vacants

Tests
==================================================

Le fichier contient **3 tests unitaires** dans le module ``tests``:

test_create_unit_success
--------------------------------------------------

**Description:**

Vérifie la création réussie d'un Unit avec toutes les données valides.

**Ce qui est testé:**

.. code-block:: rust

    #[test]
    fn test_create_unit_success() {
        let building_id = Uuid::new_v4();
        let unit = Unit::new(
            building_id,
            "A101".to_string(),
            UnitType::Apartment,
            Some(1),
            75.5,
            50.0,
        );

        assert!(unit.is_ok());
        let unit = unit.unwrap();
        assert_eq!(unit.unit_number, "A101");
        assert_eq!(unit.surface_area, 75.5);
    }

**Assertions:**

1. ✅ La création retourne ``Ok``
2. ✅ Le numéro de lot est correct
3. ✅ La surface est correcte

test_create_unit_invalid_surface_fails
--------------------------------------------------

**Description:**

Vérifie que la création échoue avec une surface invalide (≤ 0).

**Ce qui est testé:**

.. code-block:: rust

    #[test]
    fn test_create_unit_invalid_surface_fails() {
        let building_id = Uuid::new_v4();
        let unit = Unit::new(
            building_id,
            "A101".to_string(),
            UnitType::Apartment,
            Some(1),
            0.0,
            50.0,
        );

        assert!(unit.is_err());
    }

**Assertions:**

1. ✅ La création retourne ``Err`` avec surface = 0

test_assign_owner
--------------------------------------------------

**Description:**

Vérifie l'attribution d'un propriétaire à un lot.

**Ce qui est testé:**

.. code-block:: rust

    #[test]
    fn test_assign_owner() {
        let building_id = Uuid::new_v4();
        let mut unit = Unit::new(
            building_id,
            "A101".to_string(),
            UnitType::Apartment,
            Some(1),
            75.5,
            50.0,
        )
        .unwrap();

        let owner_id = Uuid::new_v4();
        unit.assign_owner(owner_id);

        assert_eq!(unit.owner_id, Some(owner_id));
    }

**Assertions:**

1. ✅ Le propriétaire est correctement attribué

Couverture des Tests
--------------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 40 30 30

   * - Fonctionnalité
     - Testée
     - Cas de test
   * - Création avec données valides
     - ✅
     - ``test_create_unit_success``
   * - Validation surface invalide (≤ 0)
     - ✅
     - ``test_create_unit_invalid_surface_fails``
   * - Attribution d'un propriétaire
     - ✅
     - ``test_assign_owner``
   * - Validation numéro vide
     - ❌
     - Manquant
   * - Validation quota invalide (≤ 0)
     - ❌
     - Manquant
   * - Validation quota invalide (> 1000)
     - ❌
     - Manquant
   * - Retrait d'un propriétaire
     - ❌
     - Manquant
   * - Génération UUID unique
     - ❌
     - Manquant
   * - Timestamps automatiques
     - ❌
     - Manquant
   * - Mise à jour de updated_at
     - ❌
     - Manquant

**Tests manquants recommandés:**

.. code-block:: rust

    #[test]
    fn test_create_unit_empty_number_fails() {
        let result = Unit::new(
            Uuid::new_v4(),
            "".to_string(),
            UnitType::Apartment,
            Some(1),
            75.0,
            50.0,
        );
        assert_eq!(result.unwrap_err(), "Unit number cannot be empty");
    }

    #[test]
    fn test_create_unit_invalid_quota_too_low_fails() {
        let result = Unit::new(
            Uuid::new_v4(),
            "A101".to_string(),
            UnitType::Apartment,
            Some(1),
            75.0,
            0.0,  // Quota = 0 → invalide
        );
        assert_eq!(result.unwrap_err(), "Quota must be between 0 and 1000");
    }

    #[test]
    fn test_create_unit_invalid_quota_too_high_fails() {
        let result = Unit::new(
            Uuid::new_v4(),
            "A101".to_string(),
            UnitType::Apartment,
            Some(1),
            75.0,
            1500.0,  // Quota > 1000 → invalide
        );
        assert_eq!(result.unwrap_err(), "Quota must be between 0 and 1000");
    }

    #[test]
    fn test_remove_owner() {
        let mut unit = Unit::new(
            Uuid::new_v4(),
            "A101".to_string(),
            UnitType::Apartment,
            Some(1),
            75.0,
            50.0,
        ).unwrap();

        let owner_id = Uuid::new_v4();
        unit.assign_owner(owner_id);
        assert_eq!(unit.owner_id, Some(owner_id));

        unit.remove_owner();
        assert_eq!(unit.owner_id, None);
    }

    #[test]
    fn test_updated_at_changes_on_assign_owner() {
        let mut unit = Unit::new(...).unwrap();
        let initial_time = unit.updated_at;

        std::thread::sleep(std::time::Duration::from_millis(10));
        unit.assign_owner(Uuid::new_v4());

        assert!(unit.updated_at > initial_time);
    }

Architecture et Diagrammes
==================================================

Relation avec les autres entités
--------------------------------------------------

.. code-block:: text

    ┌─────────────────┐
    │  Organization   │
    │   (Syndic)      │
    └────────┬────────┘
             │
             │ 1:N
             │
    ┌────────▼────────┐
    │    Building     │
    │  (Immeuble)     │
    └────────┬────────┘
             │
             │ 1:N
             │
    ┌────────▼────────┐       ┌──────────────┐
    │      Unit    ◄──────────┤    Owner     │
    │     (Lot)       │   N:1  │(Copropriétaire)│
    └────────┬────────┘        └──────────────┘
             │
             │ 1:N
             │
    ┌────────▼────────┐
    │    Expense      │
    │   (Charge)      │
    └─────────────────┘

**Relations:**

- Un Unit appartient à un Building (relation N:1 via ``building_id``)
- Un Unit peut avoir un Owner (relation N:1 via ``owner_id``)
- Un Owner peut posséder plusieurs Units (relation 1:N)
- Un Unit génère des Expenses (charges) selon sa quote-part

Modèle de calcul des charges
--------------------------------------------------

.. code-block:: text

    [Charge commune totale: 10,000 €]
                │
                ▼
    ┌───────────────────────────┐
    │  Répartition par quote-   │
    │  part (millièmes)         │
    └───────────┬───────────────┘
                │
                ├─► Unit A (100‰) → 10,000 € × (100/1000) = 1,000 €
                ├─► Unit B (150‰) → 10,000 € × (150/1000) = 1,500 €
                ├─► Unit C (50‰)  → 10,000 € × (50/1000)  = 500 €
                └─► ... (total = 1000‰)

**Formule:**

.. code-block:: text

    Charge du lot = Charge totale × (quota du lot / 1000)

Cycle de vie d'un Unit
--------------------------------------------------

.. code-block:: text

    [Création]
        │
        ├─► Validation (numéro, surface, quota)
        ├─► Génération UUID
        ├─► owner_id = None (lot vacant)
        │
        ▼
    [Lot vacant]
        │
        ├─► assign_owner(owner_id)
        │
        ▼
    [Lot attribué]
        │
        ├─► Calcul des charges
        ├─► Association aux dépenses
        ├─► Génération de documents
        │
        ├─► Vente / Transfert
        │   ├─► remove_owner()
        │   └─► assign_owner(new_owner_id)
        │
        ├─► Modification (surface, quota)
        │
        ▼
    [Lot actif en copropriété]

Utilisation dans l'Application
==================================================

Création d'un Unit (Use Case)
--------------------------------------------------

**Couche Application - Use Case:**

.. code-block:: rust

    // backend/src/application/use_cases/create_unit.rs
    pub async fn execute(
        repo: &impl UnitRepository,
        building_repo: &impl BuildingRepository,
        dto: CreateUnitDto,
    ) -> Result<UnitDto, ApplicationError> {
        // 1. Vérifier que le building existe
        building_repo.find_by_id(dto.building_id).await?
            .ok_or(ApplicationError::NotFound("Building not found".to_string()))?;

        // 2. Créer l'entité Unit (validation automatique)
        let unit = Unit::new(
            dto.building_id,
            dto.unit_number,
            dto.unit_type,
            dto.floor,
            dto.surface_area,
            dto.quota,
        ).map_err(|e| ApplicationError::ValidationError(e))?;

        // 3. Vérifier l'unicité du numéro de lot dans le building
        if repo.exists_by_building_and_number(dto.building_id, &unit.unit_number).await? {
            return Err(ApplicationError::DuplicateError("Unit number already exists".to_string()));
        }

        // 4. Persister dans la base de données
        let saved_unit = repo.create(unit).await?;

        Ok(UnitDto::from(saved_unit))
    }

Calcul de la charge d'un Unit
--------------------------------------------------

**Couche Domain - Service:**

.. code-block:: rust

    // backend/src/domain/services/expense_calculator.rs
    pub fn calculate_unit_expense(
        total_expense: f64,
        unit_quota: f64,
    ) -> f64 {
        // Charge du lot = Charge totale × (quota / 1000)
        total_expense * (unit_quota / 1000.0)
    }

    // Exemple d'utilisation
    let total_expense = 10_000.0;  // 10,000 €
    let unit_quota = 50.0;          // 50 millièmes

    let unit_expense = calculate_unit_expense(total_expense, unit_quota);
    // unit_expense = 500.0 € (10,000 × 50/1000)

Récupération des Units d'un Building
--------------------------------------------------

**Couche Application - Use Case:**

.. code-block:: rust

    // backend/src/application/use_cases/list_building_units.rs
    pub async fn execute(
        repo: &impl UnitRepository,
        building_id: Uuid,
    ) -> Result<Vec<UnitDto>, ApplicationError> {
        let units = repo.find_by_building_id(building_id).await?;

        Ok(units.into_iter()
            .map(|unit| UnitDto::from(unit))
            .collect())
    }

Transfert de propriété
--------------------------------------------------

**Couche Application - Use Case:**

.. code-block:: rust

    // backend/src/application/use_cases/transfer_unit_ownership.rs
    pub async fn execute(
        unit_repo: &impl UnitRepository,
        unit_id: Uuid,
        new_owner_id: Uuid,
    ) -> Result<UnitDto, ApplicationError> {
        // 1. Récupérer le unit
        let mut unit = unit_repo.find_by_id(unit_id).await?
            .ok_or(ApplicationError::NotFound("Unit not found".to_string()))?;

        // 2. Attribuer le nouveau propriétaire
        unit.assign_owner(new_owner_id);

        // 3. Sauvegarder
        let updated_unit = unit_repo.update(unit).await?;

        Ok(UnitDto::from(updated_unit))
    }

Dépendances
==================================================

Dépendances Externes
--------------------------------------------------

.. code-block:: rust

    use chrono::{DateTime, Utc};  // Gestion des dates et timestamps UTC
    use serde::{Deserialize, Serialize};  // Sérialisation/désérialisation JSON
    use uuid::Uuid;  // Génération et manipulation d'UUID v4

Dépendances Internes
--------------------------------------------------

Cette entité dépend de:

- **Building** (via ``building_id``)
- **Owner** (via ``owner_id``)

Elle est utilisée par:

.. code-block:: text

    backend/src/domain/entities/unit.rs
            ▲
            │ used by
            │
            ├─► backend/src/application/dto/unit_dto.rs
            ├─► backend/src/application/ports/unit_repository.rs
            ├─► backend/src/application/use_cases/create_unit.rs
            ├─► backend/src/application/use_cases/get_unit.rs
            ├─► backend/src/application/use_cases/list_building_units.rs
            ├─► backend/src/application/use_cases/transfer_unit_ownership.rs
            ├─► backend/src/domain/services/expense_calculator.rs
            ├─► backend/src/infrastructure/repositories/postgres_unit_repository.rs
            └─► backend/src/web/handlers/units.rs

Notes de Conception
==================================================

Système de Quote-part (Tantièmes)
--------------------------------------------------

**Principe:**

En France, la quote-part est exprimée en **millièmes** (‰). La somme des quotes-parts de tous les lots d'un immeuble doit toujours égaler 1000 millièmes.

**Calcul:**

La quote-part est généralement calculée en fonction:

1. De la **surface** du lot (loi Carrez)
2. De la **situation** (étage, orientation, vue)
3. De la **destination** (habitation, commerce, parking)

**Exemple:**

.. code-block:: text

    Immeuble de 3 lots:

    - Lot A: 100 m², appartement, 2ème étage → 500‰
    - Lot B: 75 m², appartement, 1er étage  → 400‰
    - Lot C: 12 m², parking sous-sol        → 100‰

    Total: 500 + 400 + 100 = 1000‰ ✓

**Utilisation:**

.. code-block:: rust

    // Calcul de la charge annuelle d'un lot
    fn calculate_annual_charge(total_building_charges: f64, unit_quota: f64) -> f64 {
        total_building_charges * (unit_quota / 1000.0)
    }

    // Exemple:
    let total_charges = 50_000.0;  // 50,000 € de charges pour l'immeuble
    let unit_quota = 50.0;          // Lot avec 50 millièmes

    let annual_charge = calculate_annual_charge(total_charges, unit_quota);
    // annual_charge = 2,500 € (50,000 × 50/1000)

Validation de la Surface
--------------------------------------------------

**Problème:**

La validation actuelle accepte toute surface > 0, mais ne vérifie pas la cohérence métier.

**Améliorations potentielles:**

.. code-block:: rust

    pub fn new(..., surface_area: f64, ...) -> Result<Self, String> {
        // Validation basique
        if surface_area <= 0.0 {
            return Err("Surface area must be greater than 0".to_string());
        }

        // Validation métier selon le type
        match unit_type {
            UnitType::Apartment if surface_area < 9.0 => {
                return Err("Apartment must have at least 9m² (loi Carrez)".to_string());
            }
            UnitType::Parking if surface_area > 50.0 => {
                return Err("Parking cannot exceed 50m²".to_string());
            }
            UnitType::Cellar if surface_area > 30.0 => {
                return Err("Cellar cannot exceed 30m²".to_string());
            }
            _ => {}
        }

        // ...
    }

Unicité du Numéro de Lot
--------------------------------------------------

**Problème:**

La méthode ``new()`` ne vérifie pas l'unicité du ``unit_number`` dans le building.

**Solution:**

La vérification d'unicité doit être effectuée au niveau du **repository** ou de l'**use case**, car elle nécessite un accès à la base de données.

.. code-block:: rust

    // Dans le repository
    async fn exists_by_building_and_number(
        &self,
        building_id: Uuid,
        unit_number: &str,
    ) -> Result<bool, RepositoryError>;

    // Dans l'use case
    if repo.exists_by_building_and_number(building_id, &unit_number).await? {
        return Err(ApplicationError::DuplicateError("Unit number already exists".to_string()));
    }

**Contrainte base de données:**

.. code-block:: sql

    CREATE UNIQUE INDEX idx_units_building_number
    ON units(building_id, unit_number);

Gestion des Lots Vacants
--------------------------------------------------

**Problème:**

Un lot sans propriétaire (``owner_id = None``) pose des questions:

1. Qui paie les charges?
2. Comment gérer les assemblées générales?
3. Comment envoyer les documents?

**Solutions:**

.. code-block:: rust

    pub struct Unit {
        // ...
        pub owner_id: Option<Uuid>,
        pub temporary_manager_id: Option<Uuid>,  // Administrateur temporaire
        pub is_vacant: bool,  // Indicateur explicite
    }

    impl Unit {
        pub fn is_vacant(&self) -> bool {
            self.owner_id.is_none()
        }

        pub fn assign_temporary_manager(&mut self, manager_id: Uuid) {
            self.temporary_manager_id = Some(manager_id);
            self.updated_at = Utc::now();
        }
    }

**Règles métier:**

- Les charges d'un lot vacant peuvent être gérées par le syndic
- Un administrateur temporaire peut être nommé en cas de succession longue

Immuabilité des Caractéristiques
--------------------------------------------------

**Observation:**

Tous les champs sont ``pub`` (publics et modifiables), permettant des modifications directes non contrôlées.

**Recommandation:**

.. code-block:: rust

    pub struct Unit {
        id: Uuid,  // Privé - immuable
        building_id: Uuid,  // Privé - immuable
        unit_number: String,  // Privé - modifiable via méthode
        unit_type: UnitType,  // Privé - immuable (ou très rarement modifiable)
        floor: Option<i32>,  // Privé - immuable
        surface_area: f64,  // Privé - modifiable via méthode (rare)
        quota: f64,  // Privé - modifiable via méthode (assemblée générale)
        owner_id: Option<Uuid>,  // Privé - géré par assign_owner/remove_owner
        created_at: DateTime<Utc>,  // Privé - immuable
        updated_at: DateTime<Utc>,  // Privé - géré automatiquement
    }

    impl Unit {
        pub fn id(&self) -> Uuid { self.id }
        pub fn unit_number(&self) -> &str { &self.unit_number }
        pub fn quota(&self) -> f64 { self.quota }
        // ... autres getters

        pub fn update_quota(&mut self, new_quota: f64) -> Result<(), String> {
            if new_quota <= 0.0 || new_quota > 1000.0 {
                return Err("Quota must be between 0 and 1000".to_string());
            }
            self.quota = new_quota;
            self.updated_at = Utc::now();
            Ok(())
        }

        pub fn update_surface(&mut self, new_surface: f64) -> Result<(), String> {
            if new_surface <= 0.0 {
                return Err("Surface must be greater than 0".to_string());
            }
            self.surface_area = new_surface;
            self.updated_at = Utc::now();
            Ok(())
        }
    }

Avertissements
==================================================

⚠️ **Validation Quote-part Incomplète**

La validation actuelle vérifie que ``quota ≤ 1000``, mais ne garantit pas que la **somme des quotes-parts** de tous les lots d'un building égale 1000.

**Recommandation:** Implémenter une validation au niveau du Building ou de l'use case.

⚠️ **Pas de Validation d'Unicité**

Le ``unit_number`` peut être dupliqué dans le même building.

**Recommandation:** Contrainte UNIQUE en base de données + vérification dans l'use case.

⚠️ **Champs Publics**

Tous les champs sont publics, permettant des modifications directes non contrôlées.

**Recommandation:** Rendre les champs privés et exposer via getters/setters.

⚠️ **Pas de Gestion des Lots Vacants**

Il n'y a pas de logique métier pour gérer les lots sans propriétaire.

**Recommandation:** Ajouter ``temporary_manager_id`` ou ``is_vacant`` flag.

⚠️ **Pas de Validation de Cohérence**

La surface n'est pas validée selon le type de lot (ex: appartement < 9m²).

**Recommandation:** Ajouter des règles de validation métier par type.

⚠️ **Modification de building_id Possible**

Le champ ``building_id`` est public et peut être modifié, ce qui n'a aucun sens métier.

**Recommandation:** Rendre ce champ immuable (privé sans setter).

Fichiers Associés
==================================================

.. code-block:: text

    backend/src/
    ├── domain/
    │   ├── entities/
    │   │   ├── unit.rs                     ← CE FICHIER
    │   │   ├── building.rs                 (parent)
    │   │   └── owner.rs                    (propriétaire)
    │   │
    │   └── services/
    │       └── expense_calculator.rs       (calcul charges)
    │
    ├── application/
    │   ├── dto/
    │   │   └── unit_dto.rs                 (représentation DTO)
    │   │
    │   ├── ports/
    │   │   └── unit_repository.rs          (trait repository)
    │   │
    │   └── use_cases/
    │       ├── create_unit.rs              (création)
    │       ├── get_unit.rs                 (récupération)
    │       ├── list_building_units.rs      (listing)
    │       └── transfer_unit_ownership.rs  (transfert)
    │
    ├── infrastructure/
    │   └── repositories/
    │       └── postgres_unit_repository.rs (implémentation PostgreSQL)
    │
    └── web/
        └── handlers/
            └── units.rs                    (endpoints API REST)

Base de Données (Schema SQL)
--------------------------------------------------

.. code-block:: sql

    -- migrations/XXXXXX_create_units_table.sql
    CREATE TABLE units (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
        unit_number VARCHAR(50) NOT NULL,
        unit_type VARCHAR(50) NOT NULL CHECK (unit_type IN ('Apartment', 'Parking', 'Cellar', 'Commercial', 'Other')),
        floor INTEGER,
        surface_area DOUBLE PRECISION NOT NULL CHECK (surface_area > 0),
        quota DOUBLE PRECISION NOT NULL CHECK (quota > 0 AND quota <= 1000),
        owner_id UUID REFERENCES owners(id) ON DELETE SET NULL,
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

        -- Contraintes
        CONSTRAINT units_building_number_unique UNIQUE (building_id, unit_number)
    );

    -- Index pour recherche par building
    CREATE INDEX idx_units_building_id ON units(building_id);

    -- Index pour recherche par propriétaire
    CREATE INDEX idx_units_owner_id ON units(owner_id);

    -- Index pour recherche par type
    CREATE INDEX idx_units_type ON units(unit_type);

Points d'Amélioration Suggérés
==================================================

1. **Validation Quote-part Globale**

   Vérifier que la somme des quotes-parts d'un building = 1000‰

2. **Champs Privés**

   Encapsuler les champs et exposer via getters/setters

3. **Tests Complets**

   Ajouter tests pour validation quota, numéro vide, remove_owner

4. **Validation Métier par Type**

   Surface minimale pour appartements (9m² loi Carrez), maximale pour parkings

5. **Gestion Lots Vacants**

   Ajouter ``temporary_manager_id`` ou flag ``is_vacant``

6. **Historique des Propriétaires**

   Créer une table ``unit_ownership_history`` pour traçabilité

7. **Événements de Domaine**

   Émettre ``UnitCreated``, ``OwnerAssigned``, ``OwnerRemoved``

8. **Value Objects**

   Créer ``Quota`` value object avec validation encapsulée

9. **Méthode update_quota**

   Permettre la modification de la quote-part (assemblée générale)

10. **Documentation Inline**

    Ajouter doc comments Rust (``///``) pour rustdoc
