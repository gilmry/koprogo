//! Story 4.6 — Runner BDD pour `resolution_auto_evaluation.feature`.
//!
//! Teste `GenerateAgoResolutionsUseCase` et la garde
//! `ResolutionAutoNotRemovable` de `ResolutionUseCases` sans dépendance DB /
//! testcontainers — même pattern que `bdd_meeting_hybrid_quorum.rs` : des
//! dépôts en mémoire, aucun réseau, aucune base.
//!
//! Couvre la taxonomie 4-cat exigée par CRITICAL.md §3 :
//! `@happy` + `@edge` + `@security` + `@negative`.

use async_trait::async_trait;
use chrono::{Duration, Utc};
use cucumber::{given, then, when, World};
use koprogo_api::application::error::AppError;
use koprogo_api::application::ports::{
    MeetingRepository, ResolutionRepository, UnitOwnerRepository, UnitRepository, VoteRepository,
};
use koprogo_api::application::use_cases::{GenerateAgoResolutionsUseCase, ResolutionUseCases};
use koprogo_api::domain::entities::{
    LotHolder, Meeting, MeetingType, Resolution, ResolutionKind, ResolutionStatus, Unit, UnitOwner,
    Vote,
};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// ============================================================
// Dépôts en mémoire — MeetingRepository / ResolutionRepository
// ============================================================

#[derive(Debug)]
struct FakeMeetingRepository {
    meetings: Mutex<HashMap<Uuid, Meeting>>,
}

impl FakeMeetingRepository {
    fn new() -> Self {
        Self {
            meetings: Mutex::new(HashMap::new()),
        }
    }

    fn pose(&self, meeting: Meeting) {
        self.meetings.lock().unwrap().insert(meeting.id, meeting);
    }
}

#[async_trait]
impl MeetingRepository for FakeMeetingRepository {
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
    async fn find_by_building(&self, _building_id: Uuid) -> Result<Vec<Meeting>, String> {
        Ok(vec![])
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
        _page_request: &koprogo_api::application::dto::PageRequest,
        _organization_id: Option<Uuid>,
    ) -> Result<(Vec<Meeting>, i64), String> {
        Ok((vec![], 0))
    }
}

#[derive(Debug)]
struct FakeResolutionRepository {
    resolutions: Mutex<HashMap<Uuid, Resolution>>,
}

