use crate::application::dto::{
    AddDecisionNotesDto, BoardDecisionResponseDto, CreateBoardDecisionDto, DecisionStatsDto,
    UpdateBoardDecisionDto,
};
use crate::application::ports::{BoardDecisionRepository, BuildingRepository, MeetingRepository};
use crate::domain::entities::{BoardDecision, DecisionStatus};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

/// Use cases pour la gestion des décisions du conseil de copropriété
pub struct BoardDecisionUseCases {
    decision_repository: Arc<dyn BoardDecisionRepository>,
    building_repository: Arc<dyn BuildingRepository>,
    meeting_repository: Arc<dyn MeetingRepository>,
}

impl BoardDecisionUseCases {
    pub fn new(
        decision_repository: Arc<dyn BoardDecisionRepository>,
        building_repository: Arc<dyn BuildingRepository>,
        meeting_repository: Arc<dyn MeetingRepository>,
    ) -> Self {
        Self {
            decision_repository,
            building_repository,
            meeting_repository,
        }
    }

    /// Crée une nouvelle décision à suivre suite à une AG
    pub async fn create_decision(
        &self,
        dto: CreateBoardDecisionDto,
    ) -> Result<BoardDecisionResponseDto, String> {
        // Valider que l'immeuble existe
        let building_id = Uuid::parse_str(&dto.building_id)
            .map_err(|_| "Invalid building ID format".to_string())?;

        self.building_repository
            .find_by_id(building_id)
            .await?
            .ok_or_else(|| "Building not found".to_string())?;

        // Valider que la réunion existe
        let meeting_id = Uuid::parse_str(&dto.meeting_id)
            .map_err(|_| "Invalid meeting ID format".to_string())?;

        self.meeting_repository
            .find_by_id(meeting_id)
            .await?
            .ok_or_else(|| "Meeting not found".to_string())?;

        // Parser la deadline optionnelle
        let deadline = if let Some(deadline_str) = &dto.deadline {
            Some(
                DateTime::parse_from_rfc3339(deadline_str)
                    .map_err(|_| "Invalid deadline format".to_string())?
                    .with_timezone(&Utc),
            )
        } else {
            None
        };

        // Créer l'entité BoardDecision
        let decision = BoardDecision::new(
            building_id,
            meeting_id,
            dto.subject,
            dto.decision_text,
            deadline,
        )?;

        // Persister
        let created = self.decision_repository.create(&decision).await?;

        Ok(Self::to_response_dto(created))
    }

    /// Récupère une décision par ID
    pub async fn get_decision(&self, id: Uuid) -> Result<BoardDecisionResponseDto, String> {
        let decision = self
            .decision_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Decision not found".to_string())?;

        Ok(Self::to_response_dto(decision))
    }

    /// Liste toutes les décisions d'un immeuble
    pub async fn list_decisions_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<BoardDecisionResponseDto>, String> {
        let decisions = self
            .decision_repository
            .find_by_building(building_id)
            .await?;

        Ok(decisions.into_iter().map(Self::to_response_dto).collect())
    }

    /// Liste les décisions avec un statut donné
    pub async fn list_decisions_by_status(
        &self,
        building_id: Uuid,
        status: &str,
    ) -> Result<Vec<BoardDecisionResponseDto>, String> {
        let status_enum = status
            .parse::<DecisionStatus>()
            .map_err(|e| e.to_string())?;

        let decisions = self
            .decision_repository
            .find_by_status(building_id, status_enum)
            .await?;

        Ok(decisions.into_iter().map(Self::to_response_dto).collect())
    }

