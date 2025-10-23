# Issue #004 - Pagination et Filtrage des Endpoints API

**Priorité**: 🔴 CRITIQUE
**Estimation**: 3-4 heures
**Labels**: `enhancement`, `backend`, `performance`, `critical`

---

## 📋 Description

Ajouter la pagination et le filtrage à tous les endpoints de liste (GET collections). Actuellement, tous les endpoints retournent l'intégralité des résultats sans pagination, ce qui pose des problèmes de performance et d'expérience utilisateur.

**Problème actuel** :
- `GET /buildings` retourne tous les immeubles (potentiellement des milliers)
- `GET /expenses` retourne toutes les dépenses sans limite
- Aucun filtre par date, statut, ou autres critères
- Impossible de trier les résultats

**Impact** :
- ❌ Performance dégradée avec datasets larges
- ❌ Expérience utilisateur médiocre (chargement lent)
- ❌ Consommation mémoire excessive côté client
- ❌ Non conforme aux best practices REST API

---

## 🎯 Objectifs

- [ ] Implémenter pagination générique pour tous les endpoints de liste
- [ ] Ajouter support de filtrage par champs communs
- [ ] Permettre le tri (ordering) par colonnes
- [ ] Créer des structs réutilisables (`PageRequest`, `PageResponse`)
- [ ] Documenter format de query parameters
- [ ] Maintenir rétrocompatibilité (pagination optionnelle)

---

## 📐 Spécifications Techniques

### Format de Pagination (Standard)

**Query Parameters** :
```
?page=1&per_page=20&sort_by=created_at&order=desc
```

**Response Format** :
```json
{
  "data": [...],
  "pagination": {
    "current_page": 1,
    "per_page": 20,
    "total_items": 156,
    "total_pages": 8,
    "has_next": true,
    "has_previous": false
  }
}
```

### Endpoints à modifier

| Endpoint | Filtres | Tri |
|----------|---------|-----|
| `GET /buildings` | `city`, `construction_year` | `name`, `created_at`, `total_units` |
| `GET /units` | `unit_type`, `building_id` | `unit_number`, `area` |
| `GET /owners` | `email`, `phone` | `last_name`, `created_at` |
| `GET /expenses` | `status`, `category`, `date_from`, `date_to` | `amount`, `due_date`, `created_at` |
| `GET /buildings/:id/units` | `unit_type`, `has_owner` | `unit_number`, `floor` |
| `GET /buildings/:id/expenses` | `status`, `paid` | `due_date`, `amount` |
| `GET /meetings` | `status`, `meeting_type`, `date_from` | `scheduled_at` |
| `GET /documents` | `document_type`, `building_id` | `created_at`, `title` |

---

## 🔧 Détails d'Implémentation

### 1. Structs Génériques

**Fichier** : `backend/src/application/dto/pagination.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PageRequest {
    #[serde(default = "default_page")]
    pub page: i64,

    #[serde(default = "default_per_page")]
    pub per_page: i64,

    pub sort_by: Option<String>,

    #[serde(default)]
    pub order: SortOrder,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    20
}

impl PageRequest {
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.per_page
    }

    pub fn limit(&self) -> i64 {
        self.per_page.min(100) // Max 100 items per page
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.page < 1 {
            return Err("page must be >= 1".to_string());
        }
        if self.per_page < 1 || self.per_page > 100 {
            return Err("per_page must be between 1 and 100".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}

impl SortOrder {
    pub fn to_sql(&self) -> &str {
        match self {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PageResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub current_page: i64,
    pub per_page: i64,
    pub total_items: i64,
    pub total_pages: i64,
    pub has_next: bool,
    pub has_previous: bool,
}

impl PaginationMeta {
    pub fn new(current_page: i64, per_page: i64, total_items: i64) -> Self {
        let total_pages = (total_items as f64 / per_page as f64).ceil() as i64;
        Self {
            current_page,
            per_page,
            total_items,
            total_pages,
            has_next: current_page < total_pages,
            has_previous: current_page > 1,
        }
    }
}
```

### 2. Filtres Spécifiques

**Fichier** : `backend/src/application/dto/filters.rs`

```rust
use serde::Deserialize;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::domain::entities::expense::{ExpenseStatus, ExpenseCategory};
use crate::domain::entities::meeting::MeetingStatus;

#[derive(Debug, Deserialize)]
pub struct BuildingFilters {
    pub city: Option<String>,
    pub construction_year: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ExpenseFilters {
    pub status: Option<ExpenseStatus>,
    pub category: Option<ExpenseCategory>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub building_id: Option<Uuid>,
    pub paid: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct MeetingFilters {
    pub status: Option<MeetingStatus>,
    pub building_id: Option<Uuid>,
    pub date_from: Option<DateTime<Utc>>,
}
```

### 3. Repository Updates

