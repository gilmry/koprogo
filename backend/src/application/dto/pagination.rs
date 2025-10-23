use serde::{Deserialize, Serialize};

/// Page request parameters for pagination
#[derive(Debug, Deserialize, Clone)]
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
    /// Calculate offset for SQL query
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.per_page
    }

    /// Get limit with max cap of 100 items
    pub fn limit(&self) -> i64 {
        self.per_page.min(100)
    }

    /// Validate page request parameters
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

impl Default for PageRequest {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
            sort_by: None,
            order: SortOrder::default(),
        }
    }
}

/// Sort order for pagination
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}

impl SortOrder {
    /// Convert to SQL ORDER BY clause
    pub fn to_sql(&self) -> &str {
        match self {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        }
    }
}

/// Paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PageResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

impl<T> PageResponse<T> {
    pub fn new(data: Vec<T>, page: i64, per_page: i64, total_items: i64) -> Self {
        Self {
            data,
            pagination: PaginationMeta::new(page, per_page, total_items),
        }
    }
}

/// Pagination metadata
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
        let total_pages = if total_items == 0 {
            0
        } else {
            ((total_items as f64) / (per_page as f64)).ceil() as i64
        };

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_request_default() {
        let req = PageRequest::default();
        assert_eq!(req.page, 1);
        assert_eq!(req.per_page, 20);
        assert_eq!(req.offset(), 0);
        assert_eq!(req.limit(), 20);
    }

    #[test]
    fn test_page_request_offset() {
        let req = PageRequest {
            page: 3,
            per_page: 20,
            sort_by: None,
            order: SortOrder::default(),
        };
        assert_eq!(req.offset(), 40); // (3-1) * 20 = 40
    }

    #[test]
    fn test_page_request_limit_capped() {
        let req = PageRequest {
            page: 1,
            per_page: 500, // Excessive
            sort_by: None,
            order: SortOrder::default(),
        };
        assert_eq!(req.limit(), 100); // Capped at 100
    }

    #[test]
    fn test_page_request_validation_valid() {
        let req = PageRequest {
            page: 1,
            per_page: 20,
            sort_by: None,
            order: SortOrder::default(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_page_request_validation_invalid_page() {
        let req = PageRequest {
            page: 0, // Invalid
            per_page: 20,
            sort_by: None,
            order: SortOrder::default(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_page_request_validation_invalid_per_page() {
        let req = PageRequest {
            page: 1,
            per_page: 101, // Invalid (> 100)
            sort_by: None,
            order: SortOrder::default(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_pagination_meta_calculation() {
        let meta = PaginationMeta::new(2, 20, 45);
        assert_eq!(meta.current_page, 2);
        assert_eq!(meta.per_page, 20);
        assert_eq!(meta.total_items, 45);
        assert_eq!(meta.total_pages, 3); // ceil(45/20) = 3
        assert!(meta.has_next);
        assert!(meta.has_previous);
    }

    #[test]
    fn test_pagination_meta_first_page() {
        let meta = PaginationMeta::new(1, 20, 100);
        assert!(!meta.has_previous);
        assert!(meta.has_next);
    }

    #[test]
    fn test_pagination_meta_last_page() {
        let meta = PaginationMeta::new(5, 20, 100);
        assert!(meta.has_previous);
        assert!(!meta.has_next);
    }

    #[test]
    fn test_pagination_meta_empty() {
        let meta = PaginationMeta::new(1, 20, 0);
        assert_eq!(meta.total_pages, 0);
        assert!(!meta.has_next);
        assert!(!meta.has_previous);
    }

    #[test]
    fn test_sort_order_to_sql() {
        assert_eq!(SortOrder::Asc.to_sql(), "ASC");
        assert_eq!(SortOrder::Desc.to_sql(), "DESC");
    }
}
