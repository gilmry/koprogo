# Issue #001 - Gestion des Assemblées Générales (API complète)

**Priorité**: 🔴 CRITIQUE
**Estimation**: 6-8 heures
**Labels**: `enhancement`, `backend`, `critical`, `domain-logic`

---

## 📋 Description

Implémenter la couche Application et Infrastructure pour la gestion complète des assemblées générales. L'entité de domaine `Meeting` et le schéma de base de données existent déjà, mais aucune API n'est exposée.

**Contexte métier** : Les assemblées générales sont une obligation légale pour toute copropriété. Le système doit permettre de planifier, gérer et archiver les AG (ordinaires et extraordinaires).

---

## 🎯 Objectifs

- [ ] Créer les Use Cases pour les opérations métier sur les meetings
- [ ] Implémenter les handlers HTTP (Actix-web)
- [ ] Exposer les endpoints REST API
- [ ] Ajouter les tests E2E pour valider le flux complet
- [ ] Documenter l'API (commentaires OpenAPI-ready)

---

## 📐 Spécifications Techniques

### Architecture (Hexagonal)

```
Domain (✅ EXISTANT)
  └─ entities/meeting.rs (Meeting, MeetingType, MeetingStatus)

Application (❌ À CRÉER)
  ├─ use_cases/meeting_use_cases.rs
  └─ dto/meeting_dto.rs (CreateMeetingRequest, UpdateMeetingRequest, MeetingResponse)

Infrastructure (❌ À CRÉER)
  ├─ web/handlers/meeting_handlers.rs
  └─ web/routes.rs (ajouter les routes meetings)
```

### Endpoints à implémenter

| Méthode | Endpoint | Description | Auth |
|---------|----------|-------------|------|
| `POST` | `/api/v1/meetings` | Créer une assemblée | Syndic+ |
| `GET` | `/api/v1/meetings` | Lister toutes les assemblées | Owner+ |
| `GET` | `/api/v1/meetings/:id` | Détails d'une assemblée | Owner+ |
| `PUT` | `/api/v1/meetings/:id` | Modifier une assemblée | Syndic+ |
| `DELETE` | `/api/v1/meetings/:id` | Annuler une assemblée | Syndic+ |
| `GET` | `/api/v1/buildings/:id/meetings` | Meetings d'un immeuble | Owner+ |
| `PUT` | `/api/v1/meetings/:id/agenda` | Mettre à jour l'ordre du jour | Syndic+ |
| `PUT` | `/api/v1/meetings/:id/minutes` | Ajouter le procès-verbal | Syndic+ |
| `PUT` | `/api/v1/meetings/:id/attendance` | Enregistrer présence | Syndic+ |
| `GET` | `/api/v1/meetings/:id/attendees` | Liste des participants | Owner+ |

---

## 📝 User Stories

### US1 - Création d'assemblée (Syndic)
```gherkin
En tant que syndic
Je veux créer une nouvelle assemblée générale
Afin de convoquer les copropriétaires

Scénario: Création AG ordinaire
  Étant donné que je suis authentifié en tant que Syndic
  Quand je crée une AG avec :
    - building_id: <uuid>
    - meeting_type: Ordinary
    - scheduled_at: 2025-11-15T14:00:00Z
    - location: "Salle municipale"
    - agenda: [{"item": "Approbation des comptes", "order": 1}]
  Alors l'assemblée est créée avec statut "Scheduled"
  Et les copropriétaires peuvent la consulter
```

### US2 - Consultation assemblées (Copropriétaire)
```gherkin
En tant que copropriétaire
Je veux voir la liste des assemblées de mon immeuble
Afin de connaître les dates et ordres du jour

Scénario: Liste des AG futures
  Étant donné que je suis copropriétaire du building_id <uuid>
  Quand je consulte GET /buildings/<uuid>/meetings?status=Scheduled
  Alors je vois toutes les assemblées à venir
  Avec leur date, type et ordre du jour
```

### US3 - Mise à jour procès-verbal (Syndic)
```gherkin
En tant que syndic
Je veux ajouter le procès-verbal après une AG
Afin d'archiver les décisions prises

Scénario: Upload procès-verbal
  Étant donné qu'une AG est en statut "Completed"
  Quand j'envoie PUT /meetings/<id>/minutes avec :
    - minutes: "Compte-rendu de l'AG du 15/11/2025..."
  Alors le procès-verbal est enregistré
  Et le statut reste "Completed"
```

