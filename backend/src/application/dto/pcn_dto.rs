use crate::domain::services::{PcnAccount, PcnReportLine};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request to generate PCN report for a building
#[derive(Debug, Serialize, Deserialize)]
pub struct PcnReportRequest {
    /// Building ID to generate report for
    pub building_id: Uuid,
    /// Optional start date for filtering expenses
    pub start_date: Option<DateTime<Utc>>,
    /// Optional end date for filtering expenses
    pub end_date: Option<DateTime<Utc>>,
}

/// Response containing PCN report data
#[derive(Debug, Serialize, Deserialize)]
pub struct PcnReportResponse {
    /// Building ID
    pub building_id: Uuid,
    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Period start date (if filtered)
    pub period_start: Option<DateTime<Utc>>,
    /// Period end date (if filtered)
    pub period_end: Option<DateTime<Utc>>,
    /// PCN report lines (one per account with expenses)
    pub lines: Vec<PcnReportLineDto>,
    /// Total amount across all accounts
    pub total_amount: f64,
    /// Total number of expense entries
    pub total_entries: usize,
}

/// DTO for a single PCN report line
#[derive(Debug, Serialize, Deserialize)]
pub struct PcnReportLineDto {
    /// PCN account code (e.g., "611")
    pub account_code: String,
    /// Account label in Dutch (Nederlands - 60%)
    pub account_label_nl: String,
    /// Account label in French (Français - 40%)
    pub account_label_fr: String,
    /// Account label in German (Deutsch - <1%)
    pub account_label_de: String,
    /// Account label in English (International)
    pub account_label_en: String,
    /// Total amount for this account
    pub total_amount: f64,
    /// Number of expense entries for this account
    pub entry_count: usize,
}

impl From<PcnReportLine> for PcnReportLineDto {
    fn from(line: PcnReportLine) -> Self {
        Self {
            account_code: line.account.code,
            account_label_nl: line.account.label_nl,
            account_label_fr: line.account.label_fr,
            account_label_de: line.account.label_de,
            account_label_en: line.account.label_en,
            total_amount: line.total_amount,
            entry_count: line.entry_count,
        }
    }
}

/// Response for PCN account mapping lookup
#[derive(Debug, Serialize, Deserialize)]
pub struct PcnAccountDto {
    /// PCN account code
    pub code: String,
    /// Account label in Dutch
    pub label_nl: String,
    /// Account label in French
    pub label_fr: String,
    /// Account label in German
    pub label_de: String,
    /// Account label in English
    pub label_en: String,
}

impl From<PcnAccount> for PcnAccountDto {
    fn from(account: PcnAccount) -> Self {
        Self {
            code: account.code,
            label_nl: account.label_nl,
            label_fr: account.label_fr,
            label_de: account.label_de,
            label_en: account.label_en,
        }
    }
}