**Exemple** : `backend/src/application/ports/building_repository.rs`

```rust
use crate::domain::entities::building::Building;
use crate::application::dto::pagination::PageRequest;
use crate::application::dto::filters::BuildingFilters;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait BuildingRepository: Send + Sync {
    async fn create(&self, building: &Building) -> Result<Building, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Building, String>;

    // Nouvelle méthode avec pagination
    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        filters: &BuildingFilters,
    ) -> Result<(Vec<Building>, i64), String>;

    // Ancienne méthode (deprecated, pour rétrocompatibilité)
    async fn find_all(&self) -> Result<Vec<Building>, String>;

    async fn update(&self, building: &Building) -> Result<Building, String>;
    async fn delete(&self, id: Uuid) -> Result<(), String>;
}
```

### 4. Repository Implementation

**Exemple** : `backend/src/infrastructure/database/repositories/building_repository_impl.rs`

```rust
use crate::application::ports::building_repository::BuildingRepository;
use crate::application::dto::pagination::PageRequest;
use crate::application::dto::filters::BuildingFilters;
use crate::domain::entities::building::Building;
use sqlx::PgPool;
use async_trait::async_trait;

pub struct PostgresBuildingRepository {
    pool: PgPool,
}

#[async_trait]
impl BuildingRepository for PostgresBuildingRepository {
    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        filters: &BuildingFilters,
    ) -> Result<(Vec<Building>, i64), String> {
        page_request.validate()?;

        // Construire WHERE clause dynamiquement
        let mut where_clauses = Vec::new();
        let mut params: Vec<String> = Vec::new();

        if let Some(city) = &filters.city {
            where_clauses.push(format!("city ILIKE ${}",  params.len() + 1));
            params.push(format!("%{}%", city));
        }

        if let Some(year) = filters.construction_year {
            where_clauses.push(format!("construction_year = ${}", params.len() + 1));
            params.push(year.to_string());
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Déterminer colonne de tri
        let sort_column = page_request
            .sort_by
            .as_deref()
            .unwrap_or("created_at");
        let allowed_columns = vec!["name", "created_at", "total_units", "city"];
        if !allowed_columns.contains(&sort_column) {
            return Err(format!("Invalid sort column: {}", sort_column));
        }

        // Query avec COUNT total
        let count_query = format!(
            "SELECT COUNT(*) FROM buildings {}",
            where_clause
        );

        let total_items: i64 = sqlx::query_scalar(&count_query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        // Query paginée
        let data_query = format!(
            "SELECT * FROM buildings {} ORDER BY {} {} LIMIT $1 OFFSET $2",
            where_clause,
            sort_column,
            page_request.order.to_sql()
        );

        let buildings = sqlx::query_as::<_, Building>(&data_query)
            .bind(page_request.limit())
            .bind(page_request.offset())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok((buildings, total_items))
    }

    // ... autres méthodes
}
```

**Note** : Pour éviter les injections SQL, utiliser query builder ou macro sqlx! avec paramètres.

### 5. Use Cases Update

**Exemple** : `backend/src/application/use_cases/building_use_cases.rs`

```rust
use crate::application::dto::pagination::{PageRequest, PageResponse, PaginationMeta};
use crate::application::dto::filters::BuildingFilters;

impl BuildingUseCases {
    pub async fn find_all_paginated(
        &self,
        page_request: PageRequest,
        filters: BuildingFilters,
    ) -> Result<PageResponse<BuildingResponse>, String> {
        let (buildings, total_items) = self
            .building_repo
            .find_all_paginated(&page_request, &filters)
            .await?;

        let data: Vec<BuildingResponse> = buildings
            .into_iter()
            .map(BuildingResponse::from)
            .collect();

        Ok(PageResponse {
            data,
            pagination: PaginationMeta::new(
                page_request.page,
                page_request.per_page,
                total_items,
            ),
        })
    }
}
```

### 6. Handler Update

**Exemple** : `backend/src/infrastructure/web/handlers/building_handlers.rs`

```rust
use actix_web::{web, HttpResponse, Result};
use crate::application::use_cases::building_use_cases::BuildingUseCases;
use crate::application::dto::pagination::PageRequest;
use crate::application::dto::filters::BuildingFilters;

pub async fn list_buildings(
    use_cases: web::Data<Arc<BuildingUseCases>>,
    page_request: web::Query<PageRequest>,
    filters: web::Query<BuildingFilters>,
) -> Result<HttpResponse> {
    match use_cases
        .find_all_paginated(page_request.into_inner(), filters.into_inner())
        .await
    {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}
```

---

## ✅ Critères d'Acceptation

