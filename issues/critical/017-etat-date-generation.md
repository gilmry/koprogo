# Issue #017 - Génération État Daté (Mutations Immobilières)

**Priorité**: 🔴 CRITIQUE
**Estimation**: 6-8 heures
**Labels**: `enhancement`, `backend`, `frontend`, `critical`, `legal-compliance`, `pdf-generation`

---

## 📋 Description

Implémenter la génération automatique d'**états datés** conformes à la législation belge pour les mutations immobilières (ventes, donations, successions). L'état daté est un document **obligatoire** fourni par le syndic au notaire lors de toute transaction immobilière concernant un lot en copropriété.

**Contexte légal** : En Belgique, l'article 577-2 du Code Civil oblige le syndic à délivrer un état daté dans les 15 jours suivant la demande du notaire. Ce document certifie la situation financière du lot (charges payées/impayées, travaux votés, etc.) à une date précise.

**Impact métier** : Sans état daté conforme, les mutations immobilières sont bloquées. C'est un document critique pour le bon fonctionnement du marché immobilier.

---

## 🎯 Objectifs

- [ ] Créer l'entité domain `EtatDate`
- [ ] Implémenter la génération de données de l'état daté
- [ ] Générer un PDF conforme au format légal belge
- [ ] Exposer endpoint API pour demande d'état daté
- [ ] Créer interface frontend pour syndic
- [ ] Archiver les états datés générés
- [ ] Implémenter suivi des délais (15 jours max)

---

## 📐 Spécifications Techniques

### Contenu Légal d'un État Daté

Un état daté belge doit obligatoirement contenir :

#### 1. Informations d'Identification
- Nom de la copropriété (immeuble)
- Adresse complète de l'immeuble
- Numéro du lot concerné
- Nom du propriétaire actuel
- Quote-part dans les charges (tantièmes)
- Date de référence de l'état daté

#### 2. Situation Financière du Lot
- **Charges courantes** :
  - Montant des provisions trimestrielles/mensuelles
  - Solde dû ou créditeur au jour de l'état daté
  - Détail des impayés (montants, périodes, date de mise en demeure)
- **Charges extraordinaires** :
  - Travaux votés en AG non encore payés
  - Appels de fonds exceptionnels en cours

#### 3. Situation Juridique
- Résumé des décisions des 5 dernières AG concernant :
  - Travaux votés (montant, nature, échéance)
  - Modifications du règlement de copropriété
  - Litiges en cours impliquant la copropriété
  - Procédures judiciaires
- Existence d'un fonds de réserve (montant)
- Existence de dettes de la copropriété (emprunts, etc.)

#### 4. Informations Syndic
- Nom et coordonnées du syndic
- Date de fin de mandat
- Montant des honoraires annuels

#### 5. Certification
- Date de génération
- Signature du syndic (ou cachet électronique)
- Mention légale "Certifié conforme le [date]"

---

## 🔧 Détails d'Implémentation

### 1. Domain Layer - Entity EtatDate

**Fichier** : `backend/src/domain/entities/etat_date.rs`

