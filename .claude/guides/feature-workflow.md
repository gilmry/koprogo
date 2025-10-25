# Workflow de Développement de Features

Guide complet pour développer une nouvelle feature avec Claude Code sur KoproGo.

## Vue d'ensemble du workflow

```
1. Analyse & Design
2. Setup de branche Git
3. Tests-First (TDD)
4. Implémentation (Domain → Application → Infrastructure)
5. Tests d'intégration
6. Pre-commit checks
7. Documentation
8. Pre-push validation (CI)
9. Pull Request
```

---

## Étape 1 : Analyse & Design

### Questions à se poser

1. **Quelle est la feature demandée ?**
   - User story claire
   - Critères d'acceptation
   - Cas d'usage concrets

2. **Quel est le modèle de domaine ?**
   - Nouvelles entités ?
   - Nouveaux value objects ?
   - Nouvelles règles métier ?

3. **Quels sont les use cases ?**
   - Commandes (CUD)
   - Queries (R)
   - Services de domaine nécessaires ?

4. **Quelles sont les dépendances ?**
   - Bases de données (PostgreSQL)
   - Services externes
   - APIs tierces

### Template d'analyse

```markdown
## Feature: [Nom]

### User Story
En tant que [role], je veux [action], afin de [bénéfice]

### Critères d'Acceptation
- [ ] Critère 1
- [ ] Critère 2
- [ ] Critère 3

### Design Technique

#### Domaine
- Entités : [Entity1, Entity2]
- Value Objects : [VO1, VO2]
- Règles métier : [Rule1, Rule2]

#### Application
- Use Cases : [UseCase1, UseCase2]
- Ports (interfaces) : [Port1, Port2]
- DTOs : [DTO1, DTO2]

#### Infrastructure
- Repositories : [Repo1, Repo2]
- Handlers HTTP : [Handler1, Handler2]
- Migrations DB : [Migration1]
```

---

## Étape 2 : Setup de Branche Git

```bash
# Créer une nouvelle branche depuis main
git checkout main
git pull origin main
git checkout -b feature/ma-nouvelle-feature

# Vérifier le statut
git status
```

**Convention de nommage** :
- `feature/nom-de-la-feature` pour nouvelles features
- `fix/nom-du-bug` pour bugfixes
- `refactor/description` pour refactoring
- `docs/description` pour documentation

---

## Étape 3 : Tests-First (TDD)

### 3.1 Tests Unitaires du Domaine

Commencez TOUJOURS par les tests du domaine.

**Exemple : Créer une entité Payment**

```rust
// backend/src/domain/entities/payment.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_payment_success() {
        let payment = Payment::new(
            Uuid::new_v4(),
            Decimal::from(100),
            "expense_id".to_string(),
            "user_id".to_string(),
        );

        assert!(payment.is_ok());
        let p = payment.unwrap();
        assert_eq!(p.amount(), Decimal::from(100));
        assert_eq!(p.status(), PaymentStatus::Pending);
    }

    #[test]
    fn test_create_payment_invalid_amount() {
        let payment = Payment::new(
            Uuid::new_v4(),
            Decimal::from(-100), // Montant négatif
            "expense_id".to_string(),
            "user_id".to_string(),
        );

        assert!(payment.is_err());
        assert_eq!(payment.unwrap_err(), "Amount must be positive");
    }

    #[test]
    fn test_mark_payment_as_paid() {
        let mut payment = Payment::new(
            Uuid::new_v4(),
            Decimal::from(100),
            "expense_id".to_string(),
            "user_id".to_string(),
        ).unwrap();

        let result = payment.mark_as_paid();
        assert!(result.is_ok());
        assert_eq!(payment.status(), PaymentStatus::Paid);
    }
}
```

**Lancer les tests** :
```bash
cargo test --lib test_create_payment
```

### 3.2 Tests BDD (optionnel mais recommandé)

