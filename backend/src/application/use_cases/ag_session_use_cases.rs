use crate::application::dto::ag_session_dto::{
    AgSessionResponse, CombinedQuorumResponse, CreateAgSessionDto, EndAgSessionDto,
    RecordRemoteJoinDto,
};
use crate::application::ports::ag_session_repository::AgSessionRepository;
use crate::application::ports::meeting_repository::MeetingRepository;
use crate::domain::entities::ag_session::{AgSession, VideoPlatform};
use std::sync::Arc;
use uuid::Uuid;

pub struct AgSessionUseCases {
    pub ag_session_repo: Arc<dyn AgSessionRepository>,
    pub meeting_repo: Arc<dyn MeetingRepository>,
}

impl AgSessionUseCases {
    pub fn new(
        ag_session_repo: Arc<dyn AgSessionRepository>,
        meeting_repo: Arc<dyn MeetingRepository>,
    ) -> Self {
        Self {
            ag_session_repo,
            meeting_repo,
        }
    }

    /// Crée une session de visioconférence pour une AG (B15-3)
    pub async fn create_session(
        &self,
        organization_id: Uuid,
        dto: CreateAgSessionDto,
        created_by: Uuid,
    ) -> Result<AgSessionResponse, String> {
        // Vérifier que la réunion existe et appartient à l'organisation
        let meeting = self
            .meeting_repo
            .find_by_id(dto.meeting_id)
            .await?
            .ok_or_else(|| format!("Réunion {} introuvable", dto.meeting_id))?;

        if meeting.organization_id != organization_id {
            return Err(
                "Accès refusé : la réunion n'appartient pas à votre organisation".to_string(),
            );
        }

        // Vérifier qu'il n'y a pas déjà une session pour cette réunion
        if let Some(existing) = self
            .ag_session_repo
            .find_by_meeting_id(dto.meeting_id)
            .await?
        {
            return Err(format!(
                "Une session de visioconférence existe déjà pour cette réunion (id: {})",
                existing.id
            ));
        }

        let platform = VideoPlatform::from_db_string(&dto.platform)?;

        let session = AgSession::new(
            organization_id,
            dto.meeting_id,
            platform,
            dto.video_url,
            dto.host_url,
            dto.scheduled_start,
            dto.access_password,
            dto.waiting_room_enabled.unwrap_or(true),
            dto.recording_enabled.unwrap_or(false),
            created_by,
        )?;

        let created = self.ag_session_repo.create(&session).await?;
        Ok(AgSessionResponse::from(&created))
    }

    /// Récupère une session par ID
    pub async fn get_session(
        &self,
        id: Uuid,
        organization_id: Uuid,
    ) -> Result<AgSessionResponse, String> {
        let session = self
            .ag_session_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Session {} introuvable", id))?;

        if session.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }

        Ok(AgSessionResponse::from(&session))
    }

    /// Récupère la session associée à une réunion
    pub async fn get_session_for_meeting(
        &self,
        meeting_id: Uuid,
        organization_id: Uuid,
    ) -> Result<Option<AgSessionResponse>, String> {
        match self.ag_session_repo.find_by_meeting_id(meeting_id).await? {
            Some(session) if session.organization_id == organization_id => {
                Ok(Some(AgSessionResponse::from(&session)))
            }
            Some(_) => Err("Accès refusé".to_string()),
            None => Ok(None),
        }
    }

    /// Liste les sessions de l'organisation
    pub async fn list_sessions(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<AgSessionResponse>, String> {
        let sessions = self
            .ag_session_repo
            .find_by_organization(organization_id)
            .await?;
        Ok(sessions.iter().map(AgSessionResponse::from).collect())
    }

    /// Démarre une session (Scheduled → Live)
    pub async fn start_session(
        &self,
        id: Uuid,
        organization_id: Uuid,
    ) -> Result<AgSessionResponse, String> {
        let mut session = self
            .ag_session_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Session {} introuvable", id))?;

        if session.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }

        session.start()?;
        let updated = self.ag_session_repo.update(&session).await?;
        Ok(AgSessionResponse::from(&updated))
    }

    /// Termine une session (Live → Ended)
    pub async fn end_session(
        &self,
        id: Uuid,
        organization_id: Uuid,
        dto: EndAgSessionDto,
    ) -> Result<AgSessionResponse, String> {
        let mut session = self
            .ag_session_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Session {} introuvable", id))?;

        if session.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }

        session.end(dto.recording_url)?;
        let updated = self.ag_session_repo.update(&session).await?;
        Ok(AgSessionResponse::from(&updated))
    }

    /// Annule une session (Scheduled → Cancelled)
    pub async fn cancel_session(
        &self,
        id: Uuid,
        organization_id: Uuid,
    ) -> Result<AgSessionResponse, String> {
        let mut session = self
            .ag_session_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Session {} introuvable", id))?;

        if session.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }

        session.cancel()?;
        let updated = self.ag_session_repo.update(&session).await?;
        Ok(AgSessionResponse::from(&updated))
    }

    /// Enregistre un participant distant et recalcule le quorum distanciel
    pub async fn record_remote_join(
        &self,
        id: Uuid,
        organization_id: Uuid,
        dto: RecordRemoteJoinDto,
    ) -> Result<AgSessionResponse, String> {
        let mut session = self
            .ag_session_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Session {} introuvable", id))?;

        if session.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }

        session.record_remote_join(dto.voting_power, dto.total_building_quotas)?;
        let updated = self.ag_session_repo.update(&session).await?;
        Ok(AgSessionResponse::from(&updated))
    }

    /// Calcule le quorum combiné (présentiel + distanciel) — Art. 3.87 §5 CC
    pub async fn calculate_combined_quorum(
        &self,
        id: Uuid,
        organization_id: Uuid,
        physical_quotas: f64,
        total_building_quotas: f64,
    ) -> Result<CombinedQuorumResponse, String> {
        let session = self
            .ag_session_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Session {} introuvable", id))?;

        if session.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }

        let combined_pct =
            session.calculate_combined_quorum(physical_quotas, total_building_quotas)?;

        Ok(CombinedQuorumResponse {
            session_id: session.id,
            meeting_id: session.meeting_id,
            physical_quotas,
            remote_quotas: session.remote_voting_power,
            total_building_quotas,
            combined_percentage: combined_pct,
            quorum_reached: combined_pct > 50.0,
        })
    }

    /// Supprime une session (uniquement si Scheduled ou Cancelled)
    pub async fn delete_session(&self, id: Uuid, organization_id: Uuid) -> Result<(), String> {
        let session = self
            .ag_session_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Session {} introuvable", id))?;

        if session.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }

        use crate::domain::entities::ag_session::AgSessionStatus;
        if session.status == AgSessionStatus::Live {
            return Err("Impossible de supprimer une session en cours".to_string());
        }

        self.ag_session_repo.delete(id).await?;
        Ok(())
    }

    /// Liste les sessions en attente de démarrage (platform stats helper)
    pub async fn list_pending_sessions(&self) -> Result<Vec<AgSessionResponse>, String> {
        let sessions = self.ag_session_repo.find_pending_start().await?;
        Ok(sessions.iter().map(AgSessionResponse::from).collect())
    }
}
