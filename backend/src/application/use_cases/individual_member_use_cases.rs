use crate::application::dto::individual_member_dto::IndividualMemberResponseDto;
use crate::application::ports::individual_member_repository::IndividualMemberRepository;
use crate::domain::entities::IndividualMember;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct IndividualMemberUseCases {
    pub repo: Arc<dyn IndividualMemberRepository>,
}

impl IndividualMemberUseCases {
    pub fn new(repo: Arc<dyn IndividualMemberRepository>) -> Self {
        Self { repo }
    }

    /// Join a campaign as an individual member
    pub async fn join_campaign(
        &self,
        campaign_id: Uuid,
        email: String,
        postal_code: String,
    ) -> Result<IndividualMemberResponseDto, String> {
        // Check for duplicate email in campaign
        if let Some(_existing) = self
            .repo
            .find_by_email_and_campaign(&email, campaign_id)
            .await?
        {
            return Err("Email already registered for this campaign".to_string());
        }

        let member = IndividualMember::new(campaign_id, email, postal_code)?;
        let created = self.repo.create(&member).await?;
        Ok(IndividualMemberResponseDto::from(created))
    }

    /// Grant GDPR consent
    pub async fn grant_consent(
        &self,
        campaign_id: Uuid,
        member_id: Uuid,
    ) -> Result<IndividualMemberResponseDto, String> {
        let mut member = self
            .repo
            .find_by_id(member_id)
            .await?
            .ok_or_else(|| format!("Member {} not found", member_id))?;

        if member.campaign_id != campaign_id {
            return Err("Member does not belong to this campaign".to_string());
        }

        member.has_gdpr_consent = true;
        member.consent_at = Some(Utc::now());

        let updated = self.repo.update(&member).await?;
        Ok(IndividualMemberResponseDto::from(updated))
    }

    /// Update consumption data
    pub async fn update_consumption(
        &self,
        campaign_id: Uuid,
        member_id: Uuid,
        annual_kwh: Option<f64>,
        current_provider: Option<String>,
        ean_code: Option<String>,
    ) -> Result<IndividualMemberResponseDto, String> {
        let mut member = self
            .repo
            .find_by_id(member_id)
            .await?
            .ok_or_else(|| format!("Member {} not found", member_id))?;

        if member.campaign_id != campaign_id {
            return Err("Member does not belong to this campaign".to_string());
        }

        if let Some(kwh) = annual_kwh {
            if kwh < 0.0 {
                return Err("Consumption cannot be negative".to_string());
            }
            member.annual_consumption_kwh = Some(kwh);
        }
        if let Some(provider) = current_provider {
            member.current_provider = Some(provider);
        }
        if let Some(ean) = ean_code {
            member.ean_code = Some(ean);
        }

        let updated = self.repo.update(&member).await?;
        Ok(IndividualMemberResponseDto::from(updated))
    }

    /// Withdraw from campaign (GDPR Art. 17)
    pub async fn withdraw(&self, campaign_id: Uuid, member_id: Uuid) -> Result<String, String> {
        let member = self
            .repo
            .find_by_id(member_id)
            .await?
            .ok_or_else(|| format!("Member {} not found", member_id))?;

        if member.campaign_id != campaign_id {
            return Err("Member does not belong to this campaign".to_string());
        }

        let email = member.email.clone();
        self.repo.withdraw_consent(member_id).await?;
        Ok(email)
    }
}
