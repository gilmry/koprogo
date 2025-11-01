# Issue #020 - Carnet d'Entretien et Suivi Travaux

**Priorité**: 🟡 HIGH
**Estimation**: 10-12 heures
**Labels**: `enhancement`, `backend`, `frontend`, `maintenance`, `legal-compliance`

---

## 📋 Description

Implémenter un **carnet d'entretien numérique** pour tracer tous les travaux, interventions, maintenances et contrôles techniques obligatoires effectués dans l'immeuble. Le carnet d'entretien est une bonne pratique (recommandé légalement) et devient indispensable pour la gestion professionnelle d'une copropriété.

**Contexte légal** : Bien que non strictement obligatoire en Belgique, le carnet d'entretien est **fortement recommandé** pour :
- Traçabilité des travaux (garanties décennales)
- Contrôles techniques obligatoires (ascenseurs, chaufferies, électricité)
- Valorisation de l'immeuble lors de ventes
- Prévention litiges avec prestataires

**Impact métier** : Un carnet d'entretien bien tenu réduit les coûts de maintenance, facilite les audits, et améliore la gestion préventive.

---

## 🎯 Objectifs

- [ ] Créer l'entité `WorkReport` pour tracer interventions
- [ ] Gérer les garanties constructeurs et dates d'expiration
- [ ] Planifier les contrôles techniques obligatoires
- [ ] Alertes automatiques avant expiration contrôles
- [ ] Upload photos avant/après interventions
- [ ] Historique chronologique complet par bâtiment
- [ ] Export PDF du carnet d'entretien

---

## 📐 Spécifications Techniques

### Types de Travaux à Tracer

1. **Maintenance Préventive** :
   - Nettoyage gouttières
   - Révision chaudière
   - Contrôle ascenseur
   - Entretien espaces verts

2. **Maintenance Corrective** :
   - Réparations urgentes (fuites, pannes)
   - Remplacement équipements défectueux
   - Interventions dépannage

3. **Travaux d'Amélioration** :
   - Rénovation façade
   - Remplacement toiture
   - Installation interphone
   - Travaux d'isolation

4. **Contrôles Techniques Obligatoires** :
   - Ascenseur (annuel)
   - Chaudière (annuel)
   - Détection incendie (semestriel)
   - Installation électrique (tous les 5 ans)
   - Performance énergétique (DPE) (tous les 10 ans)

---

## 🔧 Détails d'Implémentation

### 1. Domain Layer - Entity WorkReport

**Fichier** : `backend/src/domain/entities/work_report.rs`

```rust
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkReport {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub work_type: WorkType,
    pub category: WorkCategory,
    pub title: String,
    pub description: String,
    pub contractor_name: Option<String>,
    pub contractor_id: Option<Uuid>, // Si contractor entity existe
    pub work_date: NaiveDate,
    pub cost: Option<f64>,
    pub invoice_number: Option<String>,
    pub warranty: Option<Warranty>,
    pub documents: Vec<Uuid>, // Photos, invoices, certificates
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "work_type", rename_all = "snake_case")]
pub enum WorkType {
    Maintenance,    // Entretien régulier
    Repair,         // Réparation
    Installation,   // Installation nouveau équipement
    Inspection,     // Contrôle technique
    Improvement,    // Travaux d'amélioration
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "work_category", rename_all = "snake_case")]
pub enum WorkCategory {
    Elevator,
    Heating,
    Plumbing,
    Electricity,
    Roofing,
    Facade,
    Painting,
    Cleaning,
    Gardening,
    FireSafety,
    Security,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warranty {
    pub warranty_type: WarrantyType,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub provider: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WarrantyType {
    Contractual,      // Garantie contractuelle (1-2 ans)
    Biennale,         // Garantie biennale (2 ans)
    Decennale,        // Garantie décennale (10 ans)
}

impl WorkReport {
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        work_type: WorkType,
        category: WorkCategory,
        title: String,
        description: String,
        work_date: NaiveDate,
        created_by: Uuid,
    ) -> Result<Self, String> {
        if title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }
        if description.trim().is_empty() {
            return Err("Description cannot be empty".to_string());
        }

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            work_type,
            category,
            title,
            description,
            contractor_name: None,
            contractor_id: None,
            work_date,
            cost: None,
            invoice_number: None,
            warranty: None,
            documents: Vec::new(),
            created_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub fn add_warranty(
        &mut self,
        warranty_type: WarrantyType,
        provider: String,
        start_date: NaiveDate,
    ) -> Result<(), String> {
        let duration_years = match warranty_type {
            WarrantyType::Contractual => 2,
            WarrantyType::Biennale => 2,
            WarrantyType::Decennale => 10,
        };

        let end_date = start_date
            .checked_add_signed(chrono::Duration::days(duration_years * 365))
            .ok_or("Invalid warranty end date")?;

        self.warranty = Some(Warranty {
            warranty_type,
            start_date,
            end_date,
            provider,
            description: None,
        });

        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn is_warranty_active(&self) -> bool {
        if let Some(warranty) = &self.warranty {
            let today = chrono::Local::now().naive_local().date();
            today >= warranty.start_date && today <= warranty.end_date
        } else {
            false
        }
    }
}
```

