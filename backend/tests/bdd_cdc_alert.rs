//! Story 4.7 — Runner BDD pour `cdc_alert.feature`.
//!
//! `CdcUseCases` est de la logique pure derrière des ports (traits) : même
//! pattern que `bdd_meeting_hybrid_quorum.rs` / `bdd_meeting_complete.rs` —
//! pas de testcontainer Postgres, des repositories en mémoire suffisent à
//! exercer les 4 catégories (@happy @edge @security @negative).

use async_trait::async_trait;
use chrono::{Duration, Utc};
use cucumber::{given, then, when, World};
use koprogo_api::application::dto::{
    BoardAlertResponseDto, BoardMemberResponseDto, CdcCandidateDto, CreateBoardAlertDto,
    ElectCdcMembersDto, PageRequest,
};
use koprogo_api::application::error::AppError;
use koprogo_api::application::ports::{
    BoardAlertRepository, BoardMemberRepository, MeetingRepository,
};
use koprogo_api::application::use_cases::CdcUseCases;
use koprogo_api::domain::entities::{BoardAlert, BoardMember, Meeting, MeetingStatus, MeetingType};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// ============================================================================
// Repositories en mémoire — implémentent les mêmes ports que la production.
// ============================================================================

#[derive(Default)]
struct InMemoryBoardMemberRepository {
    members: Mutex<HashMap<Uuid, BoardMember>>,
}