---

## 🔧 Détails d'Implémentation

### 1. Application Layer - Use Cases

**Fichier** : `backend/src/application/use_cases/meeting_use_cases.rs`

```rust
use crate::application::ports::meeting_repository::MeetingRepository;
use crate::application::dto::meeting_dto::*;
use crate::domain::entities::meeting::{Meeting, MeetingType, MeetingStatus};
use std::sync::Arc;
use uuid::Uuid;

pub struct MeetingUseCases {
    meeting_repo: Arc<dyn MeetingRepository>,
}

impl MeetingUseCases {
    pub fn new(meeting_repo: Arc<dyn MeetingRepository>) -> Self {
        Self { meeting_repo }
    }

    pub async fn create_meeting(
        &self,
        request: CreateMeetingRequest,
    ) -> Result<MeetingResponse, String> {
        // 1. Valider building_id existe (appel repository)
        // 2. Créer entité Meeting via constructeur domain
        // 3. Persister via repository
        // 4. Convertir en DTO et retourner
    }

    pub async fn find_by_building(
        &self,
        building_id: Uuid,
        status: Option<MeetingStatus>,
    ) -> Result<Vec<MeetingResponse>, String> {
        // Filtrer par building et optionnellement par statut
    }

    pub async fn update_agenda(
        &self,
        meeting_id: Uuid,
        agenda: serde_json::Value,
    ) -> Result<MeetingResponse, String> {
        // 1. Récupérer meeting
        // 2. Appeler meeting.update_agenda(agenda) (méthode domain)
        // 3. Sauvegarder
    }

    pub async fn add_minutes(
        &self,
        meeting_id: Uuid,
        minutes: String,
    ) -> Result<MeetingResponse, String> {
        // 1. Récupérer meeting
        // 2. Vérifier statut = Completed
        // 3. Mettre à jour minutes
        // 4. Sauvegarder
    }

    pub async fn record_attendance(
        &self,
        meeting_id: Uuid,
        attendee_count: i32,
    ) -> Result<MeetingResponse, String> {
        // Mettre à jour attendee_count
    }
}
```

### 2. Application Layer - DTOs

**Fichier** : `backend/src/application/dto/meeting_dto.rs`

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::domain::entities::meeting::{MeetingType, MeetingStatus};

