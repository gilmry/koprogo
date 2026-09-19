//! Use cases du conseil de copropriété — Story 4.7 (#582).
//!
//! `create_alert` : seul un membre dont le mandat court peut alerter la
//! prochaine assemblée générale (Art. 3.90 §1er CC — la mission de contrôle
//! du conseil a besoin d'un instrument, pas seulement d'un titre).
//!
//! `elect_members` : élire le conseil suppose une AG qui a abouti. La preuve
//! du quorum double (Art. 3.87 §5 CC) est déjà portée par
//! `Meeting::assert_can_complete` au moment de la clôture — ce use case ne la
//! recalcule pas, il exige simplement que l'AG référencée soit `Completed`.

use crate::application::dto::{
    BoardAlertResponseDto, BoardMemberResponseDto, CreateBoardAlertDto, ElectCdcMembersDto,
};
use crate::application::error::AppError;
use crate::application::ports::{BoardAlertRepository, BoardMemberRepository, MeetingRepository};
use crate::domain::entities::{
    AlertSeverity, BoardAlert, BoardMember, BoardPosition, MeetingStatus,
};
use std::sync::Arc;
use uuid::Uuid;

pub struct CdcUseCases {
    alert_repository: Arc<dyn BoardAlertRepository>,
    board_member_repository: Arc<dyn BoardMemberRepository>,
    meeting_repository: Arc<dyn MeetingRepository>,
}

impl CdcUseCases {
    pub fn new(
        alert_repository: Arc<dyn BoardAlertRepository>,
        board_member_repository: Arc<dyn BoardMemberRepository>,
        meeting_repository: Arc<dyn MeetingRepository>,
    ) -> Self {
        Self {
            alert_repository,
            board_member_repository,
            meeting_repository,
        }
    }

    /// Émet une alerte du conseil à destination de la prochaine AG.
    ///
    /// `dto.target_meeting_id` est fourni par l'appelant (typiquement la
    /// prochaine AGO déjà planifiée pour l'immeuble) plutôt que résolu
    /// automatiquement : le dépôt n'a aujourd'hui aucune notion de « la
    /// prochaine AG » côté `MeetingRepository` (une seule requête ad hoc,
    /// fenêtrée à 7 jours, existe pour le dashboard — non généralisable).
    /// Ce use case valide en revanche que la cible est bien une AG à venir
    /// du même immeuble, ce qui est la substance de `target=AG_next`.
    pub async fn create_alert(
        &self,
        caller_owner_id: Uuid,
        building_id: Uuid,
        dto: CreateBoardAlertDto,
    ) -> Result<BoardAlertResponseDto, AppError> {
        let target_meeting_id = Uuid::parse_str(&dto.target_meeting_id)
            .map_err(|_| AppError::Validation("Invalid target_meeting_id format".to_string()))?;
        let severity: AlertSeverity = dto.severity.parse().map_err(AppError::Validation)?;

        // Autorisation : seul un membre du conseil dont le mandat court sur
        // cet immeuble peut alerter (Story 4.7 @security).
        let member = self
            .board_member_repository
            .find_by_owner_and_building(caller_owner_id, building_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| {
                AppError::Forbidden(
                    "Vous n'êtes pas membre du conseil de copropriété de cet immeuble".to_string(),
                )
            })?;
        if !member.is_active() {
            return Err(AppError::Forbidden(
                "Votre mandat au conseil de copropriété a pris fin".to_string(),
            ));
        }

        let meeting = self
            .meeting_repository
            .find_by_id(target_meeting_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound("Meeting not found".to_string()))?;
        if meeting.building_id != building_id {
            return Err(AppError::Forbidden(
                "Cette assemblée n'appartient pas à cet immeuble".to_string(),
            ));
        }
        if meeting.status != MeetingStatus::Scheduled {
            return Err(AppError::Validation(
                "La cible d'une alerte doit être une assemblée générale à venir".to_string(),
            ));
        }

        let alert = BoardAlert::new(building_id, member.id, dto.text, severity, meeting.id)?;
        let created = self.alert_repository.create(&alert).await?;

        Ok(to_alert_dto(&created))
    }

    /// Liste les alertes visibles à une AG donnée.
    pub async fn list_alerts_for_meeting(
        &self,
        meeting_id: Uuid,
    ) -> Result<Vec<BoardAlertResponseDto>, AppError> {
        let alerts = self
            .alert_repository
            .find_by_target_meeting(meeting_id)
            .await?;
        Ok(alerts.iter().map(to_alert_dto).collect())
    }

