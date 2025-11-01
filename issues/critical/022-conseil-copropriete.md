# Issue #022 - Conseil de Copropriété (Board of Directors)

**Priorité**: 🔴 CRITIQUE
**Estimation**: 12-15 heures
**Labels**: `enhancement`, `backend`, `frontend`, `critical`, `legal-compliance`, `governance`

---

## 📋 Description

Implémenter le **Conseil de Copropriété** complet, qui est une **obligation légale** en Belgique pour les immeubles de plus de 20 lots. Le conseil surveille le syndic, suit l'exécution des décisions d'AG, et génère des rapports semestriels et annuels.

**Contexte légal** : En Belgique, pour les copropriétés de **>20 lots** (hors caves et garages), un conseil de copropriété est **obligatoire** (Article 577-8/4 du Code Civil belge). Sans conseil, la gestion de la copropriété est non conforme.

**Impact métier** : BLOQUANT pour toute copropriété >20 lots. 0% actuellement implémenté dans KoproGo.

---

## 🎯 Objectifs

- [ ] Créer rôle `BoardMember` avec permissions spéciales
- [ ] Gérer élections et mandats annuels
- [ ] Dashboard suivi exécution décisions AG
- [ ] Tracking délais (devis demandés, travaux planifiés)
- [ ] Système alertes retards/manquements syndic
- [ ] Accès lecture seule tous documents
- [ ] Génération rapports semestriels automatiques
- [ ] Génération rapports annuels pour AG
- [ ] Vérification incompatibilités (syndic ≠ membre conseil)

---

## 📐 Spécifications Techniques

### Rôles et Permissions

#### Nouveau Rôle: BoardMember

```rust
// Ajouter à UserRole enum
pub enum UserRole {
    SuperAdmin,
    Syndic,
    Accountant,
    BoardMember,  // NOUVEAU
    Owner,
}
```

**Permissions BoardMember** :
- ✅ **Lecture** : Tous documents, décisions AG, finances, travaux
- ✅ **Écriture** : Rapports conseil, notes internes
- ❌ **Interdictions** : Créer dépenses, modifier contrats, convoquer AG (réservé syndic)

**Incompatibilités** :
- Un user ne peut PAS être à la fois `Syndic` ET `BoardMember` pour le même building
- Vérification dans domain logic

---

### 1. Domain Layer - Entity BoardMember

**Fichier** : `backend/src/domain/entities/board_member.rs`

```rust
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardMember {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub user_id: Uuid,
    pub owner_id: Option<Uuid>, // Lien avec Owner si copropriétaire
    pub position: BoardPosition,
    pub elected_date: NaiveDate,
    pub mandate_start: NaiveDate,
    pub mandate_end: NaiveDate,
    pub is_active: bool,
    pub elected_by_meeting_id: Uuid, // AG où élu
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "board_position", rename_all = "snake_case")]
pub enum BoardPosition {
    President,       // Président
    VicePresident,   // Vice-président
    Secretary,       // Secrétaire
    Treasurer,       // Trésorier (si différent du syndic)
    Member,          // Membre simple
}

impl BoardMember {
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        user_id: Uuid,
        position: BoardPosition,
        elected_date: NaiveDate,
        elected_by_meeting_id: Uuid,
    ) -> Result<Self, String> {
        let mandate_start = elected_date;
        let mandate_end = elected_date
            .checked_add_signed(chrono::Duration::days(365))
            .ok_or("Invalid mandate end date")?;

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            user_id,
            owner_id: None,
            position,
            elected_date,
            mandate_start,
            mandate_end,
            is_active: true,
            elected_by_meeting_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub fn is_mandate_active(&self) -> bool {
        let today = chrono::Local::now().naive_local().date();
        self.is_active && today >= self.mandate_start && today <= self.mandate_end
    }

    pub fn days_until_mandate_end(&self) -> i64 {
        let today = chrono::Local::now().naive_local().date();
        (self.mandate_end - today).num_days()
    }

    pub fn renew_mandate(&mut self, new_mandate_start: NaiveDate) -> Result<(), String> {
        let new_mandate_end = new_mandate_start
            .checked_add_signed(chrono::Duration::days(365))
            .ok_or("Invalid new mandate end date")?;

        self.mandate_start = new_mandate_start;
        self.mandate_end = new_mandate_end;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn terminate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }
}
```

---

### 2. Domain Layer - Entity BoardDecision

**Fichier** : `backend/src/domain/entities/board_decision.rs`