    /// Liste les décisions en retard
    pub async fn list_overdue_decisions(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<BoardDecisionResponseDto>, String> {
        let decisions = self.decision_repository.find_overdue(building_id).await?;

        Ok(decisions.into_iter().map(Self::to_response_dto).collect())
    }

    /// Met à jour le statut d'une décision
    pub async fn update_decision_status(
        &self,
        id: Uuid,
        dto: UpdateBoardDecisionDto,
    ) -> Result<BoardDecisionResponseDto, String> {
        let mut decision = self
            .decision_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Decision not found".to_string())?;

        // Parser et mettre à jour le statut
        let new_status = dto
            .status
            .parse::<DecisionStatus>()
            .map_err(|e| e.to_string())?;
        decision.update_status(new_status)?;

        // Mettre à jour les notes si fournies
        if let Some(notes) = dto.notes {
            decision.add_notes(notes);
        }

        // Vérifier si la décision est en retard
        decision.check_and_update_overdue_status();

        // Persister les changements
        let updated = self.decision_repository.update(&decision).await?;

        Ok(Self::to_response_dto(updated))
    }

    /// Ajoute des notes à une décision
    pub async fn add_notes(
        &self,
        id: Uuid,
        dto: AddDecisionNotesDto,
    ) -> Result<BoardDecisionResponseDto, String> {
        let mut decision = self
            .decision_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Decision not found".to_string())?;

        decision.add_notes(dto.notes);

        let updated = self.decision_repository.update(&decision).await?;

        Ok(Self::to_response_dto(updated))
    }

    /// Marque une décision comme complétée
    pub async fn complete_decision(&self, id: Uuid) -> Result<BoardDecisionResponseDto, String> {
        let mut decision = self
            .decision_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Decision not found".to_string())?;

        decision.update_status(DecisionStatus::Completed)?;

        let updated = self.decision_repository.update(&decision).await?;

        Ok(Self::to_response_dto(updated))
    }

    /// Obtient des statistiques sur les décisions d'un immeuble
    pub async fn get_decision_stats(&self, building_id: Uuid) -> Result<DecisionStatsDto, String> {
        let pending = self
            .decision_repository
            .count_by_status(building_id, DecisionStatus::Pending)
            .await?;

        let in_progress = self
            .decision_repository
            .count_by_status(building_id, DecisionStatus::InProgress)
            .await?;

        let completed = self
            .decision_repository
            .count_by_status(building_id, DecisionStatus::Completed)
            .await?;

        let overdue = self.decision_repository.count_overdue(building_id).await?;

        let cancelled = self
            .decision_repository
            .count_by_status(building_id, DecisionStatus::Cancelled)
            .await?;

        let total = pending + in_progress + completed + overdue + cancelled;

        Ok(DecisionStatsDto {
            building_id: building_id.to_string(),
            total_decisions: total,
            pending,
            in_progress,
            completed,
            overdue,
            cancelled,
        })
    }

    /// Convertit une entité BoardDecision en DTO de réponse
    fn to_response_dto(decision: BoardDecision) -> BoardDecisionResponseDto {
        let days_until_deadline = decision.deadline.map(|deadline| {
            let now = Utc::now();
            (deadline - now).num_days()
        });

        BoardDecisionResponseDto {
            id: decision.id.to_string(),
            building_id: decision.building_id.to_string(),
            meeting_id: decision.meeting_id.to_string(),
            subject: decision.subject.clone(),
            decision_text: decision.decision_text.clone(),
            deadline: decision.deadline.map(|d| d.to_rfc3339()),
            status: decision.status.to_string(),
            completed_at: decision.completed_at.map(|d| d.to_rfc3339()),
            notes: decision.notes.clone(),
            is_overdue: decision.is_overdue(),
            days_until_deadline,
            created_at: decision.created_at.to_rfc3339(),
            updated_at: decision.updated_at.to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{
        BoardDecisionRepository, BuildingRepository, MeetingRepository,
    };
    use crate::domain::entities::{Building, Meeting};
    use mockall::mock;
    use mockall::predicate::*;

    // Mock du repository BoardDecision
    mock! {
        pub DecisionRepository {}

        #[async_trait::async_trait]
        impl BoardDecisionRepository for DecisionRepository {
            async fn create(&self, decision: &BoardDecision) -> Result<BoardDecision, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<BoardDecision>, String>;
            async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<BoardDecision>, String>;
            async fn find_by_meeting(&self, meeting_id: Uuid) -> Result<Vec<BoardDecision>, String>;
            async fn find_by_status(&self, building_id: Uuid, status: DecisionStatus) -> Result<Vec<BoardDecision>, String>;
            async fn find_overdue(&self, building_id: Uuid) -> Result<Vec<BoardDecision>, String>;
            async fn find_deadline_approaching(&self, building_id: Uuid, days_threshold: i32) -> Result<Vec<BoardDecision>, String>;
            async fn update(&self, decision: &BoardDecision) -> Result<BoardDecision, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
            async fn count_by_status(&self, building_id: Uuid, status: DecisionStatus) -> Result<i64, String>;
            async fn count_overdue(&self, building_id: Uuid) -> Result<i64, String>;
        }
    }

    // Mock du repository Building
    mock! {
        pub BuildingRepo {}

        #[async_trait::async_trait]
        impl BuildingRepository for BuildingRepo {
            async fn create(&self, building: &Building) -> Result<Building, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Building>, String>;
            async fn find_all(&self) -> Result<Vec<Building>, String>;
            async fn find_all_paginated(
                &self,
                page_request: &crate::application::dto::PageRequest,
                filters: &crate::application::dto::BuildingFilters,
            ) -> Result<(Vec<Building>, i64), String>;
            async fn update(&self, building: &Building) -> Result<Building, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
        }
    }

    // Mock du repository Meeting
    mock! {
        pub MeetingRepo {}

        #[async_trait::async_trait]
        impl MeetingRepository for MeetingRepo {
            async fn create(&self, meeting: &Meeting) -> Result<Meeting, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Meeting>, String>;
            async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Meeting>, String>;
            async fn find_all_paginated(
                &self,
                page_request: &crate::application::dto::PageRequest,
                organization_id: Option<Uuid>,
            ) -> Result<(Vec<Meeting>, i64), String>;
            async fn update(&self, meeting: &Meeting) -> Result<Meeting, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
        }
    }

    #[tokio::test]
    async fn test_create_decision_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let mut decision_repo = MockDecisionRepository::new();
        let mut building_repo = MockBuildingRepo::new();
        let mut meeting_repo = MockMeetingRepo::new();

        // Mock building exists
        let building = Building::new(
            org_id,
            "Test Building".to_string(),
            "123 Main St".to_string(),
            "Brussels".to_string(),
            "1000".to_string(),
            "Belgium".to_string(),
            25,
            1000,
            Some(2020),
        )
        .unwrap();
        building_repo
            .expect_find_by_id()
            .with(eq(building_id))
            .times(1)
            .returning(move |_| Ok(Some(building.clone())));

        // Mock meeting exists
        use crate::domain::entities::MeetingType;
        let meeting = Meeting::new(
            org_id,
            building_id,
            MeetingType::Ordinary,
            "Test AG".to_string(),
            None,
            Utc::now(),
            "Test Location".to_string(),
        )
        .unwrap();
        meeting_repo
            .expect_find_by_id()
            .with(eq(meeting_id))
            .times(1)
            .returning(move |_| Ok(Some(meeting.clone())));

        // Mock create decision
        decision_repo
            .expect_create()
            .times(1)
            .returning(|decision| Ok(decision.clone()));

        let use_cases = BoardDecisionUseCases::new(
            Arc::new(decision_repo),
            Arc::new(building_repo),
            Arc::new(meeting_repo),
        );

        let dto = CreateBoardDecisionDto {
            building_id: building_id.to_string(),
            meeting_id: meeting_id.to_string(),
            subject: "Travaux urgents".to_string(),
            decision_text: "Effectuer les travaux de toiture".to_string(),
            deadline: Some((Utc::now() + chrono::Duration::days(30)).to_rfc3339()),
        };

        let result = use_cases.create_decision(dto).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.subject, "Travaux urgents");
        assert_eq!(response.status, "pending");
    }

    #[tokio::test]
    async fn test_create_decision_fails_building_not_found() {
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let decision_repo = MockDecisionRepository::new();
        let mut building_repo = MockBuildingRepo::new();
        let meeting_repo = MockMeetingRepo::new();

        // Mock building not found
        building_repo
            .expect_find_by_id()
            .with(eq(building_id))
            .times(1)
            .returning(|_| Ok(None));

        let use_cases = BoardDecisionUseCases::new(
            Arc::new(decision_repo),
            Arc::new(building_repo),
            Arc::new(meeting_repo),
        );

        let dto = CreateBoardDecisionDto {
            building_id: building_id.to_string(),
            meeting_id: meeting_id.to_string(),
            subject: "Test".to_string(),
            decision_text: "Test".to_string(),
            deadline: None,
        };

        let result = use_cases.create_decision(dto).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Building not found");
    }

    #[tokio::test]
    async fn test_create_decision_fails_meeting_not_found() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let decision_repo = MockDecisionRepository::new();
        let mut building_repo = MockBuildingRepo::new();
        let mut meeting_repo = MockMeetingRepo::new();

        // Mock building exists
        let building = Building::new(
            org_id,
            "Test Building".to_string(),
            "123 Main St".to_string(),
            "Brussels".to_string(),
            "1000".to_string(),
            "Belgium".to_string(),
            25,
            1000,
            Some(2020),
        )
        .unwrap();
        building_repo
            .expect_find_by_id()
            .with(eq(building_id))
            .times(1)
            .returning(move |_| Ok(Some(building.clone())));

        // Mock meeting not found
        meeting_repo
            .expect_find_by_id()
            .with(eq(meeting_id))
            .times(1)
            .returning(|_| Ok(None));

        let use_cases = BoardDecisionUseCases::new(
            Arc::new(decision_repo),
            Arc::new(building_repo),
            Arc::new(meeting_repo),
        );

        let dto = CreateBoardDecisionDto {
            building_id: building_id.to_string(),
            meeting_id: meeting_id.to_string(),
            subject: "Test".to_string(),
            decision_text: "Test".to_string(),
            deadline: None,
        };

        let result = use_cases.create_decision(dto).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Meeting not found");
    }

