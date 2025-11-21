use crate::domain::entities::{Notice, NoticeCategory, NoticeStatus, NoticeType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DTO for creating a new notice (Draft status)
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNoticeDto {
    pub building_id: Uuid,
    pub notice_type: NoticeType,
    pub category: NoticeCategory,
    pub title: String,
    pub content: String,
    // Event-specific fields (required for Event type)
    pub event_date: Option<DateTime<Utc>>,
    pub event_location: Option<String>,
    // Contact info for LostAndFound and ClassifiedAd
    pub contact_info: Option<String>,
    // Optional expiration date
    pub expires_at: Option<DateTime<Utc>>,
}

/// DTO for updating a notice (Draft only)
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateNoticeDto {
    pub title: Option<String>,
    pub content: Option<String>,
    pub category: Option<NoticeCategory>,
    pub event_date: Option<Option<DateTime<Utc>>>,
    pub event_location: Option<Option<String>>,
    pub contact_info: Option<Option<String>>,
    pub expires_at: Option<Option<DateTime<Utc>>>,
}

/// DTO for setting expiration date
#[derive(Debug, Serialize, Deserialize)]
pub struct SetExpirationDto {
    pub expires_at: Option<DateTime<Utc>>,
}

/// Complete notice response with author information
#[derive(Debug, Serialize, Clone)]
pub struct NoticeResponseDto {
    pub id: Uuid,
    pub building_id: Uuid,
    pub author_id: Uuid,
    pub author_name: String, // Enriched from Owner
    pub notice_type: NoticeType,
    pub category: NoticeCategory,
    pub title: String,
    pub content: String,
    pub status: NoticeStatus,
    pub is_pinned: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
    // Event-specific fields
    pub event_date: Option<DateTime<Utc>>,
    pub event_location: Option<String>,
    // Contact info
    pub contact_info: Option<String>,
    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Computed fields
    pub is_expired: bool,
    pub days_until_event: Option<i64>, // For Event type
}

impl NoticeResponseDto {
    /// Create from Notice with author name enrichment
    pub fn from_notice(notice: Notice, author_name: String) -> Self {
        let is_expired = notice.is_expired();
        let days_until_event = if notice.notice_type == NoticeType::Event {
            notice.event_date.map(|event_date| {
                let now = Utc::now();
                (event_date - now).num_days()
            })
        } else {
            None
        };

        Self {
            id: notice.id,
            building_id: notice.building_id,
            author_id: notice.author_id,
            author_name,
            notice_type: notice.notice_type,
            category: notice.category,
            title: notice.title,
            content: notice.content,
            status: notice.status,
            is_pinned: notice.is_pinned,
            published_at: notice.published_at,
            expires_at: notice.expires_at,
            archived_at: notice.archived_at,
            event_date: notice.event_date,
            event_location: notice.event_location,
            contact_info: notice.contact_info,
            created_at: notice.created_at,
            updated_at: notice.updated_at,
            is_expired,
            days_until_event,
        }
    }
}

/// Summary notice response for list views
#[derive(Debug, Serialize, Clone)]
pub struct NoticeSummaryDto {
    pub id: Uuid,
    pub building_id: Uuid,
    pub author_name: String,
    pub notice_type: NoticeType,
    pub category: NoticeCategory,
    pub title: String,
    pub status: NoticeStatus,
    pub is_pinned: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub event_date: Option<DateTime<Utc>>, // For Event type
    pub created_at: DateTime<Utc>,
    pub is_expired: bool,
}

impl NoticeSummaryDto {
    /// Create from Notice with author name enrichment
    pub fn from_notice(notice: Notice, author_name: String) -> Self {
        let is_expired = notice.is_expired();

        Self {
            id: notice.id,
            building_id: notice.building_id,
            author_name,
            notice_type: notice.notice_type,
            category: notice.category,
            title: notice.title,
            status: notice.status.clone(),
            is_pinned: notice.is_pinned,
            published_at: notice.published_at,
            event_date: notice.event_date,
            created_at: notice.created_at,
            is_expired,
        }
    }
}
