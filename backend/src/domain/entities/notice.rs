use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Notice type for community board
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NoticeType {
    /// General announcement (info, rules, reminders)
    Announcement,
    /// Community event (party, meeting, workshop)
    Event,
    /// Lost and found items
    LostAndFound,
    /// Classified ad (buy, sell, services)
    ClassifiedAd,
}

/// Notice category for filtering
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NoticeCategory {
    /// General information
    General,
    /// Maintenance and repairs
    Maintenance,
    /// Social events and activities
    Social,
    /// Security and safety
    Security,
    /// Environment and recycling
    Environment,
    /// Parking and transportation
    Parking,
    /// Other category
    Other,
}

/// Notice status workflow
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NoticeStatus {
    /// Draft (not visible to others)
    Draft,
    /// Published (visible to all building members)
    Published,
    /// Archived (moved to history)
    Archived,
    /// Expired (automatically expired based on expires_at)
    Expired,
}

/// Community notice board entry
///
/// Represents an announcement, event, lost & found item, or classified ad
/// posted on the building's community board.
///
/// # Business Rules
/// - Title must be 5-255 characters
/// - Content must be non-empty (max 10,000 characters)
/// - Draft notices cannot be pinned
/// - Only published notices are visible to building members
/// - Expired notices are automatically marked as Expired
/// - Events must have event_date and event_location
/// - Lost & Found and Classified Ads should have contact_info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notice {
    pub id: Uuid,
    pub building_id: Uuid,
    pub author_id: Uuid, // Owner who created the notice
    pub notice_type: NoticeType,
    pub category: NoticeCategory,
    pub title: String,
    pub content: String,
    pub status: NoticeStatus,
    pub is_pinned: bool, // Pin important notices to top
    pub published_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
    // Event-specific fields
    pub event_date: Option<DateTime<Utc>>,
    pub event_location: Option<String>,
    // Contact info for LostAndFound and ClassifiedAd
    pub contact_info: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Notice {
    /// Create a new notice (Draft status)
    ///
    /// # Validation
    /// - Title must be 5-255 characters
    /// - Content must be non-empty (max 10,000 characters)
    /// - Events must have event_date and event_location
    pub fn new(
        building_id: Uuid,
        author_id: Uuid,
        notice_type: NoticeType,
        category: NoticeCategory,
        title: String,
        content: String,
        event_date: Option<DateTime<Utc>>,
        event_location: Option<String>,
        contact_info: Option<String>,
    ) -> Result<Self, String> {
        // Validate title
        if title.len() < 5 {
            return Err("Notice title must be at least 5 characters".to_string());
        }
        if title.len() > 255 {
            return Err("Notice title cannot exceed 255 characters".to_string());
        }

        // Validate content
        if content.trim().is_empty() {
            return Err("Notice content cannot be empty".to_string());
        }
        if content.len() > 10_000 {
            return Err("Notice content cannot exceed 10,000 characters".to_string());
        }

        // Validate event fields
        if notice_type == NoticeType::Event {
            if event_date.is_none() {
                return Err("Event notices must have an event_date".to_string());
            }
            if event_location.is_none() || event_location.as_ref().unwrap().trim().is_empty() {
                return Err("Event notices must have an event_location".to_string());
            }
        }

        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            building_id,
            author_id,
            notice_type,
            category,
            title,
            content,
            status: NoticeStatus::Draft,
            is_pinned: false,
            published_at: None,
            expires_at: None,
            archived_at: None,
            event_date,
            event_location,
            contact_info,
            created_at: now,
            updated_at: now,
        })
    }

    /// Publish a draft notice
    ///
    /// # Transitions
    /// Draft → Published
    ///
    /// # Business Rules
    /// - Only Draft notices can be published
    /// - Sets published_at timestamp
    pub fn publish(&mut self) -> Result<(), String> {
        if self.status != NoticeStatus::Draft {
            return Err(format!(
                "Cannot publish notice in status {:?}. Only Draft notices can be published.",
                self.status
            ));
        }

        self.status = NoticeStatus::Published;
        self.published_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Archive a notice
    ///
    /// # Transitions
    /// Published → Archived
    /// Expired → Archived
    ///
    /// # Business Rules
    /// - Only Published or Expired notices can be archived
    /// - Sets archived_at timestamp
    /// - Unpins notice if pinned
    pub fn archive(&mut self) -> Result<(), String> {
        match self.status {
            NoticeStatus::Published | NoticeStatus::Expired => {
                self.status = NoticeStatus::Archived;
                self.archived_at = Some(Utc::now());
                self.is_pinned = false; // Unpin when archiving
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!(
                "Cannot archive notice in status {:?}. Only Published or Expired notices can be archived.",
                self.status
            )),
        }
    }

    /// Mark notice as expired
    ///
    /// # Transitions
    /// Published → Expired
    ///
    /// # Business Rules
    /// - Only Published notices can expire
    /// - Unpins notice if pinned
    pub fn expire(&mut self) -> Result<(), String> {
        if self.status != NoticeStatus::Published {
            return Err(format!(
                "Cannot expire notice in status {:?}. Only Published notices can expire.",
                self.status
            ));
        }

        self.status = NoticeStatus::Expired;
        self.is_pinned = false; // Unpin when expiring
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Pin notice to top of board
    ///
    /// # Business Rules
    /// - Only Published notices can be pinned
    /// - Draft, Archived, Expired notices cannot be pinned
    pub fn pin(&mut self) -> Result<(), String> {
        if self.status != NoticeStatus::Published {
            return Err(format!(
                "Cannot pin notice in status {:?}. Only Published notices can be pinned.",
                self.status
            ));
        }

        if self.is_pinned {
            return Err("Notice is already pinned".to_string());
        }

        self.is_pinned = true;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Unpin notice
    pub fn unpin(&mut self) -> Result<(), String> {
        if !self.is_pinned {
            return Err("Notice is not pinned".to_string());
        }

        self.is_pinned = false;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check if notice is expired
    ///
    /// Returns true if expires_at is set and is in the past
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Update notice content
    ///
    /// # Business Rules
    /// - Only Draft notices can be updated
    /// - Same validation as new()
    pub fn update_content(
        &mut self,
        title: Option<String>,
        content: Option<String>,
        category: Option<NoticeCategory>,
        event_date: Option<Option<DateTime<Utc>>>,
        event_location: Option<Option<String>>,
        contact_info: Option<Option<String>>,
        expires_at: Option<Option<DateTime<Utc>>>,
    ) -> Result<(), String> {
        if self.status != NoticeStatus::Draft {
            return Err(format!(
                "Cannot update notice in status {:?}. Only Draft notices can be updated.",
                self.status
            ));
        }

        // Update title if provided
        if let Some(new_title) = title {
            if new_title.len() < 5 {
                return Err("Notice title must be at least 5 characters".to_string());
            }
            if new_title.len() > 255 {
                return Err("Notice title cannot exceed 255 characters".to_string());
            }
            self.title = new_title;
        }

        // Update content if provided
        if let Some(new_content) = content {
            if new_content.trim().is_empty() {
                return Err("Notice content cannot be empty".to_string());
            }
            if new_content.len() > 10_000 {
                return Err("Notice content cannot exceed 10,000 characters".to_string());
            }
            self.content = new_content;
        }

        // Update category if provided
        if let Some(new_category) = category {
            self.category = new_category;
        }

        // Update event fields if provided
        if let Some(new_event_date) = event_date {
            self.event_date = new_event_date;
        }
        if let Some(new_event_location) = event_location {
            self.event_location = new_event_location;
        }

        // Validate event fields for Event type
        if self.notice_type == NoticeType::Event {
            if self.event_date.is_none() {
                return Err("Event notices must have an event_date".to_string());
            }
            if self.event_location.is_none()
                || self.event_location.as_ref().unwrap().trim().is_empty()
            {
                return Err("Event notices must have an event_location".to_string());
            }
        }

        // Update contact info if provided
        if let Some(new_contact_info) = contact_info {
            self.contact_info = new_contact_info;
        }

        // Update expires_at if provided
        if let Some(new_expires_at) = expires_at {
            self.expires_at = new_expires_at;
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Set expiration date
    ///
    /// # Business Rules
    /// - Expiration date must be in the future
    /// - Can be set for Draft or Published notices
    pub fn set_expiration(&mut self, expires_at: Option<DateTime<Utc>>) -> Result<(), String> {
        if let Some(expiration) = expires_at {
            if expiration <= Utc::now() {
                return Err("Expiration date must be in the future".to_string());
            }
        }

        self.expires_at = expires_at;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_create_announcement_success() {
        let building_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let notice = Notice::new(
            building_id,
            author_id,
            NoticeType::Announcement,
            NoticeCategory::General,
            "Important Announcement".to_string(),
            "This is an important announcement for all residents.".to_string(),
            None,
            None,
            None,
        );

        assert!(notice.is_ok());
        let notice = notice.unwrap();
        assert_eq!(notice.building_id, building_id);
        assert_eq!(notice.author_id, author_id);
        assert_eq!(notice.status, NoticeStatus::Draft);
        assert!(!notice.is_pinned);
    }

    #[test]
    fn test_create_event_success() {
        let building_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let event_date = Utc::now() + Duration::days(7);

        let notice = Notice::new(
            building_id,
            author_id,
            NoticeType::Event,
            NoticeCategory::Social,
            "Summer BBQ Party".to_string(),
            "Join us for a fun summer BBQ party!".to_string(),
            Some(event_date),
            Some("Garden courtyard".to_string()),
            Some("contact@example.com".to_string()),
        );

        assert!(notice.is_ok());
        let notice = notice.unwrap();
        assert_eq!(notice.notice_type, NoticeType::Event);
        assert!(notice.event_date.is_some());
        assert_eq!(notice.event_location, Some("Garden courtyard".to_string()));
    }

    #[test]
    fn test_create_event_without_date_fails() {
        let building_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let result = Notice::new(
            building_id,
            author_id,
            NoticeType::Event,
            NoticeCategory::Social,
            "Summer BBQ Party".to_string(),
            "Join us for a fun summer BBQ party!".to_string(),
            None, // Missing event_date
            Some("Garden courtyard".to_string()),
            None,
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Event notices must have an event_date"
        );
    }

    #[test]
    fn test_create_event_without_location_fails() {
        let building_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let event_date = Utc::now() + Duration::days(7);

        let result = Notice::new(
            building_id,
            author_id,
            NoticeType::Event,
            NoticeCategory::Social,
            "Summer BBQ Party".to_string(),
            "Join us for a fun summer BBQ party!".to_string(),
            Some(event_date),
            None, // Missing event_location
            None,
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Event notices must have an event_location"
        );
    }

    #[test]
    fn test_title_too_short_fails() {
        let building_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let result = Notice::new(
            building_id,
            author_id,
            NoticeType::Announcement,
            NoticeCategory::General,
            "Hi".to_string(), // Too short (< 5 chars)
            "This is the content.".to_string(),
            None,
            None,
            None,
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Notice title must be at least 5 characters"
        );
    }

    #[test]
    fn test_title_too_long_fails() {
        let building_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let long_title = "A".repeat(256); // 256 chars (> 255)

        let result = Notice::new(
            building_id,
            author_id,
            NoticeType::Announcement,
            NoticeCategory::General,
            long_title,
            "This is the content.".to_string(),
            None,
            None,
            None,
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Notice title cannot exceed 255 characters"
        );
    }

    #[test]
    fn test_empty_content_fails() {
        let building_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let result = Notice::new(
            building_id,
            author_id,
            NoticeType::Announcement,
            NoticeCategory::General,
            "Valid Title".to_string(),
            "   ".to_string(), // Empty/whitespace content
            None,
            None,
            None,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Notice content cannot be empty");
    }

    #[test]
    fn test_publish_draft_success() {
        let building_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let mut notice = Notice::new(
            building_id,
            author_id,
            NoticeType::Announcement,
            NoticeCategory::General,
            "Important Announcement".to_string(),
            "This is an important announcement.".to_string(),
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(notice.status, NoticeStatus::Draft);
        assert!(notice.published_at.is_none());

        let result = notice.publish();
        assert!(result.is_ok());
        assert_eq!(notice.status, NoticeStatus::Published);
        assert!(notice.published_at.is_some());
    }

    #[test]
    fn test_publish_non_draft_fails() {
        let building_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let mut notice = Notice::new(
            building_id,
            author_id,
            NoticeType::Announcement,
            NoticeCategory::General,
            "Important Announcement".to_string(),
            "This is an important announcement.".to_string(),
            None,
            None,
            None,
        )
        .unwrap();

        notice.publish().unwrap();
        assert_eq!(notice.status, NoticeStatus::Published);

        // Try to publish again
        let result = notice.publish();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Only Draft notices can be published"));
    }

    #[test]
    fn test_archive_published_success() {
        let building_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let mut notice = Notice::new(
            building_id,
            author_id,
            NoticeType::Announcement,
            NoticeCategory::General,
            "Important Announcement".to_string(),
            "This is an important announcement.".to_string(),
            None,
            None,
            None,
        )
        .unwrap();

        notice.publish().unwrap();
        notice.pin().unwrap();
        assert!(notice.is_pinned);

        let result = notice.archive();
        assert!(result.is_ok());
        assert_eq!(notice.status, NoticeStatus::Archived);
        assert!(notice.archived_at.is_some());
        assert!(!notice.is_pinned); // Should be unpinned
    }

    #[test]
    fn test_archive_expired_success() {
        let building_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let mut notice = Notice::new(
            building_id,
            author_id,
            NoticeType::Announcement,
            NoticeCategory::General,
            "Important Announcement".to_string(),
            "This is an important announcement.".to_string(),
            None,
            None,
            None,
        )
        .unwrap();

        notice.publish().unwrap();
        notice.expire().unwrap();
        assert_eq!(notice.status, NoticeStatus::Expired);

        let result = notice.archive();
        assert!(result.is_ok());
        assert_eq!(notice.status, NoticeStatus::Archived);
    }

    #[test]
    fn test_pin_published_success() {
        let building_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let mut notice = Notice::new(
            building_id,
            author_id,
            NoticeType::Announcement,
            NoticeCategory::General,
            "Important Announcement".to_string(),
            "This is an important announcement.".to_string(),
            None,
            None,
            None,
        )
        .unwrap();

        notice.publish().unwrap();
        assert!(!notice.is_pinned);

        let result = notice.pin();
        assert!(result.is_ok());
        assert!(notice.is_pinned);
    }

    #[test]
    fn test_pin_draft_fails() {
        let building_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let mut notice = Notice::new(
            building_id,
            author_id,
            NoticeType::Announcement,
            NoticeCategory::General,
            "Important Announcement".to_string(),
            "This is an important announcement.".to_string(),
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(notice.status, NoticeStatus::Draft);

        let result = notice.pin();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Only Published notices can be pinned"));
    }

    #[test]
    fn test_unpin_success() {
        let building_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let mut notice = Notice::new(
            building_id,
            author_id,
            NoticeType::Announcement,
            NoticeCategory::General,
            "Important Announcement".to_string(),
            "This is an important announcement.".to_string(),
            None,
            None,
            None,
        )
        .unwrap();

        notice.publish().unwrap();
        notice.pin().unwrap();
        assert!(notice.is_pinned);

        let result = notice.unpin();
        assert!(result.is_ok());
        assert!(!notice.is_pinned);
    }

    #[test]
    fn test_is_expired() {
        let building_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let mut notice = Notice::new(
            building_id,
            author_id,
            NoticeType::Announcement,
            NoticeCategory::General,
            "Important Announcement".to_string(),
            "This is an important announcement.".to_string(),
            None,
            None,
            None,
        )
        .unwrap();

        // No expiration set
        assert!(!notice.is_expired());

        // Set expiration in the past
        notice.expires_at = Some(Utc::now() - Duration::days(1));
        assert!(notice.is_expired());

        // Set expiration in the future
        notice.expires_at = Some(Utc::now() + Duration::days(1));
        assert!(!notice.is_expired());
    }

    #[test]
    fn test_update_content_draft_success() {
        let building_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let mut notice = Notice::new(
            building_id,
            author_id,
            NoticeType::Announcement,
            NoticeCategory::General,
            "Original Title".to_string(),
            "Original content.".to_string(),
            None,
            None,
            None,
        )
        .unwrap();

        let result = notice.update_content(
            Some("Updated Title".to_string()),
            Some("Updated content.".to_string()),
            Some(NoticeCategory::Maintenance),
            None,
            None,
            None,
            None,
        );

        assert!(result.is_ok());
        assert_eq!(notice.title, "Updated Title");
        assert_eq!(notice.content, "Updated content.");
        assert_eq!(notice.category, NoticeCategory::Maintenance);
    }

    #[test]
    fn test_update_content_published_fails() {
        let building_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let mut notice = Notice::new(
            building_id,
            author_id,
            NoticeType::Announcement,
            NoticeCategory::General,
            "Original Title".to_string(),
            "Original content.".to_string(),
            None,
            None,
            None,
        )
        .unwrap();

        notice.publish().unwrap();

        let result = notice.update_content(
            Some("Updated Title".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Only Draft notices can be updated"));
    }

    #[test]
    fn test_set_expiration_future_success() {
        let building_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let mut notice = Notice::new(
            building_id,
            author_id,
            NoticeType::Announcement,
            NoticeCategory::General,
            "Important Announcement".to_string(),
            "This is an important announcement.".to_string(),
            None,
            None,
            None,
        )
        .unwrap();

        let future_date = Utc::now() + Duration::days(7);
        let result = notice.set_expiration(Some(future_date));

        assert!(result.is_ok());
        assert_eq!(notice.expires_at, Some(future_date));
    }

    #[test]
    fn test_set_expiration_past_fails() {
        let building_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let mut notice = Notice::new(
            building_id,
            author_id,
            NoticeType::Announcement,
            NoticeCategory::General,
            "Important Announcement".to_string(),
            "This is an important announcement.".to_string(),
            None,
            None,
            None,
        )
        .unwrap();

        let past_date = Utc::now() - Duration::days(1);
        let result = notice.set_expiration(Some(past_date));

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Expiration date must be in the future"
        );
    }

    #[test]
    fn test_expire_published_success() {
        let building_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();

        let mut notice = Notice::new(
            building_id,
            author_id,
            NoticeType::Announcement,
            NoticeCategory::General,
            "Important Announcement".to_string(),
            "This is an important announcement.".to_string(),
            None,
            None,
            None,
        )
        .unwrap();

        notice.publish().unwrap();
        notice.pin().unwrap();
        assert!(notice.is_pinned);

        let result = notice.expire();
        assert!(result.is_ok());
        assert_eq!(notice.status, NoticeStatus::Expired);
        assert!(!notice.is_pinned); // Should be unpinned
    }
}