    #[tokio::test]
    async fn test_get_decision_stats() {
        let building_id = Uuid::new_v4();

        let mut decision_repo = MockDecisionRepository::new();
        let building_repo = MockBuildingRepo::new();
        let meeting_repo = MockMeetingRepo::new();

        // Mock count_by_status for each status
        decision_repo
            .expect_count_by_status()
            .withf(move |id, status| *id == building_id && *status == DecisionStatus::Pending)
            .times(1)
            .returning(|_, _| Ok(3));

        decision_repo
            .expect_count_by_status()
            .withf(move |id, status| *id == building_id && *status == DecisionStatus::InProgress)
            .times(1)
            .returning(|_, _| Ok(2));

        decision_repo
            .expect_count_by_status()
            .withf(move |id, status| *id == building_id && *status == DecisionStatus::Completed)
            .times(1)
            .returning(|_, _| Ok(4));

        decision_repo
            .expect_count_by_status()
            .withf(move |id, status| *id == building_id && *status == DecisionStatus::Cancelled)
            .times(1)
            .returning(|_, _| Ok(1));

        // Mock count_overdue
        decision_repo
            .expect_count_overdue()
            .with(eq(building_id))
            .times(1)
            .returning(|_| Ok(2));

        let use_cases = BoardDecisionUseCases::new(
            Arc::new(decision_repo),
            Arc::new(building_repo),
            Arc::new(meeting_repo),
        );

        let result = use_cases.get_decision_stats(building_id).await;
        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.total_decisions, 12); // 3 + 2 + 4 + 2 + 1
        assert_eq!(stats.pending, 3);
        assert_eq!(stats.in_progress, 2);
        assert_eq!(stats.completed, 4);
        assert_eq!(stats.overdue, 2);
        assert_eq!(stats.cancelled, 1);
    }

    #[tokio::test]
    async fn test_update_decision_status() {
        let decision_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let mut decision_repo = MockDecisionRepository::new();
        let building_repo = MockBuildingRepo::new();
        let meeting_repo = MockMeetingRepo::new();

        let decision = BoardDecision::new(
            building_id,
            meeting_id,
            "Test".to_string(),
            "Test decision".to_string(),
            None,
        )
        .unwrap();

        // Mock find_by_id
        decision_repo
            .expect_find_by_id()
            .with(eq(decision_id))
            .times(1)
            .returning(move |_| Ok(Some(decision.clone())));

        // Mock update
        decision_repo
            .expect_update()
            .times(1)
            .returning(|decision| Ok(decision.clone()));

        let use_cases = BoardDecisionUseCases::new(
            Arc::new(decision_repo),
            Arc::new(building_repo),
            Arc::new(meeting_repo),
        );

        let dto = UpdateBoardDecisionDto {
            status: "in_progress".to_string(),
            notes: Some("Work in progress".to_string()),
        };

        let result = use_cases.update_decision_status(decision_id, dto).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, "in_progress");
        assert!(response.notes.is_some());
    }
}