---

### 2. Domain Layer - Entity TechnicalInspection

**Fichier** : `backend/src/domain/entities/technical_inspection.rs`

```rust
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalInspection {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub inspection_type: InspectionType,
    pub last_inspection_date: NaiveDate,
    pub next_inspection_date: NaiveDate,
    pub frequency_days: i32,
    pub responsible_entity: String, // Organisme agréé
    pub alert_before_days: i32,     // Alerter X jours avant
    pub status: InspectionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "inspection_type", rename_all = "snake_case")]
pub enum InspectionType {
    Elevator,           // Annuel
    Boiler,             // Annuel
    FireDetection,      // Semestriel
    ElectricalSystem,   // Tous les 5 ans
    EnergyPerformance,  // Tous les 10 ans (DPE)
    GasSafety,          // Annuel
    WaterQuality,       // Variable
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "inspection_status", rename_all = "snake_case")]
pub enum InspectionStatus {
    UpToDate,           // À jour
    DueSoon,            // À faire prochainement
    Overdue,            // En retard
    NotApplicable,      // N/A (équipement inexistant)
}

impl TechnicalInspection {
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        inspection_type: InspectionType,
        last_inspection_date: NaiveDate,
        responsible_entity: String,
    ) -> Self {
        let frequency_days = Self::get_default_frequency(&inspection_type);
        let next_inspection_date = last_inspection_date
            .checked_add_signed(chrono::Duration::days(frequency_days as i64))
            .unwrap();

        Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            inspection_type,
            last_inspection_date,
            next_inspection_date,
            frequency_days,
            responsible_entity,
            alert_before_days: 30, // Alerter 30 jours avant par défaut
            status: InspectionStatus::UpToDate,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn get_default_frequency(inspection_type: &InspectionType) -> i32 {
        match inspection_type {
            InspectionType::Elevator => 365,
            InspectionType::Boiler => 365,
            InspectionType::FireDetection => 182,
            InspectionType::ElectricalSystem => 1825, // 5 ans
            InspectionType::EnergyPerformance => 3650, // 10 ans
            InspectionType::GasSafety => 365,
            InspectionType::WaterQuality => 365,
        }
    }

    pub fn update_status(&mut self) {
        let today = chrono::Local::now().naive_local().date();
        let days_until_next = (self.next_inspection_date - today).num_days();

        self.status = if days_until_next < 0 {
            InspectionStatus::Overdue
        } else if days_until_next <= self.alert_before_days as i64 {
            InspectionStatus::DueSoon
        } else {
            InspectionStatus::UpToDate
        };
    }

    pub fn record_inspection(&mut self, inspection_date: NaiveDate) {
        self.last_inspection_date = inspection_date;
        self.next_inspection_date = inspection_date
            .checked_add_signed(chrono::Duration::days(self.frequency_days as i64))
            .unwrap();
        self.update_status();
        self.updated_at = Utc::now();
    }

    pub fn needs_alert(&self) -> bool {
        matches!(
            self.status,
            InspectionStatus::DueSoon | InspectionStatus::Overdue
        )
    }
}
```

