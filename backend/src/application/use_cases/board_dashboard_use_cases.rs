use crate::application::dto::{
    BoardDecisionResponseDto, BoardMemberResponseDto, DeadlineAlertDto, DecisionStatsDto,
};
use crate::application::ports::{
    BoardDecisionRepository, BoardMemberRepository, BuildingRepository,
};
#[cfg(test)]
use crate::domain::entities::BoardPosition;
use crate::domain::entities::{BoardDecision, BoardMember, DecisionStatus};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

/// Board Dashboard Use Cases
/// Provides aggregated data and alerts for board members
pub struct BoardDashboardUseCases {
    board_member_repo: Arc<dyn BoardMemberRepository>,
    board_decision_repo: Arc<dyn BoardDecisionRepository>,
    building_repo: Arc<dyn BuildingRepository>,
}

/// Dashboard response containing all aggregated data
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BoardDashboardResponse {
    pub my_mandate: Option<BoardMemberResponseDto>,
    pub decisions_stats: DecisionStatsDto,
    pub overdue_decisions: Vec<BoardDecisionResponseDto>,
    pub upcoming_deadlines: Vec<DeadlineAlertDto>,
}

impl BoardDashboardUseCases {
    pub fn new(
        board_member_repo: Arc<dyn BoardMemberRepository>,
        board_decision_repo: Arc<dyn BoardDecisionRepository>,
        building_repo: Arc<dyn BuildingRepository>,
    ) -> Self {
        Self {
            board_member_repo,
            board_decision_repo,
            building_repo,
        }
    }

    /// Get complete dashboard data for a board member
    pub async fn get_dashboard(
        &self,
        building_id: Uuid,
        owner_id: Uuid,
    ) -> Result<BoardDashboardResponse, String> {
        // Verify building exists
        self.building_repo
            .find_by_id(building_id)
            .await?
            .ok_or_else(|| "Building not found".to_string())?;

        // Get board member's mandate (if any)
        let my_mandate = self
            .board_member_repo
            .find_by_owner_and_building(owner_id, building_id)
            .await?
            .map(|bm| self.to_board_member_dto(&bm));

        // Get all decisions for the building
        let decisions = self
            .board_decision_repo
            .find_by_building(building_id)
            .await?;

        // Calculate statistics
        let decisions_stats = self.calculate_decision_stats(&decisions, building_id);

        // Get overdue decisions
        let overdue_decisions = self.get_overdue_decisions(&decisions);

        // Get upcoming deadlines
        let upcoming_deadlines = self.get_upcoming_deadlines(&decisions);

        Ok(BoardDashboardResponse {
            my_mandate,
            decisions_stats,
            overdue_decisions,
            upcoming_deadlines,
        })
    }

    /// Calculate decision statistics
    fn calculate_decision_stats(
        &self,
        decisions: &[BoardDecision],
        building_id: Uuid,
    ) -> DecisionStatsDto {
        let pending = decisions
            .iter()
            .filter(|d| d.status == DecisionStatus::Pending)
            .count() as i64;
        let in_progress = decisions
            .iter()
            .filter(|d| d.status == DecisionStatus::InProgress)
            .count() as i64;
        let completed = decisions
            .iter()
            .filter(|d| d.status == DecisionStatus::Completed)
            .count() as i64;
        let overdue = decisions
            .iter()
            .filter(|d| d.status == DecisionStatus::Overdue)
            .count() as i64;
        let cancelled = decisions
            .iter()
            .filter(|d| d.status == DecisionStatus::Cancelled)
            .count() as i64;

        DecisionStatsDto {
            building_id: building_id.to_string(),
            total_decisions: decisions.len() as i64,
            pending,
            in_progress,
            completed,
            overdue,
            cancelled,
        }
    }

    /// Get overdue decisions
    fn get_overdue_decisions(&self, decisions: &[BoardDecision]) -> Vec<BoardDecisionResponseDto> {
        decisions
            .iter()
            .filter(|d| d.status == DecisionStatus::Overdue)
            .map(|d| self.to_decision_dto(d))
            .collect()
    }

