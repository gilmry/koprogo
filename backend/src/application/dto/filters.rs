use crate::domain::entities::ApprovalStatus;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

/// Filters for building list queries
#[derive(Debug, Deserialize, Default, Clone)]
pub struct BuildingFilters {
    /// Story 1.3 — Scope organisation : filtre les buildings dont
    /// l'ACP parente appartient à cette organisation. Le repository
    /// traduit en `acp_id IN (SELECT id FROM acps WHERE organization_id = $)`
    /// (la colonne `buildings.organization_id` a été DROP en migration 040000).
    pub organization_id: Option<Uuid>,
    /// Story 1.2 — Scope ACP direct (filtre `buildings.acp_id = $`).
    pub acp_id: Option<Uuid>,
    pub city: Option<String>,
    pub construction_year: Option<i32>,
    pub min_units: Option<i32>,
    pub max_units: Option<i32>,
    /// BUG-WF14-2: Si défini, filtre les buildings où cet user possède un lot (via owners.user_id → unit_owners → units)
    pub owner_user_id: Option<Uuid>,
    /// Recherche libre (ILIKE) sur name/city/address — évite au frontend de
    /// devoir fetch les 100 premiers buildings puis filtrer en mémoire
    /// (BuildingSelector : ratait les buildings récents une fois >100
    /// buildings créés globalement en CI, cf. searchBuildings côté frontend).
    pub search: Option<String>,
}

/// Filters for expense list queries
#[derive(Debug, Deserialize, Default, Clone)]
pub struct ExpenseFilters {
    pub organization_id: Option<Uuid>,
    pub building_id: Option<Uuid>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub paid: Option<bool>,
    pub approval_status: Option<ApprovalStatus>, // Nouveau: pour filtrer par statut workflow
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub min_amount: Option<Decimal>,
    pub max_amount: Option<Decimal>,
}

/// Filters for unit list queries
#[derive(Debug, Deserialize, Default, Clone)]
pub struct UnitFilters {
    /// Story H15 — Scope organisation : filtre les lots dont l'ACP appartient
    /// à cette organisation. Le repository traduit en
    /// `acp_id IN (SELECT id FROM acps WHERE organization_id = $)` (la colonne
    /// `units.organization_id` a été DROP en migration 20260630030000).
    pub organization_id: Option<Uuid>,
    /// Story H15 — Scope ACP direct (filtre `units.acp_id = $`).
    pub acp_id: Option<Uuid>,
    pub building_id: Option<Uuid>,
    pub unit_type: Option<String>,
    pub has_owner: Option<bool>,
    pub floor: Option<i32>,
    pub min_area: Option<f64>,
    pub max_area: Option<f64>,
}

/// Filters for owner list queries
#[derive(Debug, Deserialize, Default, Clone)]
pub struct OwnerFilters {
    pub organization_id: Option<Uuid>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub last_name: Option<String>,
    pub first_name: Option<String>,
}

/// Filters for work report list queries
#[derive(Debug, Deserialize, Default, Clone)]
pub struct WorkReportFilters {
    pub organization_id: Option<Uuid>,
    pub building_id: Option<Uuid>,
    pub work_type: Option<String>,
    pub warranty_type: Option<String>,
    pub contractor_name: Option<String>,
    pub work_date_from: Option<DateTime<Utc>>,
    pub work_date_to: Option<DateTime<Utc>>,
    // ADR-0008 : bornes de coût en `Decimal`, comme la colonne
    // `work_reports.cost` qu'elles filtrent.
    //
    // ATTENTION — ces deux bornes, comme `warranty_type`, `contractor_name`,
    // `work_date_from`, `work_date_to` et `warranty_active`, sont acceptées
    // par l'API mais **jamais appliquées** : `work_report_repository_impl`
    // ne lit que `building_id` et `work_type`. Un appelant qui passe
    // `?min_cost=1000` reçoit la liste NON filtrée en croyant l'inverse.
    // Défaut constaté en convertissant ces champs — tracé, non corrigé ici
    // (le corriger demande 7 filtres + tests 4-cat, hors périmètre ADR-0008).
    pub min_cost: Option<Decimal>,
    pub max_cost: Option<Decimal>,
    pub warranty_active: Option<bool>,
}

/// Filters for technical inspection list queries
#[derive(Debug, Deserialize, Default, Clone)]
pub struct TechnicalInspectionFilters {
    pub organization_id: Option<Uuid>,
    pub building_id: Option<Uuid>,
    pub inspection_type: Option<String>,
    pub status: Option<String>,
    pub inspector_name: Option<String>,
    pub inspector_company: Option<String>,
    pub inspection_date_from: Option<DateTime<Utc>>,
    pub inspection_date_to: Option<DateTime<Utc>>,
    pub overdue: Option<bool>,
    pub compliant: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_filters_default() {
        let filters = BuildingFilters::default();
        assert!(filters.city.is_none());
        assert!(filters.construction_year.is_none());
        assert!(filters.min_units.is_none());
        assert!(filters.max_units.is_none());
    }

    #[test]
    fn test_expense_filters_default() {
        let filters = ExpenseFilters::default();
        assert!(filters.building_id.is_none());
        assert!(filters.category.is_none());
        assert!(filters.paid.is_none());
    }

    #[test]
    fn test_unit_filters_default() {
        let filters = UnitFilters::default();
        assert!(filters.building_id.is_none());
        assert!(filters.has_owner.is_none());
    }

    #[test]
    fn test_owner_filters_default() {
        let filters = OwnerFilters::default();
        assert!(filters.email.is_none());
        assert!(filters.last_name.is_none());
    }
}