Créer un fichier Gherkin pour les scénarios utilisateur.

```gherkin
# backend/tests/features/payments.feature

Feature: Payment Management
  As a building manager
  I want to manage payments
  So that I can track expenses

  Scenario: Create a new payment
    Given I am authenticated as a syndic
    When I create a payment for expense "exp-123" with amount 150.00
    Then the payment should be created successfully
    And the payment status should be "pending"

  Scenario: Mark payment as paid
    Given I have a pending payment "pay-123"
    When I mark the payment as paid
    Then the payment status should be "paid"
    And the paid_at timestamp should be set
```

---

## Étape 4 : Implémentation (Architecture Hexagonale)

### 4.1 Layer Domain (Core Business Logic)

**Créer l'entité** :

```rust
// backend/src/domain/entities/payment.rs

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Paid,
    Cancelled,
    Refunded,
}

#[derive(Debug, Clone)]
pub struct Payment {
    id: Uuid,
    expense_id: Uuid,
    user_id: Uuid,
    amount: Decimal,
    status: PaymentStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    paid_at: Option<DateTime<Utc>>,
}

impl Payment {
    /// Crée un nouveau paiement (validation des invariants)
    pub fn new(
        id: Uuid,
        amount: Decimal,
        expense_id: Uuid,
        user_id: Uuid,
    ) -> Result<Self, String> {
        // Invariant : montant positif
        if amount <= Decimal::ZERO {
            return Err("Amount must be positive".to_string());
        }

        let now = Utc::now();
        Ok(Self {
            id,
            expense_id,
            user_id,
            amount,
            status: PaymentStatus::Pending,
            created_at: now,
            updated_at: now,
            paid_at: None,
        })
    }

    /// Marque le paiement comme payé
    pub fn mark_as_paid(&mut self) -> Result<(), String> {
        if self.status != PaymentStatus::Pending {
            return Err(format!("Cannot mark {:?} payment as paid", self.status));
        }

        self.status = PaymentStatus::Paid;
        self.paid_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    // Getters
    pub fn id(&self) -> Uuid { self.id }
    pub fn amount(&self) -> Decimal { self.amount }
    pub fn status(&self) -> &PaymentStatus { &self.status }
    pub fn paid_at(&self) -> Option<DateTime<Utc>> { self.paid_at }
}
```

**Enregistrer le module** dans `backend/src/domain/entities/mod.rs` :
```rust
pub mod payment;
```

### 4.2 Layer Application (Use Cases + Ports)

**Définir le Port (interface)** :

```rust
// backend/src/application/ports/payment_repository.rs

use crate::domain::entities::payment::Payment;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait PaymentRepository: Send + Sync {
    async fn create(&self, payment: &Payment) -> Result<Payment, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Payment>, String>;
    async fn find_by_expense(&self, expense_id: Uuid) -> Result<Vec<Payment>, String>;
    async fn update(&self, payment: &Payment) -> Result<Payment, String>;
    async fn delete(&self, id: Uuid) -> Result<(), String>;
}
```

**Créer le Use Case** :

```rust
// backend/src/application/use_cases/payment_use_cases.rs

use crate::application::ports::payment_repository::PaymentRepository;
use crate::domain::entities::payment::Payment;
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

pub struct PaymentUseCases {
    payment_repo: Arc<dyn PaymentRepository>,
}

impl PaymentUseCases {
    pub fn new(payment_repo: Arc<dyn PaymentRepository>) -> Self {
        Self { payment_repo }
    }

    pub async fn create_payment(
        &self,
        expense_id: Uuid,
        user_id: Uuid,
        amount: Decimal,
    ) -> Result<Payment, String> {
        // Créer l'entité (validation des invariants)
        let payment = Payment::new(Uuid::new_v4(), amount, expense_id, user_id)?;

        // Persister via le port
        self.payment_repo.create(&payment).await
    }

    pub async fn mark_payment_as_paid(&self, id: Uuid) -> Result<Payment, String> {
        // Récupérer le paiement
        let mut payment = self
            .payment_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Payment not found".to_string())?;

        // Appliquer la logique métier
        payment.mark_as_paid()?;

        // Persister les changements
        self.payment_repo.update(&payment).await
    }
}
```

