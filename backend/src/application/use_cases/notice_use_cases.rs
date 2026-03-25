use crate::application::dto::{
    CreateNoticeDto, NoticeResponseDto, NoticeSummaryDto, SetExpirationDto, UpdateNoticeDto,
};
use crate::application::ports::{NoticeRepository, UserRepository};
use crate::domain::entities::{Notice, NoticeCategory, NoticeStatus, NoticeType};
use std::sync::Arc;
use uuid::Uuid;

pub struct NoticeUseCases {
    notice_repo: Arc<dyn NoticeRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl NoticeUseCases {
    pub fn new(notice_repo: Arc<dyn NoticeRepository>, user_repo: Arc<dyn UserRepository>) -> Self {
        Self {
            notice_repo,
            user_repo,
        }
    }

    /// Check if user has building admin privileges (admin, superadmin, or syndic)
    fn is_building_admin(role: &str) -> bool {
        role == "admin" || role == "superadmin" || role == "syndic"
    }

    /// Resolve user_id to display name via user lookup
    async fn resolve_author_name(&self, user_id: Uuid) -> String {
        match self.user_repo.find_by_id(user_id).await {
            Ok(Some(user)) => format!("{} {}", user.first_name, user.last_name),
            _ => "Unknown Author".to_string(),
        }
    }

    /// Create a new notice (Draft status)
    ///
    /// # Authorization
    /// - Any authenticated user in the organization can post a notice
    ///   (syndic, admin, owner — all are valid authors)
    pub async fn create_notice(
        &self,
        user_id: Uuid,
        _organization_id: Uuid,
        dto: CreateNoticeDto,
    ) -> Result<NoticeResponseDto, String> {
        // author_id is the user's own ID (notices.author_id now references users.id)
        let notice = Notice::new(
            dto.building_id,
            user_id,
            dto.notice_type,
            dto.category,
            dto.title,
            dto.content,
            dto.event_date,
            dto.event_location,
            dto.contact_info,
        )?;

        // Set expiration if provided
        let mut notice = notice;
        if let Some(expires_at) = dto.expires_at {
            notice.set_expiration(Some(expires_at))?;
        }

        // Persist notice
        let created = self.notice_repo.create(&notice).await?;

        let author_name = self.resolve_author_name(user_id).await;
        Ok(NoticeResponseDto::from_notice(created, author_name))
    }

    /// Get notice by ID with author name enrichment
    pub async fn get_notice(&self, notice_id: Uuid) -> Result<NoticeResponseDto, String> {
        let notice = self
            .notice_repo
            .find_by_id(notice_id)
            .await?
            .ok_or("Notice not found".to_string())?;

        let author_name = self.resolve_author_name(notice.author_id).await;
        Ok(NoticeResponseDto::from_notice(notice, author_name))
    }

    /// List all notices for a building (all statuses)
    ///
    /// # Returns
    /// - Notices sorted by pinned (DESC), created_at (DESC)
    pub async fn list_building_notices(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<NoticeSummaryDto>, String> {
        let notices = self.notice_repo.find_by_building(building_id).await?;
        self.enrich_notices_summary(notices).await
    }

    /// List published notices for a building (visible to members)
    ///
    /// # Returns
    /// - Only Published notices, sorted by pinned (DESC), published_at (DESC)
    pub async fn list_published_notices(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<NoticeSummaryDto>, String> {
        let notices = self
            .notice_repo
            .find_published_by_building(building_id)
            .await?;
        self.enrich_notices_summary(notices).await
    }

    /// List pinned notices for a building (important announcements)
    pub async fn list_pinned_notices(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<NoticeSummaryDto>, String> {
        let notices = self
            .notice_repo
            .find_pinned_by_building(building_id)
            .await?;
        self.enrich_notices_summary(notices).await
    }

    /// List notices by type (Announcement, Event, LostAndFound, ClassifiedAd)
    pub async fn list_notices_by_type(
        &self,
        building_id: Uuid,
        notice_type: NoticeType,
    ) -> Result<Vec<NoticeSummaryDto>, String> {
        let notices = self
            .notice_repo
            .find_by_type(building_id, notice_type)
            .await?;
        self.enrich_notices_summary(notices).await
    }

    /// List notices by category (General, Maintenance, Social, etc.)
    pub async fn list_notices_by_category(
        &self,
        building_id: Uuid,
        category: NoticeCategory,
    ) -> Result<Vec<NoticeSummaryDto>, String> {
        let notices = self
            .notice_repo
            .find_by_category(building_id, category)
            .await?;
        self.enrich_notices_summary(notices).await
    }

    /// List notices by status (Draft, Published, Archived, Expired)
    pub async fn list_notices_by_status(
        &self,
        building_id: Uuid,
        status: NoticeStatus,
    ) -> Result<Vec<NoticeSummaryDto>, String> {
        let notices = self.notice_repo.find_by_status(building_id, status).await?;
        self.enrich_notices_summary(notices).await
    }

    /// List all notices created by an author
    pub async fn list_author_notices(
        &self,
        author_id: Uuid,
    ) -> Result<Vec<NoticeSummaryDto>, String> {
        let notices = self.notice_repo.find_by_author(author_id).await?;
        self.enrich_notices_summary(notices).await
    }

    /// Update a notice (Draft only)
    ///
    /// # Authorization
    /// - Only author can update their notice
    /// - Only Draft notices can be updated
    pub async fn update_notice(
        &self,
        notice_id: Uuid,
        user_id: Uuid,
        _organization_id: Uuid,
        dto: UpdateNoticeDto,
    ) -> Result<NoticeResponseDto, String> {
        let mut notice = self
            .notice_repo
            .find_by_id(notice_id)
            .await?
            .ok_or("Notice not found".to_string())?;

        // Authorization: only author can update
        if notice.author_id != user_id {
            return Err("Unauthorized: only author can update notice".to_string());
        }

        // Update content (domain validates Draft status)
        notice.update_content(
            dto.title,
            dto.content,
            dto.category,
            dto.event_date,
            dto.event_location,
            dto.contact_info,
            dto.expires_at,
        )?;

        // Persist changes
        let updated = self.notice_repo.update(&notice).await?;

        // Return enriched response
        self.get_notice(updated.id).await
    }

    /// Publish a notice (Draft → Published)
    ///
    /// # Authorization
    /// - Only author can publish their notice
    pub async fn publish_notice(
        &self,
        notice_id: Uuid,
        user_id: Uuid,
        _organization_id: Uuid,
    ) -> Result<NoticeResponseDto, String> {
        let mut notice = self
            .notice_repo
            .find_by_id(notice_id)
            .await?
            .ok_or("Notice not found".to_string())?;

        // Authorization: only author can publish
        if notice.author_id != user_id {
            return Err("Unauthorized: only author can publish notice".to_string());
        }

        // Publish (domain validates state transition)
        notice.publish()?;

        // Persist changes
        let updated = self.notice_repo.update(&notice).await?;

        // Return enriched response
        self.get_notice(updated.id).await
    }

    /// Archive a notice (Published/Expired → Archived)
    ///
    /// # Authorization
    /// - Only author or building admin can archive
    pub async fn archive_notice(
        &self,
        notice_id: Uuid,
        user_id: Uuid,
        _organization_id: Uuid,
        actor_role: &str,
    ) -> Result<NoticeResponseDto, String> {
        let mut notice = self
            .notice_repo
            .find_by_id(notice_id)
            .await?
            .ok_or("Notice not found".to_string())?;

        // Authorization: only author or building admin can archive
        let is_author = notice.author_id == user_id;
        let is_admin = Self::is_building_admin(actor_role);

        if !is_author && !is_admin {
            return Err(
                "Unauthorized: only author or building admin can archive notice".to_string(),
            );
        }

        // Archive (domain validates state transition)
        notice.archive()?;

        // Persist changes
        let updated = self.notice_repo.update(&notice).await?;

        // Return enriched response
        self.get_notice(updated.id).await
    }

    /// Pin a notice to top of board (Published only)
    ///
    /// # Authorization
    /// - Only building admin (admin, superadmin, or syndic) can pin notices
    pub async fn pin_notice(
        &self,
        notice_id: Uuid,
        actor_role: &str,
    ) -> Result<NoticeResponseDto, String> {
        // Authorization: only building admin can pin
        if !Self::is_building_admin(actor_role) {
            return Err(
                "Unauthorized: only building admin (admin, superadmin, or syndic) can pin notices"
                    .to_string(),
            );
        }

        let mut notice = self
            .notice_repo
            .find_by_id(notice_id)
            .await?
            .ok_or("Notice not found".to_string())?;

        // Pin (domain validates Published status)
        notice.pin()?;

        // Persist changes
        let updated = self.notice_repo.update(&notice).await?;

        // Return enriched response
        self.get_notice(updated.id).await
    }

    /// Unpin a notice
    ///
    /// # Authorization
    /// - Only building admin (admin, superadmin, or syndic) can unpin notices
    pub async fn unpin_notice(
        &self,
        notice_id: Uuid,
        actor_role: &str,
    ) -> Result<NoticeResponseDto, String> {
        // Authorization: only building admin can unpin
        if !Self::is_building_admin(actor_role) {
            return Err("Unauthorized: only building admin (admin, superadmin, or syndic) can unpin notices".to_string());
        }

        let mut notice = self
            .notice_repo
            .find_by_id(notice_id)
            .await?
            .ok_or("Notice not found".to_string())?;

        // Unpin
        notice.unpin()?;

        // Persist changes
        let updated = self.notice_repo.update(&notice).await?;

        // Return enriched response
        self.get_notice(updated.id).await
    }

    /// Set expiration date for a notice
    ///
    /// # Authorization
    /// - Only author can set expiration
    pub async fn set_expiration(
        &self,
        notice_id: Uuid,
        user_id: Uuid,
        _organization_id: Uuid,
        dto: SetExpirationDto,
    ) -> Result<NoticeResponseDto, String> {
        let mut notice = self
            .notice_repo
            .find_by_id(notice_id)
            .await?
            .ok_or("Notice not found".to_string())?;

        // Authorization: only author can set expiration
        if notice.author_id != user_id {
            return Err("Unauthorized: only author can set expiration".to_string());
        }

        // Set expiration (domain validates future date)
        notice.set_expiration(dto.expires_at)?;

        // Persist changes
        let updated = self.notice_repo.update(&notice).await?;

        // Return enriched response
        self.get_notice(updated.id).await
    }

    /// Delete a notice
    ///
    /// # Authorization
    /// - Only author can delete their notice
    /// - Cannot delete Published/Archived notices (must archive first)
    pub async fn delete_notice(
        &self,
        notice_id: Uuid,
        user_id: Uuid,
        _organization_id: Uuid,
    ) -> Result<(), String> {
        let notice = self
            .notice_repo
            .find_by_id(notice_id)
            .await?
            .ok_or("Notice not found".to_string())?;

        // Authorization: only author can delete
        if notice.author_id != user_id {
            return Err("Unauthorized: only author can delete notice".to_string());
        }

        // Business rule: cannot delete Published or Archived notices
        match notice.status {
            NoticeStatus::Published | NoticeStatus::Archived => {
                return Err(format!(
                    "Cannot delete notice in status {:?}. Archive it first.",
                    notice.status
                ));
            }
            _ => {}
        }

        // Delete notice
        self.notice_repo.delete(notice_id).await?;

        Ok(())
    }

    /// Automatically expire notices that have passed their expiration date
    ///
    /// # Background Job
    /// - Should be called periodically (e.g., daily cron job)
    /// - Finds all Published notices with expires_at in the past
    /// - Transitions them to Expired status
    pub async fn auto_expire_notices(&self, building_id: Uuid) -> Result<Vec<Uuid>, String> {
        let expired_notices = self.notice_repo.find_expired(building_id).await?;

        let mut expired_ids = Vec::new();

        for mut notice in expired_notices {
            // Expire notice (domain validates state transition)
            if let Err(e) = notice.expire() {
                log::warn!("Failed to expire notice {}: {}. Skipping.", notice.id, e);
                continue;
            }

            // Persist changes
            match self.notice_repo.update(&notice).await {
                Ok(_) => {
                    expired_ids.push(notice.id);
                    log::info!("Auto-expired notice: {}", notice.id);
                }
                Err(e) => {
                    log::error!("Failed to update expired notice {}: {}", notice.id, e);
                }
            }
        }

        Ok(expired_ids)
    }

    /// Get notice statistics for a building
    pub async fn get_statistics(&self, building_id: Uuid) -> Result<NoticeStatistics, String> {
        let total_count = self.notice_repo.count_by_building(building_id).await?;
        let published_count = self
            .notice_repo
            .count_published_by_building(building_id)
            .await?;
        let pinned_count = self
            .notice_repo
            .count_pinned_by_building(building_id)
            .await?;

        Ok(NoticeStatistics {
            total_count,
            published_count,
            pinned_count,
        })
    }

    // Helper method to enrich notices with author names
    async fn enrich_notices_summary(
        &self,
        notices: Vec<Notice>,
    ) -> Result<Vec<NoticeSummaryDto>, String> {
        let mut enriched = Vec::new();

        for notice in notices {
            let author_name = self.resolve_author_name(notice.author_id).await;
            enriched.push(NoticeSummaryDto::from_notice(notice, author_name));
        }

        Ok(enriched)
    }
}

/// Notice statistics for a building
#[derive(Debug, serde::Serialize)]
pub struct NoticeStatistics {
    pub total_count: i64,
    pub published_count: i64,
    pub pinned_count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{NoticeRepository, UserRepository};
    use crate::domain::entities::{
        Notice, NoticeCategory, NoticeStatus, NoticeType, User, UserRole,
    };
    use async_trait::async_trait;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use uuid::Uuid;

    // ─── Mock NoticeRepository ──────────────────────────────────────────

    struct MockNoticeRepo {
        notices: Mutex<HashMap<Uuid, Notice>>,
    }

    impl MockNoticeRepo {
        fn new() -> Self {
            Self {
                notices: Mutex::new(HashMap::new()),
            }
        }

        fn with_notice(notice: Notice) -> Self {
            let mut map = HashMap::new();
            map.insert(notice.id, notice);
            Self {
                notices: Mutex::new(map),
            }
        }
    }

    #[async_trait]
    impl NoticeRepository for MockNoticeRepo {
        async fn create(&self, notice: &Notice) -> Result<Notice, String> {
            let mut store = self.notices.lock().unwrap();
            store.insert(notice.id, notice.clone());
            Ok(notice.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Notice>, String> {
            let store = self.notices.lock().unwrap();
            Ok(store.get(&id).cloned())
        }

        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Notice>, String> {
            let store = self.notices.lock().unwrap();
            Ok(store
                .values()
                .filter(|n| n.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn find_published_by_building(
            &self,
            building_id: Uuid,
        ) -> Result<Vec<Notice>, String> {
            let store = self.notices.lock().unwrap();
            Ok(store
                .values()
                .filter(|n| n.building_id == building_id && n.status == NoticeStatus::Published)
                .cloned()
                .collect())
        }

        async fn find_pinned_by_building(&self, building_id: Uuid) -> Result<Vec<Notice>, String> {
            let store = self.notices.lock().unwrap();
            Ok(store
                .values()
                .filter(|n| n.building_id == building_id && n.is_pinned)
                .cloned()
                .collect())
        }

        async fn find_by_type(
            &self,
            building_id: Uuid,
            notice_type: NoticeType,
        ) -> Result<Vec<Notice>, String> {
            let store = self.notices.lock().unwrap();
            Ok(store
                .values()
                .filter(|n| n.building_id == building_id && n.notice_type == notice_type)
                .cloned()
                .collect())
        }

        async fn find_by_category(
            &self,
            building_id: Uuid,
            category: NoticeCategory,
        ) -> Result<Vec<Notice>, String> {
            let store = self.notices.lock().unwrap();
            Ok(store
                .values()
                .filter(|n| n.building_id == building_id && n.category == category)
                .cloned()
                .collect())
        }

        async fn find_by_status(
            &self,
            building_id: Uuid,
            status: NoticeStatus,
        ) -> Result<Vec<Notice>, String> {
            let store = self.notices.lock().unwrap();
            Ok(store
                .values()
                .filter(|n| n.building_id == building_id && n.status == status)
                .cloned()
                .collect())
        }

        async fn find_by_author(&self, author_id: Uuid) -> Result<Vec<Notice>, String> {
            let store = self.notices.lock().unwrap();
            Ok(store
                .values()
                .filter(|n| n.author_id == author_id)
                .cloned()
                .collect())
        }

        async fn find_expired(&self, building_id: Uuid) -> Result<Vec<Notice>, String> {
            let store = self.notices.lock().unwrap();
            Ok(store
                .values()
                .filter(|n| {
                    n.building_id == building_id
                        && n.status == NoticeStatus::Published
                        && n.is_expired()
                })
                .cloned()
                .collect())
        }

        async fn update(&self, notice: &Notice) -> Result<Notice, String> {
            let mut store = self.notices.lock().unwrap();
            store.insert(notice.id, notice.clone());
            Ok(notice.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<(), String> {
            let mut store = self.notices.lock().unwrap();
            store.remove(&id);
            Ok(())
        }

        async fn count_by_building(&self, building_id: Uuid) -> Result<i64, String> {
            let store = self.notices.lock().unwrap();
            Ok(store
                .values()
                .filter(|n| n.building_id == building_id)
                .count() as i64)
        }

        async fn count_published_by_building(&self, building_id: Uuid) -> Result<i64, String> {
            let store = self.notices.lock().unwrap();
            Ok(store
                .values()
                .filter(|n| n.building_id == building_id && n.status == NoticeStatus::Published)
                .count() as i64)
        }

        async fn count_pinned_by_building(&self, building_id: Uuid) -> Result<i64, String> {
            let store = self.notices.lock().unwrap();
            Ok(store
                .values()
                .filter(|n| n.building_id == building_id && n.is_pinned)
                .count() as i64)
        }
    }

    // ─── Mock UserRepository ────────────────────────────────────────────

    struct MockUserRepo {
        users: Mutex<HashMap<Uuid, User>>,
    }

    impl MockUserRepo {
        fn new() -> Self {
            Self {
                users: Mutex::new(HashMap::new()),
            }
        }

        fn with_user(user: User) -> Self {
            let mut map = HashMap::new();
            map.insert(user.id, user);
            Self {
                users: Mutex::new(map),
            }
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepo {
        async fn create(&self, user: &User) -> Result<User, String> {
            let mut store = self.users.lock().unwrap();
            store.insert(user.id, user.clone());
            Ok(user.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, String> {
            let store = self.users.lock().unwrap();
            Ok(store.get(&id).cloned())
        }

        async fn find_by_email(&self, email: &str) -> Result<Option<User>, String> {
            let store = self.users.lock().unwrap();
            Ok(store.values().find(|u| u.email == email).cloned())
        }

        async fn find_all(&self) -> Result<Vec<User>, String> {
            let store = self.users.lock().unwrap();
            Ok(store.values().cloned().collect())
        }

        async fn find_by_organization(&self, org_id: Uuid) -> Result<Vec<User>, String> {
            let store = self.users.lock().unwrap();
            Ok(store
                .values()
                .filter(|u| u.organization_id == Some(org_id))
                .cloned()
                .collect())
        }

        async fn update(&self, user: &User) -> Result<User, String> {
            let mut store = self.users.lock().unwrap();
            store.insert(user.id, user.clone());
            Ok(user.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            let mut store = self.users.lock().unwrap();
            Ok(store.remove(&id).is_some())
        }

        async fn count_by_organization(&self, org_id: Uuid) -> Result<i64, String> {
            let store = self.users.lock().unwrap();
            Ok(store
                .values()
                .filter(|u| u.organization_id == Some(org_id))
                .count() as i64)
        }
    }

    // ─── Helpers ────────────────────────────────────────────────────────

    fn make_user(id: Uuid) -> User {
        User {
            id,
            email: format!("user-{}@example.com", &id.to_string()[..8]),
            password_hash: "hash".to_string(),
            first_name: "Jean".to_string(),
            last_name: "Dupont".to_string(),
            role: UserRole::Owner,
            organization_id: Some(Uuid::new_v4()),
            is_active: true,
            processing_restricted: false,
            processing_restricted_at: None,
            marketing_opt_out: false,
            marketing_opt_out_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_draft_notice(building_id: Uuid, author_id: Uuid) -> Notice {
        Notice::new(
            building_id,
            author_id,
            NoticeType::Announcement,
            NoticeCategory::General,
            "Test Notice Title".to_string(),
            "This is a test notice content for unit testing.".to_string(),
            None,
            None,
            None,
        )
        .unwrap()
    }

    fn make_published_notice(building_id: Uuid, author_id: Uuid) -> Notice {
        let mut notice = make_draft_notice(building_id, author_id);
        notice.publish().unwrap();
        notice
    }

    // ─── Tests ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_create_notice_success() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let user = make_user(user_id);

        let uc = NoticeUseCases::new(
            Arc::new(MockNoticeRepo::new()),
            Arc::new(MockUserRepo::with_user(user)),
        );

        let dto = CreateNoticeDto {
            building_id,
            notice_type: NoticeType::Announcement,
            category: NoticeCategory::General,
            title: "Important Building Notice".to_string(),
            content: "Please be aware of upcoming maintenance work.".to_string(),
            event_date: None,
            event_location: None,
            contact_info: None,
            expires_at: None,
        };

        let result = uc.create_notice(user_id, org_id, dto).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.title, "Important Building Notice");
        assert_eq!(resp.status, NoticeStatus::Draft);
        assert_eq!(resp.author_id, user_id);
        assert_eq!(resp.author_name, "Jean Dupont");
        assert!(!resp.is_pinned);
    }

    #[tokio::test]
    async fn test_get_notice_success() {
        let user_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let notice = make_draft_notice(building_id, user_id);
        let notice_id = notice.id;
        let user = make_user(user_id);

        let uc = NoticeUseCases::new(
            Arc::new(MockNoticeRepo::with_notice(notice)),
            Arc::new(MockUserRepo::with_user(user)),
        );

        let result = uc.get_notice(notice_id).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.id, notice_id);
        assert_eq!(resp.author_name, "Jean Dupont");
    }

    #[tokio::test]
    async fn test_get_notice_not_found() {
        let uc = NoticeUseCases::new(
            Arc::new(MockNoticeRepo::new()),
            Arc::new(MockUserRepo::new()),
        );

        let result = uc.get_notice(Uuid::new_v4()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Notice not found");
    }

    #[tokio::test]
    async fn test_publish_notice_success() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let notice = make_draft_notice(building_id, user_id);
        let notice_id = notice.id;
        let user = make_user(user_id);

        let uc = NoticeUseCases::new(
            Arc::new(MockNoticeRepo::with_notice(notice)),
            Arc::new(MockUserRepo::with_user(user)),
        );

        let result = uc.publish_notice(notice_id, user_id, org_id).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.status, NoticeStatus::Published);
        assert!(resp.published_at.is_some());
    }

    #[tokio::test]
    async fn test_publish_notice_unauthorized() {
        let author_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let notice = make_draft_notice(building_id, author_id);
        let notice_id = notice.id;

        let uc = NoticeUseCases::new(
            Arc::new(MockNoticeRepo::with_notice(notice)),
            Arc::new(MockUserRepo::new()),
        );

        let result = uc.publish_notice(notice_id, other_user_id, org_id).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Unauthorized: only author can publish notice"));
    }

    #[tokio::test]
    async fn test_archive_notice_by_author() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let notice = make_published_notice(building_id, user_id);
        let notice_id = notice.id;
        let user = make_user(user_id);

        let uc = NoticeUseCases::new(
            Arc::new(MockNoticeRepo::with_notice(notice)),
            Arc::new(MockUserRepo::with_user(user)),
        );

        let result = uc.archive_notice(notice_id, user_id, org_id, "owner").await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.status, NoticeStatus::Archived);
        assert!(resp.archived_at.is_some());
    }

    #[tokio::test]
    async fn test_archive_notice_by_admin() {
        let author_id = Uuid::new_v4();
        let admin_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let notice = make_published_notice(building_id, author_id);
        let notice_id = notice.id;
        let admin_user = make_user(admin_id);

        let uc = NoticeUseCases::new(
            Arc::new(MockNoticeRepo::with_notice(notice)),
            Arc::new(MockUserRepo::with_user(admin_user)),
        );

        // Admin (not the author) can archive
        let result = uc
            .archive_notice(notice_id, admin_id, org_id, "admin")
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, NoticeStatus::Archived);
    }

    #[tokio::test]
    async fn test_archive_notice_unauthorized_non_author_non_admin() {
        let author_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let notice = make_published_notice(building_id, author_id);
        let notice_id = notice.id;

        let uc = NoticeUseCases::new(
            Arc::new(MockNoticeRepo::with_notice(notice)),
            Arc::new(MockUserRepo::new()),
        );

        let result = uc
            .archive_notice(notice_id, other_user_id, org_id, "owner")
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Unauthorized: only author or building admin can archive notice"));
    }

    #[tokio::test]
    async fn test_pin_notice_admin_success() {
        let user_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let notice = make_published_notice(building_id, user_id);
        let notice_id = notice.id;
        let user = make_user(user_id);

        let uc = NoticeUseCases::new(
            Arc::new(MockNoticeRepo::with_notice(notice)),
            Arc::new(MockUserRepo::with_user(user)),
        );

        let result = uc.pin_notice(notice_id, "syndic").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_pinned);
    }

    #[tokio::test]
    async fn test_pin_notice_unauthorized_owner() {
        let user_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let notice = make_published_notice(building_id, user_id);
        let notice_id = notice.id;

        let uc = NoticeUseCases::new(
            Arc::new(MockNoticeRepo::with_notice(notice)),
            Arc::new(MockUserRepo::new()),
        );

        let result = uc.pin_notice(notice_id, "owner").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Unauthorized: only building admin"));
    }

    #[tokio::test]
    async fn test_delete_notice_success_draft() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let notice = make_draft_notice(building_id, user_id);
        let notice_id = notice.id;

        let uc = NoticeUseCases::new(
            Arc::new(MockNoticeRepo::with_notice(notice)),
            Arc::new(MockUserRepo::new()),
        );

        let result = uc.delete_notice(notice_id, user_id, org_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_notice_blocked_for_published() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let notice = make_published_notice(building_id, user_id);
        let notice_id = notice.id;

        let uc = NoticeUseCases::new(
            Arc::new(MockNoticeRepo::with_notice(notice)),
            Arc::new(MockUserRepo::new()),
        );

        let result = uc.delete_notice(notice_id, user_id, org_id).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Cannot delete notice in status"));
    }
}