---

### 3. Migration Database

**Fichier** : `backend/migrations/20251101000003_create_work_reports.sql`

```sql
CREATE TYPE work_type AS ENUM ('maintenance', 'repair', 'installation', 'inspection', 'improvement');
CREATE TYPE work_category AS ENUM (
    'elevator', 'heating', 'plumbing', 'electricity', 'roofing',
    'facade', 'painting', 'cleaning', 'gardening', 'fire_safety',
    'security', 'other'
);

CREATE TABLE work_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    work_type work_type NOT NULL,
    category work_category NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    contractor_name VARCHAR(255),
    contractor_id UUID, -- Future: REFERENCES contractors(id)
    work_date DATE NOT NULL,
    cost DECIMAL(12, 2),
    invoice_number VARCHAR(100),
    warranty JSONB, -- Warranty object
    documents UUID[], -- Array of document IDs
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_work_reports_building ON work_reports(building_id);
CREATE INDEX idx_work_reports_work_date ON work_reports(work_date DESC);
CREATE INDEX idx_work_reports_category ON work_reports(category);
CREATE INDEX idx_work_reports_work_type ON work_reports(work_type);

-- Table pour contrôles techniques
CREATE TYPE inspection_type AS ENUM (
    'elevator', 'boiler', 'fire_detection', 'electrical_system',
    'energy_performance', 'gas_safety', 'water_quality'
);
CREATE TYPE inspection_status AS ENUM ('up_to_date', 'due_soon', 'overdue', 'not_applicable');

CREATE TABLE technical_inspections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    inspection_type inspection_type NOT NULL,
    last_inspection_date DATE NOT NULL,
    next_inspection_date DATE NOT NULL,
    frequency_days INTEGER NOT NULL,
    responsible_entity VARCHAR(255) NOT NULL,
    alert_before_days INTEGER NOT NULL DEFAULT 30,
    status inspection_status NOT NULL DEFAULT 'up_to_date',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(building_id, inspection_type)
);

-- Indexes
CREATE INDEX idx_inspections_building ON technical_inspections(building_id);
CREATE INDEX idx_inspections_next_date ON technical_inspections(next_inspection_date);
CREATE INDEX idx_inspections_status ON technical_inspections(status);
```

---

### 4. Application Layer - Use Cases

**Fichier** : `backend/src/application/use_cases/maintenance_use_cases.rs`

