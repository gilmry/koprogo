use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

/// Filters for building list queries
#[derive(Debug, Deserialize, Default, Clone)]
pub struct BuildingFilters {
    pub organization_id: Option<Uuid>,
    pub city: Option<String>,
    pub construction_year: Option<i32>,
    pub min_units: Option<i32>,
    pub max_units: Option<i32>,
}

/// Filters for expense list queries
#[derive(Debug, Deserialize, Default, Clone)]
pub struct ExpenseFilters {
    pub organization_id: Option<Uuid>,
    pub building_id: Option<Uuid>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub paid: Option<bool>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub min_amount: Option<f64>,
    pub max_amount: Option<f64>,
}

/// Filters for unit list queries
#[derive(Debug, Deserialize, Default, Clone)]
pub struct UnitFilters {
    pub organization_id: Option<Uuid>,
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
