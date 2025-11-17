use crate::domain::entities::{Notice, NoticeCategory, NoticeStatus, NoticeType};
use async_trait::async_trait;
use uuid::Uuid;

/// Repository port for Notice aggregate
#[async_trait]
pub trait NoticeRepository: Send + Sync {
    /// Create a new notice
    async fn create(&self, notice: &Notice) -> Result<Notice, String>;

    /// Find notice by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Notice>, String>;

    /// Find all notices for a building
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Notice>, String>;

    /// Find all published notices for a building (visible to members)
    async fn find_published_by_building(&self, building_id: Uuid)
        -> Result<Vec<Notice>, String>;

    /// Find all pinned notices for a building (important announcements)
    async fn find_pinned_by_building(&self, building_id: Uuid) -> Result<Vec<Notice>, String>;

    /// Find notices by type (Announcement, Event, LostAndFound, ClassifiedAd)
    async fn find_by_type(
        &self,
        building_id: Uuid,
        notice_type: NoticeType,
    ) -> Result<Vec<Notice>, String>;

    /// Find notices by category (General, Maintenance, Social, Security, etc.)
    async fn find_by_category(
        &self,
        building_id: Uuid,
        category: NoticeCategory,
    ) -> Result<Vec<Notice>, String>;

    /// Find notices by status (Draft, Published, Archived, Expired)
    async fn find_by_status(
        &self,
        building_id: Uuid,
        status: NoticeStatus,
    ) -> Result<Vec<Notice>, String>;

    /// Find all notices created by an author (Owner)
    async fn find_by_author(&self, author_id: Uuid) -> Result<Vec<Notice>, String>;

    /// Find all expired notices for a building (for auto-archiving)
    async fn find_expired(&self, building_id: Uuid) -> Result<Vec<Notice>, String>;

    /// Update an existing notice
    async fn update(&self, notice: &Notice) -> Result<Notice, String>;

    /// Delete a notice
    async fn delete(&self, id: Uuid) -> Result<(), String>;

    /// Count total notices for a building
    async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String>;

    /// Count published notices for a building
    async fn count_published_by_building(&self, building_id: Uuid) -> Result<i64, String>;

    /// Count pinned notices for a building
    async fn count_pinned_by_building(&self, building_id: Uuid) -> Result<i64, String>;
}