**Enregistrer les modules** :
```rust
// backend/src/application/ports/mod.rs
pub mod payment_repository;

// backend/src/application/use_cases/mod.rs
pub mod payment_use_cases;
```

### 4.3 Layer Infrastructure (Adapters)

**Créer la migration DB** :

```bash
cd backend
sqlx migrate add create_payments_table
```

Éditer le fichier de migration :

```sql
-- backend/migrations/XXXXXX_create_payments_table.sql

CREATE TYPE payment_status AS ENUM ('pending', 'paid', 'cancelled', 'refunded');

CREATE TABLE payments (
    id UUID PRIMARY KEY,
    expense_id UUID NOT NULL REFERENCES expenses(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    amount DECIMAL(15, 2) NOT NULL CHECK (amount > 0),
    status payment_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    paid_at TIMESTAMPTZ,

    CONSTRAINT amount_positive CHECK (amount > 0)
);

CREATE INDEX idx_payments_expense_id ON payments(expense_id);
CREATE INDEX idx_payments_user_id ON payments(user_id);
CREATE INDEX idx_payments_organization_id ON payments(organization_id);
CREATE INDEX idx_payments_status ON payments(status);
```

**Implémenter le Repository (adapter PostgreSQL)** :

```rust
// backend/src/infrastructure/database/repositories/payment_repository_impl.rs

use crate::application::ports::payment_repository::PaymentRepository;
use crate::domain::entities::payment::{Payment, PaymentStatus};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PostgresPaymentRepository {
    pool: PgPool,
}

impl PostgresPaymentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PaymentRepository for PostgresPaymentRepository {
    async fn create(&self, payment: &Payment) -> Result<Payment, String> {
        sqlx::query!(
            r#"
            INSERT INTO payments (id, expense_id, user_id, amount, status, created_at, updated_at, paid_at)
            VALUES ($1, $2, $3, $4, $5::payment_status, $6, $7, $8)
            "#,
            payment.id(),
            payment.expense_id(),
            payment.user_id(),
            payment.amount(),
            format!("{:?}", payment.status()).to_lowercase(),
            payment.created_at(),
            payment.updated_at(),
            payment.paid_at(),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create payment: {}", e))?;

        Ok(payment.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Payment>, String> {
        let result = sqlx::query!(
            r#"
            SELECT id, expense_id, user_id, amount, status as "status: String", created_at, updated_at, paid_at
            FROM payments
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find payment: {}", e))?;

        if let Some(row) = result {
            let status = match row.status.as_str() {
                "pending" => PaymentStatus::Pending,
                "paid" => PaymentStatus::Paid,
                "cancelled" => PaymentStatus::Cancelled,
                "refunded" => PaymentStatus::Refunded,
                _ => return Err("Invalid payment status".to_string()),
            };

            // Reconstruire l'entité depuis les données DB
            // (En production, utiliser un builder ou un constructeur spécial)
            Ok(Some(Payment::from_db(
                row.id,
                row.expense_id,
                row.user_id,
                row.amount,
                status,
                row.created_at,
                row.updated_at,
                row.paid_at,
            )))
        } else {
            Ok(None)
        }
    }

    // ... autres méthodes
}
```

**Créer le Handler HTTP** :