```rust
use crate::domain::entities::work_report::*;
use crate::domain::entities::technical_inspection::*;
use crate::application::ports::work_report_repository::WorkReportRepository;
use crate::application::ports::inspection_repository::TechnicalInspectionRepository;
use std::sync::Arc;
use uuid::Uuid;
use chrono::NaiveDate;

pub struct MaintenanceUseCases {
    work_report_repo: Arc<dyn WorkReportRepository>,
    inspection_repo: Arc<dyn TechnicalInspectionRepository>,
}

impl MaintenanceUseCases {
    pub fn new(
        work_report_repo: Arc<dyn WorkReportRepository>,
        inspection_repo: Arc<dyn TechnicalInspectionRepository>,
    ) -> Self {
        Self {
            work_report_repo,
            inspection_repo,
        }
    }

    pub async fn create_work_report(
        &self,
        organization_id: Uuid,
        building_id: Uuid,
        work_type: WorkType,
        category: WorkCategory,
        title: String,
        description: String,
        work_date: NaiveDate,
        created_by: Uuid,
    ) -> Result<WorkReport, String> {
        let work_report = WorkReport::new(
            organization_id,
            building_id,
            work_type,
            category,
            title,
            description,
            work_date,
            created_by,
        )?;

        self.work_report_repo.create(&work_report).await
    }

    pub async fn get_maintenance_history(
        &self,
        building_id: Uuid,
        limit: usize,
    ) -> Result<Vec<WorkReport>, String> {
        self.work_report_repo
            .find_by_building_paginated(building_id, 0, limit)
            .await
    }

    pub async fn get_active_warranties(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<WorkReport>, String> {
        let all_reports = self.work_report_repo.find_by_building(building_id).await?;
        let active_warranties: Vec<WorkReport> = all_reports
            .into_iter()
            .filter(|r| r.is_warranty_active())
            .collect();
        Ok(active_warranties)
    }

    pub async fn schedule_inspection(
        &self,
        organization_id: Uuid,
        building_id: Uuid,
        inspection_type: InspectionType,
        last_inspection_date: NaiveDate,
        responsible_entity: String,
    ) -> Result<TechnicalInspection, String> {
        let inspection = TechnicalInspection::new(
            organization_id,
            building_id,
            inspection_type,
            last_inspection_date,
            responsible_entity,
        );

        self.inspection_repo.create(&inspection).await
    }

    pub async fn get_upcoming_inspections(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<TechnicalInspection>, String> {
        let mut inspections = self.inspection_repo.find_by_building(building_id).await?;

        // Mettre à jour les statuts
        for inspection in &mut inspections {
            inspection.update_status();
        }

        // Filtrer ceux qui nécessitent une alerte
        let upcoming: Vec<TechnicalInspection> = inspections
            .into_iter()
            .filter(|i| i.needs_alert())
            .collect();

        Ok(upcoming)
    }

    pub async fn record_inspection_done(
        &self,
        inspection_id: Uuid,
        inspection_date: NaiveDate,
    ) -> Result<TechnicalInspection, String> {
        let mut inspection = self
            .inspection_repo
            .find_by_id(inspection_id)
            .await?
            .ok_or("Inspection not found")?;

        inspection.record_inspection(inspection_date);
        self.inspection_repo.update(&inspection).await
    }

    pub async fn export_maintenance_logbook_pdf(
        &self,
        building_id: Uuid,
    ) -> Result<String, String> {
        // TODO: Générer PDF complet du carnet d'entretien
        // - Liste chronologique de tous les work_reports
        // - État des garanties actives
        // - Planning des inspections
        todo!("Implement PDF generation")
    }
}
```

---

## ✅ Critères d'Acceptation

### Fonctionnels
- [ ] Créer un rapport de travaux avec photos
- [ ] Enregistrer garantie décennale avec dates
- [ ] Planifier contrôles techniques récurrents
- [ ] Alertes 30 jours avant contrôle obligatoire
- [ ] Historique chronologique complet
- [ ] Export PDF carnet d'entretien

### Techniques
- [ ] Tests unitaires + E2E
- [ ] Frontend timeline des interventions
- [ ] Upload photos travaux (multi-upload)

---

## 🔗 Dépendances

- ✅ Building entity
- Issue #002 : Document upload (photos travaux)
- Issue #047 : PDF generation (carnet PDF)
- Issue #009 : Notifications (alertes inspections)

---

## 🚀 Checklist

- [ ] 1. Créer entities `work_report.rs` + `technical_inspection.rs`
- [ ] 2. Migration SQL
- [ ] 3. Repositories
- [ ] 4. Use cases
- [ ] 5. Handlers HTTP
- [ ] 6. Tests (15+ tests)
- [ ] 7. Frontend: page carnet d'entretien
- [ ] 8. Frontend: timeline travaux
- [ ] 9. Cron job alertes inspections
- [ ] 10. Commit : `feat: implement digital maintenance logbook`

---

**Créé le** : 2025-11-01
**Milestone** : v1.1 - Maintenance Features