impl FakeResolutionRepository {
    fn new() -> Self {
        Self {
            resolutions: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl ResolutionRepository for FakeResolutionRepository {
    async fn create(&self, resolution: &Resolution) -> Result<Resolution, String> {
        self.resolutions
            .lock()
            .unwrap()
            .insert(resolution.id, resolution.clone());
        Ok(resolution.clone())
    }
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Resolution>, String> {
        Ok(self.resolutions.lock().unwrap().get(&id).cloned())
    }
    async fn find_by_meeting_id(&self, meeting_id: Uuid) -> Result<Vec<Resolution>, String> {
        Ok(self
            .resolutions
            .lock()
            .unwrap()
            .values()
            .filter(|r| r.meeting_id == meeting_id)
            .cloned()
            .collect())
    }
    async fn find_by_status(&self, status: ResolutionStatus) -> Result<Vec<Resolution>, String> {
        Ok(self
            .resolutions
            .lock()
            .unwrap()
            .values()
            .filter(|r| r.status == status)
            .cloned()
            .collect())
    }
    async fn update(&self, resolution: &Resolution) -> Result<Resolution, String> {
        self.resolutions
            .lock()
            .unwrap()
            .insert(resolution.id, resolution.clone());
        Ok(resolution.clone())
    }
    async fn delete(&self, id: Uuid) -> Result<bool, String> {
        Ok(self.resolutions.lock().unwrap().remove(&id).is_some())
    }
    async fn update_vote_counts(
        &self,
        _resolution_id: Uuid,
        _vote_count_pour: i32,
        _vote_count_contre: i32,
        _vote_count_abstention: i32,
        _total_voting_power_pour: Decimal,
        _total_voting_power_contre: Decimal,
        _total_voting_power_abstention: Decimal,
    ) -> Result<(), String> {
        Ok(())
    }
    async fn close_voting(
        &self,
        _resolution_id: Uuid,
        _final_status: ResolutionStatus,
        _voix_plafonnees: Option<serde_json::Value>,
    ) -> Result<(), String> {
        Ok(())
    }
    async fn get_meeting_vote_summary(&self, meeting_id: Uuid) -> Result<Vec<Resolution>, String> {
        self.find_by_meeting_id(meeting_id).await
    }
}

// ============================================================
// Dépôts en mémoire — jamais exercés par delete/update_resolution, mais
// requis par la signature de `ResolutionUseCases::new`.
// ============================================================

#[derive(Debug)]
struct FakeVoteRepository;

#[async_trait]
impl VoteRepository for FakeVoteRepository {
    async fn create(&self, vote: &Vote) -> Result<Vote, String> {
        Ok(vote.clone())
    }
    async fn find_by_id(&self, _id: Uuid) -> Result<Option<Vote>, String> {
        Ok(None)
    }
    async fn find_by_resolution_id(&self, _resolution_id: Uuid) -> Result<Vec<Vote>, String> {
        Ok(vec![])
    }
    async fn find_by_owner_id(&self, _owner_id: Uuid) -> Result<Vec<Vote>, String> {
        Ok(vec![])
    }
    async fn find_by_resolution_and_unit(
        &self,
        _resolution_id: Uuid,
        _unit_id: Uuid,
    ) -> Result<Option<Vote>, String> {
        Ok(None)
    }
    async fn has_voted(&self, _resolution_id: Uuid, _unit_id: Uuid) -> Result<bool, String> {
        Ok(false)
    }
    async fn update(&self, vote: &Vote) -> Result<Vote, String> {
        Ok(vote.clone())
    }
    async fn delete(&self, _id: Uuid) -> Result<bool, String> {
        Ok(true)
    }
    async fn count_by_resolution_and_choice(
        &self,
        _resolution_id: Uuid,
    ) -> Result<(i32, i32, i32), String> {
        Ok((0, 0, 0))
    }
    async fn sum_voting_power_by_resolution(
        &self,
        _resolution_id: Uuid,
    ) -> Result<(Decimal, Decimal, Decimal), String> {
        Ok((Decimal::ZERO, Decimal::ZERO, Decimal::ZERO))
    }
    async fn count_proxy_votes_for_mandataire(
        &self,
        _resolution_id: Uuid,
        _proxy_owner_id: Uuid,
    ) -> Result<(i64, Decimal), String> {
        Ok((0, Decimal::ZERO))
    }
}

#[derive(Debug)]
struct FakeUnitOwnerRepository;

#[async_trait]
impl UnitOwnerRepository for FakeUnitOwnerRepository {
    async fn create(&self, unit_owner: &UnitOwner) -> Result<UnitOwner, String> {
        Ok(unit_owner.clone())
    }
    async fn find_by_id(&self, _id: Uuid) -> Result<Option<UnitOwner>, String> {
        Ok(None)
    }
    async fn find_current_owners_by_unit(&self, _unit_id: Uuid) -> Result<Vec<UnitOwner>, String> {
        Ok(vec![])
    }
    async fn find_current_units_by_owner(&self, _owner_id: Uuid) -> Result<Vec<UnitOwner>, String> {
        Ok(vec![])
    }
    async fn find_all_owners_by_unit(&self, _unit_id: Uuid) -> Result<Vec<UnitOwner>, String> {
        Ok(vec![])
    }
    async fn find_all_units_by_owner(&self, _owner_id: Uuid) -> Result<Vec<UnitOwner>, String> {
        Ok(vec![])
    }
    async fn update(&self, unit_owner: &UnitOwner) -> Result<UnitOwner, String> {
        Ok(unit_owner.clone())
    }
    async fn delete(&self, _id: Uuid) -> Result<(), String> {
        Ok(())
    }
    async fn has_active_owners(&self, _unit_id: Uuid) -> Result<bool, String> {
        Ok(true)
    }
    async fn get_total_ownership_percentage(&self, _unit_id: Uuid) -> Result<Decimal, String> {
        Ok(Decimal::ONE)
    }
    async fn find_active_by_unit_and_owner(
        &self,
        _unit_id: Uuid,
        _owner_id: Uuid,
    ) -> Result<Option<UnitOwner>, String> {
        Ok(None)
    }
    async fn find_active_by_building(
        &self,
        _building_id: Uuid,
    ) -> Result<Vec<(Uuid, Uuid, Decimal)>, String> {
        Ok(vec![])
    }
    async fn find_active_quota_shares_by_building(
        &self,
        _building_id: Uuid,
    ) -> Result<Vec<(Uuid, Uuid, Decimal)>, String> {
        Ok(vec![])
    }
    async fn find_voting_holders_by_unit(&self, _unit_id: Uuid) -> Result<Vec<LotHolder>, String> {
        Ok(vec![])
    }
    async fn is_voting_representative(&self, _unit_owner_id: Uuid) -> Result<bool, String> {
        Ok(false)
    }
    async fn set_voting_representative(&self, _unit_owner_id: Uuid) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug)]
struct FakeUnitRepository;

#[async_trait]
impl UnitRepository for FakeUnitRepository {
    async fn create(&self, unit: &Unit) -> Result<Unit, String> {
        Ok(unit.clone())
    }
    async fn find_by_id(&self, _id: Uuid) -> Result<Option<Unit>, String> {
        Ok(None)
    }
    async fn find_by_building(&self, _building_id: Uuid) -> Result<Vec<Unit>, String> {
        Ok(vec![])
    }
    async fn find_by_owner(&self, _owner_id: Uuid) -> Result<Vec<Unit>, String> {
        Ok(vec![])
    }
    async fn find_all_paginated(
        &self,
        _page_request: &koprogo_api::application::dto::PageRequest,
        _filters: &koprogo_api::application::dto::UnitFilters,
    ) -> Result<(Vec<Unit>, i64), String> {
        Ok((vec![], 0))
    }
    async fn update(&self, unit: &Unit) -> Result<Unit, String> {
        Ok(unit.clone())
    }
    async fn delete(&self, _id: Uuid) -> Result<bool, String> {
        Ok(true)
    }
}

// ============================================================
// World
// ============================================================

// `Debug` est exigé par `cucumber::World` — sans lui, ce harnais ne compile
// pas. Il ne l'avait jamais fait : rien ne l'exécutait, donc rien ne le
// compilait. Le câbler dans `ci.yml` le 2026-09-16 l'a révélé, ce qui est
// exactement la fonction de `garde_harnais_executes` :
//
//     Ils compilent, ils passent en local, et la CI reste verte sans les
//     avoir vus.
//
// Ici, ils ne compilaient même pas.
#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct ResolutionAutoEvaluationWorld {
    meeting_repo: Arc<FakeMeetingRepository>,
    resolution_repo: Arc<FakeResolutionRepository>,
    vote_repo: Arc<FakeVoteRepository>,
    unit_owner_repo: Arc<FakeUnitOwnerRepository>,
    unit_repo: Arc<FakeUnitRepository>,
    meeting_id: Uuid,
    generated: Vec<Resolution>,
    last_error: Option<AppError>,
}

impl ResolutionAutoEvaluationWorld {
    fn new() -> Self {
        Self {
            meeting_repo: Arc::new(FakeMeetingRepository::new()),
            resolution_repo: Arc::new(FakeResolutionRepository::new()),
            vote_repo: Arc::new(FakeVoteRepository),
            unit_owner_repo: Arc::new(FakeUnitOwnerRepository),
            unit_repo: Arc::new(FakeUnitRepository),
            meeting_id: Uuid::nil(),
            generated: Vec::new(),
            last_error: None,
        }
    }