    /// Élit les membres du conseil à l'issue d'une AG clôturée.
    ///
    /// L'AG doit être `Completed` : sa clôture suppose déjà le quorum double
    /// vérifié (Art. 3.87 §5 CC, `Meeting::assert_can_complete`). Une AG
    /// encore `Scheduled` ou `Cancelled` n'a jamais prouvé son quorum — 422
    /// (Story 4.7 @negative).
    pub async fn elect_members(
        &self,
        building_id: Uuid,
        dto: ElectCdcMembersDto,
    ) -> Result<Vec<BoardMemberResponseDto>, AppError> {
        let meeting_id = Uuid::parse_str(&dto.meeting_id)
            .map_err(|_| AppError::Validation("Invalid meeting_id format".to_string()))?;

        let meeting = self
            .meeting_repository
            .find_by_id(meeting_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound("Meeting not found".to_string()))?;
        if meeting.building_id != building_id {
            return Err(AppError::Validation(
                "Cette assemblée n'appartient pas à cet immeuble".to_string(),
            ));
        }
        if meeting.status != MeetingStatus::Completed {
            return Err(AppError::CdcElectionQuorumNotReached { meeting_id });
        }

        let mandate_start = chrono::Utc::now();
        let mandate_end = mandate_start + chrono::Duration::days(365);

        let mut elected = Vec::with_capacity(dto.candidates.len());
        for candidate in &dto.candidates {
            let owner_id = Uuid::parse_str(&candidate.owner_id)
                .map_err(|_| AppError::Validation("Invalid owner_id format".to_string()))?;
            let position: BoardPosition =
                candidate.position.parse().map_err(AppError::Validation)?;

            let member = BoardMember::new(
                owner_id,
                building_id,
                position,
                mandate_start,
                mandate_end,
                meeting_id,
            )
            .map_err(AppError::Validation)?;

            let created = self
                .board_member_repository
                .create(&member)
                .await
                .map_err(AppError::from)?;
            elected.push(to_member_dto(&created));
        }

        Ok(elected)
    }
}

fn to_alert_dto(alert: &BoardAlert) -> BoardAlertResponseDto {
    BoardAlertResponseDto {
        id: alert.id.to_string(),
        building_id: alert.building_id.to_string(),
        raised_by_board_member_id: alert.raised_by_board_member_id.to_string(),
        text: alert.text.clone(),
        severity: alert.severity.to_string(),
        target_meeting_id: alert.target_meeting_id.to_string(),
        created_at: alert.created_at.to_rfc3339(),
    }
}

