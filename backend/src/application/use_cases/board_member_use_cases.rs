use crate::application::dto::{
    BoardMemberResponseDto, BoardStatsDto, CreateBoardMemberDto, RenewMandateDto,
};
use crate::application::ports::{BoardMemberRepository, BuildingRepository};
use crate::domain::entities::{BoardMember, BoardPosition};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct BoardMemberUseCases {
    repository: Arc<dyn BoardMemberRepository>,
    building_repository: Arc<dyn BuildingRepository>,
}

impl BoardMemberUseCases {
    pub fn new(
        repository: Arc<dyn BoardMemberRepository>,
        building_repository: Arc<dyn BuildingRepository>,
    ) -> Self {
        Self {
            repository,
            building_repository,
        }
    }

    /// Élit un nouveau membre du conseil de copropriété
    /// Vérifie que l'immeuble a plus de 20 lots (obligation légale)
    pub async fn elect_board_member(
        &self,
        dto: CreateBoardMemberDto,
    ) -> Result<BoardMemberResponseDto, String> {
        // Parse UUIDs
        let owner_id =
            Uuid::parse_str(&dto.owner_id).map_err(|_| "Invalid owner_id format".to_string())?;
        let building_id = Uuid::parse_str(&dto.building_id)
            .map_err(|_| "Invalid building_id format".to_string())?;
        let elected_by_meeting_id = Uuid::parse_str(&dto.elected_by_meeting_id)
            .map_err(|_| "Invalid elected_by_meeting_id format".to_string())?;

        // Vérifier que l'immeuble existe et a plus de 20 lots
        let building = self
            .building_repository
            .find_by_id(building_id)
            .await?
            .ok_or_else(|| "Building not found".to_string())?;

        if building.total_units <= 20 {
            return Err(
                "Board of directors is only required for buildings with more than 20 units"
                    .to_string(),
            );
        }

        // Parse position
        let position: BoardPosition = dto
            .position
            .parse()
            .map_err(|e| format!("Invalid position: {}", e))?;

        // Parse dates
        let mandate_start = DateTime::parse_from_rfc3339(&dto.mandate_start)
            .map_err(|_| "Invalid mandate_start format".to_string())?
            .with_timezone(&Utc);

        let mandate_end = DateTime::parse_from_rfc3339(&dto.mandate_end)
            .map_err(|_| "Invalid mandate_end format".to_string())?
            .with_timezone(&Utc);

        // Créer l'entité domain avec validation
        let board_member = BoardMember::new(
            owner_id,
            building_id,
            position,
            mandate_start,
            mandate_end,
            elected_by_meeting_id,
        )?;

        // Persister
        let created = self.repository.create(&board_member).await?;

        Ok(self.to_response_dto(&created))
    }

    /// Obtient un membre du conseil par son ID
    pub async fn get_board_member(
        &self,
        id: Uuid,
    ) -> Result<Option<BoardMemberResponseDto>, String> {
        let member = self.repository.find_by_id(id).await?;
        Ok(member.map(|m| self.to_response_dto(&m)))
    }