```rust
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardDecision {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub decision_type: DecisionType,
    pub title: String,
    pub description: String,
    pub decided_at: DateTime<Utc>,
    pub deadline: Option<NaiveDate>,
    pub status: DecisionStatus,
    pub assigned_to: Option<Uuid>, // User ID responsable (souvent syndic)
    pub ag_meeting_id: Option<Uuid>, // Décision d'AG à surveiller
    pub created_by: Uuid, // Board member qui a créé
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "decision_type", rename_all = "snake_case")]
pub enum DecisionType {
    AgDecisionTracking,      // Suivi décision AG
    QuoteRequest,            // Demande devis
    WorkSupervision,         // Surveillance travaux
    ContractReview,          // Révision contrat
    FinancialControl,        // Contrôle financier
    Recommendation,          // Recommandation au syndic
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "decision_status", rename_all = "snake_case")]
pub enum DecisionStatus {
    Pending,        // En attente
    InProgress,     // En cours
    Completed,      // Terminée
    Overdue,        // En retard
    Cancelled,      // Annulée
}

impl BoardDecision {
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        decision_type: DecisionType,
        title: String,
        description: String,
        deadline: Option<NaiveDate>,
        created_by: Uuid,
    ) -> Result<Self, String> {
        if title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            decision_type,
            title,
            description,
            decided_at: Utc::now(),
            deadline,
            status: DecisionStatus::Pending,
            assigned_to: None,
            ag_meeting_id: None,
            created_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub fn update_status(&mut self) {
        if let Some(deadline) = self.deadline {
            let today = chrono::Local::now().naive_local().date();
            if today > deadline && self.status == DecisionStatus::Pending {
                self.status = DecisionStatus::Overdue;
                self.updated_at = Utc::now();
            }
        }
    }

    pub fn mark_completed(&mut self) {
        self.status = DecisionStatus::Completed;
        self.updated_at = Utc::now();
    }

    pub fn is_overdue(&self) -> bool {
        matches!(self.status, DecisionStatus::Overdue)
    }
}
```

---

### 3. Domain Layer - Entity BoardReport

**Fichier** : `backend/src/domain/entities/board_report.rs`

```rust
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardReport {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub report_type: ReportType,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub summary: String,
    pub ag_decisions_status: Vec<AgDecisionStatus>,
    pub recommendations: Vec<String>,
    pub issues_identified: Vec<String>,
    pub approved: bool,
    pub approved_by_meeting_id: Option<Uuid>,
    pub pdf_path: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "report_type", rename_all = "snake_case")]
pub enum ReportType {
    Semester,    // Rapport semestriel
    Annual,      // Rapport annuel (pour AG)
    Extraordinary, // Rapport extraordinaire
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgDecisionStatus {
    pub meeting_id: Uuid,
    pub meeting_date: NaiveDate,
    pub decision_summary: String,
    pub status: String, // "Executed", "In Progress", "Delayed", "Not Started"
    pub comments: Option<String>,
}

impl BoardReport {
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        report_type: ReportType,
        period_start: NaiveDate,
        period_end: NaiveDate,
        summary: String,
        created_by: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            report_type,
            period_start,
            period_end,
            summary,
            ag_decisions_status: Vec::new(),
            recommendations: Vec::new(),
            issues_identified: Vec::new(),
            approved: false,
            approved_by_meeting_id: None,
            pdf_path: None,
            created_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn add_ag_decision_status(&mut self, status: AgDecisionStatus) {
        self.ag_decisions_status.push(status);
        self.updated_at = Utc::now();
    }

    pub fn add_recommendation(&mut self, recommendation: String) {
        self.recommendations.push(recommendation);
        self.updated_at = Utc::now();
    }

    pub fn approve(&mut self, meeting_id: Uuid) {
        self.approved = true;
        self.approved_by_meeting_id = Some(meeting_id);
        self.updated_at = Utc::now();
    }
}
```

---

### 4. Migration Database

**Fichier** : `backend/migrations/20251101000004_create_board_system.sql`

