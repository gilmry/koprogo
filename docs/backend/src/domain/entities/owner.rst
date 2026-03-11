==================================================
backend/src/domain/entities/owner.rs
==================================================

Description et Responsabilités
==================================================

Le fichier ``owner.rs`` définit l'entité de domaine **Owner** (Copropriétaire) dans le système KoproGo. Cette entité représente les propriétaires d'unités dans une copropriété et gère leurs informations personnelles et de contact.

**Responsabilités principales:**

- Représenter l'identité d'un copropriétaire avec ses informations complètes
- Valider les données lors de la création (noms, email)
- Gérer les informations de contact (email, téléphone)
- Fournir une représentation formatée du nom complet
- Maintenir les métadonnées temporelles (création, mise à jour)

**Contexte métier:**

Un Owner est une personne physique propriétaire d'une ou plusieurs unités (lots) dans une copropriété. Chaque owner possède des informations d'identité complètes (nom, prénom) et de contact (email, téléphone, adresse postale complète). Ces informations sont essentielles pour la communication avec le syndic et pour l'envoi de documents officiels.

Structures et Types
==================================================

Owner
--------------------------------------------------

**Signature:**

.. code-block:: rust

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct Owner {
        pub id: Uuid,
        pub first_name: String,
        pub last_name: String,
        pub email: String,
        pub phone: Option<String>,
        pub address: String,
        pub city: String,
        pub postal_code: String,
        pub country: String,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

**Description:**

Structure représentant un copropriétaire avec toutes ses informations personnelles et de contact.

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
   * - ``first_name``
     - ``String``
     - Prénom du copropriétaire (obligatoire, non vide)
   * - ``last_name``
     - ``String``
     - Nom de famille du copropriétaire (obligatoire, non vide)
   * - ``email``
     - ``String``
     - Adresse email de contact (obligatoire, validée)
   * - ``phone``
     - ``Option<String>``
     - Numéro de téléphone (optionnel)
   * - ``address``
     - ``String``
     - Adresse postale (rue, numéro)
   * - ``city``
     - ``String``
     - Ville de résidence
   * - ``postal_code``
     - ``String``
     - Code postal
   * - ``country``
     - ``String``
     - Pays de résidence
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

- L'adresse est structurée en plusieurs champs séparés (address, city, postal_code, country) pour faciliter les recherches et le tri géographique
- Le téléphone est optionnel car tous les copropriétaires ne souhaitent pas fournir cette information
- L'email est obligatoire et validé car c'est le canal principal de communication
- Les timestamps permettent de tracer l'historique des modifications

Méthodes
==================================================

Owner::new
--------------------------------------------------

**Signature:**

.. code-block:: rust

    pub fn new(
        first_name: String,
        last_name: String,
        email: String,
        phone: Option<String>,
        address: String,
        city: String,
        postal_code: String,
        country: String,
    ) -> Result<Self, String>

**Description:**

Constructeur pour créer une nouvelle instance d'Owner avec validation des données.

**Comportement:**

1. Valide que ``first_name`` n'est pas vide
2. Valide que ``last_name`` n'est pas vide
3. Valide le format de l'email avec ``is_valid_email()``
4. Génère un nouvel UUID v4 pour ``id``
5. Capture le timestamp actuel UTC pour ``created_at`` et ``updated_at``
6. Retourne une instance Owner si toutes les validations passent
7. Retourne une erreur descriptive si une validation échoue

**Paramètres:**

.. list-table::
   :header-rows: 1
   :widths: 20 20 60

   * - Paramètre
     - Type
     - Description
   * - ``first_name``
     - ``String``
     - Prénom du copropriétaire (doit être non vide)
   * - ``last_name``
     - ``String``
     - Nom de famille (doit être non vide)
   * - ``email``
     - ``String``
     - Adresse email (doit contenir @ et .)
   * - ``phone``
     - ``Option<String>``
     - Numéro de téléphone optionnel
   * - ``address``
     - ``String``
     - Adresse postale complète
   * - ``city``
     - ``String``
     - Ville de résidence
   * - ``postal_code``
     - ``String``
     - Code postal
   * - ``country``
     - ``String``
     - Pays de résidence

**Retour:**

- ``Ok(Owner)``: Instance Owner valide avec ID généré et timestamps
- ``Err(String)``: Message d'erreur descriptif si validation échoue

**Erreurs possibles:**

- ``"First name cannot be empty"``: Le prénom est vide
- ``"Last name cannot be empty"``: Le nom est vide
- ``"Invalid email format"``: L'email ne contient pas @ ou .

**Exemple d'utilisation:**

.. code-block:: rust

    // Création réussie
    let owner = Owner::new(
        "Jean".to_string(),
        "Dupont".to_string(),
        "jean.dupont@example.com".to_string(),
        Some("+33612345678".to_string()),
        "123 Rue de la Paix".to_string(),
        "Paris".to_string(),
        "75001".to_string(),
        "France".to_string(),
    ).unwrap();

    println!("Owner créé: {} (ID: {})", owner.full_name(), owner.id);

    // Création échouée - email invalide
    let result = Owner::new(
        "Marie".to_string(),
        "Martin".to_string(),
        "invalid-email".to_string(),
        None,
        "45 Avenue des Champs".to_string(),
        "Lyon".to_string(),
        "69001".to_string(),
        "France".to_string(),
    );

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Invalid email format");

Owner::is_valid_email
--------------------------------------------------

**Signature:**

.. code-block:: rust

    fn is_valid_email(email: &str) -> bool

**Description:**

Méthode privée de validation basique du format email.

**Comportement:**

1. Vérifie la présence du caractère ``@`` dans l'email
2. Vérifie la présence du caractère ``.`` dans l'email
3. Retourne ``true`` si les deux conditions sont remplies

**Paramètres:**

.. list-table::
   :header-rows: 1
   :widths: 20 20 60

   * - Paramètre
     - Type
     - Description
   * - ``email``
     - ``&str``
     - Chaîne de caractères à valider

**Retour:**

- ``bool``: ``true`` si l'email contient @ et ., ``false`` sinon

**Notes d'implémentation:**

Cette validation est **délibérément basique** et ne suit pas strictement la RFC 5322. Elle accepte les formats suivants:

- ✅ ``user@example.com`` → valide
- ✅ ``firstname.lastname@company.co.uk`` → valide
- ❌ ``invalid-email`` → invalide (pas de @)
- ❌ ``missing-domain@`` → invalide (pas de .)
- ❌ ``@no-user.com`` → valide (mais incorrecte sémantiquement)

**Justification:**

Une validation stricte de l'email nécessiterait une bibliothèque externe (comme ``email_address``). Cette validation simple empêche les erreurs de saisie évidentes tout en restant permissive pour les cas edge valides.

**Exemple d'utilisation:**

.. code-block:: rust

    assert!(Owner::is_valid_email("user@example.com"));
    assert!(Owner::is_valid_email("firstname.lastname@company.co.uk"));
    assert!(!Owner::is_valid_email("invalid-email"));
    assert!(!Owner::is_valid_email("missing@domain"));

Owner::full_name
--------------------------------------------------

**Signature:**

.. code-block:: rust

    pub fn full_name(&self) -> String

**Description:**

Retourne le nom complet formaté du copropriétaire.

**Comportement:**

Concatène ``first_name`` et ``last_name`` avec un espace entre les deux.

**Paramètres:**

Aucun (méthode sur ``&self``)

**Retour:**

``String``: Nom complet au format ``"Prénom Nom"``

**Exemple d'utilisation:**

.. code-block:: rust

    let owner = Owner::new(
        "Jean".to_string(),
        "Dupont".to_string(),
        "jean.dupont@example.com".to_string(),
        None,
        "123 Rue de la Paix".to_string(),
        "Paris".to_string(),
        "75001".to_string(),
        "France".to_string(),
    ).unwrap();

    assert_eq!(owner.full_name(), "Jean Dupont");

    // Utilisation dans un template
    println!("Bonjour {}", owner.full_name()); // "Bonjour Jean Dupont"

**Notes:**

Cette méthode est particulièrement utile pour:

- Affichage dans les interfaces utilisateur
- Génération de documents (convocations, courriers)
- Logs et traces d'audit
- Export CSV/Excel

Owner::update_contact
--------------------------------------------------

**Signature:**

.. code-block:: rust

    pub fn update_contact(&mut self, email: String, phone: Option<String>) -> Result<(), String>

**Description:**

Met à jour les informations de contact (email et téléphone) avec validation.

**Comportement:**

1. Valide le nouveau format email avec ``is_valid_email()``
2. Si valide, met à jour ``email`` et ``phone``
3. Met à jour ``updated_at`` avec le timestamp actuel
4. Retourne ``Ok(())`` si succès, ``Err`` si email invalide

**Paramètres:**

.. list-table::
   :header-rows: 1
   :widths: 20 20 60

   * - Paramètre
     - Type
     - Description
   * - ``email``
     - ``String``
     - Nouvelle adresse email (validée)
   * - ``phone``
     - ``Option<String>``
     - Nouveau numéro de téléphone (peut être None)

**Retour:**

- ``Ok(())``: Mise à jour effectuée avec succès
- ``Err(String)``: Erreur de validation avec message

**Erreurs possibles:**

- ``"Invalid email format"``: Le nouvel email ne contient pas @ ou .

**Exemple d'utilisation:**

.. code-block:: rust

    let mut owner = Owner::new(
        "Jean".to_string(),
        "Dupont".to_string(),
        "jean.dupont@example.com".to_string(),
        None,
        "123 Rue de la Paix".to_string(),
        "Paris".to_string(),
        "75001".to_string(),
        "France".to_string(),
    ).unwrap();

    // Mise à jour réussie
    let result = owner.update_contact(
        "new.email@example.com".to_string(),
        Some("+33699999999".to_string()),
    );

    assert!(result.is_ok());
    assert_eq!(owner.email, "new.email@example.com");
    assert_eq!(owner.phone, Some("+33699999999".to_string()));

    // Mise à jour échouée - email invalide
    let result = owner.update_contact(
        "invalid".to_string(),
        None,
    );

    assert!(result.is_err());
    assert_eq!(owner.email, "new.email@example.com"); // Inchangé

**Notes:**

- Cette méthode modifie l'instance (``&mut self``)
- Le timestamp ``updated_at`` est automatiquement mis à jour
- Le téléphone peut être supprimé en passant ``None``
- L'adresse postale complète ne peut pas être modifiée par cette méthode (nécessiterait une méthode dédiée)

Tests
==================================================

Le fichier contient **3 tests unitaires** dans le module ``tests``:

test_create_owner_success
--------------------------------------------------

**Description:**

Vérifie la création réussie d'un Owner avec toutes les données valides.

**Ce qui est testé:**

.. code-block:: rust

    #[test]
    fn test_create_owner_success() {
        let owner = Owner::new(
            "Jean".to_string(),
            "Dupont".to_string(),
            "jean.dupont@example.com".to_string(),
            Some("+33612345678".to_string()),
            "123 Rue de la Paix".to_string(),
            "Paris".to_string(),
            "75001".to_string(),
            "France".to_string(),
        );

        assert!(owner.is_ok());
        let owner = owner.unwrap();
        assert_eq!(owner.full_name(), "Jean Dupont");
    }

**Assertions:**

1. ✅ La création retourne ``Ok``
2. ✅ Le nom complet est correctement formaté

test_create_owner_invalid_email_fails
--------------------------------------------------

**Description:**

Vérifie que la création échoue avec un email invalide.

**Ce qui est testé:**

.. code-block:: rust

    #[test]
    fn test_create_owner_invalid_email_fails() {
        let owner = Owner::new(
            "Jean".to_string(),
            "Dupont".to_string(),
            "invalid-email".to_string(),
            None,
            "123 Rue de la Paix".to_string(),
            "Paris".to_string(),
            "75001".to_string(),
            "France".to_string(),
        );

        assert!(owner.is_err());
        assert_eq!(owner.unwrap_err(), "Invalid email format");
    }

**Assertions:**

1. ✅ La création retourne ``Err``
2. ✅ Le message d'erreur est correct

test_update_contact
--------------------------------------------------

**Description:**

Vérifie la mise à jour des informations de contact.

**Ce qui est testé:**

.. code-block:: rust

    #[test]
    fn test_update_contact() {
        let mut owner = Owner::new(
            "Jean".to_string(),
            "Dupont".to_string(),
            "jean.dupont@example.com".to_string(),
            None,
            "123 Rue de la Paix".to_string(),
            "Paris".to_string(),
            "75001".to_string(),
            "France".to_string(),
        )
        .unwrap();

        let result = owner.update_contact(
            "new.email@example.com".to_string(),
            Some("+33699999999".to_string()),
        );

        assert!(result.is_ok());
        assert_eq!(owner.email, "new.email@example.com");
    }

**Assertions:**

1. ✅ La mise à jour retourne ``Ok``
2. ✅ L'email est correctement modifié

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
     - ``test_create_owner_success``
   * - Validation email invalide
     - ✅
     - ``test_create_owner_invalid_email_fails``
   * - Mise à jour contact
     - ✅
     - ``test_update_contact``
   * - Validation prénom vide
     - ❌
     - Manquant
   * - Validation nom vide
     - ❌
     - Manquant
   * - Mise à jour avec email invalide
     - ❌
     - Manquant
   * - Génération UUID unique
     - ❌
     - Manquant
   * - Timestamps automatiques
     - ❌
     - Manquant

**Tests manquants recommandés:**

.. code-block:: rust

    #[test]
    fn test_create_owner_empty_first_name_fails() {
        let result = Owner::new(
            "".to_string(),
            "Dupont".to_string(),
            "jean.dupont@example.com".to_string(),
            None,
            "123 Rue".to_string(),
            "Paris".to_string(),
            "75001".to_string(),
            "France".to_string(),
        );
        assert_eq!(result.unwrap_err(), "First name cannot be empty");
    }

    #[test]
    fn test_create_owner_empty_last_name_fails() {
        let result = Owner::new(
            "Jean".to_string(),
            "".to_string(),
            "jean.dupont@example.com".to_string(),
            None,
            "123 Rue".to_string(),
            "Paris".to_string(),
            "75001".to_string(),
            "France".to_string(),
        );
        assert_eq!(result.unwrap_err(), "Last name cannot be empty");
    }

    #[test]
    fn test_update_contact_invalid_email_fails() {
        let mut owner = Owner::new(...).unwrap();
        let result = owner.update_contact("invalid".to_string(), None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid email format");
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
    ┌────────▼────────┐
    │      Unit       │
    │     (Lot)       │
    └────────┬────────┘
             │
             │ N:1
             │
    ┌────────▼────────┐       ┌──────────────┐
    │     Owner    ◄──────────┤    User      │
    │ (Copropriétaire)│   1:1  │ (Compte)     │
    └─────────────────┘        └──────────────┘

**Relations:**

- Un Owner peut posséder plusieurs Units (relation 1:N)
- Un Owner est associé à un User (compte d'authentification)
- Les Units appartiennent à des Buildings gérés par une Organization

Cycle de vie d'un Owner
--------------------------------------------------

.. code-block:: text

    [Création]
        │
        ├─► Validation prénom/nom/email
        ├─► Génération UUID
        ├─► Timestamp created_at/updated_at
        │
        ▼
    [Owner actif]
        │
        ├─► Mise à jour contact (update_contact)
        ├─► Association à des Units
        ├─► Création compte User
        ├─► Réception documents
        ├─► Paiement charges
        │
        ▼
    [Archivage potentiel]
        │
        └─► Transfert de propriété
            └─► Nouvel Owner créé

Utilisation dans l'Application
==================================================

Création d'un Owner (Use Case)
--------------------------------------------------

**Couche Application - Use Case:**

.. code-block:: rust

    // backend/src/application/use_cases/create_owner.rs
    pub async fn execute(
        repo: &impl OwnerRepository,
        dto: CreateOwnerDto,
    ) -> Result<OwnerDto, ApplicationError> {
        // 1. Créer l'entité Owner (validation automatique)
        let owner = Owner::new(
            dto.first_name,
            dto.last_name,
            dto.email,
            dto.phone,
            dto.address,
            dto.city,
            dto.postal_code,
            dto.country,
        ).map_err(|e| ApplicationError::ValidationError(e))?;

        // 2. Persister dans la base de données
        let saved_owner = repo.create(owner).await?;

        // 3. Retourner le DTO
        Ok(OwnerDto::from(saved_owner))
    }

**Flux complet:**

.. code-block:: text

    [API Handler]
          │
          ├─► Reçoit CreateOwnerDto
          │
          ▼
    [Create Owner Use Case]
          │
          ├─► Owner::new() → Validation
          ├─► OwnerRepository::create() → Persistence
          │
          ▼
    [Owner créé et retourné]

Récupération d'un Owner
--------------------------------------------------

**Couche Application - Use Case:**

.. code-block:: rust

    // backend/src/application/use_cases/get_owner.rs
    pub async fn execute(
        repo: &impl OwnerRepository,
        owner_id: Uuid,
    ) -> Result<OwnerDto, ApplicationError> {
        let owner = repo.find_by_id(owner_id).await?
            .ok_or(ApplicationError::NotFound("Owner not found".to_string()))?;

        Ok(OwnerDto::from(owner))
    }

Mise à jour des coordonnées
--------------------------------------------------

**Couche Application - Use Case:**

.. code-block:: rust

    // backend/src/application/use_cases/update_owner_contact.rs
    pub async fn execute(
        repo: &impl OwnerRepository,
        owner_id: Uuid,
        email: String,
        phone: Option<String>,
    ) -> Result<OwnerDto, ApplicationError> {
        // 1. Récupérer l'owner
        let mut owner = repo.find_by_id(owner_id).await?
            .ok_or(ApplicationError::NotFound("Owner not found".to_string()))?;

        // 2. Mettre à jour (validation automatique)
        owner.update_contact(email, phone)
            .map_err(|e| ApplicationError::ValidationError(e))?;

        // 3. Persister les changements
        let updated_owner = repo.update(owner).await?;

        Ok(OwnerDto::from(updated_owner))
    }

**Notes importantes:**

Le timestamp ``updated_at`` est automatiquement mis à jour par la méthode ``update_contact()``, garantissant la traçabilité des modifications.

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

Cette entité est **autonome** et ne dépend d'aucune autre entité de domaine. Elle est utilisée par:

.. code-block:: text

    backend/src/domain/entities/owner.rs
            ▲
            │ used by
            │
            ├─► backend/src/application/dto/owner_dto.rs
            ├─► backend/src/application/ports/owner_repository.rs
            ├─► backend/src/application/use_cases/create_owner.rs
            ├─► backend/src/application/use_cases/get_owner.rs
            ├─► backend/src/application/use_cases/update_owner_contact.rs
            ├─► backend/src/infrastructure/repositories/postgres_owner_repository.rs
            └─► backend/src/web/handlers/owners.rs

Notes de Conception
==================================================

Validation des Données
--------------------------------------------------

**Principe:**

La validation est effectuée **à la création** et **lors des mises à jour** pour garantir l'intégrité des données.

**Règles de validation:**

1. ✅ Prénom non vide (business rule)
2. ✅ Nom non vide (business rule)
3. ✅ Email contient @ et . (validation basique)
4. ⚠️ Téléphone optionnel (pas de validation de format)
5. ⚠️ Adresse/ville/code postal non validés (acceptent chaînes vides)

**Améliorations potentielles:**

.. code-block:: rust

    // Validation stricte de l'email avec bibliothèque externe
    use email_address::EmailAddress;

    fn is_valid_email(email: &str) -> bool {
        EmailAddress::is_valid(email)
    }

    // Validation du format téléphone international
    use phonenumber::PhoneNumber;

    fn is_valid_phone(phone: &str) -> bool {
        PhoneNumber::parse(None, phone).is_ok()
    }

    // Validation code postal français
    fn is_valid_french_postal_code(code: &str) -> bool {
        code.len() == 5 && code.chars().all(|c| c.is_digit(10))
    }

Immuabilité Partielle
--------------------------------------------------

**Design actuel:**

- Tous les champs sont ``pub`` (publics et modifiables)
- Seule ``update_contact()`` offre une mise à jour contrôlée

**Alternative recommandée:**

.. code-block:: rust

    pub struct Owner {
        id: Uuid,  // Privé - immuable
        first_name: String,  // Privé
        last_name: String,  // Privé
        email: String,  // Privé - modifiable via update_contact
        phone: Option<String>,  // Privé - modifiable via update_contact
        address: String,  // Privé - modifiable via update_address
        // ...
        created_at: DateTime<Utc>,  // Privé - immuable
        updated_at: DateTime<Utc>,  // Privé - géré automatiquement
    }

    impl Owner {
        pub fn id(&self) -> Uuid { self.id }
        pub fn first_name(&self) -> &str { &self.first_name }
        pub fn last_name(&self) -> &str { &self.last_name }
        pub fn email(&self) -> &str { &self.email }
        // ... autres getters

        pub fn update_name(&mut self, first: String, last: String) -> Result<(), String> {
            // Validation + mise à jour
        }

        pub fn update_address(&mut self, address: String, city: String, postal: String, country: String) -> Result<(), String> {
            // Mise à jour adresse complète
        }
    }

**Avantages:**

- Encapsulation complète des données
- Contrôle strict des modifications
- Traçabilité via ``updated_at``
- Impossible de modifier ``id`` ou ``created_at``

Gestion des Timestamps
--------------------------------------------------

**Comportement actuel:**

- ``created_at``: Défini à la création, jamais modifié
- ``updated_at``: Défini à la création, mis à jour par ``update_contact()``

**Limitation:**

D'autres modifications (changement de nom, adresse) ne mettent pas à jour ``updated_at`` automatiquement.

**Solution:**

.. code-block:: rust

    impl Owner {
        fn touch(&mut self) {
            self.updated_at = Utc::now();
        }

        pub fn update_name(&mut self, first: String, last: String) -> Result<(), String> {
            // Validation...
            self.first_name = first;
            self.last_name = last;
            self.touch();  // Mise à jour automatique
            Ok(())
        }

        pub fn update_contact(&mut self, email: String, phone: Option<String>) -> Result<(), String> {
            // Validation...
            self.email = email;
            self.phone = phone;
            self.touch();  // Mise à jour automatique
            Ok(())
        }
    }

Avertissements
==================================================

⚠️ **Validation Email Basique**

La validation email actuelle est **trop permissive** et accepte des formats invalides comme ``@domain.com`` (pas d'utilisateur) ou ``user@`` (pas de domaine complet).

**Recommandation:** Utiliser une bibliothèque de validation stricte en production.

⚠️ **Pas de Validation de Duplicata**

La méthode ``new()`` ne vérifie pas l'unicité de l'email. Deux owners peuvent avoir le même email.

**Recommandation:** Implémenter une contrainte UNIQUE en base de données et gérer l'erreur au niveau du repository.

⚠️ **Champs Publics**

Tous les champs sont publics, permettant des modifications directes non contrôlées.

**Recommandation:** Rendre les champs privés et exposer via getters/setters avec validation.

⚠️ **Pas de Validation d'Adresse**

Les champs ``address``, ``city``, ``postal_code`` acceptent des chaînes vides.

**Recommandation:** Ajouter des validations spécifiques selon le pays.

⚠️ **Pas de Gestion de Soft Delete**

Il n'y a pas de champ ``deleted_at`` ou ``is_active`` pour gérer la désactivation d'un owner.

**Recommandation:** Ajouter un champ ``is_active: bool`` ou ``deleted_at: Option<DateTime<Utc>>`` pour le soft delete.

Fichiers Associés
==================================================

.. code-block:: text

    backend/src/
    ├── domain/
    │   └── entities/
    │       ├── owner.rs                    ← CE FICHIER
    │       ├── unit.rs                     (référence Owner)
    │       └── user.rs                     (lié à Owner)
    │
    ├── application/
    │   ├── dto/
    │   │   └── owner_dto.rs                (représentation DTO)
    │   │
    │   ├── ports/
    │   │   └── owner_repository.rs         (trait repository)
    │   │
    │   └── use_cases/
    │       ├── create_owner.rs             (création)
    │       ├── get_owner.rs                (récupération)
    │       ├── update_owner_contact.rs     (mise à jour)
    │       └── list_owners.rs              (listing)
    │
    ├── infrastructure/
    │   └── repositories/
    │       └── postgres_owner_repository.rs (implémentation PostgreSQL)
    │
    └── web/
        └── handlers/
            └── owners.rs                   (endpoints API REST)

Base de Données (Schema SQL)
--------------------------------------------------

.. code-block:: sql

    -- migrations/XXXXXX_create_owners_table.sql
    CREATE TABLE owners (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        first_name VARCHAR(255) NOT NULL,
        last_name VARCHAR(255) NOT NULL,
        email VARCHAR(255) NOT NULL,
        phone VARCHAR(50),
        address TEXT NOT NULL,
        city VARCHAR(255) NOT NULL,
        postal_code VARCHAR(20) NOT NULL,
        country VARCHAR(255) NOT NULL,
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

        -- Contraintes
        CONSTRAINT owners_email_unique UNIQUE (email),
        CONSTRAINT owners_first_name_not_empty CHECK (LENGTH(first_name) > 0),
        CONSTRAINT owners_last_name_not_empty CHECK (LENGTH(last_name) > 0),
        CONSTRAINT owners_email_format CHECK (email LIKE '%@%.%')
    );

    -- Index pour recherche par email
    CREATE INDEX idx_owners_email ON owners(email);

    -- Index pour recherche par nom
    CREATE INDEX idx_owners_name ON owners(last_name, first_name);

    -- Index pour recherche géographique
    CREATE INDEX idx_owners_location ON owners(country, city);

Points d'Amélioration Suggérés
==================================================

1. **Validation Email Stricte**

   Remplacer la validation basique par une bibliothèque robuste

2. **Champs Privés**

   Encapsuler les champs et exposer via getters/setters

3. **Tests Complets**

   Ajouter les tests manquants (nom vide, email invalide en update)

4. **Validation Téléphone**

   Valider le format international du téléphone

5. **Validation Adresse**

   Valider les champs d'adresse selon le pays

6. **Soft Delete**

   Ajouter un champ ``is_active`` ou ``deleted_at``

7. **Méthode update_address**

   Créer une méthode dédiée pour mettre à jour l'adresse complète

8. **Événements de Domaine**

   Émettre des événements lors de la création/modification (``OwnerCreated``, ``OwnerContactUpdated``)

9. **Value Objects**

   Créer des Value Objects pour ``Email``, ``PhoneNumber``, ``Address``

10. **Documentation Inline**

    Ajouter des doc comments Rust (``///``) pour générer la documentation avec rustdoc