    /// Liste tous les membres actifs du conseil pour un immeuble
    pub async fn list_active_board_members(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<BoardMemberResponseDto>, String> {
        let members = self.repository.find_active_by_building(building_id).await?;
        Ok(members.iter().map(|m| self.to_response_dto(m)).collect())
    }

    /// Liste tous les membres du conseil (incluant historique) pour un immeuble
    pub async fn list_all_board_members(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<BoardMemberResponseDto>, String> {
        let members = self.repository.find_by_building(building_id).await?;
        Ok(members.iter().map(|m| self.to_response_dto(m)).collect())
    }

    /// Renouvelle le mandat d'un membre du conseil
    pub async fn renew_mandate(
        &self,
        id: Uuid,
        dto: RenewMandateDto,
    ) -> Result<BoardMemberResponseDto, String> {
        let mut member = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Board member not found".to_string())?;

        let new_meeting_id = Uuid::parse_str(&dto.new_elected_by_meeting_id)
            .map_err(|_| "Invalid meeting_id format".to_string())?;

        // Renouveler le mandat (validation dans l'entity)
        member.extend_mandate(new_meeting_id)?;

        // Persister
        let updated = self.repository.update(&member).await?;

        Ok(self.to_response_dto(&updated))
    }

    /// Démissionne un membre du conseil (suppression)
    pub async fn remove_board_member(&self, id: Uuid) -> Result<bool, String> {
        self.repository.delete(id).await
    }

    /// Obtient les statistiques du conseil pour un immeuble
    pub async fn get_board_stats(&self, building_id: Uuid) -> Result<BoardStatsDto, String> {
        let all_members = self.repository.find_by_building(building_id).await?;
        let active_members = self.repository.find_active_by_building(building_id).await?;
        let expiring_soon = self.repository.find_expiring_soon(building_id, 60).await?;

        let has_president = active_members
            .iter()
            .any(|m| m.position == BoardPosition::President);
        let has_treasurer = active_members
            .iter()
            .any(|m| m.position == BoardPosition::Treasurer);

        Ok(BoardStatsDto {
            building_id: building_id.to_string(),
            total_members: all_members.len() as i64,
            active_members: active_members.len() as i64,
            expiring_soon: expiring_soon.len() as i64,
            has_president,
            has_treasurer,
        })
    }

    /// Vérifie si un propriétaire a un mandat actif pour un immeuble
    /// Utilisé pour l'autorisation d'accès au tableau de bord
    pub async fn has_active_board_mandate(
        &self,
        owner_id: Uuid,
        building_id: Uuid,
    ) -> Result<bool, String> {
        self.repository
            .has_active_mandate(owner_id, building_id)
            .await
    }

    /// Mapper entity → DTO response
    fn to_response_dto(&self, member: &BoardMember) -> BoardMemberResponseDto {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::Building;
    use chrono::Duration;
    use mockall::mock;
    use mockall::predicate::*;

    // Mock du BoardMemberRepository
    mock! {
        pub BoardMemberRepo {}

        #[async_trait::async_trait]
        impl BoardMemberRepository for BoardMemberRepo {
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

    // Mock du BuildingRepository
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

    fn create_test_building(total_units: i32) -> Building {
        Building::new(
            Uuid::new_v4(),
            "Test Building".to_string(),
            "123 Test St".to_string(),
            "Brussels".to_string(),
            "1000".to_string(),
            "Belgium".to_string(),
            total_units,
            1000,
            Some(2020),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_elect_board_member_success() {
        // Arrange
        let mut mock_board_repo = MockBoardMemberRepo::new();
        let mut mock_building_repo = MockBuildingRepo::new();

        let building = create_test_building(25); // >20 lots
        let building_id = building.id;

        mock_building_repo
            .expect_find_by_id()
            .with(eq(building_id))
            .times(1)
            .returning(move |_| Ok(Some(create_test_building(25))));

        mock_board_repo
            .expect_create()
            .times(1)
            .returning(|member| Ok(member.clone()));

        let use_cases =
            BoardMemberUseCases::new(Arc::new(mock_board_repo), Arc::new(mock_building_repo));

        let dto = CreateBoardMemberDto {
            owner_id: Uuid::new_v4().to_string(),
            building_id: building_id.to_string(),
            position: "president".to_string(),
            mandate_start: Utc::now().to_rfc3339(),
            mandate_end: (Utc::now() + Duration::days(365)).to_rfc3339(),
            elected_by_meeting_id: Uuid::new_v4().to_string(),
        };

        // Act
        let result = use_cases.elect_board_member(dto).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.position, "president");
        assert!(response.is_active);
    }

    #[tokio::test]
    async fn test_elect_board_member_fails_building_not_found() {
        // Arrange
        let mock_board_repo = MockBoardMemberRepo::new();
        let mut mock_building_repo = MockBuildingRepo::new();

        let building_id = Uuid::new_v4();

        mock_building_repo
            .expect_find_by_id()
            .with(eq(building_id))
            .times(1)
            .returning(|_| Ok(None)); // Building not found

        let use_cases =
            BoardMemberUseCases::new(Arc::new(mock_board_repo), Arc::new(mock_building_repo));

        let dto = CreateBoardMemberDto {
            owner_id: Uuid::new_v4().to_string(),
            building_id: building_id.to_string(),
            position: "president".to_string(),
            mandate_start: Utc::now().to_rfc3339(),
            mandate_end: (Utc::now() + Duration::days(365)).to_rfc3339(),
            elected_by_meeting_id: Uuid::new_v4().to_string(),
        };

        // Act
        let result = use_cases.elect_board_member(dto).await;

        // Assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Building not found");
    }

    #[tokio::test]
    async fn test_elect_board_member_fails_building_too_small() {
        // Arrange
        let mock_board_repo = MockBoardMemberRepo::new();
        let mut mock_building_repo = MockBuildingRepo::new();

        let building = create_test_building(15); // ≤20 lots
        let building_id = building.id;

        mock_building_repo
            .expect_find_by_id()
            .with(eq(building_id))
            .times(1)
            .returning(move |_| Ok(Some(create_test_building(15))));

        let use_cases =
            BoardMemberUseCases::new(Arc::new(mock_board_repo), Arc::new(mock_building_repo));

        let dto = CreateBoardMemberDto {
            owner_id: Uuid::new_v4().to_string(),
            building_id: building_id.to_string(),
            position: "president".to_string(),
            mandate_start: Utc::now().to_rfc3339(),
            mandate_end: (Utc::now() + Duration::days(365)).to_rfc3339(),
            elected_by_meeting_id: Uuid::new_v4().to_string(),
        };

        // Act
        let result = use_cases.elect_board_member(dto).await;

        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Board of directors is only required for buildings with more than 20 units"
        );
    }

    #[tokio::test]
    async fn test_get_board_stats() {
        // Arrange
        let mut mock_board_repo = MockBoardMemberRepo::new();
        let mock_building_repo = MockBuildingRepo::new();

        let building_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Créer des membres test
        let president = BoardMember::new(
            user_id,
            building_id,
            BoardPosition::President,
            Utc::now() - Duration::days(100),
            Utc::now() + Duration::days(265),
            Uuid::new_v4(),
        )
        .unwrap();

        let treasurer = BoardMember::new(
            Uuid::new_v4(),
            building_id,
            BoardPosition::Treasurer,
            Utc::now() - Duration::days(320),
            Utc::now() + Duration::days(45), // Expire soon
            Uuid::new_v4(),
        )
        .unwrap();

        let all_members = vec![president.clone(), treasurer.clone()];
        let active_members = vec![president.clone(), treasurer.clone()];
        let expiring = vec![treasurer.clone()];

        mock_board_repo
            .expect_find_by_building()
            .with(eq(building_id))
            .times(1)
            .return_once(move |_| Ok(all_members));

        mock_board_repo
            .expect_find_active_by_building()
            .with(eq(building_id))
            .times(1)
            .return_once(move |_| Ok(active_members));

        mock_board_repo
            .expect_find_expiring_soon()
            .with(eq(building_id), eq(60))
            .times(1)
            .return_once(move |_, _| Ok(expiring));

        let use_cases =
            BoardMemberUseCases::new(Arc::new(mock_board_repo), Arc::new(mock_building_repo));

        // Act
        let result = use_cases.get_board_stats(building_id).await;

        // Assert
        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.total_members, 2);
        assert_eq!(stats.active_members, 2);
        assert_eq!(stats.expiring_soon, 1);
        assert!(stats.has_president);
        assert!(stats.has_treasurer);
    }

    #[tokio::test]
    async fn test_renew_mandate_success() {
        // Arrange
        let mut mock_board_repo = MockBoardMemberRepo::new();
        let mock_building_repo = MockBuildingRepo::new();

        let member_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        // Membre avec mandat qui expire bientôt
        let member = BoardMember::new(
            Uuid::new_v4(),
            building_id,
            BoardPosition::President,
            Utc::now() - Duration::days(320),
            Utc::now() + Duration::days(45), // Expire dans 45 jours
            Uuid::new_v4(),
        )
        .unwrap();

        let member_clone = member.clone();

        mock_board_repo
            .expect_find_by_id()
            .with(eq(member_id))
            .times(1)
            .return_once(move |_| Ok(Some(member_clone)));

        mock_board_repo
            .expect_update()
            .times(1)
            .returning(|m| Ok(m.clone()));

        let use_cases =
            BoardMemberUseCases::new(Arc::new(mock_board_repo), Arc::new(mock_building_repo));

        let dto = RenewMandateDto {
            new_elected_by_meeting_id: Uuid::new_v4().to_string(),
        };

        // Act
        let result = use_cases.renew_mandate(member_id, dto).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.days_remaining > 300); // Nouveau mandat d'un an
    }
}
