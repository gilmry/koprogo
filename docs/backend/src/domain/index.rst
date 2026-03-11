Domain - Couche Domaine (Core Métier)
======================================

La couche Domain contient la logique métier pure, complètement indépendante des frameworks et technologies.

**Principe** : Le Domain ne dépend de PERSONNE. Zéro dépendance externe (sauf crates utilitaires : uuid, chrono, serde).

Structure
---------

.. code-block:: text

   domain/
   ├── entities/          # Entités métier avec invariants
   │   ├── user.rs
   │   ├── organization.rs
   │   ├── building.rs
   │   ├── unit.rs
   │   ├── owner.rs
   │   ├── expense.rs
   │   ├── meeting.rs
   │   ├── document.rs
   │   └── refresh_token.rs
   └── services/          # Services domaine
       ├── expense_calculator.rs
       ├── pcn_mapper.rs
       └── pcn_exporter.rs

Entities (Entités Métier)
--------------------------

Les entités encapsulent les règles métier et garantissent les invariants.

.. toctree::
   :maxdepth: 1

   entities/building
   entities/unit
   entities/owner
   entities/expense
   entities/meeting
   entities/document
   entities/user
   entities/organization
   entities/refresh_token

**Caractéristiques Communes** :

- ✅ Identifiant UUID unique
- ✅ Timestamps ``created_at`` / ``updated_at``
- ✅ Validation dans constructeur ``new()``
- ✅ Méthodes métier (ex: ``Building::update_info()``)
- ✅ Tests unitaires in-module

**Exemple Pattern** :

.. code-block:: rust

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct Building {
       pub id: Uuid,
       pub name: String,
       // ... autres champs
       pub created_at: DateTime<Utc>,
       pub updated_at: DateTime<Utc>,
   }

   impl Building {
       pub fn new(...) -> Result<Self, String> {
           // Validation invariants métier
           if name.is_empty() {
               return Err("Building name cannot be empty".to_string());
           }

           Ok(Self {
               id: Uuid::new_v4(),
               name,
               created_at: Utc::now(),
               updated_at: Utc::now(),
               // ...
           })
       }

       pub fn update_info(&mut self, ...) {
           // Logique métier
           self.updated_at = Utc::now();
       }
   }

   #[cfg(test)]
   mod tests {
       #[test]
       fn test_create_building_success() { ... }

       #[test]
       fn test_create_building_empty_name_fails() { ... }
   }

Relations Entre Entités
------------------------

.. code-block:: text

   Organization (Multi-tenant)
        │
        ├──> Users (1:N)
        │
        └──> Buildings (1:N)
              │
              ├──> Units (1:N)
              │     │
              │     └──> Owners (N:1)
              │
              ├──> Expenses (1:N)
              │
              ├──> Meetings (1:N)
              │
              └──> Documents (1:N)

Domain Services
---------------

Services domaine pour logique métier complexe impliquant plusieurs entités.

.. toctree::
   :maxdepth: 1

   services/expense_calculator
   services/pcn_mapper
   services/pcn_exporter

**ExpenseCalculator** :

Calcule la répartition des charges selon les quotes-parts.

.. code-block:: rust

   impl ExpenseCalculator {
       pub fn calculate_share(
           expense: &Expense,
           unit: &Unit,
           total_shares: i32
       ) -> f64 {
           (expense.amount as f64 * unit.ownership_share as f64)
               / total_shares as f64
       }
   }

**PCNMapper** :

Mappe les données pour génération Précompte de Charge Notariale (PCN).

**PCNExporter** :

Exporte PCN en PDF ou Excel.

Validation Métier
-----------------

Toute validation est dans les entités :

.. code-block:: rust

   // ✅ BON : Validation dans l'entité
   impl Building {
       pub fn new(name: String, total_units: i32, ...) -> Result<Self, String> {
           if name.is_empty() {
               return Err("Name cannot be empty".to_string());
           }
           if total_units <= 0 {
               return Err("Total units must be > 0".to_string());
           }
           // ...
       }
   }

   // ❌ MAUVAIS : Validation dans le handler HTTP
   async fn create_building(req: HttpRequest) -> Result<HttpResponse> {
       if dto.name.is_empty() {  // NON ! Ceci appartient au domain
           return Err(...)
       }
   }