    fn generate_use_case(&self) -> GenerateAgoResolutionsUseCase {
        GenerateAgoResolutionsUseCase::new(self.meeting_repo.clone(), self.resolution_repo.clone())
    }

    fn resolution_use_cases(&self) -> ResolutionUseCases {
        ResolutionUseCases::new(
            self.resolution_repo.clone(),
            self.vote_repo.clone(),
            self.meeting_repo.clone(),
            self.unit_owner_repo.clone(),
            self.unit_repo.clone(),
        )
    }
}

fn nouvelle_reunion(meeting_type: MeetingType) -> Meeting {
    Meeting::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        meeting_type,
        "Assemblée de test".to_string(),
        None,
        Utc::now() + Duration::days(30),
        "Salle des fêtes".to_string(),
    )
    .expect("réunion de test valide")
}

// ============================================================
// Given
// ============================================================

#[given("une assemblée générale ordinaire")]
async fn given_ago(world: &mut ResolutionAutoEvaluationWorld) {
    let meeting = nouvelle_reunion(MeetingType::Ordinary);
    world.meeting_id = meeting.id;
    world.meeting_repo.pose(meeting);
}

#[given("une assemblée générale extraordinaire")]
async fn given_age(world: &mut ResolutionAutoEvaluationWorld) {
    let meeting = nouvelle_reunion(MeetingType::Extraordinary);
    world.meeting_id = meeting.id;
    world.meeting_repo.pose(meeting);
}