```rust
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtatDate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub unit_id: Uuid,
    pub reference_date: NaiveDate,
    pub requested_by: String, // Notaire name
    pub requested_by_email: String,
    pub requested_at: DateTime<Utc>,
    pub generated_at: Option<DateTime<Utc>>,
    pub pdf_path: Option<String>,
    pub status: EtatDateStatus,
    pub data: EtatDateData,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "etat_date_status", rename_all = "snake_case")]
pub enum EtatDateStatus {
    Pending,     // Demandé, pas encore généré
    Generated,   // PDF généré
    Sent,        // Envoyé au notaire
    Expired,     // > 15 jours, doit être regénéré
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtatDateData {
    // Identification
    pub building_name: String,
    pub building_address: String,
    pub unit_number: String,
    pub owner_name: String,
    pub ownership_percentage: f64,
    pub tantiemes: i32,

    // Situation financière
    pub financial_situation: FinancialSituation,

    // Décisions AG
    pub recent_decisions: Vec<AgDecision>,

    // Informations syndic
    pub syndic_info: SyndicInfo,

    // Fonds de réserve
    pub reserve_fund: Option<ReserveFund>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialSituation {
    pub quarterly_provision: f64,
    pub balance: f64, // Négatif = impayé, Positif = créditeur
    pub unpaid_charges: Vec<UnpaidCharge>,
    pub extraordinary_calls: Vec<ExtraordinaryCall>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnpaidCharge {
    pub amount: f64,
    pub due_date: NaiveDate,
    pub description: String,
    pub notice_sent_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtraordinaryCall {
    pub amount: f64,
    pub description: String,
    pub voted_at: NaiveDate,
    pub due_date: NaiveDate,
    pub paid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgDecision {
    pub meeting_date: NaiveDate,
    pub decision_type: String, // "Travaux", "Règlement", "Litige", etc.
    pub description: String,
    pub amount: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyndicInfo {
    pub name: String,
    pub address: String,
    pub phone: String,
    pub email: String,
    pub mandate_end_date: NaiveDate,
    pub annual_fees: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReserveFund {
    pub total_amount: f64,
    pub unit_share: f64, // Part du lot
}

impl EtatDate {
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        unit_id: Uuid,
        reference_date: NaiveDate,
        requested_by: String,
        requested_by_email: String,
        data: EtatDateData,
    ) -> Result<Self, String> {
        if requested_by.trim().is_empty() {
            return Err("Requested by (notaire) cannot be empty".to_string());
        }
        if !requested_by_email.contains('@') {
            return Err("Invalid email format".to_string());
        }

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            unit_id,
            reference_date,
            requested_by,
            requested_by_email,
            requested_at: Utc::now(),
            generated_at: None,
            pdf_path: None,
            status: EtatDateStatus::Pending,
            data,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub fn mark_as_generated(&mut self, pdf_path: String) {
        self.generated_at = Some(Utc::now());
        self.pdf_path = Some(pdf_path);
        self.status = EtatDateStatus::Generated;
        self.updated_at = Utc::now();
    }

    pub fn mark_as_sent(&mut self) {
        self.status = EtatDateStatus::Sent;
        self.updated_at = Utc::now();
    }

    pub fn is_expired(&self) -> bool {
        // Un état daté expire après 15 jours (délai légal)
        let days_since_request = (Utc::now() - self.requested_at).num_days();
        days_since_request > 15
    }
}
```

---

### 2. Migration Database

**Fichier** : `backend/migrations/20251101000001_create_etat_date.sql`

```sql
-- Table pour les états datés
CREATE TYPE etat_date_status AS ENUM ('pending', 'generated', 'sent', 'expired');

CREATE TABLE etat_dates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    unit_id UUID NOT NULL REFERENCES units(id) ON DELETE CASCADE,
    reference_date DATE NOT NULL,
    requested_by VARCHAR(255) NOT NULL,
    requested_by_email VARCHAR(255) NOT NULL,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    generated_at TIMESTAMPTZ,
    pdf_path TEXT,
    status etat_date_status NOT NULL DEFAULT 'pending',
    data JSONB NOT NULL, -- EtatDateData complet en JSON
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_etat_dates_building ON etat_dates(building_id);
CREATE INDEX idx_etat_dates_unit ON etat_dates(unit_id);
CREATE INDEX idx_etat_dates_status ON etat_dates(status);
CREATE INDEX idx_etat_dates_requested_at ON etat_dates(requested_at);

-- Index pour recherche par email notaire
CREATE INDEX idx_etat_dates_requested_by_email ON etat_dates(requested_by_email);
```

---

### 3. Application Layer - Use Cases

**Fichier** : `backend/src/application/use_cases/etat_date_use_cases.rs`