    /// Get upcoming deadlines (within 30 days)
    fn get_upcoming_deadlines(&self, decisions: &[BoardDecision]) -> Vec<DeadlineAlertDto> {
        let now = Utc::now();
        let thirty_days = chrono::Duration::days(30);

        decisions
            .iter()
            .filter(|d| {
                if let Some(deadline) = d.deadline {
                    let diff = deadline - now;
                    diff > chrono::Duration::zero() && diff <= thirty_days
                } else {
                    false
                }
            })
            .map(|d| {
                let days_remaining = (d.deadline.unwrap() - now).num_days();
                let urgency = if days_remaining <= 7 {
                    "critical"
                } else if days_remaining <= 14 {
                    "high"
                } else {
                    "medium"
                };

                DeadlineAlertDto {
                    decision_id: d.id.to_string(),
                    subject: d.subject.clone(),
                    deadline: d.deadline.unwrap().to_rfc3339(),
                    days_remaining,
                    urgency: urgency.to_string(),
                }
            })
            .collect()
    }

    /// Convert BoardMember to DTO
    fn to_board_member_dto(&self, bm: &BoardMember) -> BoardMemberResponseDto {
        let now = Utc::now();
        let days_remaining = (bm.mandate_end - now).num_days();
        let expires_soon = days_remaining <= 60 && days_remaining > 0;
        let is_active = now >= bm.mandate_start && now <= bm.mandate_end;

        BoardMemberResponseDto {
            id: bm.id.to_string(),
            owner_id: bm.owner_id.to_string(),
            building_id: bm.building_id.to_string(),
            position: bm.position.to_string(),
            mandate_start: bm.mandate_start.to_rfc3339(),
            mandate_end: bm.mandate_end.to_rfc3339(),
            elected_by_meeting_id: bm.elected_by_meeting_id.to_string(),
            is_active,
            days_remaining,
            expires_soon,
            created_at: bm.created_at.to_rfc3339(),
            updated_at: bm.updated_at.to_rfc3339(),
        }
    }

