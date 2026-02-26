use crate::application::dto::{
    CreateNoticeDto, NoticeResponseDto, NoticeSummaryDto, SetExpirationDto, UpdateNoticeDto,
};
use crate::application::ports::{NoticeRepository, OwnerRepository};
use crate::domain::entities::{Notice, NoticeCategory, NoticeStatus, NoticeType};
use std::sync::Arc;
use uuid::Uuid;

pub struct NoticeUseCases {
    notice_repo: Arc<dyn NoticeRepository>,
    owner_repo: Arc<dyn OwnerRepository>,
}

impl NoticeUseCases {
    pub fn new(
        notice_repo: Arc<dyn NoticeRepository>,
        owner_repo: Arc<dyn OwnerRepository>,
    ) -> Self {
        Self {
            notice_repo,
            owner_repo,
        }
    }

    /// Check if user has building admin privileges (admin, superadmin, or syndic)
    fn is_building_admin(role: &str) -> bool {
        role == "admin" || role == "superadmin" || role == "syndic"
    }

    /// Resolve user_id to owner_id via organization lookup
    async fn resolve_owner(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
    ) -> Result<crate::domain::entities::Owner, String> {
        self.owner_repo
            .find_by_user_id_and_organization(user_id, organization_id)
            .await?
            .ok_or_else(|| "Owner not found for this user in the organization".to_string())
    }

    /// Create a new notice (Draft status)
    ///
    /// # Authorization
    /// - Author must be a member of the building (validated by owner_repo)
    pub async fn create_notice(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        dto: CreateNoticeDto,
    ) -> Result<NoticeResponseDto, String> {
        // Resolve user_id to owner
        let author = self.resolve_owner(user_id, organization_id).await?;
        let author_id = author.id;

        // Create notice entity (validates business rules)
        let notice = Notice::new(
            dto.building_id,
            author_id,
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

        // Return enriched response
        let author_name = format!("{} {}", author.first_name, author.last_name);
        Ok(NoticeResponseDto::from_notice(created, author_name))
    }

    /// Get notice by ID with author name enrichment
    pub async fn get_notice(&self, notice_id: Uuid) -> Result<NoticeResponseDto, String> {
        let notice = self
            .notice_repo
            .find_by_id(notice_id)
            .await?
            .ok_or("Notice not found".to_string())?;

        // Enrich with author name
        let author = self
            .owner_repo
            .find_by_id(notice.author_id)
            .await?
            .ok_or("Author not found".to_string())?;

        let author_name = format!("{} {}", author.first_name, author.last_name);
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
        organization_id: Uuid,
        dto: UpdateNoticeDto,
    ) -> Result<NoticeResponseDto, String> {
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let mut notice = self
            .notice_repo
            .find_by_id(notice_id)
            .await?
            .ok_or("Notice not found".to_string())?;

        // Authorization: only author can update
        if notice.author_id != owner.id {
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
        organization_id: Uuid,
    ) -> Result<NoticeResponseDto, String> {
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let mut notice = self
            .notice_repo
            .find_by_id(notice_id)
            .await?
            .ok_or("Notice not found".to_string())?;

        // Authorization: only author can publish
        if notice.author_id != owner.id {
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
        organization_id: Uuid,
        actor_role: &str,
    ) -> Result<NoticeResponseDto, String> {
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let mut notice = self
            .notice_repo
            .find_by_id(notice_id)
            .await?
            .ok_or("Notice not found".to_string())?;

        // Authorization: only author or building admin can archive
        let is_author = notice.author_id == owner.id;
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
        organization_id: Uuid,
        dto: SetExpirationDto,
    ) -> Result<NoticeResponseDto, String> {
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let mut notice = self
            .notice_repo
            .find_by_id(notice_id)
            .await?
            .ok_or("Notice not found".to_string())?;

        // Authorization: only author can set expiration
        if notice.author_id != owner.id {
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
        organization_id: Uuid,
    ) -> Result<(), String> {
        let owner = self.resolve_owner(user_id, organization_id).await?;
        let notice = self
            .notice_repo
            .find_by_id(notice_id)
            .await?
            .ok_or("Notice not found".to_string())?;

        // Authorization: only author can delete
        if notice.author_id != owner.id {
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
            // Get author name
            let author = self.owner_repo.find_by_id(notice.author_id).await?;
            let author_name = if let Some(owner) = author {
                format!("{} {}", owner.first_name, owner.last_name)
            } else {
                "Unknown Author".to_string()
            };

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
