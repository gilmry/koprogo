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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::ag_session_dto::{CreateAgSessionDto, RecordRemoteJoinDto};
    use crate::application::dto::PageRequest;
    use crate::application::ports::ag_session_repository::AgSessionRepository;
    use crate::application::ports::meeting_repository::MeetingRepository;
    use crate::domain::entities::ag_session::{AgSession, AgSessionStatus};
    use crate::domain::entities::meeting::{Meeting, MeetingType};
    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ========== Mock AgSessionRepository ==========

    struct MockAgSessionRepository {
        sessions: Mutex<HashMap<Uuid, AgSession>>,
    }

    impl MockAgSessionRepository {
        fn new() -> Self {
            Self {
                sessions: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl AgSessionRepository for MockAgSessionRepository {
        async fn create(&self, session: &AgSession) -> Result<AgSession, String> {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(session.id, session.clone());
            Ok(session.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<AgSession>, String> {
            let sessions = self.sessions.lock().unwrap();
            Ok(sessions.get(&id).cloned())
        }

        async fn find_by_meeting_id(
            &self,
            meeting_id: Uuid,
        ) -> Result<Option<AgSession>, String> {
            let sessions = self.sessions.lock().unwrap();
            Ok(sessions
                .values()
                .find(|s| s.meeting_id == meeting_id)
                .cloned())
        }

        async fn find_by_organization(
            &self,
            organization_id: Uuid,
        ) -> Result<Vec<AgSession>, String> {
            let sessions = self.sessions.lock().unwrap();
            Ok(sessions
                .values()
                .filter(|s| s.organization_id == organization_id)
                .cloned()
                .collect())
        }

        async fn update(&self, session: &AgSession) -> Result<AgSession, String> {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(session.id, session.clone());
            Ok(session.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            let mut sessions = self.sessions.lock().unwrap();
            Ok(sessions.remove(&id).is_some())
        }

        async fn find_pending_start(&self) -> Result<Vec<AgSession>, String> {
            let sessions = self.sessions.lock().unwrap();
            Ok(sessions
                .values()
                .filter(|s| s.status == AgSessionStatus::Scheduled)
                .cloned()
                .collect())
        }
    }

    // ========== Mock MeetingRepository ==========

    struct MockMeetingRepository {
        meetings: Mutex<HashMap<Uuid, Meeting>>,
    }

    impl MockMeetingRepository {
        fn new() -> Self {
            Self {
                meetings: Mutex::new(HashMap::new()),
            }
        }

        fn with_meeting(meeting: Meeting) -> Self {
            let mut map = HashMap::new();
            map.insert(meeting.id, meeting);
            Self {
                meetings: Mutex::new(map),
            }
        }
    }

    #[async_trait]
    impl MeetingRepository for MockMeetingRepository {
        async fn create(&self, meeting: &Meeting) -> Result<Meeting, String> {
            let mut meetings = self.meetings.lock().unwrap();
            meetings.insert(meeting.id, meeting.clone());
            Ok(meeting.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Meeting>, String> {
            let meetings = self.meetings.lock().unwrap();
            Ok(meetings.get(&id).cloned())
        }

        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Meeting>, String> {
            let meetings = self.meetings.lock().unwrap();
            Ok(meetings
                .values()
                .filter(|m| m.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn update(&self, meeting: &Meeting) -> Result<Meeting, String> {
            let mut meetings = self.meetings.lock().unwrap();
            meetings.insert(meeting.id, meeting.clone());
            Ok(meeting.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            let mut meetings = self.meetings.lock().unwrap();
            Ok(meetings.remove(&id).is_some())
        }

        async fn find_all_paginated(
            &self,
            _page_request: &PageRequest,
            _organization_id: Option<Uuid>,
        ) -> Result<(Vec<Meeting>, i64), String> {
            let meetings = self.meetings.lock().unwrap();
            let all: Vec<Meeting> = meetings.values().cloned().collect();
            let count = all.len() as i64;
            Ok((all, count))
        }
    }

    // ========== Helpers ==========

    fn make_meeting(org_id: Uuid) -> Meeting {
        let future_date = Utc::now() + Duration::days(30);
        Meeting::new(
            org_id,
            Uuid::new_v4(),
            MeetingType::Ordinary,
            "AGO 2026".to_string(),
            Some("Assemblée générale ordinaire".to_string()),
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap()
    }

    fn make_create_dto(meeting_id: Uuid) -> CreateAgSessionDto {
        CreateAgSessionDto {
            meeting_id,
            platform: "jitsi".to_string(),
            video_url: "https://meet.jit.si/koprogo-ago-2026".to_string(),
            host_url: None,
            scheduled_start: Utc::now() + Duration::hours(2),
            access_password: None,
            waiting_room_enabled: Some(true),
            recording_enabled: Some(false),
        }
    }

    fn make_use_cases(
        ag_repo: MockAgSessionRepository,
        meeting_repo: MockMeetingRepository,
    ) -> AgSessionUseCases {
        AgSessionUseCases::new(Arc::new(ag_repo), Arc::new(meeting_repo))
    }

    /// Helper: insert a Scheduled session into the mock repo and return its ID + org_id
    fn insert_scheduled_session(
        ag_repo: &MockAgSessionRepository,
        org_id: Uuid,
        meeting_id: Uuid,
    ) -> Uuid {
        let future = Utc::now() + Duration::hours(2);
        let session = AgSession::new(
            org_id,
            meeting_id,
            VideoPlatform::Jitsi,
            "https://meet.jit.si/koprogo-test".to_string(),
            None,
            future,
            None,
            true,
            false,
            Uuid::new_v4(),
        )
        .unwrap();
        let session_id = session.id;
        ag_repo.sessions.lock().unwrap().insert(session_id, session);
        session_id
    }

    // ========== Tests ==========

    #[tokio::test]
    async fn test_create_session_success() {
        let org_id = Uuid::new_v4();
        let meeting = make_meeting(org_id);
        let meeting_id = meeting.id;

        let ag_repo = MockAgSessionRepository::new();
        let meeting_repo = MockMeetingRepository::with_meeting(meeting);
        let uc = make_use_cases(ag_repo, meeting_repo);
        let created_by = Uuid::new_v4();

        let dto = make_create_dto(meeting_id);
        let result = uc.create_session(org_id, dto, created_by).await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.organization_id, org_id);
        assert_eq!(resp.meeting_id, meeting_id);
        assert_eq!(resp.platform, "jitsi");
        assert_eq!(resp.status, "scheduled");
        assert_eq!(resp.remote_attendees_count, 0);
        assert!(resp.waiting_room_enabled);
        assert!(!resp.recording_enabled);
        assert_eq!(resp.created_by, created_by);
    }

    #[tokio::test]
    async fn test_create_session_fail_meeting_not_found() {
        let org_id = Uuid::new_v4();
        let fake_meeting_id = Uuid::new_v4();

        let ag_repo = MockAgSessionRepository::new();
        let meeting_repo = MockMeetingRepository::new(); // Empty, no meetings
        let uc = make_use_cases(ag_repo, meeting_repo);
        let created_by = Uuid::new_v4();

        let dto = make_create_dto(fake_meeting_id);
        let result = uc.create_session(org_id, dto, created_by).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("introuvable"));
    }

    #[tokio::test]
    async fn test_create_session_fail_wrong_organization() {
        let org_id_a = Uuid::new_v4();
        let org_id_b = Uuid::new_v4();
        // Meeting belongs to org_id_a
        let meeting = make_meeting(org_id_a);
        let meeting_id = meeting.id;

        let ag_repo = MockAgSessionRepository::new();
        let meeting_repo = MockMeetingRepository::with_meeting(meeting);
        let uc = make_use_cases(ag_repo, meeting_repo);
        let created_by = Uuid::new_v4();

        // Try to create session with org_id_b
        let dto = make_create_dto(meeting_id);
        let result = uc.create_session(org_id_b, dto, created_by).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("n'appartient pas à votre organisation"));
    }

    #[tokio::test]
    async fn test_start_session_success() {
        let org_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let ag_repo = MockAgSessionRepository::new();
        let session_id = insert_scheduled_session(&ag_repo, org_id, meeting_id);
        let meeting_repo = MockMeetingRepository::new();
        let uc = make_use_cases(ag_repo, meeting_repo);

        let result = uc.start_session(session_id, org_id).await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.status, "live");
        assert!(resp.actual_start.is_some());
    }

    #[tokio::test]
    async fn test_cancel_session_success() {
        let org_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let ag_repo = MockAgSessionRepository::new();
        let session_id = insert_scheduled_session(&ag_repo, org_id, meeting_id);
        let meeting_repo = MockMeetingRepository::new();
        let uc = make_use_cases(ag_repo, meeting_repo);

        let result = uc.cancel_session(session_id, org_id).await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.status, "cancelled");
    }

    #[tokio::test]
    async fn test_delete_session_fail_live_session() {
        let org_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let ag_repo = MockAgSessionRepository::new();
        let session_id = insert_scheduled_session(&ag_repo, org_id, meeting_id);

        // Start the session to make it Live
        {
            let mut sessions = ag_repo.sessions.lock().unwrap();
            let session = sessions.get_mut(&session_id).unwrap();
            session.start().unwrap();
        }

        let meeting_repo = MockMeetingRepository::new();
        let uc = make_use_cases(ag_repo, meeting_repo);

        let result = uc.delete_session(session_id, org_id).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Impossible de supprimer une session en cours"));
    }

    #[tokio::test]
    async fn test_record_remote_join_success() {
        let org_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let ag_repo = MockAgSessionRepository::new();
        let session_id = insert_scheduled_session(&ag_repo, org_id, meeting_id);

        // Start the session first (record_remote_join requires Live status)
        {
            let mut sessions = ag_repo.sessions.lock().unwrap();
            let session = sessions.get_mut(&session_id).unwrap();
            session.start().unwrap();
        }

        let meeting_repo = MockMeetingRepository::new();
        let uc = make_use_cases(ag_repo, meeting_repo);

        let dto = RecordRemoteJoinDto {
            voting_power: 150.0,
            total_building_quotas: 1000.0,
        };

        let result = uc.record_remote_join(session_id, org_id, dto).await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.remote_attendees_count, 1);
        assert!((resp.remote_voting_power - 150.0).abs() < 0.01);
        assert!((resp.quorum_remote_contribution - 15.0).abs() < 0.01);
    }
}
