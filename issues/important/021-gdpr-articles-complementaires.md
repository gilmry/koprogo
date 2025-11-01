# Issue #021 - GDPR Articles Complémentaires (16, 18, 21)

**Priorité**: 🟡 HIGH
**Estimation**: 5-7 heures
**Labels**: `enhancement`, `backend`, `frontend`, `gdpr`, `legal-compliance`, `privacy`

---

## 📋 Description

Compléter l'implémentation GDPR en ajoutant les **Articles 16, 18 et 21** du RGPD. Actuellement, KoproGo implémente les Articles 15 (Right to Access) et 17 (Right to Erasure). Pour une conformité GDPR complète, il manque :
- **Article 16** : Droit à la rectification
- **Article 18** : Droit à la limitation du traitement
- **Article 21** : Droit d'opposition

**Contexte légal** : Le RGPD impose aux responsables de traitement de données de fournir des mécanismes pour que les utilisateurs puissent exercer **tous** leurs droits, pas seulement certains. Ne pas implémenter ces droits expose à des sanctions.

**Impact métier** : Conformité légale complète + renforce la confiance utilisateurs.

---

## 🎯 Objectifs

- [ ] Implémenter Article 16 (Rectification)
- [ ] Implémenter Article 18 (Limitation du traitement)
- [ ] Implémenter Article 21 (Droit d'opposition)
- [ ] Exposer endpoints API pour chaque droit
- [ ] Interface frontend pour exercer ces droits
- [ ] Audit logs pour toutes les demandes
- [ ] Rate limiting (max 10 demandes/jour)

---

## 📐 Spécifications Techniques

### Article 16 : Droit à la Rectification

Permet à l'utilisateur de **corriger ses données personnelles** si elles sont inexactes ou incomplètes.

**Use cases** :
- Corriger nom/prénom mal orthographié
- Mettre à jour email/téléphone
- Corriger adresse postale

**Implémentation** :
```rust
// Endpoint: PATCH /api/v1/gdpr/rectify
#[derive(Deserialize)]
pub struct RectificationRequest {
    pub changes: Vec<FieldChange>,
}

#[derive(Deserialize)]
pub struct FieldChange {
    pub field: String,       // "first_name", "email", "phone", etc.
    pub old_value: String,
    pub new_value: String,
    pub reason: String,      // Justification
}
```

**Workflow** :
1. User soumet demande rectification
2. System valide les changements (format email, etc.)
3. Admin (optionnel) approuve les changements sensibles (email)
4. System applique les changements
5. Log audit
6. Notification email confirmation

---

### Article 18 : Droit à la Limitation du Traitement

Permet à l'utilisateur de **geler temporairement** le traitement de ses données, sans les supprimer.

**Use cases** :
- Contestation exactitude données (pendant vérification)
- Opposition traitement (en attente décision)
- Besoin conservation pour défense droits

**Implémentation** :
```rust
// Endpoint: POST /api/v1/gdpr/restrict
#[derive(Deserialize)]
pub struct RestrictionRequest {
    pub reason: RestrictionReason,
    pub effective_from: Option<NaiveDate>,
    pub effective_until: Option<NaiveDate>,
    pub description: String,
}

#[derive(Deserialize, Clone)]
pub enum RestrictionReason {
    DataAccuracyDispute,     // Contestation exactitude
    ProcessingObjection,     // Opposition traitement
    UnlawfulProcessing,      // Traitement illicite
    LegalClaimPreservation,  // Conservation pour défense droits
}
```

**Comportement** :
- Données marquées `processing_restricted = true`
- Pas de traitement marketing/analytique
- Données conservées mais non utilisées
- Levée restriction sur demande ou après délai

---

### Article 21 : Droit d'Opposition

Permet à l'utilisateur de **s'opposer** à certains traitements, notamment marketing et profilage.

**Use cases** :
- Opt-out marketing emails
- Refus profilage
- Opposition traitement basé sur intérêt légitime

**Implémentation** :
```rust
// Endpoint: POST /api/v1/gdpr/object
#[derive(Deserialize)]
pub struct ObjectionRequest {
    pub objection_type: ObjectionType,
    pub processing_purposes: Vec<String>,
    pub description: String,
}

#[derive(Deserialize, Clone)]
pub enum ObjectionType {
    Marketing,              // Marketing direct
    Profiling,              // Profilage/décisions automatisées
    LegitimateInterest,     // Traitement basé intérêt légitime
    Research,               // Recherche scientifique/historique
}
```

**Comportement** :
- Arrêt immédiat traitement marketing si demandé
- Flag `marketing_consent = false`
- Pas d'impact sur traitements nécessaires au service

---

## 🔧 Détails d'Implémentation

### 1. Domain Layer - Entities (Existantes à modifier)

**Fichier** : `backend/src/domain/entities/gdpr_rectification.rs` (déjà existe)

Vérifier que l'entity `GdprRectificationRequest` est complète et ajouter méthodes si nécessaire.

**Fichier** : `backend/src/domain/entities/gdpr_restriction.rs` (déjà existe)

Vérifier `GdprRestrictionRequest`.

**Fichier** : `backend/src/domain/entities/gdpr_objection.rs` (déjà existe)

Vérifier `GdprObjectionRequest`.

---

### 2. Application Layer - Use Cases

**Fichier** : `backend/src/application/use_cases/gdpr_use_cases.rs` (modifier existant)

Ajouter les méthodes :

```rust
impl GdprUseCases {
    // Article 16 - Rectification
    pub async fn request_rectification(
        &self,
        user_id: Uuid,
        changes: Vec<FieldChange>,
    ) -> Result<GdprRectificationRequest, String> {
        // 1. Valider les changements
        for change in &changes {
            self.validate_field_change(change)?;
        }

        // 2. Créer demande rectification
        let request = GdprRectificationRequest::new(user_id, changes);

        // 3. Sauvegarder
        self.rectification_repo.create(&request).await?;

        // 4. Log audit
        self.audit_logger.log(AuditEvent::GdprRectificationRequested {
            user_id,
            changes_count: request.changes.len(),
        }).await;

        Ok(request)
    }

    pub async fn approve_rectification(
        &self,
        request_id: Uuid,
        approved_by: Uuid,
    ) -> Result<(), String> {
        let mut request = self.rectification_repo.find_by_id(request_id).await?
            .ok_or("Request not found")?;

        request.approve();
        self.rectification_repo.update(&request).await?;

        // Appliquer les changements
        self.apply_field_changes(request.user_id, &request.changes).await?;

        // Log audit
        self.audit_logger.log(AuditEvent::GdprRectificationApplied {
            user_id: request.user_id,
            approved_by,
        }).await;

        Ok(())
    }

    // Article 18 - Restriction
    pub async fn request_restriction(
        &self,
        user_id: Uuid,
        reason: RestrictionReason,
        effective_from: Option<NaiveDate>,
        effective_until: Option<NaiveDate>,
        description: String,
    ) -> Result<GdprRestrictionRequest, String> {
        let request = GdprRestrictionRequest::new(
            user_id,
            reason,
            effective_from,
            effective_until,
            description,
        );

        self.restriction_repo.create(&request).await?;

        // Activer restriction
        self.activate_restriction(user_id).await?;

        // Log audit
        self.audit_logger.log(AuditEvent::GdprRestrictionRequested {
            user_id,
            reason: request.reason.clone(),
        }).await;

        Ok(request)
    }

    pub async fn lift_restriction(
        &self,
        user_id: Uuid,
    ) -> Result<(), String> {
        // Mettre fin à la restriction
        let mut request = self.restriction_repo.find_active_by_user(user_id).await?
            .ok_or("No active restriction found")?;

        request.lift();
        self.restriction_repo.update(&request).await?;

        // Désactiver flag
        self.deactivate_restriction(user_id).await?;

        // Log audit
        self.audit_logger.log(AuditEvent::GdprRestrictionLifted {
            user_id,
        }).await;

        Ok(())
    }

    // Article 21 - Objection
    pub async fn request_objection(
        &self,
        user_id: Uuid,
        objection_type: ObjectionType,
        processing_purposes: Vec<String>,
        description: String,
    ) -> Result<GdprObjectionRequest, String> {
        let request = GdprObjectionRequest::new(
            user_id,
            objection_type.clone(),
            processing_purposes.clone(),
            description,
        );

        self.objection_repo.create(&request).await?;

        // Accepter automatiquement les objections marketing
        if objection_type == ObjectionType::Marketing {
            self.accept_objection(request.id, None).await?;
        }

        // Log audit
        self.audit_logger.log(AuditEvent::GdprObjectionRequested {
            user_id,
            objection_type,
        }).await;

        Ok(request)
    }

    pub async fn accept_objection(
        &self,
        request_id: Uuid,
        accepted_by: Option<Uuid>,
    ) -> Result<(), String> {
        let mut request = self.objection_repo.find_by_id(request_id).await?
            .ok_or("Request not found")?;

        request.accept();
        self.objection_repo.update(&request).await?;

        // Appliquer objection (stop marketing, etc.)
        self.apply_objection(request.user_id, &request.objection_type, &request.processing_purposes).await?;

        // Log audit
        self.audit_logger.log(AuditEvent::GdprObjectionAccepted {
            user_id: request.user_id,
            accepted_by,
        }).await;

        Ok(())
    }

    async fn validate_field_change(&self, change: &FieldChange) -> Result<(), String> {
        match change.field.as_str() {
            "email" => {
                if !change.new_value.contains('@') {
                    return Err("Invalid email format".to_string());
                }
            }
            "phone" => {
                // Validation téléphone basique
                if change.new_value.len() < 8 {
                    return Err("Invalid phone format".to_string());
                }
            }
            "first_name" | "last_name" => {
                if change.new_value.trim().is_empty() {
                    return Err(format!("{} cannot be empty", change.field));
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn apply_field_changes(
        &self,
        user_id: Uuid,
        changes: &[FieldChange],
    ) -> Result<(), String> {
        let mut user = self.user_repo.find_by_id(user_id).await?
            .ok_or("User not found")?;

        for change in changes {
            match change.field.as_str() {
                "email" => user.email = change.new_value.clone(),
                "first_name" => user.first_name = change.new_value.clone(),
                "last_name" => user.last_name = change.new_value.clone(),
                "phone" => { /* Update phone if field exists */ }
                _ => {}
            }
        }

        self.user_repo.update(&user).await?;
        Ok(())
    }

    async fn activate_restriction(&self, user_id: Uuid) -> Result<(), String> {
        // Marquer user.processing_restricted = true
        let mut user = self.user_repo.find_by_id(user_id).await?
            .ok_or("User not found")?;

        // Ajouter champ processing_restricted à User entity si n'existe pas
        // user.processing_restricted = true;

        self.user_repo.update(&user).await?;
        Ok(())
    }

    async fn deactivate_restriction(&self, user_id: Uuid) -> Result<(), String> {
        // Marquer user.processing_restricted = false
        let mut user = self.user_repo.find_by_id(user_id).await?
            .ok_or("User not found")?;

        // user.processing_restricted = false;

        self.user_repo.update(&user).await?;
        Ok(())
    }

    async fn apply_objection(
        &self,
        user_id: Uuid,
        objection_type: &ObjectionType,
        processing_purposes: &[String],
    ) -> Result<(), String> {
        match objection_type {
            ObjectionType::Marketing => {
                // Stop tous les emails marketing
                // user.marketing_consent = false;
                // TODO: Update user preferences
            }
            ObjectionType::Profiling => {
                // Désactiver analytics/profilage
                // user.profiling_consent = false;
            }
            _ => {}
        }
        Ok(())
    }
}
```

---

### 3. Infrastructure Layer - Handlers

**Fichier** : `backend/src/infrastructure/web/handlers/gdpr_handlers.rs` (modifier existant)

Ajouter les endpoints :

```rust
// Article 16
pub async fn request_rectification(
    use_cases: web::Data<Arc<GdprUseCases>>,
    user: AuthenticatedUser,
    request: web::Json<RectificationRequest>,
) -> Result<HttpResponse> {
    match use_cases.request_rectification(user.user_id, request.changes.clone()).await {
        Ok(req) => Ok(HttpResponse::Created().json(req)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

// Article 18
pub async fn request_restriction(
    use_cases: web::Data<Arc<GdprUseCases>>,
    user: AuthenticatedUser,
    request: web::Json<RestrictionRequest>,
) -> Result<HttpResponse> {
    match use_cases.request_restriction(
        user.user_id,
        request.reason.clone(),
        request.effective_from,
        request.effective_until,
        request.description.clone(),
    ).await {
        Ok(req) => Ok(HttpResponse::Created().json(req)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

pub async fn lift_restriction(
    use_cases: web::Data<Arc<GdprUseCases>>,
    user: AuthenticatedUser,
) -> Result<HttpResponse> {
    match use_cases.lift_restriction(user.user_id).await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({"message": "Restriction lifted"}))),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

// Article 21
pub async fn request_objection(
    use_cases: web::Data<Arc<GdprUseCases>>,
    user: AuthenticatedUser,
    request: web::Json<ObjectionRequest>,
) -> Result<HttpResponse> {
    match use_cases.request_objection(
        user.user_id,
        request.objection_type.clone(),
        request.processing_purposes.clone(),
        request.description.clone(),
    ).await {
        Ok(req) => Ok(HttpResponse::Created().json(req)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}
```

---

### 4. Frontend - GDPR Panel Extension

**Fichier** : `frontend/src/components/GdprDataPanel.svelte` (modifier existant)

Ajouter sections :

```svelte
<div class="gdpr-section">
    <h3>Article 16 - Droit à la Rectification</h3>
    <p>Corriger vos données personnelles inexactes ou incomplètes.</p>
    <button on:click={openRectificationModal}>Demander une rectification</button>
</div>

<div class="gdpr-section">
    <h3>Article 18 - Limitation du Traitement</h3>
    <p>Geler temporairement l'utilisation de vos données.</p>
    {#if restrictionActive}
        <p class="warning">⚠️ Restriction active</p>
        <button on:click={liftRestriction}>Lever la restriction</button>
    {:else}
        <button on:click={requestRestriction}>Demander une restriction</button>
    {/if}
</div>

<div class="gdpr-section">
    <h3>Article 21 - Droit d'Opposition</h3>
    <p>S'opposer au traitement de vos données pour certains usages.</p>
    <label>
        <input type="checkbox" bind:checked={marketingConsent} on:change={updateMarketingConsent}>
        J'accepte de recevoir des emails marketing
    </label>
    <label>
        <input type="checkbox" bind:checked={profilingConsent} on:change={updateProfilingConsent}>
        J'accepte le profilage de mes données
    </label>
</div>
```

---

## ✅ Critères d'Acceptation

### Fonctionnels
- [ ] User peut demander rectification données
- [ ] User peut restreindre traitement temporairement
- [ ] User peut s'opposer au marketing
- [ ] Admin peut approuver rectifications sensibles
- [ ] Tous les événements sont loggés (audit trail)

### Techniques
- [ ] Tests unitaires (15+ tests)
- [ ] Tests E2E pour chaque endpoint
- [ ] Rate limiting implémenté
- [ ] Frontend complet avec modals

---

## 🧪 Plan de Tests

```rust
#[actix_rt::test]
async fn test_request_rectification() {
    // Créer user
    // Demander rectification email
    // Vérifier request créée
    // Approuver rectification
    // Vérifier email modifié
}

#[actix_rt::test]
async fn test_restrict_processing() {
    // Créer user
    // Demander restriction
    // Vérifier processing_restricted = true
    // Lever restriction
    // Vérifier processing_restricted = false
}
```

---

## 🔗 Dépendances

### Bloquantes
- ✅ GDPR Articles 15 & 17 implémentés
- ✅ Audit logs existants

### Optionnelles
- Issue #009 : Notifications (email confirmation)

---

## 🚀 Checklist

- [ ] 1. Vérifier entities GDPR existantes
- [ ] 2. Compléter `gdpr_use_cases.rs`
- [ ] 3. Ajouter endpoints API
- [ ] 4. Ajouter routes
- [ ] 5. Tests unitaires (15+ tests)
- [ ] 6. Tests E2E (8+ tests)
- [ ] 7. Frontend: modifier `GdprDataPanel.svelte`
- [ ] 8. Frontend: modals rectification/restriction
- [ ] 9. Documentation GDPR compliance
- [ ] 10. Commit : `feat: implement GDPR Articles 16, 18, 21 (rectification, restriction, objection)`

---

**Créé le** : 2025-11-01
**Milestone** : v1.0 - GDPR Compliance Complete
**Impact** : HIGH - Conformité légale RGPD complète
