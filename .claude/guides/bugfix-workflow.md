# Workflow de Correction de Bugs

Guide pour corriger efficacement les bugs avec Claude Code sur KoproGo.

## Vue d'ensemble

```
1. Reproduction du bug (écrire test qui échoue)
2. Investigation de la cause racine
3. Correction ciblée
4. Validation (test passe)
5. Tests de régression
6. Pre-commit + Pre-push
7. Commit + Push + PR
```

---

## Étape 1 : Reproduction du Bug

**TOUJOURS commencer par écrire un test qui reproduit le bug.**

### 1.1 Créer un test qui échoue

```rust
#[test]
fn test_bug_negative_payment_amount() {
    // Arrange: Setup qui provoque le bug
    let payment = Payment::new(
        Uuid::new_v4(),
        Decimal::from(-100),  // Montant négatif (bug)
        Uuid::new_v4(),
        Uuid::new_v4(),
    );

    // Assert: Le test échoue actuellement (reproduit le bug)
    assert!(payment.is_err());
    assert_eq!(payment.unwrap_err(), "Amount must be positive");
}
```

**Lancer le test** pour confirmer qu'il échoue :
```bash
cargo test --lib test_bug_negative_payment_amount
```

---

## Étape 2 : Investigation

### 2.1 Analyser le code concerné

Utiliser Claude Code pour explorer :

```
Claude, analyse le code de Payment::new() pour comprendre pourquoi
les montants négatifs ne sont pas rejetés.
```

### 2.2 Vérifier les logs

```bash
# Activer les logs verbeux
RUST_LOG=debug cargo test --lib test_bug_negative_payment_amount -- --nocapture
```

### 2.3 Utiliser le debugger (optionnel)

```rust
#[test]
fn test_bug_negative_payment_amount() {
    dbg!("Testing negative amount");
    let amount = Decimal::from(-100);
    dbg!(&amount);  // Inspecter la valeur

    let payment = Payment::new(...);
    dbg!(&payment);  // Inspecter le résultat
}
```

---

## Étape 3 : Correction Ciblée

### 3.1 Identifier la ligne problématique

```rust
// AVANT (bug)
pub fn new(id: Uuid, amount: Decimal, ...) -> Result<Self, String> {
    // ❌ Pas de validation du montant !
    Ok(Self {
        id,
        amount,  // Montant négatif accepté
        ...
    })
}
```

### 3.2 Appliquer la correction minimale

```rust
// APRÈS (fix)
pub fn new(id: Uuid, amount: Decimal, ...) -> Result<Self, String> {
    // ✅ Validation ajoutée
    if amount <= Decimal::ZERO {
        return Err("Amount must be positive".to_string());
    }

    Ok(Self {
        id,
        amount,
        ...
    })
}
```

### 3.3 Vérifier que le test passe

```bash
cargo test --lib test_bug_negative_payment_amount
```

✅ Le test doit maintenant passer !

---

## Étape 4 : Tests de Régression

Ajouter des tests supplémentaires pour couvrir les cas limites :

```rust
#[test]
fn test_payment_amount_zero() {
    let payment = Payment::new(
        Uuid::new_v4(),
        Decimal::ZERO,  // Montant nul
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    assert!(payment.is_err());
}

#[test]
fn test_payment_amount_very_small() {
    let payment = Payment::new(
        Uuid::new_v4(),
        Decimal::new(1, 2),  // 0.01€
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    assert!(payment.is_ok());
}

#[test]
fn test_payment_amount_very_large() {
    let payment = Payment::new(
        Uuid::new_v4(),
        Decimal::from(1_000_000),  // 1M€
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    assert!(payment.is_ok());
}
```

---

## Étape 5 : Pre-commit et Pre-push

### 5.1 Pre-commit

```bash
make pre-commit
```

Vérifie :
- ✅ Format du code
- ✅ Lint (clippy)

### 5.2 Pre-push (CI complète)

```bash
make ci
```

Vérifie :
- ✅ Lint
- ✅ Tous les tests
- ✅ Audit sécurité

---

## Étape 6 : Documentation

### 6.1 Mettre à jour CHANGELOG.md

```markdown
### Fixed - Payment Amount Validation (2025-XX-XX)

**Bug Fix**
- Fixed: Negative payment amounts were accepted
- Added validation to reject amounts <= 0
- Added regression tests for edge cases (zero, very small, very large amounts)

Fixes #456
```

### 6.2 Ajouter un commentaire explicatif