```rust
use crate::domain::entities::etat_date::*;
use crate::application::ports::etat_date_repository::EtatDateRepository;
use crate::application::ports::expense_repository::ExpenseRepository;
use crate::application::ports::unit_repository::UnitRepository;
use crate::application::ports::meeting_repository::MeetingRepository;
use std::sync::Arc;
use uuid::Uuid;
use chrono::NaiveDate;

pub struct EtatDateUseCases {
    etat_date_repo: Arc<dyn EtatDateRepository>,
    expense_repo: Arc<dyn ExpenseRepository>,
    unit_repo: Arc<dyn UnitRepository>,
    meeting_repo: Arc<dyn MeetingRepository>,
}

impl EtatDateUseCases {
    pub fn new(
        etat_date_repo: Arc<dyn EtatDateRepository>,
        expense_repo: Arc<dyn ExpenseRepository>,
        unit_repo: Arc<dyn UnitRepository>,
        meeting_repo: Arc<dyn MeetingRepository>,
    ) -> Self {
        Self {
            etat_date_repo,
            expense_repo,
            unit_repo,
            meeting_repo,
        }
    }

    pub async fn request_etat_date(
        &self,
        building_id: Uuid,
        unit_id: Uuid,
        reference_date: NaiveDate,
        requested_by: String,
        requested_by_email: String,
    ) -> Result<EtatDate, String> {
        // 1. Récupérer informations du unit
        let unit = self.unit_repo.find_by_id(unit_id).await?
            .ok_or("Unit not found")?;

        // 2. Récupérer situation financière (expenses impayées)
        let financial_situation = self.calculate_financial_situation(unit_id, reference_date).await?;

        // 3. Récupérer décisions des 5 dernières AG
        let recent_decisions = self.get_recent_ag_decisions(building_id, 5).await?;

        // 4. Construire SyndicInfo (TODO: récupérer depuis organization)
        let syndic_info = SyndicInfo {
            name: "Syndic KoproGo".to_string(), // TODO: dynamic
            address: "Rue Example 1, 1000 Bruxelles".to_string(),
            phone: "+32 2 123 45 67".to_string(),
            email: "syndic@koprogo.be".to_string(),
            mandate_end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
            annual_fees: 1200.0,
        };

        // 5. Créer EtatDateData
        let data = EtatDateData {
            building_name: "Building Name".to_string(), // TODO: dynamic
            building_address: "Building Address".to_string(),
            unit_number: unit.unit_number.clone(),
            owner_name: "Owner Name".to_string(), // TODO: from unit_owners
            ownership_percentage: unit.quota.unwrap_or(0.0),
            tantiemes: (unit.quota.unwrap_or(0.0) * 1000.0) as i32,
            financial_situation,
            recent_decisions,
            syndic_info,
            reserve_fund: None, // TODO: calculate
        };

        // 6. Créer EtatDate entity
        let etat_date = EtatDate::new(
            unit.organization_id,
            building_id,
            unit_id,
            reference_date,
            requested_by,
            requested_by_email,
            data,
        )?;

        // 7. Sauvegarder
        self.etat_date_repo.create(&etat_date).await
    }

    async fn calculate_financial_situation(
        &self,
        unit_id: Uuid,
        reference_date: NaiveDate,
    ) -> Result<FinancialSituation, String> {
        // TODO: Implémenter calcul situation financière
        // - Récupérer toutes les expenses liées au unit
        // - Calculer provisions trimestrielles
        // - Identifier impayés
        // - Identifier appels de fonds extraordinaires
        todo!("Implement financial situation calculation")
    }

    async fn get_recent_ag_decisions(
        &self,
        building_id: Uuid,
        limit: usize,
    ) -> Result<Vec<AgDecision>, String> {
        // TODO: Récupérer les N dernières meetings et extraire décisions importantes
        todo!("Implement AG decisions extraction")
    }

    pub async fn generate_pdf(
        &self,
        etat_date_id: Uuid,
    ) -> Result<String, String> {
        // TODO: Implémenter génération PDF
        // - Utiliser template HTML + wkhtmltopdf ou printpdf
        // - Inclure toutes les sections légales
        // - Ajouter watermark/signature
        // - Sauvegarder PDF dans storage
        // - Marquer etat_date comme Generated
        todo!("Implement PDF generation")
    }

    pub async fn send_to_notary(
        &self,
        etat_date_id: Uuid,
    ) -> Result<(), String> {
        // TODO: Envoyer email au notaire avec PDF en pièce jointe
        // - Récupérer etat_date
        // - Générer PDF si pas encore fait
        // - Envoyer email via SMTP
        // - Marquer comme Sent
        todo!("Implement email sending")
    }
}
```

---

### 4. Endpoints API

**Fichier** : `backend/src/infrastructure/web/handlers/etat_date_handlers.rs`

```rust
use actix_web::{web, HttpResponse, Result};
use crate::application::use_cases::etat_date_use_cases::EtatDateUseCases;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDate;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct RequestEtatDateRequest {
    pub unit_id: Uuid,
    pub reference_date: NaiveDate,
    pub requested_by: String,
    pub requested_by_email: String,
}

pub async fn request_etat_date(
    use_cases: web::Data<Arc<EtatDateUseCases>>,
    building_id: web::Path<Uuid>,
    request: web::Json<RequestEtatDateRequest>,
) -> Result<HttpResponse> {
    match use_cases.request_etat_date(
        *building_id,
        request.unit_id,
        request.reference_date,
        request.requested_by.clone(),
        request.requested_by_email.clone(),
    ).await {
        Ok(etat_date) => Ok(HttpResponse::Created().json(etat_date)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

pub async fn generate_pdf(
    use_cases: web::Data<Arc<EtatDateUseCases>>,
    etat_date_id: web::Path<Uuid>,
) -> Result<HttpResponse> {
    match use_cases.generate_pdf(*etat_date_id).await {
        Ok(pdf_path) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "pdf_path": pdf_path
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(e)),
    }
}

pub async fn download_pdf(
    etat_date_id: web::Path<Uuid>,
) -> Result<HttpResponse> {
    // TODO: Serve PDF file
    todo!()
}

pub async fn send_to_notary(
    use_cases: web::Data<Arc<EtatDateUseCases>>,
    etat_date_id: web::Path<Uuid>,
) -> Result<HttpResponse> {
    match use_cases.send_to_notary(*etat_date_id).await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "État daté envoyé au notaire"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(e)),
    }
}
```