```sql
-- Ajouter BoardMember au rôle
ALTER TYPE user_role ADD VALUE IF NOT EXISTS 'board_member';

-- Table membres du conseil
CREATE TYPE board_position AS ENUM ('president', 'vice_president', 'secretary', 'treasurer', 'member');

CREATE TABLE board_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    owner_id UUID REFERENCES owners(id),
    position board_position NOT NULL,
    elected_date DATE NOT NULL,
    mandate_start DATE NOT NULL,
    mandate_end DATE NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    elected_by_meeting_id UUID REFERENCES meetings(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Un user ne peut avoir qu'une seule position active par building
    UNIQUE(building_id, user_id, is_active)
);

CREATE INDEX idx_board_members_building ON board_members(building_id);
CREATE INDEX idx_board_members_user ON board_members(user_id);
CREATE INDEX idx_board_members_active ON board_members(is_active);

-- Table décisions conseil
CREATE TYPE decision_type AS ENUM (
    'ag_decision_tracking',
    'quote_request',
    'work_supervision',
    'contract_review',
    'financial_control',
    'recommendation',
    'other'
);

CREATE TYPE decision_status AS ENUM ('pending', 'in_progress', 'completed', 'overdue', 'cancelled');

CREATE TABLE board_decisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    decision_type decision_type NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    decided_at TIMESTAMPTZ NOT NULL,
    deadline DATE,
    status decision_status NOT NULL DEFAULT 'pending',
    assigned_to UUID REFERENCES users(id),
    ag_meeting_id UUID REFERENCES meetings(id),
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_board_decisions_building ON board_decisions(building_id);
CREATE INDEX idx_board_decisions_status ON board_decisions(status);
CREATE INDEX idx_board_decisions_deadline ON board_decisions(deadline);

-- Table rapports conseil
CREATE TYPE report_type AS ENUM ('semester', 'annual', 'extraordinary');

CREATE TABLE board_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    report_type report_type NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    summary TEXT NOT NULL,
    ag_decisions_status JSONB NOT NULL DEFAULT '[]',
    recommendations JSONB NOT NULL DEFAULT '[]',
    issues_identified JSONB NOT NULL DEFAULT '[]',
    approved BOOLEAN NOT NULL DEFAULT FALSE,
    approved_by_meeting_id UUID REFERENCES meetings(id),
    pdf_path TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_board_reports_building ON board_reports(building_id);
CREATE INDEX idx_board_reports_period ON board_reports(period_start, period_end);
CREATE INDEX idx_board_reports_type ON board_reports(report_type);

-- Contrainte: Un syndic ne peut pas être membre du conseil
CREATE OR REPLACE FUNCTION check_syndic_board_incompatibility()
RETURNS TRIGGER AS $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM user_roles ur
        WHERE ur.user_id = NEW.user_id
        AND ur.organization_id = NEW.organization_id
        AND ur.role = 'syndic'
        AND ur.is_primary = TRUE
    ) THEN
        RAISE EXCEPTION 'A syndic cannot be a board member (incompatibility)';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER enforce_syndic_board_incompatibility
BEFORE INSERT OR UPDATE ON board_members
FOR EACH ROW EXECUTE FUNCTION check_syndic_board_incompatibility();
```

---

## ✅ Critères d'Acceptation

### Fonctionnels
- [ ] Élire membres du conseil en AG (vote majoritaire)
- [ ] Mandats annuels avec rappels renouvellement 30 jours avant
- [ ] Dashboard conseil: décisions AG en cours, tâches en retard
- [ ] Tracking délais: devis demandés, travaux planifiés
- [ ] Alertes automatiques retards syndic
- [ ] Accès lecture seule tous documents
- [ ] Génération rapport semestriel automatique
- [ ] Génération rapport annuel pour AG avec vote décharge
- [ ] Vérification incompatibilité syndic ≠ conseil

### Légaux
- [ ] Conforme Article 577-8/4 Code Civil belge
- [ ] Conseil obligatoire >20 lots (vérification automatique)
- [ ] Rapports contiennent toutes sections obligatoires

### Techniques
- [ ] Tests unitaires (20+ tests)
- [ ] Tests E2E complets
- [ ] Frontend dashboard conseil responsive
- [ ] Performance: calcul statut décisions < 500ms

---

## 🔗 Dépendances

### Bloquantes
- ✅ Meeting entity exists
- ✅ User/Owner entities exist

### Recommandées
- Issue #001 : Meeting Management (vote AG)
- Issue #046 : Voting System (élection membres)
- Issue #047 : PDF Generation (rapports)

---

## 🚀 Checklist

- [ ] 1. Créer entities (board_member, board_decision, board_report)
- [ ] 2. Migration SQL avec trigger incompatibilité
- [ ] 3. Repositories
- [ ] 4. Use cases (élection, tracking, rapports)
- [ ] 5. Handlers HTTP
- [ ] 6. Tests (20+ tests)
- [ ] 7. Frontend: dashboard conseil
- [ ] 8. Frontend: génération rapports
- [ ] 9. Cron job alertes renouvellement mandats
- [ ] 10. Documentation utilisateur
- [ ] 11. Commit : `feat: implement Board of Directors (Conseil de Copropriété)`

---

**Créé le** : 2025-11-01
**Milestone** : v1.0 - MVP Complet - Conformité Légale Belgique
**Bloque** : Production pour copropriétés >20 lots (obligation légale)
**Impact Business** : CRITIQUE - 0% implémenté, requis légalement