#[async_trait]
impl BoardMemberRepository for InMemoryBoardMemberRepository {
    async fn create(&self, board_member: &BoardMember) -> Result<BoardMember, String> {
        self.members
            .lock()
            .unwrap()
            .insert(board_member.id, board_member.clone());
        Ok(board_member.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<BoardMember>, String> {
        Ok(self.members.lock().unwrap().get(&id).cloned())
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<BoardMember>, String> {
        Ok(self
            .members
            .lock()
            .unwrap()
            .values()
            .filter(|m| m.building_id == building_id)
            .cloned()
            .collect())
    }

    async fn find_active_by_building(&self, building_id: Uuid) -> Result<Vec<BoardMember>, String> {
        Ok(self
            .members
            .lock()
            .unwrap()
            .values()
            .filter(|m| m.building_id == building_id && m.is_active())
            .cloned()
            .collect())
    }

    async fn find_expiring_soon(
        &self,
        _building_id: Uuid,
        _days_threshold: i32,
    ) -> Result<Vec<BoardMember>, String> {
        Ok(Vec::new())
    }

    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<BoardMember>, String> {
        Ok(self
            .members
            .lock()
            .unwrap()
            .values()
            .filter(|m| m.owner_id == owner_id)
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
            .values()
            .find(|m| m.owner_id == owner_id && m.building_id == building_id)
            .cloned())
    }

    async fn has_active_mandate(&self, owner_id: Uuid, building_id: Uuid) -> Result<bool, String> {
        Ok(self
            .members
            .lock()
            .unwrap()
            .values()
            .any(|m| m.owner_id == owner_id && m.building_id == building_id && m.is_active()))
    }

    async fn update(&self, board_member: &BoardMember) -> Result<BoardMember, String> {
        self.members
            .lock()
            .unwrap()
            .insert(board_member.id, board_member.clone());
        Ok(board_member.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        Ok(self.members.lock().unwrap().remove(&id).is_some())
    }

    async fn count_active_by_building(&self, building_id: Uuid) -> Result<i64, String> {
        Ok(self
            .members
            .lock()
            .unwrap()
            .values()
            .filter(|m| m.building_id == building_id && m.is_active())
            .count() as i64)
    }
}

#[derive(Default)]
struct InMemoryMeetingRepository {
    meetings: Mutex<HashMap<Uuid, Meeting>>,
}

#[async_trait]
impl MeetingRepository for InMemoryMeetingRepository {
    async fn create(&self, meeting: &Meeting) -> Result<Meeting, String> {
        self.meetings
            .lock()
            .unwrap()
            .insert(meeting.id, meeting.clone());
        Ok(meeting.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Meeting>, String> {
        Ok(self.meetings.lock().unwrap().get(&id).cloned())
    }

    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Meeting>, String> {
        Ok(self
            .meetings
            .lock()
            .unwrap()
            .values()
            .filter(|m| m.building_id == building_id)
            .cloned()
            .collect())
    }

    async fn update(&self, meeting: &Meeting) -> Result<Meeting, String> {
        self.meetings
            .lock()
            .unwrap()
            .insert(meeting.id, meeting.clone());
        Ok(meeting.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        Ok(self.meetings.lock().unwrap().remove(&id).is_some())
    }

    async fn find_all_paginated(
        &self,
        _page_request: &PageRequest,
        _organization_id: Option<Uuid>,
    ) -> Result<(Vec<Meeting>, i64), String> {
        let all: Vec<Meeting> = self.meetings.lock().unwrap().values().cloned().collect();
        let total = all.len() as i64;
        Ok((all, total))
    }
}

#[derive(Default)]
struct InMemoryBoardAlertRepository {
    alerts: Mutex<HashMap<Uuid, BoardAlert>>,
}

#[async_trait]
impl BoardAlertRepository for InMemoryBoardAlertRepository {
    async fn create(&self, alert: &BoardAlert) -> Result<BoardAlert, AppError> {
        self.alerts.lock().unwrap().insert(alert.id, alert.clone());
        Ok(alert.clone())
    }

    async fn find_by_target_meeting(&self, meeting_id: Uuid) -> Result<Vec<BoardAlert>, AppError> {
        Ok(self
            .alerts
            .lock()
            .unwrap()
            .values()
            .filter(|a| a.target_meeting_id == meeting_id)
            .cloned()
            .collect())
    }
}

// ============================================================================
// World
// ============================================================================

#[derive(World)]
#[world(init = Self::new)]
pub struct CdcAlertWorld {
    use_cases: Option<Arc<CdcUseCases>>,
    member_repo: Option<Arc<InMemoryBoardMemberRepository>>,
    meeting_repo: Option<Arc<InMemoryMeetingRepository>>,
    buildings: HashMap<String, Uuid>,
    meetings: HashMap<String, Uuid>,
    last_building_id: Option<Uuid>,
    elected_owner_ids: Vec<Uuid>,
    election_result: Option<Result<Vec<BoardMemberResponseDto>, AppError>>,
    alert_result: Option<Result<BoardAlertResponseDto, AppError>>,
}

impl std::fmt::Debug for CdcAlertWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CdcAlertWorld")
            .field("buildings", &self.buildings)
            .field("meetings", &self.meetings)
            .field("election_result", &self.election_result)
            .field("alert_result", &self.alert_result)
            .finish()
    }
}

impl CdcAlertWorld {
    async fn new() -> Self {
        Self {
            use_cases: None,
            member_repo: None,
            meeting_repo: None,
            buildings: HashMap::new(),
            meetings: HashMap::new(),
            last_building_id: None,
            elected_owner_ids: Vec::new(),
            election_result: None,
            alert_result: None,
        }
    }

    fn ensure_setup(&mut self) {
        if self.use_cases.is_none() {
            let member_repo = Arc::new(InMemoryBoardMemberRepository::default());
            let meeting_repo = Arc::new(InMemoryMeetingRepository::default());
            let alert_repo = Arc::new(InMemoryBoardAlertRepository::default());
            self.use_cases = Some(Arc::new(CdcUseCases::new(
                alert_repo,
                member_repo.clone(),
                meeting_repo.clone(),
            )));
            self.member_repo = Some(member_repo);
            self.meeting_repo = Some(meeting_repo);
        }
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
    .expect("valid meeting");
    meeting.status = status;
    meeting
}

// ============================================================================
// Given
// ============================================================================

#[given("a coproperty management system")]
fn given_system(world: &mut CdcAlertWorld) {
    world.ensure_setup();
}

#[given(regex = r#"^a building "([^"]+)"$"#)]
fn given_building(world: &mut CdcAlertWorld, name: String) {
    world.ensure_setup();
    let id = Uuid::new_v4();
    world.buildings.insert(name, id);
    world.last_building_id = Some(id);
}

#[given(regex = r#"^a (completed|scheduled) meeting "([^"]+)" for that building$"#)]
async fn given_meeting(world: &mut CdcAlertWorld, status: String, name: String) {
    world.ensure_setup();
    let building_id = world.last_building_id.expect("a building must exist");
    let status = if status == "completed" {
        MeetingStatus::Completed
    } else {
        MeetingStatus::Scheduled
    };
    let meeting = make_meeting(building_id, status);
    let meeting_id = meeting.id;
    world
        .meeting_repo
        .as_ref()
        .unwrap()
        .create(&meeting)
        .await
        .expect("create meeting");
    world.meetings.insert(name, meeting_id);
}

#[given(regex = r#"^the CdC is elected with (\d+) members? from "([^"]+)"$"#)]
async fn given_elect_members(world: &mut CdcAlertWorld, count: usize, meeting_name: String) {
    elect_members(world, count, meeting_name).await;
}

#[given("the elected member's mandate has already ended")]
async fn given_mandate_ended(world: &mut CdcAlertWorld) {
    let owner_id = world.elected_owner_ids[0];
    let building_id = world.last_building_id.expect("a building must exist");
    let repo = world.member_repo.as_ref().unwrap();
    let mut member = repo
        .find_by_owner_and_building(owner_id, building_id)
        .await
        .unwrap()
        .expect("elected member must exist");
    member.resign(Utc::now() - Duration::days(1));
    repo.update(&member).await.unwrap();
}

// ============================================================================
// When
// ============================================================================

#[when(regex = r#"^the CdC is elected with (\d+) members? from "([^"]+)"$"#)]
async fn when_elect_members(world: &mut CdcAlertWorld, count: usize, meeting_name: String) {
    elect_members(world, count, meeting_name).await;
}

async fn elect_members(world: &mut CdcAlertWorld, count: usize, meeting_name: String) {
    let building_id = world.last_building_id.expect("a building must exist");
    let meeting_id = *world
        .meetings
        .get(&meeting_name)
        .expect("meeting must exist");

    let owner_ids: Vec<Uuid> = (0..count).map(|_| Uuid::new_v4()).collect();
    world.elected_owner_ids = owner_ids.clone();

    let candidates = owner_ids
        .into_iter()
        .enumerate()
        .map(|(i, owner_id)| CdcCandidateDto {
            owner_id: owner_id.to_string(),
            position: if i == 0 {
                "president".to_string()
            } else {
                "member".to_string()
            },
        })
        .collect();

    let dto = ElectCdcMembersDto {
        meeting_id: meeting_id.to_string(),
        candidates,
    };

    let uc = world.use_cases.as_ref().unwrap().clone();
    world.election_result = Some(uc.elect_members(building_id, dto).await);
}

#[when("the elected member resigns")]
async fn when_resign(world: &mut CdcAlertWorld) {
    let owner_id = world.elected_owner_ids[0];
    let building_id = world.last_building_id.expect("a building must exist");
    let repo = world.member_repo.as_ref().unwrap();
    let mut member = repo
        .find_by_owner_and_building(owner_id, building_id)
        .await
        .unwrap()
        .expect("elected member must exist");
    member.resign(Utc::now());
    repo.update(&member).await.unwrap();
}

#[when(
    regex = r#"^(the first elected member|a non-elected owner) creates an alert "([^"]+)" with severity "([^"]+)" targeting "([^"]+)"$"#
)]
async fn when_create_alert(
    world: &mut CdcAlertWorld,
    actor: String,
    text: String,
    severity: String,
    meeting_name: String,
) {
    let building_id = world.last_building_id.expect("a building must exist");
    let owner_id = if actor == "the first elected member" {
        world.elected_owner_ids[0]
    } else {
        Uuid::new_v4() // jamais élu sur cet immeuble
    };
    let target_meeting_id = *world
        .meetings
        .get(&meeting_name)
        .expect("meeting must exist");

    let dto = CreateBoardAlertDto {
        text,
        severity,
        target_meeting_id: target_meeting_id.to_string(),
    };

    let uc = world.use_cases.as_ref().unwrap().clone();
    world.alert_result = Some(uc.create_alert(owner_id, building_id, dto).await);
}

// ============================================================================
// Then
// ============================================================================

#[then(regex = r#"^the election should succeed with (\d+) members?$"#)]
fn then_election_succeeds(world: &mut CdcAlertWorld, count: usize) {
    match world.election_result.as_ref() {
        Some(Ok(members)) => assert_eq!(members.len(), count),
        other => panic!("expected Ok with {count} members, got {:?}", other),
    }
}

#[then("the election should fail with a quorum not reached error")]
fn then_election_fails_quorum(world: &mut CdcAlertWorld) {
    match world.election_result.as_ref() {
        Some(Err(AppError::CdcElectionQuorumNotReached { .. })) => {}
        other => panic!("expected CdcElectionQuorumNotReached, got {:?}", other),
    }
}

#[then("the alert operation should succeed")]
fn then_alert_succeeds(world: &mut CdcAlertWorld) {
    match world.alert_result.as_ref() {
        Some(Ok(_)) => {}
        other => panic!("expected Ok, got {:?}", other),
    }
}

#[then("the alert operation should fail with a forbidden error")]
fn then_alert_fails_forbidden(world: &mut CdcAlertWorld) {
    match world.alert_result.as_ref() {
        Some(Err(AppError::Forbidden(_))) => {}
        other => panic!("expected Forbidden, got {:?}", other),
    }
}

#[then(regex = r#"^the alert should be visible at "([^"]+)"$"#)]
async fn then_alert_visible_at(world: &mut CdcAlertWorld, meeting_name: String) {
    let meeting_id = *world
        .meetings
        .get(&meeting_name)
        .expect("meeting must exist");
    let uc = world.use_cases.as_ref().unwrap().clone();
    let alerts = uc.list_alerts_for_meeting(meeting_id).await.unwrap();
    let expected_id = match world.alert_result.as_ref() {
        Some(Ok(a)) => a.id.clone(),
        other => panic!("expected a created alert, got {:?}", other),
    };
    assert!(
        alerts.iter().any(|a| a.id == expected_id),
        "alert {expected_id} should be visible at {meeting_name}"
    );
}

#[tokio::main]
async fn main() {
    use cucumber::writer::Stats as _;
    let writer = CdcAlertWorld::cucumber()
        .run("tests/features/cdc_alert.feature")
        .await;
    if writer.execution_has_failed() {
        std::process::exit(1);
    }
}