Règles DDD Appliquées
----------------------

1. **Ubiquitous Language** :

   Terminologie métier : Building (immeuble), Owner (copropriétaire), Unit (lot), Expense (charge)

2. **Aggregates** :

   - **Building** : Aggregate root
   - **Units** : Entités de l'aggregate Building
   - Règle : Modification Units passe toujours par Building

3. **Value Objects** (à implémenter) :

   .. code-block:: rust

      pub struct Address {
          street: String,
          city: String,
          postal_code: String,
          country: String,
      }

      pub struct Email(String);  // Email valide

4. **Domain Events** (futur) :

   .. code-block:: rust

      pub enum BuildingEvent {
          BuildingCreated { id: Uuid, name: String },
          BuildingUpdated { id: Uuid },
          BuildingDeleted { id: Uuid },
       }

Tests Domaine
-------------

**Objectif** : 100% coverage domaine (logique critique)

.. code-block:: bash

   # Tests unitaires domaine uniquement
   cargo test --lib domain::

   # Tests entité spécifique
   cargo test --lib domain::entities::building

**Pattern de Test** :

.. code-block:: rust

   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_create_building_success() {
           let building = Building::new(
               "Test Building".to_string(),
               "Address".to_string(),
               "City".to_string(),
               "12345".to_string(),
               "Country".to_string(),
               10,
               None,
           );

           assert!(building.is_ok());
           let building = building.unwrap();
           assert_eq!(building.name, "Test Building");
           assert_eq!(building.total_units, 10);
       }

       #[test]
       fn test_create_building_empty_name_fails() {
           let building = Building::new(
               "".to_string(),  // Invalide
               "Address".to_string(),
               "City".to_string(),
               "12345".to_string(),
               "Country".to_string(),
               10,
               None,
           );

           assert!(building.is_err());
           assert_eq!(
               building.unwrap_err(),
               "Building name cannot be empty"
           );
       }
   }

Dépendances Autorisées
-----------------------

Crates externes autorisées dans le Domain :

.. code-block:: toml

   [dependencies]
   uuid = { version = "1.11", features = ["v4", "serde"] }
   chrono = { version = "0.4", features = ["serde"] }
   serde = { version = "1.0", features = ["derive"] }

**Interdictions** :

- ❌ Pas d'Actix-web (framework web)
- ❌ Pas de SQLx (base de données)
- ❌ Pas de Tokio async (sauf si absolument nécessaire)
- ❌ Pas de dépendances vers Application ou Infrastructure

Évolutions Futures
-------------------

1. **Value Objects** :

   .. code-block:: rust

      pub struct Email(String);
      pub struct PhoneNumber(String);
      pub struct Address { ... }

2. **Domain Events** :

   .. code-block:: rust

      pub trait DomainEvent {
          fn event_type(&self) -> &str;
          fn aggregate_id(&self) -> Uuid;
          fn occurred_at(&self) -> DateTime<Utc>;
      }

3. **Specifications Pattern** :

   .. code-block:: rust

      pub trait Specification<T> {
          fn is_satisfied_by(&self, entity: &T) -> bool;
      }

      pub struct PaidExpenseSpecification;
      impl Specification<Expense> for PaidExpenseSpecification {
          fn is_satisfied_by(&self, expense: &Expense) -> bool {
              expense.payment_status == PaymentStatus::Paid
          }
      }

4. **Factory Pattern** :

   .. code-block:: rust

      pub struct BuildingFactory;
      impl BuildingFactory {
          pub fn create_residential(...) -> Result<Building, String> { ... }
          pub fn create_commercial(...) -> Result<Building, String> { ... }
      }

Références
----------

- Domain-Driven Design (Eric Evans)
- Implementing Domain-Driven Design (Vaughn Vernon)
- Clean Architecture (Robert C. Martin)
