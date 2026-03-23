use crate::domain::entities::individual_member::IndividualMember;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait IndividualMemberRepository: Send + Sync {
    async fn create(&self, member: &IndividualMember) -> Result<IndividualMember, String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<IndividualMember>, String>;
    async fn find_by_campaign(
        &self,
        campaign_id: Uuid,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<IndividualMember>, String>;
    async fn find_by_email(&self, email: &str) -> Result<Option<IndividualMember>, String>;
    async fn find_by_email_and_campaign(
        &self,
        email: &str,
        campaign_id: Uuid,
    ) -> Result<Option<IndividualMember>, String>;
    async fn find_active_by_campaign(
        &self,
        campaign_id: Uuid,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<IndividualMember>, String>;
    async fn find_with_consent_by_campaign(
        &self,
        campaign_id: Uuid,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<IndividualMember>, String>;
    async fn update(&self, member: &IndividualMember) -> Result<IndividualMember, String>;
    async fn delete(&self, id: Uuid) -> Result<(), String>;
    async fn withdraw_consent(&self, id: Uuid) -> Result<(), String>;
    async fn count_by_campaign(&self, campaign_id: Uuid) -> Result<i64, String>;
    async fn count_active_by_campaign(&self, campaign_id: Uuid) -> Result<i64, String>;
    async fn count_with_consent_by_campaign(&self, campaign_id: Uuid) -> Result<i64, String>;
}