```rust
// backend/src/infrastructure/web/handlers/payment_handlers.rs

use crate::application::dto::payment_dto::{CreatePaymentRequest, PaymentResponse};
use crate::application::use_cases::payment_use_cases::PaymentUseCases;
use actix_web::{web, HttpResponse};
use std::sync::Arc;
use uuid::Uuid;

pub async fn create_payment(
    payment_uc: web::Data<Arc<PaymentUseCases>>,
    req: web::Json<CreatePaymentRequest>,
) -> HttpResponse {
    match payment_uc
        .create_payment(req.expense_id, req.user_id, req.amount)
        .await
    {
        Ok(payment) => HttpResponse::Created().json(PaymentResponse::from(payment)),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({ "error": e })),
    }
}

pub async fn mark_payment_paid(
    payment_uc: web::Data<Arc<PaymentUseCases>>,
    payment_id: web::Path<Uuid>,
) -> HttpResponse {
    match payment_uc.mark_payment_as_paid(*payment_id).await {
        Ok(payment) => HttpResponse::Ok().json(PaymentResponse::from(payment)),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({ "error": e })),
    }
}
```

**Enregistrer les routes** dans `backend/src/infrastructure/web/routes.rs` :

```rust
use crate::infrastructure::web::handlers::payment_handlers;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/payments", web::post().to(payment_handlers::create_payment))
            .route("/payments/{id}/pay", web::put().to(payment_handlers::mark_payment_paid))
            // ... autres routes
    );
}
```

---

## Étape 5 : Tests d'Intégration

Créer des tests d'intégration qui testent toute la stack.

```rust
// backend/tests/integration/payment_tests.rs

use koprogo_api::*;
use sqlx::PgPool;
use testcontainers::*;

#[tokio::test]
async fn test_create_and_pay_payment_integration() {
    // Setup testcontainer avec PostgreSQL
    let container = clients::Cli::default().run(images::postgres::Postgres::default());
    let pool = setup_test_db(&container).await;

    // Créer les use cases
    let payment_repo = Arc::new(PostgresPaymentRepository::new(pool.clone()));
    let payment_uc = PaymentUseCases::new(payment_repo);

    // Test : créer un paiement
    let payment = payment_uc
        .create_payment(Uuid::new_v4(), Uuid::new_v4(), Decimal::from(100))
        .await
        .expect("Failed to create payment");

    assert_eq!(payment.amount(), Decimal::from(100));
    assert_eq!(payment.status(), &PaymentStatus::Pending);

    // Test : marquer comme payé
    let paid_payment = payment_uc
        .mark_payment_as_paid(payment.id())
        .await
        .expect("Failed to mark payment as paid");

    assert_eq!(paid_payment.status(), &PaymentStatus::Paid);
    assert!(paid_payment.paid_at().is_some());
}
```

**Lancer les tests d'intégration** :
```bash
cargo test --test integration
```

---

## Étape 6 : Pre-commit Checks

Avant de commiter, TOUJOURS lancer les pre-commit checks :

```bash
make pre-commit
```

Ce qui fait :
1. **Format le code** (cargo fmt + prettier)
2. **Lint** (clippy + prettier check)

Si des erreurs apparaissent, les corriger avant de continuer.

### Corriger les erreurs courantes

**Clippy warnings** :
```bash
# Voir les warnings
make lint

# Auto-fix quand possible
cd backend && SQLX_OFFLINE=true cargo clippy --fix --allow-dirty

# Relancer les checks
make pre-commit
```

**Formatting issues** :
```bash
# Auto-format
make format

# Vérifier
make lint
```

---

## Étape 7 : Documentation

### 7.1 Commenter le code

Ajouter des doc comments Rust :

```rust
/// Crée un nouveau paiement pour une dépense
///
/// # Arguments
///
/// * `expense_id` - ID de la dépense associée
/// * `user_id` - ID de l'utilisateur effectuant le paiement
/// * `amount` - Montant du paiement (doit être > 0)
///
/// # Returns
///
/// * `Ok(Payment)` - Le paiement créé
/// * `Err(String)` - En cas d'erreur de validation
///
/// # Examples
///
/// ```
/// let payment = Payment::new(
///     Uuid::new_v4(),
///     Decimal::from(100),
///     expense_id,
///     user_id,
/// )?;
/// ```
pub fn new(...) -> Result<Self, String> {
    // ...
}
```

### 7.2 Mettre à jour CHANGELOG.md

```markdown
### Added - Payment Management Feature (2025-XX-XX)

