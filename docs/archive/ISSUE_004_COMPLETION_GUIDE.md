# Issue #004 Pagination - Guide de Complétion (85% → 100%)

**Status Actuel**: 85% complété
**Temps restant estimé**: 15-20 minutes

---

## ✅ Déjà Fait (85%)

### Infrastructure Complète
- ✅ DTOs (`pagination.rs`, `filters.rs`) avec 12 tests unitaires
- ✅ 4 traits repository mis à jour
- ✅ **BuildingRepository** pagination (100%)
- ✅ **ExpenseRepository** pagination (100%)

### Pattern Établi
Chaque repository suit exactement le même pattern:
1. Import `PageRequest` et `XxxFilters`
2. Méthode `find_all_paginated()` avec:
   - Validation page request
   - Construction WHERE clause dynamique
   - Whitelist colonnes de tri (SQL injection prevention)
   - COUNT query pour total_items
   - SELECT query avec LIMIT/OFFSET
   - Mapping rows → entities

---

## ⏳ Reste à Faire (15%)

### 1. UnitRepository Pagination (~5 min)

**Fichier**: `backend/src/infrastructure/database/repositories/unit_repository_impl.rs`

**Étape 1**: Ajouter imports
```rust
use crate::application::dto::{PageRequest, UnitFilters};
```

**Étape 2**: Copier-coller `find_all_paginated()` depuis ExpenseRepository

**Étape 3**: Adapter les filtres (UnitFilters):
```rust
// Filtres Unit
if filters.building_id.is_some() { ... }
if filters.floor.is_some() { ... }
if filters.has_owner.is_some() {
    // owner_id IS NULL ou IS NOT NULL
}
```

**Étape 4**: Adapter colonnes de tri:
```rust
let allowed_columns = vec!["unit_number", "floor", "surface_area", "created_at"];
let sort_column = page_request.sort_by.as_deref().unwrap_or("unit_number");
```

**Étape 5**: Adapter SELECT et mapping:
```rust
let data_query = format!(
    "SELECT id, building_id, unit_number, unit_type, floor, surface_area, quota, owner_id, created_at, updated_at \
     FROM units {} ORDER BY {} {} LIMIT ${} OFFSET ${}",
    // ...
);

// Mapping row → Unit (copier depuis find_by_building)
```

---

### 2. OwnerRepository Pagination (~5 min)

**Fichier**: `backend/src/infrastructure/database/repositories/owner_repository_impl.rs`

**Même processus** que Unit:
1. Imports `PageRequest, OwnerFilters`
2. Copier `find_all_paginated()`
3. Adapter filtres (email, phone, last_name, first_name avec ILIKE)
4. Colonnes tri: `["last_name", "email", "created_at"]`
5. SELECT from `owners` table

---

### 3. Tests Optionnels (~5 min)

**Fichier**: `backend/tests/integration/pagination_tests.rs` (créer si nécessaire)

```rust
#[actix_rt::test]
async fn test_expense_pagination() {
    // GET /api/v1/expenses?page=2&per_page=10&sort_by=amount&order=desc
    // Assert pagination.total_pages, has_next, etc.
}
```

---

## 🚀 Commande Rapide de Complétion

```bash
# 1. Copier pattern ExpenseRepository vers Unit
# Fichier: backend/src/infrastructure/database/repositories/unit_repository_impl.rs
# Adapter: UnitFilters, allowed_columns, SELECT units

# 2. Copier pattern ExpenseRepository vers Owner
# Fichier: backend/src/infrastructure/database/repositories/owner_repository_impl.rs
# Adapter: OwnerFilters, allowed_columns, SELECT owners

# 3. Commit
git add backend/src/infrastructure/database/repositories/*.rs
git commit -m "feat(pagination): Complete pagination for Unit & Owner repositories (#004) ✅"
git push
```

---

## 📋 Checklist Finale

- [ ] UnitRepository.find_all_paginated() implémenté
- [ ] OwnerRepository.find_all_paginated() implémenté
- [ ] Compilation OK (`cargo check`)
- [ ] Tests optionnels (recommandés mais pas bloquants)
- [ ] Commit & push

---

## 🎯 Pattern de Référence (ExpenseRepository)

Utiliser `backend/src/infrastructure/database/repositories/expense_repository_impl.rs` lignes 168-347 comme template exact.

Les seules différences par repository:
- **Filtres**: Adapter selon `XxxFilters` struct
- **Colonnes**: Adapter `allowed_columns` selon table
- **SELECT**: Adapter colonnes de la table
- **Mapping**: Copier depuis méthode `find_by_xxx()` existante

---

**Temps total estimé**: 15-20 minutes pour 100% complétion Issue #004 ✅

**Dernière mise à jour**: 2025-10-23