    /// Convert BoardDecision to DTO
    fn to_decision_dto(&self, d: &BoardDecision) -> BoardDecisionResponseDto {
        let now = Utc::now();
        let is_overdue = d
            .deadline
            .map(|deadline| deadline < now && d.status != DecisionStatus::Completed)
            .unwrap_or(false);
        let days_until_deadline = d.deadline.map(|deadline| (deadline - now).num_days());

        BoardDecisionResponseDto {
            id: d.id.to_string(),
            building_id: d.building_id.to_string(),
            meeting_id: d.meeting_id.to_string(),
            subject: d.subject.clone(),
            decision_text: d.decision_text.clone(),
            deadline: d.deadline.map(|dt| dt.to_rfc3339()),
            status: d.status.to_string(),
            completed_at: d.completed_at.map(|dt| dt.to_rfc3339()),
            notes: d.notes.clone(),
            is_overdue,
            days_until_deadline,
            created_at: d.created_at.to_rfc3339(),
            updated_at: d.updated_at.to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{BoardDecision, BoardMember};
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::Mutex;
    use uuid::Uuid;

    // Mock BoardMemberRepository
    struct MockBoardMemberRepository {
        members: Mutex<Vec<BoardMember>>,
    }

    impl MockBoardMemberRepository {
        fn new() -> Self {
            Self {
                members: Mutex::new(vec![]),
            }
        }

        fn add_member(&self, member: BoardMember) {
            self.members.lock().unwrap().push(member);
        }
    }

    #[async_trait]
    impl BoardMemberRepository for MockBoardMemberRepository {
        async fn create(&self, _member: &BoardMember) -> Result<BoardMember, String> {
            unimplemented!()
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<BoardMember>, String> {
            Ok(self
                .members
                .lock()
                .unwrap()
                .iter()
                .find(|m| m.id == id)
                .cloned())
        }

        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<BoardMember>, String> {
            Ok(self
                .members
                .lock()
                .unwrap()
                .iter()
                .filter(|m| m.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn find_by_owner_and_building(
            &self,
            owner_id: Uuid,
            building_id: Uuid,
        ) -> Result<Option<BoardMember>, String> {
            Ok(self
                .members
                .lock()
                .unwrap()
                .iter()
                .find(|m| m.owner_id == owner_id && m.building_id == building_id)
                .cloned())
        }

        async fn find_active_by_building(
            &self,
            building_id: Uuid,
        ) -> Result<Vec<BoardMember>, String> {
            let now = Utc::now();
            Ok(self
                .members
                .lock()
                .unwrap()
                .iter()
                .filter(|m| m.building_id == building_id && m.mandate_end > now)
                .cloned()
                .collect())
        }

        async fn find_expiring_soon(
            &self,
            building_id: Uuid,
            days_threshold: i32,
        ) -> Result<Vec<BoardMember>, String> {
            let now = Utc::now();
            let threshold = now + chrono::Duration::days(days_threshold as i64);
            Ok(self
                .members
                .lock()
                .unwrap()
                .iter()
                .filter(|m| {
                    m.building_id == building_id
                        && m.mandate_end > now
                        && m.mandate_end <= threshold
                })
                .cloned()
                .collect())
        }

        async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<BoardMember>, String> {
            Ok(self
                .members
                .lock()
                .unwrap()
                .iter()
                .filter(|m| m.owner_id == owner_id)
                .cloned()
                .collect())
        }

        async fn has_active_mandate(
            &self,
            owner_id: Uuid,
            building_id: Uuid,
        ) -> Result<bool, String> {
            let now = Utc::now();
            Ok(self.members.lock().unwrap().iter().any(|m| {
                m.owner_id == owner_id && m.building_id == building_id && m.mandate_end > now
            }))
        }

        async fn update(&self, _member: &BoardMember) -> Result<BoardMember, String> {
            unimplemented!()
        }

        async fn delete(&self, _id: Uuid) -> Result<bool, String> {
            unimplemented!()
        }

        async fn count_active_by_building(&self, building_id: Uuid) -> Result<i64, String> {
            let now = Utc::now();
            Ok(self
                .members
                .lock()
                .unwrap()
                .iter()
                .filter(|m| m.building_id == building_id && m.mandate_end > now)
                .count() as i64)
        }
    }

    // Mock BoardDecisionRepository
    struct MockBoardDecisionRepository {
        decisions: Mutex<Vec<BoardDecision>>,
    }

    impl MockBoardDecisionRepository {
        fn new() -> Self {
            Self {
                decisions: Mutex::new(vec![]),
            }
        }

        fn add_decision(&self, decision: BoardDecision) {
            self.decisions.lock().unwrap().push(decision);
        }
    }

    #[async_trait]
    impl BoardDecisionRepository for MockBoardDecisionRepository {
        async fn create(&self, _decision: &BoardDecision) -> Result<BoardDecision, String> {
            unimplemented!()
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<BoardDecision>, String> {
            Ok(self
                .decisions
                .lock()
                .unwrap()
                .iter()
                .find(|d| d.id == id)
                .cloned())
        }

        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<BoardDecision>, String> {
            Ok(self
                .decisions
                .lock()
                .unwrap()
                .iter()
                .filter(|d| d.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn find_by_meeting(&self, meeting_id: Uuid) -> Result<Vec<BoardDecision>, String> {
            Ok(self
                .decisions
                .lock()
                .unwrap()
                .iter()
                .filter(|d| d.meeting_id == meeting_id)
                .cloned()
                .collect())
        }

        async fn find_by_status(
            &self,
            building_id: Uuid,
            status: DecisionStatus,
        ) -> Result<Vec<BoardDecision>, String> {
            Ok(self
                .decisions
                .lock()
                .unwrap()
                .iter()
                .filter(|d| d.building_id == building_id && d.status == status)
                .cloned()
                .collect())
        }

        async fn find_overdue(&self, building_id: Uuid) -> Result<Vec<BoardDecision>, String> {
            Ok(self
                .decisions
                .lock()
                .unwrap()
                .iter()
                .filter(|d| d.building_id == building_id && d.status == DecisionStatus::Overdue)
                .cloned()
                .collect())
        }

        async fn find_deadline_approaching(
            &self,
            building_id: Uuid,
            days_threshold: i32,
        ) -> Result<Vec<BoardDecision>, String> {
            let now = Utc::now();
            let threshold = now + chrono::Duration::days(days_threshold as i64);
            Ok(self
                .decisions
                .lock()
                .unwrap()
                .iter()
                .filter(|d| {
                    if let Some(deadline) = d.deadline {
                        d.building_id == building_id && deadline > now && deadline <= threshold
                    } else {
                        false
                    }
                })
                .cloned()
                .collect())
        }

        async fn update(&self, _decision: &BoardDecision) -> Result<BoardDecision, String> {
            unimplemented!()
        }

        async fn delete(&self, _id: Uuid) -> Result<bool, String> {
            unimplemented!()
        }

        async fn count_by_status(
            &self,
            building_id: Uuid,
            status: DecisionStatus,
        ) -> Result<i64, String> {
            Ok(self
                .decisions
                .lock()
                .unwrap()
                .iter()
                .filter(|d| d.building_id == building_id && d.status == status)
                .count() as i64)
        }

        async fn count_overdue(&self, building_id: Uuid) -> Result<i64, String> {
            Ok(self
                .decisions
                .lock()
                .unwrap()
                .iter()
                .filter(|d| d.building_id == building_id && d.status == DecisionStatus::Overdue)
                .count() as i64)
        }
    }

    // Mock BuildingRepository
    struct MockBuildingRepository {
        exists: bool,
    }

    impl MockBuildingRepository {
        fn new(exists: bool) -> Self {
            Self { exists }
        }
    }

    #[async_trait]
    impl BuildingRepository for MockBuildingRepository {
        async fn create(
            &self,
            _building: &crate::domain::entities::Building,
        ) -> Result<crate::domain::entities::Building, String> {
            unimplemented!()
        }

        async fn find_by_id(
            &self,
            _id: Uuid,
        ) -> Result<Option<crate::domain::entities::Building>, String> {
            if self.exists {
                Ok(Some(crate::domain::entities::Building {
                    id: Uuid::new_v4(),
                    organization_id: Uuid::new_v4(),
                    name: "Test Building".to_string(),
                    address: "123 Test St".to_string(),
                    city: "Brussels".to_string(),
                    postal_code: "1000".to_string(),
                    country: "Belgium".to_string(),
                    total_units: 25,
                    total_tantiemes: 1000,
                    construction_year: Some(2020),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }))
            } else {
                Ok(None)
            }
        }

        async fn find_all(&self) -> Result<Vec<crate::domain::entities::Building>, String> {
            unimplemented!()
        }

        async fn find_all_paginated(
            &self,
            _page_request: &crate::application::dto::PageRequest,
            _filters: &crate::application::dto::BuildingFilters,
        ) -> Result<(Vec<crate::domain::entities::Building>, i64), String> {
            unimplemented!()
        }

        async fn update(
            &self,
            _building: &crate::domain::entities::Building,
        ) -> Result<crate::domain::entities::Building, String> {
            unimplemented!()
        }

        async fn delete(&self, _id: Uuid) -> Result<bool, String> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_calculate_decision_stats() {
        let board_member_repo = Arc::new(MockBoardMemberRepository::new());
        let board_decision_repo = Arc::new(MockBoardDecisionRepository::new());
        let building_repo = Arc::new(MockBuildingRepository::new(true));

        let use_cases =
            BoardDashboardUseCases::new(board_member_repo, board_decision_repo, building_repo);

        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let decisions = vec![
            BoardDecision {
                id: Uuid::new_v4(),
                building_id,
                meeting_id,
                subject: "Decision 1".to_string(),
                decision_text: "Text 1".to_string(),
                deadline: None,
                status: DecisionStatus::Pending,
                completed_at: None,
                notes: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            BoardDecision {
                id: Uuid::new_v4(),
                building_id,
                meeting_id,
                subject: "Decision 2".to_string(),
                decision_text: "Text 2".to_string(),
                deadline: None,
                status: DecisionStatus::Pending,
                completed_at: None,
                notes: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            BoardDecision {
                id: Uuid::new_v4(),
                building_id,
                meeting_id,
                subject: "Decision 3".to_string(),
                decision_text: "Text 3".to_string(),
                deadline: None,
                status: DecisionStatus::Completed,
                completed_at: Some(Utc::now()),
                notes: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            BoardDecision {
                id: Uuid::new_v4(),
                building_id,
                meeting_id,
                subject: "Decision 4".to_string(),
                decision_text: "Text 4".to_string(),
                deadline: Some(Utc::now() - chrono::Duration::days(5)),
                status: DecisionStatus::Overdue,
                completed_at: None,
                notes: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        let stats = use_cases.calculate_decision_stats(&decisions, building_id);

        assert_eq!(stats.pending, 2);
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.overdue, 1);
        assert_eq!(stats.in_progress, 0);
        assert_eq!(stats.total_decisions, 4);
    }

    #[tokio::test]
    async fn test_get_upcoming_deadlines() {
        let board_member_repo = Arc::new(MockBoardMemberRepository::new());
        let board_decision_repo = Arc::new(MockBoardDecisionRepository::new());
        let building_repo = Arc::new(MockBuildingRepository::new(true));

        let use_cases =
            BoardDashboardUseCases::new(board_member_repo, board_decision_repo, building_repo);

        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let decisions = vec![
            BoardDecision {
                id: Uuid::new_v4(),
                building_id,
                meeting_id,
                subject: "Critical Decision".to_string(),
                decision_text: "Text".to_string(),
                deadline: Some(Utc::now() + chrono::Duration::days(5)),
                status: DecisionStatus::Pending,
                completed_at: None,
                notes: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            BoardDecision {
                id: Uuid::new_v4(),
                building_id,
                meeting_id,
                subject: "High Priority".to_string(),
                decision_text: "Text".to_string(),
                deadline: Some(Utc::now() + chrono::Duration::days(10)),
                status: DecisionStatus::Pending,
                completed_at: None,
                notes: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            BoardDecision {
                id: Uuid::new_v4(),
                building_id,
                meeting_id,
                subject: "Medium Priority".to_string(),
                decision_text: "Text".to_string(),
                deadline: Some(Utc::now() + chrono::Duration::days(20)),
                status: DecisionStatus::Pending,
                completed_at: None,
                notes: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            BoardDecision {
                id: Uuid::new_v4(),
                building_id,
                meeting_id,
                subject: "Too Far".to_string(),
                decision_text: "Text".to_string(),
                deadline: Some(Utc::now() + chrono::Duration::days(60)),
                status: DecisionStatus::Pending,
                completed_at: None,
                notes: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        let deadlines = use_cases.get_upcoming_deadlines(&decisions);

        assert_eq!(deadlines.len(), 3);
        assert_eq!(deadlines[0].urgency, "critical");
        assert_eq!(deadlines[1].urgency, "high");
        assert_eq!(deadlines[2].urgency, "medium");
    }

    #[tokio::test]
    async fn test_dashboard_with_expiring_mandate() {
        let board_member_repo = Arc::new(MockBoardMemberRepository::new());
        let board_decision_repo = Arc::new(MockBoardDecisionRepository::new());
        let building_repo = Arc::new(MockBuildingRepository::new(true));

        let building_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        // Add board member with expiring mandate (45 days)
        let mandate_start = Utc::now() - chrono::Duration::days(320);
        let mandate_end = mandate_start + chrono::Duration::days(365);

        board_member_repo.add_member(BoardMember {
            id: Uuid::new_v4(),
            owner_id,
            building_id,
            position: BoardPosition::President,
            mandate_start,
            mandate_end,
            elected_by_meeting_id: meeting_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        // Add some decisions
        board_decision_repo.add_decision(BoardDecision {
            id: Uuid::new_v4(),
            building_id,
            meeting_id,
            subject: "Pending Decision".to_string(),
            decision_text: "Text".to_string(),
            deadline: Some(Utc::now() + chrono::Duration::days(15)),
            status: DecisionStatus::Pending,
            completed_at: None,
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        let use_cases =
            BoardDashboardUseCases::new(board_member_repo, board_decision_repo, building_repo);

        let dashboard = use_cases
            .get_dashboard(building_id, owner_id)
            .await
            .expect("Should get dashboard");

        // Verify mandate info
        assert!(dashboard.my_mandate.is_some());
        let mandate = dashboard.my_mandate.unwrap();
        assert!(mandate.expires_soon);

        // Verify stats
        assert_eq!(dashboard.decisions_stats.pending, 1);
    }
}