---

## ✅ Critères d'Acceptation

### Fonctionnels
- [ ] Un état daté peut être demandé pour n'importe quel lot
- [ ] Les données financières sont calculées correctement
- [ ] Le PDF généré contient toutes les sections légales obligatoires
- [ ] Les états datés de plus de 15 jours sont marqués comme expirés
- [ ] Le syndic peut télécharger le PDF généré
- [ ] L'état daté peut être envoyé par email au notaire

### Légaux (Conformité Belgique)
- [ ] Toutes les informations légales obligatoires sont présentes
- [ ] Format PDF lisible et professionnel
- [ ] Mention "Certifié conforme le [date]"
- [ ] Signature ou cachet électronique du syndic

### Techniques
- [ ] PDF généré en < 3 secondes
- [ ] Stockage sécurisé des PDF (contrôle d'accès)
- [ ] Archivage illimité (legal requirement)
- [ ] Tests E2E complets

---

## 🧪 Plan de Tests

### Tests Unitaires

```rust
#[test]
fn test_etat_date_creation() {
    // Créer EtatDate avec données valides
    // Vérifier status = Pending
    // Vérifier generated_at = None
}

#[test]
fn test_etat_date_expiration() {
    // Créer EtatDate il y a 16 jours
    // Vérifier is_expired() == true
}
```

### Tests E2E

```rust
#[actix_rt::test]
async fn test_request_etat_date_success() {
    // Créer building, unit, expenses
    // Demander état daté
    // Vérifier 201 Created
    // Vérifier données retournées
}

#[actix_rt::test]
async fn test_generate_pdf() {
    // Créer état daté
    // Générer PDF
    // Vérifier fichier existe
    // Vérifier status = Generated
}
```

---

## 🔗 Dépendances

### Bloquantes
- ✅ Unit entity exists
- ✅ Expense entity exists
- ✅ Meeting entity exists

### Nécessaires pour complétion
- Issue #047 : PDF Generation Extended (templates)
- Issue #009 : Notifications (email notaire)
- Issue #016 : Plan Comptable Belge (calcul financier précis)

---

## 📚 Ressources

### Références Légales
- **Article 577-2 Code Civil belge** : Obligation état daté
- **Délai légal** : 15 jours maximum
- **Modèle officiel** : https://www.notaire.be/ (exemple état daté)

### Bibliothèques Rust
- `printpdf` : Génération PDF native Rust
- `wkhtmltopdf-rs` : HTML to PDF (si templates HTML)
- `lettre` : Envoi emails SMTP

---

## 🚀 Checklist de Développement

- [ ] 1. Créer `domain/entities/etat_date.rs`
- [ ] 2. Créer migration SQL
- [ ] 3. Créer `EtatDateRepository` trait + impl
- [ ] 4. Créer `EtatDateUseCases`
- [ ] 5. Implémenter calcul situation financière
- [ ] 6. Implémenter extraction décisions AG
- [ ] 7. Créer template PDF (HTML ou printpdf)
- [ ] 8. Implémenter génération PDF
- [ ] 9. Créer handlers HTTP
- [ ] 10. Ajouter routes dans `routes.rs`
- [ ] 11. Tests unitaires (10+ tests)
- [ ] 12. Tests E2E (5+ tests)
- [ ] 13. Frontend: page demande état daté
- [ ] 14. Frontend: liste états datés avec statuts
- [ ] 15. Documentation utilisateur
- [ ] 16. Commit : `feat: implement état daté generation for real estate transactions`

---

## 📊 Métriques de Succès

- **Conformité** : 100% des informations légales obligatoires présentes
- **Performance** : Génération PDF < 3s
- **Délai** : 0% états datés générés > 15 jours (alertes automatiques)
- **Qualité PDF** : Lisibilité professionnelle (review manuelle)

---

**Créé le** : 2025-11-01
**Assigné à** : À définir
**Milestone** : v1.0 - MVP Complet - Conformité Légale Belge
**Bloque** : Mutations immobilières (ventes, donations, successions)
**Impact Business** : CRITIQUE - Bloque toutes les ventes de lots