**Payment Domain**
- New `Payment` entity with business rules validation
- Payment status lifecycle: pending → paid/cancelled/refunded
- Amount validation (must be positive)

**Payment Use Cases**
- `create_payment`: Create new payment for expense
- `mark_payment_as_paid`: Mark payment as paid with timestamp

**Payment API**
- `POST /api/v1/payments` - Create payment
- `PUT /api/v1/payments/:id/pay` - Mark as paid
- `GET /api/v1/payments/:id` - Get payment details

**Database**
- New `payments` table with foreign keys to expenses and users
- `payment_status` ENUM type
- Indexes on expense_id, user_id, status

**Tests**
- 15+ unit tests for Payment entity
- 8 integration tests with PostgreSQL
- 5 BDD scenarios for payment workflows
```

---

## Étape 8 : Pre-push Validation (CI complète)

Avant de pusher, lancer les validations CI complètes :

```bash
make ci
```

Ce qui fait :
1. **lint** : Vérifie la qualité du code
2. **test** : Lance TOUS les tests (unit + integration + bdd)
3. **audit** : Vérifie les vulnérabilités de sécurité

**Durée estimée** : 3-5 minutes

### Si `make ci` échoue

1. **Tests échouent** :
   ```bash
   # Identifier le test qui échoue
   cargo test --lib -- --nocapture

   # Fixer et relancer
   cargo test --lib test_name
   ```

2. **Lint échoue** :
   ```bash
   make lint
   # Corriger les warnings
   make pre-commit
   ```

3. **Audit échoue** :
   ```bash
   make audit
   # Mettre à jour les dépendances vulnérables
   cd backend && cargo update
   ```

### Résolution des problèmes SQLX

Si vous voyez des erreurs `Connection refused (os error 111)` :

```bash
# Option 1 : Utiliser le mode offline
export SQLX_OFFLINE=true
make ci

# Option 2 : Démarrer PostgreSQL
make docker-up
# Attendre 5 secondes
make ci
```

---

## Étape 9 : Commit et Push

### 9.1 Commit

```bash
# Vérifier les fichiers modifiés
git status

# Ajouter les fichiers
git add backend/src/domain/entities/payment.rs
git add backend/src/application/ports/payment_repository.rs
git add backend/src/application/use_cases/payment_use_cases.rs
git add backend/src/infrastructure/database/repositories/payment_repository_impl.rs
git add backend/src/infrastructure/web/handlers/payment_handlers.rs
git add backend/migrations/XXXXXX_create_payments_table.sql
git add CHANGELOG.md

# Créer le commit avec un message descriptif
git commit -m "feat(payments): Add payment management feature

- Add Payment entity with business rules validation
- Implement payment use cases (create, mark_as_paid)
- Add PostgreSQL repository implementation
- Add HTTP handlers for payment API
- Create payments table migration
- Add 15 unit tests and 8 integration tests

Closes #123"
```

**Convention de messages de commit** :
- `feat(scope): Description` - Nouvelle feature
- `fix(scope): Description` - Correction de bug
- `refactor(scope): Description` - Refactoring
- `docs(scope): Description` - Documentation
- `test(scope): Description` - Tests
- `chore(scope): Description` - Tâches de maintenance

### 9.2 Push

```bash
# Push vers origin
git push origin feature/payment-management

# Si c'est le premier push de la branche
git push -u origin feature/payment-management
```

---

## Étape 10 : Pull Request

### 10.1 Créer la PR

```bash
# Avec gh CLI
gh pr create --title "feat: Payment management feature" --body "$(cat <<'EOF'
## Summary

Ajoute la fonctionnalité de gestion des paiements pour les dépenses.