async fn generer_resolutions(world: &mut ResolutionAutoEvaluationWorld) {
    let uc = world.generate_use_case();
    match uc.generate_ago_resolutions(world.meeting_id).await {
        Ok(v) => {
            world.generated = v;
            world.last_error = None;
        }
        Err(e) => world.last_error = Some(e),
    }
}

#[given("les résolutions automatiques sont générées")]
async fn given_generation(world: &mut ResolutionAutoEvaluationWorld) {
    generer_resolutions(world).await;
}

// ============================================================
// When
// ============================================================

#[when("les résolutions automatiques sont générées")]
async fn when_generation(world: &mut ResolutionAutoEvaluationWorld) {
    generer_resolutions(world).await;
}

#[when("le syndic tente de supprimer la résolution auto-générée")]
async fn when_delete_auto(world: &mut ResolutionAutoEvaluationWorld) {
    let id = world
        .generated
        .first()
        .expect("une résolution auto-générée doit exister")
        .id;
    let uc = world.resolution_use_cases();
    world.last_error = uc.delete_resolution(id).await.err();
}

#[when("le syndic modifie le texte de la résolution auto-générée")]
async fn when_update_auto(world: &mut ResolutionAutoEvaluationWorld) {
    let mut modifiee = world
        .generated
        .first()
        .expect("une résolution auto-générée doit exister")
        .clone();
    modifiee.description = "Texte réécrit par le syndic".to_string();
    let uc = world.resolution_use_cases();
    world.last_error = uc.update_resolution(&modifiee).await.err();
}

// ============================================================
// Then
// ============================================================

#[then("une résolution \"EvaluationContractorsAuto\" est présente")]
async fn then_resolution_present(world: &mut ResolutionAutoEvaluationWorld) {
    assert_eq!(world.generated.len(), 1, "une seule résolution attendue");
    assert_eq!(
        world.generated[0].kind,
        ResolutionKind::EvaluationContractorsAuto
    );
}

#[then("elle est marquée comme générée automatiquement")]
async fn then_is_auto_generated(world: &mut ResolutionAutoEvaluationWorld) {
    assert!(world.generated[0].is_auto_generated());
}

#[then("aucune résolution automatique n'est générée")]
async fn then_none_generated(world: &mut ResolutionAutoEvaluationWorld) {
    assert!(world.generated.is_empty());
}

#[then("la suppression est refusée avec l'erreur \"ResolutionAutoNotRemovable\"")]
async fn then_delete_refused(world: &mut ResolutionAutoEvaluationWorld) {
    assert!(
        matches!(world.last_error, Some(AppError::ResolutionAutoNotRemovable)),
        "attendu ResolutionAutoNotRemovable, got: {:?}",
        world.last_error
    );
}

#[then("la modification est refusée avec l'erreur \"ResolutionAutoNotRemovable\"")]
async fn then_update_refused(world: &mut ResolutionAutoEvaluationWorld) {
    assert!(
        matches!(world.last_error, Some(AppError::ResolutionAutoNotRemovable)),
        "attendu ResolutionAutoNotRemovable, got: {:?}",
        world.last_error
    );
}

#[tokio::main]
async fn main() {
    ResolutionAutoEvaluationWorld::cucumber()
        .run_and_exit("tests/features/resolution_auto_evaluation.feature")
        .await;
}