#[derive(Debug, Deserialize)]
pub struct CreateMeetingRequest {
    pub building_id: Uuid,
    pub meeting_type: MeetingType,
    pub scheduled_at: DateTime<Utc>,
    pub location: Option<String>,
    pub agenda: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMeetingRequest {
    pub scheduled_at: Option<DateTime<Utc>>,
    pub location: Option<String>,
    pub meeting_type: Option<MeetingType>,
    pub status: Option<MeetingStatus>,
}

#[derive(Debug, Serialize)]
pub struct MeetingResponse {
    pub id: Uuid,
    pub building_id: Uuid,
    pub meeting_type: MeetingType,
    pub scheduled_at: DateTime<Utc>,
    pub location: Option<String>,
    pub agenda: Option<serde_json::Value>,
    pub minutes: Option<String>,
    pub status: MeetingStatus,
    pub attendee_count: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 3. Infrastructure Layer - Handlers

**Fichier** : `backend/src/infrastructure/web/handlers/meeting_handlers.rs`

```rust
use actix_web::{web, HttpResponse, Result};
use crate::application::use_cases::meeting_use_cases::MeetingUseCases;
use crate::application::dto::meeting_dto::*;
use uuid::Uuid;
use std::sync::Arc;

pub async fn create_meeting(
    use_cases: web::Data<Arc<MeetingUseCases>>,
    request: web::Json<CreateMeetingRequest>,
) -> Result<HttpResponse> {
    match use_cases.create_meeting(request.into_inner()).await {
        Ok(meeting) => Ok(HttpResponse::Created().json(meeting)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

pub async fn get_meeting(
    use_cases: web::Data<Arc<MeetingUseCases>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    // Implémenter
}

pub async fn list_meetings(
    use_cases: web::Data<Arc<MeetingUseCases>>,
    query: web::Query<ListMeetingsQuery>,
) -> Result<HttpResponse> {
    // Implémenter avec filtres
}

pub async fn update_meeting(
    use_cases: web::Data<Arc<MeetingUseCases>>,
    path: web::Path<Uuid>,
    request: web::Json<UpdateMeetingRequest>,
) -> Result<HttpResponse> {
    // Implémenter
}

pub async fn delete_meeting(
    use_cases: web::Data<Arc<MeetingUseCases>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    // Soft delete ou hard delete selon stratégie
}

pub async fn get_building_meetings(
    use_cases: web::Data<Arc<MeetingUseCases>>,
    path: web::Path<Uuid>,
    query: web::Query<MeetingStatusFilter>,
) -> Result<HttpResponse> {
    // Liste meetings d'un building avec filtre statut optionnel
}

pub async fn update_agenda(
    use_cases: web::Data<Arc<MeetingUseCases>>,
    path: web::Path<Uuid>,
    agenda: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    // Implémenter
}

pub async fn add_minutes(
    use_cases: web::Data<Arc<MeetingUseCases>>,
    path: web::Path<Uuid>,
    request: web::Json<MinutesRequest>,
) -> Result<HttpResponse> {
    // Implémenter
}
```

### 4. Routes Configuration

**Fichier** : `backend/src/infrastructure/web/routes.rs`

Ajouter dans la fonction `configure_routes()` :

```rust
// Meeting routes
.service(
    web::scope("/meetings")
        .route("", web::post().to(meeting_handlers::create_meeting))
        .route("", web::get().to(meeting_handlers::list_meetings))
        .route("/{id}", web::get().to(meeting_handlers::get_meeting))
        .route("/{id}", web::put().to(meeting_handlers::update_meeting))
        .route("/{id}", web::delete().to(meeting_handlers::delete_meeting))
        .route("/{id}/agenda", web::put().to(meeting_handlers::update_agenda))
        .route("/{id}/minutes", web::put().to(meeting_handlers::add_minutes))
        .route("/{id}/attendance", web::put().to(meeting_handlers::record_attendance))
)
// Building-specific meetings
.route(
    "/buildings/{id}/meetings",
    web::get().to(meeting_handlers::get_building_meetings),
)
```

---

## ✅ Critères d'Acceptation

### Fonctionnels
- [ ] Un syndic peut créer une AG avec agenda
- [ ] Les copropriétaires voient les AG de leur immeuble
- [ ] Le syndic peut modifier l'ordre du jour avant l'AG
- [ ] Le syndic peut ajouter le procès-verbal après l'AG
- [ ] Le système enregistre le nombre de participants
- [ ] Impossible de modifier une AG annulée
- [ ] Validation : scheduled_at doit être dans le futur

### Techniques
- [ ] Respect de l'architecture hexagonale (pas de logique métier dans handlers)
- [ ] Tous les endpoints retournent du JSON
- [ ] Codes HTTP corrects (201 Created, 200 OK, 404 Not Found, 400 Bad Request)
- [ ] Gestion d'erreurs avec messages explicites
- [ ] Repository pattern utilisé (pas de SQL direct dans use cases)

### Tests
- [ ] Tests unitaires sur MeetingUseCases (min 5 tests)
- [ ] Tests E2E pour chaque endpoint (min 10 tests)
- [ ] Test BDD Gherkin pour user stories (fichier `.feature`)
- [ ] Tests de validation (agenda JSONB valide, dates cohérentes)

### Documentation
- [ ] Commentaires Rust sur fonctions publiques
- [ ] Exemples de requêtes dans commentaires
- [ ] Mise à jour du CHANGELOG.md

---

## 🧪 Plan de Tests

### Tests E2E à créer

**Fichier** : `backend/tests/e2e/meeting_tests.rs`

```rust
#[actix_rt::test]
async fn test_create_meeting_success() {
    // Créer building
    // Créer meeting avec agenda
    // Vérifier 201 Created
    // Vérifier données retournées
}

#[actix_rt::test]
async fn test_create_meeting_invalid_building() {
    // Tenter création avec building_id inexistant
    // Vérifier 400 Bad Request
}

#[actix_rt::test]
async fn test_list_meetings_by_building() {
    // Créer 2 meetings pour building A
    // Créer 1 meeting pour building B
    // GET /buildings/{A}/meetings
    // Vérifier 2 résultats
}

#[actix_rt::test]
async fn test_update_agenda() {
    // Créer meeting
    // PUT /meetings/{id}/agenda
    // Vérifier agenda mis à jour
}

#[actix_rt::test]
async fn test_add_minutes_to_completed_meeting() {
    // Créer meeting avec status Completed
    // PUT /meetings/{id}/minutes
    // Vérifier 200 OK
}

#[actix_rt::test]
async fn test_cannot_modify_cancelled_meeting() {
    // Créer meeting et annuler
    // Tenter PUT /meetings/{id}
    // Vérifier 400 Bad Request
}
```

### Tests BDD

**Fichier** : `backend/tests/features/meeting.feature`

```gherkin
Feature: Gestion des Assemblées Générales
  En tant que syndic
  Je veux gérer les assemblées générales
  Afin d'organiser la copropriété

  Background:
    Given un immeuble existe avec l'id "123e4567-e89b-12d3-a456-426614174000"
    And je suis authentifié en tant que Syndic

  Scenario: Créer une assemblée générale ordinaire
    When je crée une AG avec :
      | field        | value                                  |
      | building_id  | 123e4567-e89b-12d3-a456-426614174000   |
      | meeting_type | Ordinary                               |
      | scheduled_at | 2025-12-01T14:00:00Z                   |
      | location     | Salle des fêtes                        |
    Then la réponse HTTP est 201
    And l'assemblée a le statut "Scheduled"
    And la date planifiée est "2025-12-01T14:00:00Z"

  Scenario: Ajouter un ordre du jour
    Given une assemblée existe avec l'id "ag-123"
    When je mets à jour l'agenda avec :
      """
      [
        {"order": 1, "item": "Approbation des comptes"},
        {"order": 2, "item": "Vote travaux de ravalement"}
      ]
      """
    Then la réponse HTTP est 200
    And l'ordre du jour contient 2 points

  Scenario: Impossible de créer une AG dans le passé
    When je crée une AG avec scheduled_at "2024-01-01T10:00:00Z"
    Then la réponse HTTP est 400
    And le message d'erreur contient "must be in the future"
```

---

## 🔗 Dépendances

### Bloquantes
- ✅ Entité `Meeting` (déjà implémentée dans `domain/entities/meeting.rs`)
- ✅ Table `meetings` (migration déjà appliquée)
- ✅ Repository `MeetingRepository` (déjà implémenté dans `infrastructure/database/repositories/meeting_repository_impl.rs`)

### Optionnelles (pour évolutions futures)
- Issue #002 : Upload de documents (pour joindre PV d'AG)
- Issue #007 : Notifications (pour convoquer copropriétaires)

---

## 📚 Ressources

### Code existant à étudier
- `backend/src/domain/entities/meeting.rs` - Logique métier
- `backend/src/infrastructure/database/repositories/meeting_repository_impl.rs` - Implémentation repository
- `backend/migrations/20240101000005_create_meetings_table.sql` - Schéma DB

### Exemples similaires dans le codebase
- `backend/src/application/use_cases/building_use_cases.rs` - Pattern à suivre
- `backend/src/infrastructure/web/handlers/building_handlers.rs` - Structure handlers

### Documentation externe
- [Actix-web Handlers](https://actix.rs/docs/handlers/)
- [SQLx Query Builder](https://github.com/launchbadge/sqlx)

---

## 🚀 Checklist de Développement

- [ ] 1. Créer `application/dto/meeting_dto.rs`
- [ ] 2. Créer `application/use_cases/meeting_use_cases.rs`
- [ ] 3. Créer `infrastructure/web/handlers/meeting_handlers.rs`
- [ ] 4. Ajouter routes dans `infrastructure/web/routes.rs`
- [ ] 5. Enregistrer MeetingUseCases dans `main.rs` (dependency injection)
- [ ] 6. Écrire tests unitaires dans use_cases
- [ ] 7. Écrire tests E2E dans `tests/e2e/meeting_tests.rs`
- [ ] 8. Créer fichier BDD `tests/features/meeting.feature`
- [ ] 9. Tester manuellement avec curl/Postman
- [ ] 10. Mettre à jour CHANGELOG.md
- [ ] 11. Commit avec message : `feat: implement meeting management API`

---

## 📊 Métriques de Succès

- **Performance** : P99 < 5ms pour GET endpoints
- **Coverage** : 100% du code use_cases testé
- **Documentation** : Tous les endpoints documentés
- **Qualité** : 0 warning Clippy

---

**Créé le** : 2025-10-23
**Assigné à** : À définir
**Milestone** : v1.0 - MVP Complet