## Changes

- ✅ Payment entity avec validation métier
- ✅ Use cases pour créer et marquer les paiements comme payés
- ✅ Repository PostgreSQL
- ✅ API HTTP endpoints
- ✅ Migration DB
- ✅ Tests (unit + integration + BDD)

## Test Plan

- [x] Tests unitaires (15 tests, 100% coverage)
- [x] Tests d'intégration (8 scénarios)
- [x] Tests BDD (5 scénarios Gherkin)
- [x] Tests manuels sur environnement dev

## Checklist

- [x] Code formaté (make format)
- [x] Lint passé (make lint)
- [x] Tests passés (make test)
- [x] Audit sécurité (make audit)
- [x] CHANGELOG.md mis à jour
- [x] Documentation API ajoutée
EOF
)"
```

### 10.2 Attendre la CI/CD

GitHub Actions va automatiquement :
1. Lancer les tests
2. Vérifier le linting
3. Construire les artifacts

**Vérifier les checks** :
```bash
gh pr checks
```

### 10.3 Review et Merge

1. Demander une review à un collègue
2. Répondre aux commentaires
3. Appliquer les corrections demandées
4. Une fois approuvé, merge via :
   - Squash and merge (recommandé)
   - Rebase and merge
   - Merge commit

---

## Checklist Complète

Utilisez cette checklist pour chaque feature :

### Design
- [ ] User story claire
- [ ] Critères d'acceptation définis
- [ ] Design technique validé (entités, use cases, ports)

### Implémentation
- [ ] Tests unitaires écrits AVANT l'implémentation
- [ ] Entité Domain créée avec validation des invariants
- [ ] Port (interface) défini dans Application layer
- [ ] Use Case implémenté
- [ ] Repository PostgreSQL implémenté
- [ ] Migration DB créée
- [ ] Handler HTTP créé
- [ ] Routes enregistrées

### Tests
- [ ] Tests unitaires passent (cargo test --lib)
- [ ] Tests d'intégration passent (cargo test --test integration)
- [ ] Tests BDD passent (cargo test --test bdd)
- [ ] Coverage > 80%

### Qualité
- [ ] Pre-commit checks passent (make pre-commit)
- [ ] Code formaté (make format)
- [ ] Lint passé (make lint)
- [ ] Pre-push validation passée (make ci)
- [ ] Audit sécurité OK (make audit)

### Documentation
- [ ] Doc comments Rust ajoutés
- [ ] CHANGELOG.md mis à jour
- [ ] README.md mis à jour (si nécessaire)
- [ ] Documentation API ajoutée

### Git
- [ ] Branche créée depuis main
- [ ] Commits atomiques et descriptifs
- [ ] Messages de commit suivent la convention
- [ ] Push vers origin
- [ ] PR créée avec description complète

### CI/CD
- [ ] GitHub Actions checks passent
- [ ] Code review demandé
- [ ] Commentaires de review traités
- [ ] PR approuvée et mergée

---

## Commandes Rapides

```bash
# Workflow complet
git checkout -b feature/ma-feature
# ... développement avec TDD ...
make pre-commit          # Format + Lint
make ci                  # Tests + Audit complets
git add .
git commit -m "feat: Ma feature"
git push -u origin feature/ma-feature
gh pr create

# Debug
cargo test --lib test_name -- --nocapture   # Test avec logs
cargo test --lib -- --test-threads=1        # Tests séquentiels
RUST_LOG=debug cargo run                    # Logs verbeux

# Cleanup
git branch -D feature/old-branch            # Supprimer branche locale
git push origin --delete feature/old-branch # Supprimer branche remote
```

---

## Ressources

- [Architecture Guide](.claude/guides/architecture-guide.md)
- [Testing Guide](.claude/guides/testing-guide.md)
- [CLAUDE.md](../CLAUDE.md)
- [Templates](.claude/templates/)