### Fonctionnels
- [ ] Pagination fonctionne sur tous les endpoints de liste
- [ ] `page=1` retourne premiers résultats
- [ ] `per_page` limite nombre de résultats (max 100)
- [ ] Filtres appliqués correctement (AND logique)
- [ ] Tri ASC/DESC sur colonnes autorisées
- [ ] Métadonnées pagination correctes (total_pages, has_next, etc.)
- [ ] Requêtes sans pagination retournent tous les résultats (backward compatible)

### Performance
- [ ] Query COUNT() optimisée avec indexes
- [ ] LIMIT/OFFSET performant (< 10ms pour 10000 rows)
- [ ] Validation query parameters (éviter injection SQL)

### Sécurité
- [ ] Colonnes de tri whitelist (pas de SQL injection)
- [ ] Validation stricte des paramètres
- [ ] Pas de révélation d'informations sensibles dans erreurs

### Tests
- [ ] Tests unitaires validation PageRequest
- [ ] Tests E2E pagination (page 1, 2, dernière page)
- [ ] Tests filtres (city, status, dates)
- [ ] Tests tri ASC/DESC
- [ ] Tests limites (per_page > 100, page < 1)

---

## 🧪 Plan de Tests

```rust
#[actix_rt::test]
async fn test_pagination_first_page() {
    // Créer 50 buildings
    // GET /buildings?page=1&per_page=20
    // Vérifier 20 résultats
    // Vérifier pagination.total_items = 50
    // Vérifier has_next = true
}

#[actix_rt::test]
async fn test_pagination_last_page() {
    // Créer 45 buildings
    // GET /buildings?page=3&per_page=20
    // Vérifier 5 résultats
    // Vérifier has_next = false
}

#[actix_rt::test]
async fn test_filter_by_city() {
    // Créer 5 buildings à Paris, 3 à Lyon
    // GET /buildings?city=Paris
    // Vérifier 5 résultats
}

#[actix_rt::test]
async fn test_sort_by_name_desc() {
    // Créer buildings: "A", "C", "B"
    // GET /buildings?sort_by=name&order=desc
    // Vérifier ordre: C, B, A
}

#[actix_rt::test]
async fn test_invalid_sort_column() {
    // GET /buildings?sort_by=password
    // Vérifier 400 Bad Request
}

#[actix_rt::test]
async fn test_per_page_exceeds_limit() {
    // GET /buildings?per_page=500
    // Vérifier limité à 100 résultats
}
```

---

## 📚 Exemples d'Utilisation

### Exemple 1 : Liste paginée de buildings
```bash
curl "http://localhost:8080/api/v1/buildings?page=2&per_page=10&sort_by=name&order=asc"
```

**Response** :
```json
{
  "data": [
    {
      "id": "...",
      "name": "Building Alpha",
      ...
    }
  ],
  "pagination": {
    "current_page": 2,
    "per_page": 10,
    "total_items": 47,
    "total_pages": 5,
    "has_next": true,
    "has_previous": true
  }
}
```

### Exemple 2 : Filtrer expenses par statut et dates
```bash
curl "http://localhost:8080/api/v1/expenses?status=Unpaid&date_from=2025-01-01&date_to=2025-03-31&page=1&per_page=50"
```

### Exemple 3 : Tri par montant décroissant
```bash
curl "http://localhost:8080/api/v1/expenses?sort_by=amount&order=desc"
```

---

## 🔗 Dépendances

### Bloquantes
- Aucune (modification de code existant)

### Optionnelles
- Cursor-based pagination (pour très larges datasets)
- ElasticSearch pour recherche full-text avancée

---

## 🚀 Checklist de Développement

- [ ] 1. Créer `dto/pagination.rs` (PageRequest, PageResponse, PaginationMeta)
- [ ] 2. Créer `dto/filters.rs` (BuildingFilters, ExpenseFilters, etc.)
- [ ] 3. Modifier tous les traits Repository (ajouter méthodes `*_paginated`)
- [ ] 4. Implémenter dans PostgreSQL repositories avec LIMIT/OFFSET
- [ ] 5. Mettre à jour Use Cases
- [ ] 6. Mettre à jour Handlers (extraire query params)
- [ ] 7. Ajouter validation dans PageRequest
- [ ] 8. Tests unitaires pagination.rs
- [ ] 9. Tests E2E pour chaque endpoint
- [ ] 10. Documentation OpenAPI
- [ ] 11. Mise à jour frontend (composants List avec pagination)
- [ ] 12. Commit : `feat: add pagination and filtering to all list endpoints`

---

## 📊 Impact sur Endpoints

| Endpoint | Before | After |
|----------|--------|-------|
| `GET /buildings` | 1 query | 2 queries (COUNT + SELECT) |
| Réponse | `[...]` | `{data: [...], pagination: {...}}` |
| Performance | O(N) | O(per_page) |

---

**Créé le** : 2025-10-23
**Milestone** : v1.0 - MVP Complet
**Breaking Change** : ⚠️ Oui (format de réponse modifié, mais rétrocompatible si pagination omise)