```rust
pub fn new(id: Uuid, amount: Decimal, ...) -> Result<Self, String> {
    // Invariant: Amount must be strictly positive (> 0)
    // Fix #456: Reject negative and zero amounts
    if amount <= Decimal::ZERO {
        return Err("Amount must be positive".to_string());
    }
    ...
}
```

---

## Étape 7 : Commit et Push

```bash
# Créer la branche
git checkout -b fix/payment-negative-amount

# Ajouter les fichiers modifiés
git add backend/src/domain/entities/payment.rs
git add CHANGELOG.md

# Commit avec référence au bug
git commit -m "fix(payment): Reject negative and zero amounts

- Add validation in Payment::new() to reject amounts <= 0
- Add regression tests for edge cases
- Update CHANGELOG.md

Fixes #456"

# Push
git push -u origin fix/payment-negative-amount
```

---

## Étape 8 : Pull Request

```bash
gh pr create --title "fix: Reject negative payment amounts" --body "$(cat <<'EOF'
## Bug Description

Les paiements avec montants négatifs étaient acceptés, permettant
des transactions invalides.

## Root Cause

La méthode `Payment::new()` ne validait pas que le montant soit > 0.

## Fix

- Ajout de validation pour rejeter `amount <= 0`
- Retourne une erreur explicite
- Ajoute des tests de régression

## Test Plan

- [x] Test de reproduction du bug (échoue avant fix)
- [x] Test passe après fix
- [x] Tests de régression (zero, very small, very large)
- [x] make ci passe
- [x] Tests manuels

Fixes #456
EOF
)"
```

---

## Checklist Bugfix

- [ ] Test écrit qui reproduit le bug (échoue initialement)
- [ ] Cause racine identifiée
- [ ] Correction appliquée (minimale et ciblée)
- [ ] Test de reproduction passe
- [ ] Tests de régression ajoutés
- [ ] Pre-commit passe (make pre-commit)
- [ ] Pre-push passe (make ci)
- [ ] CHANGELOG.md mis à jour
- [ ] Branche créée (fix/description)
- [ ] Commit avec "Fixes #issue"
- [ ] PR créée avec référence à l'issue

---

## Types de Bugs Courants

### 1. Bugs de Validation

```rust
// Bug: Pas de validation
pub fn new(email: String) -> Self {
    Self { email }  // ❌ Email invalide accepté
}

// Fix: Ajouter validation
pub fn new(email: String) -> Result<Self, String> {
    if !email.contains('@') {
        return Err("Invalid email format".to_string());
    }
    Ok(Self { email })
}
```

### 2. Bugs de Logique Métier

```rust
// Bug: Logique incorrecte
pub fn calculate_total(&self) -> Decimal {
    self.items.iter().map(|i| i.price).sum()  // ❌ Oublie la quantité !
}

// Fix: Corriger la logique
pub fn calculate_total(&self) -> Decimal {
    self.items.iter().map(|i| i.price * i.quantity).sum()
}
```

### 3. Bugs de Concurrence

```rust
// Bug: Race condition
pub async fn update_balance(&mut self, amount: Decimal) {
    let current = self.balance;  // ❌ Peut être obsolète
    self.balance = current + amount;
}

// Fix: Utiliser transactions DB ou locks
pub async fn update_balance(&mut self, amount: Decimal) -> Result<(), String> {
    sqlx::query!(
        "UPDATE accounts SET balance = balance + $1 WHERE id = $2",
        amount,
        self.id
    )
    .execute(&self.pool)
    .await?;
    Ok(())
}
```

### 4. Bugs de Null/Option

```rust
// Bug: Unwrap panic
pub fn get_owner_name(&self) -> String {
    self.owner.unwrap().name  // ❌ Panic si None !
}

// Fix: Gérer le cas None
pub fn get_owner_name(&self) -> Option<String> {
    self.owner.as_ref().map(|o| o.name.clone())
}
```

---

## Commandes Rapides

```bash
# Reproduire le bug
cargo test --lib test_bug_name -- --nocapture

# Fixer et valider
cargo test --lib test_bug_name

# Tests de régression
cargo test --lib

# Pre-commit + Pre-push
make pre-commit && make ci

# Commit et PR
git checkout -b fix/description
git add .
git commit -m "fix: Description (Fixes #123)"
git push -u origin fix/description
gh pr create
```

---

## Ressources

- [Feature Workflow](.claude/guides/feature-workflow.md)
- [Testing Guide](.claude/guides/testing-guide.md)
- [Architecture Guide](.claude/guides/architecture-guide.md)