fn to_member_dto(member: &BoardMember) -> BoardMemberResponseDto {
    BoardMemberResponseDto {
        id: member.id.to_string(),
        owner_id: member.owner_id.to_string(),
        building_id: member.building_id.to_string(),
        position: member.position.to_string(),
        mandate_start: member.mandate_start.to_rfc3339(),
        mandate_end: member.mandate_end.to_rfc3339(),
        elected_by_meeting_id: member.elected_by_meeting_id.to_string(),
        is_active: member.is_active(),
        days_remaining: member.days_remaining(),
        expires_soon: member.expires_soon(),
        created_at: member.created_at.to_rfc3339(),
        updated_at: member.updated_at.to_rfc3339(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{Meeting, MeetingType};
    use chrono::{Duration, Utc};
    use mockall::mock;
    use mockall::predicate::*;

    mock! {
        pub AlertRepo {}

        #[async_trait::async_trait]
        impl BoardAlertRepository for AlertRepo {
            async fn create(&self, alert: &BoardAlert) -> Result<BoardAlert, AppError>;
            async fn find_by_target_meeting(&self, meeting_id: Uuid) -> Result<Vec<BoardAlert>, AppError>;
        }
    }

    mock! {
        pub MemberRepo {}

        #[async_trait::async_trait]
        impl BoardMemberRepository for MemberRepo {
            async fn create(&self, board_member: &BoardMember) -> Result<BoardMember, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<BoardMember>, String>;
            async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<BoardMember>, String>;
            async fn find_active_by_building(&self, building_id: Uuid) -> Result<Vec<BoardMember>, String>;
            async fn find_expiring_soon(&self, building_id: Uuid, days_threshold: i32) -> Result<Vec<BoardMember>, String>;
            async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<BoardMember>, String>;
            async fn find_by_owner_and_building(&self, owner_id: Uuid, building_id: Uuid) -> Result<Option<BoardMember>, String>;
            async fn has_active_mandate(&self, owner_id: Uuid, building_id: Uuid) -> Result<bool, String>;
            async fn update(&self, board_member: &BoardMember) -> Result<BoardMember, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
            async fn count_active_by_building(&self, building_id: Uuid) -> Result<i64, String>;
        }
    }

    mock! {
        pub MeetingRepo {}

        #[async_trait::async_trait]
        impl MeetingRepository for MeetingRepo {
            async fn create(&self, meeting: &Meeting) -> Result<Meeting, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Meeting>, String>;
            async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Meeting>, String>;
            async fn update(&self, meeting: &Meeting) -> Result<Meeting, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
            async fn find_all_paginated(
                &self,
                page_request: &crate::application::dto::PageRequest,
                organization_id: Option<Uuid>,
            ) -> Result<(Vec<Meeting>, i64), String>;
        }
    }

    fn make_meeting(building_id: Uuid, status: MeetingStatus) -> Meeting {
        let mut meeting = Meeting::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            building_id,
            MeetingType::Ordinary,
            "AGO Story 4.7".to_string(),
            None,
            Utc::now() + Duration::days(30),
            "Salle des fêtes".to_string(),
        )
        .unwrap();
        meeting.status = status;
        meeting
    }

    fn make_active_member(owner_id: Uuid, building_id: Uuid) -> BoardMember {
        BoardMember::new(
            owner_id,
            building_id,
            BoardPosition::Member,
            Utc::now() - Duration::days(10),
            Utc::now() + Duration::days(355),
            Uuid::new_v4(),
        )
        .unwrap()
    }

    fn alert_dto(meeting_id: Uuid) -> CreateBoardAlertDto {
        CreateBoardAlertDto {
            text: "L'ascenseur est en panne depuis trois jours".to_string(),
            severity: "warning".to_string(),
            target_meeting_id: meeting_id.to_string(),
        }
    }

    // ── @happy ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn happy_active_board_member_can_create_alert() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting = make_meeting(building_id, MeetingStatus::Scheduled);
        let meeting_id = meeting.id;
        let member = make_active_member(owner_id, building_id);

        let mut member_repo = MockMemberRepo::new();
        member_repo
            .expect_find_by_owner_and_building()
            .with(eq(owner_id), eq(building_id))
            .times(1)
            .return_once(move |_, _| Ok(Some(member)));

        let mut meeting_repo = MockMeetingRepo::new();
        meeting_repo
            .expect_find_by_id()
            .with(eq(meeting_id))
            .times(1)
            .return_once(move |_| Ok(Some(meeting)));

        let mut alert_repo = MockAlertRepo::new();
        alert_repo
            .expect_create()
            .times(1)
            .returning(|alert| Ok(alert.clone()));

        let uc = CdcUseCases::new(
            Arc::new(alert_repo),
            Arc::new(member_repo),
            Arc::new(meeting_repo),
        );

        let result = uc
            .create_alert(owner_id, building_id, alert_dto(meeting_id))
            .await;

        assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
        let dto = result.unwrap();
        assert_eq!(dto.severity, "warning");
        assert_eq!(dto.target_meeting_id, meeting_id.to_string());
    }

    #[tokio::test]
    async fn happy_elect_three_members_from_completed_meeting() {
        let building_id = Uuid::new_v4();
        let meeting = make_meeting(building_id, MeetingStatus::Completed);
        let meeting_id = meeting.id;

        let mut meeting_repo = MockMeetingRepo::new();
        meeting_repo
            .expect_find_by_id()
            .with(eq(meeting_id))
            .times(1)
            .return_once(move |_| Ok(Some(meeting)));

        let mut member_repo = MockMemberRepo::new();
        member_repo
            .expect_create()
            .times(3)
            .returning(|m| Ok(m.clone()));

        let uc = CdcUseCases::new(
            Arc::new(MockAlertRepo::new()),
            Arc::new(member_repo),
            Arc::new(meeting_repo),
        );

        let dto = ElectCdcMembersDto {
            meeting_id: meeting_id.to_string(),
            candidates: vec![
                crate::application::dto::CdcCandidateDto {
                    owner_id: Uuid::new_v4().to_string(),
                    position: "president".to_string(),
                },
                crate::application::dto::CdcCandidateDto {
                    owner_id: Uuid::new_v4().to_string(),
                    position: "treasurer".to_string(),
                },
                crate::application::dto::CdcCandidateDto {
                    owner_id: Uuid::new_v4().to_string(),
                    position: "member".to_string(),
                },
            ],
        };

        let result = uc.elect_members(building_id, dto).await;
        assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
        assert_eq!(result.unwrap().len(), 3);
    }

    // ── @edge ────────────────────────────────────────────────────────

    /// Un membre qui vient tout juste de démissionner (mandat échu à
    /// l'instant) perd ses droits immédiatement — pas seulement après un
    /// délai (Story 4.7 @edge, cf. `board_member.rs::edge_resign_*`).
    #[tokio::test]
    async fn edge_member_resigned_a_moment_ago_is_forbidden() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting = make_meeting(building_id, MeetingStatus::Scheduled);
        let meeting_id = meeting.id;

        let mut member = make_active_member(owner_id, building_id);
        member.resign(Utc::now() - Duration::milliseconds(1));

        let mut member_repo = MockMemberRepo::new();
        member_repo
            .expect_find_by_owner_and_building()
            .with(eq(owner_id), eq(building_id))
            .times(1)
            .return_once(move |_, _| Ok(Some(member)));

        let uc = CdcUseCases::new(
            Arc::new(MockAlertRepo::new()),
            Arc::new(member_repo),
            Arc::new(MockMeetingRepo::new()),
        );

        let result = uc
            .create_alert(owner_id, building_id, alert_dto(meeting_id))
            .await;

        match result {
            Err(AppError::Forbidden(_)) => {}
            other => panic!("expected Forbidden, got {:?}", other),
        }
    }

    // ── @security ────────────────────────────────────────────────────

    #[tokio::test]
    async fn security_non_elected_owner_cannot_create_alert() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let mut member_repo = MockMemberRepo::new();
        member_repo
            .expect_find_by_owner_and_building()
            .with(eq(owner_id), eq(building_id))
            .times(1)
            .return_once(|_, _| Ok(None));

        let uc = CdcUseCases::new(
            Arc::new(MockAlertRepo::new()),
            Arc::new(member_repo),
            Arc::new(MockMeetingRepo::new()),
        );

        let result = uc
            .create_alert(owner_id, building_id, alert_dto(meeting_id))
            .await;

        match result {
            Err(AppError::Forbidden(_)) => {}
            other => panic!("expected Forbidden, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn security_former_member_past_mandate_end_cannot_create_alert() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let expired_member = BoardMember::new(
            owner_id,
            building_id,
            BoardPosition::Member,
            Utc::now() - Duration::days(400),
            Utc::now() - Duration::days(35), // mandat déjà échu
            Uuid::new_v4(),
        )
        .unwrap();

        let mut member_repo = MockMemberRepo::new();
        member_repo
            .expect_find_by_owner_and_building()
            .with(eq(owner_id), eq(building_id))
            .times(1)
            .return_once(move |_, _| Ok(Some(expired_member)));

        let uc = CdcUseCases::new(
            Arc::new(MockAlertRepo::new()),
            Arc::new(member_repo),
            Arc::new(MockMeetingRepo::new()),
        );

        let result = uc
            .create_alert(owner_id, building_id, alert_dto(meeting_id))
            .await;

        match result {
            Err(AppError::Forbidden(_)) => {}
            other => panic!("expected Forbidden, got {:?}", other),
        }
    }

    // ── @negative ────────────────────────────────────────────────────

    #[tokio::test]
    async fn negative_election_without_quorum_returns_422() {
        let building_id = Uuid::new_v4();
        // AG encore Scheduled : jamais clôturée, donc quorum jamais prouvé.
        let meeting = make_meeting(building_id, MeetingStatus::Scheduled);
        let meeting_id = meeting.id;

        let mut meeting_repo = MockMeetingRepo::new();
        meeting_repo
            .expect_find_by_id()
            .with(eq(meeting_id))
            .times(1)
            .return_once(move |_| Ok(Some(meeting)));

        let uc = CdcUseCases::new(
            Arc::new(MockAlertRepo::new()),
            Arc::new(MockMemberRepo::new()),
            Arc::new(meeting_repo),
        );

        let dto = ElectCdcMembersDto {
            meeting_id: meeting_id.to_string(),
            candidates: vec![crate::application::dto::CdcCandidateDto {
                owner_id: Uuid::new_v4().to_string(),
                position: "president".to_string(),
            }],
        };

        let result = uc.elect_members(building_id, dto).await;
        match result {
            Err(AppError::CdcElectionQuorumNotReached { meeting_id: m }) => {
                assert_eq!(m, meeting_id)
            }
            other => panic!("expected CdcElectionQuorumNotReached, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn negative_create_alert_with_unknown_meeting_returns_not_found() {
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();
        let member = make_active_member(owner_id, building_id);

        let mut member_repo = MockMemberRepo::new();
        member_repo
            .expect_find_by_owner_and_building()
            .with(eq(owner_id), eq(building_id))
            .times(1)
            .return_once(move |_, _| Ok(Some(member)));

        let mut meeting_repo = MockMeetingRepo::new();
        meeting_repo
            .expect_find_by_id()
            .with(eq(meeting_id))
            .times(1)
            .return_once(|_| Ok(None));

        let uc = CdcUseCases::new(
            Arc::new(MockAlertRepo::new()),
            Arc::new(member_repo),
            Arc::new(meeting_repo),
        );

        let result = uc
            .create_alert(owner_id, building_id, alert_dto(meeting_id))
            .await;

        match result {
            Err(AppError::NotFound(_)) => {}
            other => panic!("expected NotFound, got {:?}", other),
        }
    }
}
