//! Story 4.1 — Runner BDD pour `meeting_hybrid_quorum.feature`.
//!
//! Teste la logique pure `compute_quorum()` et `Meeting::set_mode()` sans
//! dépendance DB / testcontainers — même pattern que `bdd_meeting_complete.rs`
//! (Track H Story H3).
//!
//! Couvre la taxonomie 4-cat exigée par CRITICAL.md §3 :
//! `@happy` + `@edge` + `@security` + `@negative`.

use chrono::{Duration, Utc};
use cucumber::{given, then, when, World};
use koprogo_api::application::error::AppError;
use koprogo_api::application::use_cases::compute_quorum_use_case::{
    compute_quorum, ComputeQuorumInput, QuorumResult,
};
use koprogo_api::domain::entities::{Meeting, MeetingMode, MeetingModeError, MeetingType};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Default, World)]
pub struct MeetingHybridQuorumWorld {
    pub meeting: Option<Meeting>,
    pub mode: Option<MeetingMode>,
    pub in_person_quotas: Decimal,
    pub remote_quotas: Decimal,
    pub proxy_quotas: Decimal,
    pub total_quotas: Decimal,
    pub remote_strong_auth_confirmed: bool,
    pub quorum_result: Option<Result<QuorumResult, AppError>>,
    pub mode_result: Option<Result<(), MeetingModeError>>,
}

fn parse_mode(s: &str) -> MeetingMode {
    match s {
        "remote" => MeetingMode::Remote,
        "hybrid" => MeetingMode::Hybrid,
        _ => MeetingMode::InPerson,
    }
}

#[given("a coproperty management system")]
fn given_system(_world: &mut MeetingHybridQuorumWorld) {
    // No-op : fixture context (mêmes conventions que bdd_meeting_complete.rs).
}

#[given(regex = r"^a meeting mode (in_person|remote|hybrid)$")]
fn given_mode(world: &mut MeetingHybridQuorumWorld, mode: String) {
    world.mode = Some(parse_mode(&mode));
    world.remote_strong_auth_confirmed = true;
}

#[given(regex = r"^un total de (-?\d+(?:\.\d+)?) quotités$")]
fn given_total(world: &mut MeetingHybridQuorumWorld, total: String) {
    world.total_quotas = Decimal::from_str(&total).expect("total quotas must parse");
}

#[given(
    regex = r"^(-?\d+(?:\.\d+)?) présentiels, (-?\d+(?:\.\d+)?) distants et (-?\d+(?:\.\d+)?) procurations$"
)]
fn given_attendance(
    world: &mut MeetingHybridQuorumWorld,
    in_person: String,
    remote: String,
    proxy: String,
) {
    world.in_person_quotas = Decimal::from_str(&in_person).expect("in_person quotas must parse");
    world.remote_quotas = Decimal::from_str(&remote).expect("remote quotas must parse");
    world.proxy_quotas = Decimal::from_str(&proxy).expect("proxy quotas must parse");
}

#[given("aucune authentification forte distancielle")]
fn given_no_strong_auth(world: &mut MeetingHybridQuorumWorld) {
    world.remote_strong_auth_confirmed = false;
}

#[given(regex = r#"^une réunion planifiée "(.+)"$"#)]
fn given_scheduled_meeting(world: &mut MeetingHybridQuorumWorld, title: String) {
    let meeting = Meeting::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        MeetingType::Ordinary,
        title,
        None,
        Utc::now() + Duration::days(30),
        "Salle des fêtes".to_string(),
    )
    .expect("meeting must build with valid fixtures");
    world.meeting = Some(meeting);
}

#[when("le quorum agrégé est calculé")]
fn when_compute_quorum(world: &mut MeetingHybridQuorumWorld) {
    let input = ComputeQuorumInput {
        mode: world.mode.expect("mode must be set"),
        in_person_quotas: world.in_person_quotas,
        remote_quotas: world.remote_quotas,
        proxy_quotas: world.proxy_quotas,
        total_quotas: world.total_quotas,
        remote_strong_auth_confirmed: world.remote_strong_auth_confirmed,
    };
    world.quorum_result = Some(compute_quorum(input));
}

#[when(
    regex = r"^le syndic configure le mode (in_person|remote|hybrid) sans URL de visioconférence$"
)]
fn when_set_mode_without_url(world: &mut MeetingHybridQuorumWorld, mode: String) {
    let result = world
        .meeting
        .as_mut()
        .expect("meeting must exist")
        .set_mode(parse_mode(&mode), None);
    world.mode_result = Some(result);
}

#[when(regex = r#"^le syndic configure le mode (in_person|remote|hybrid) avec l'URL "(.+)"$"#)]
fn when_set_mode_with_url(world: &mut MeetingHybridQuorumWorld, mode: String, url: String) {
    let result = world
        .meeting
        .as_mut()
        .expect("meeting must exist")
        .set_mode(parse_mode(&mode), Some(url));
    world.mode_result = Some(result);
}

#[then("le quorum agrégé est conforme à la somme Decimal exacte")]
fn then_quorum_sum(world: &mut MeetingHybridQuorumWorld) {
    let result = world
        .quorum_result
        .as_ref()
        .expect("result must exist")
        .as_ref()
        .expect("expected Ok");
    let expected = world.in_person_quotas + world.remote_quotas + world.proxy_quotas;
    assert_eq!(result.attended_quotas, expected);
}

#[then("le quorum est atteint")]
fn then_quorum_reached(world: &mut MeetingHybridQuorumWorld) {
    let result = world
        .quorum_result
        .as_ref()
        .expect("result must exist")
        .as_ref()
        .expect("expected Ok");
    assert!(result.quorum_reached);
}

#[then("le quorum n'est pas atteint")]
fn then_quorum_not_reached(world: &mut MeetingHybridQuorumWorld) {
    let result = world
        .quorum_result
        .as_ref()
        .expect("result must exist")
        .as_ref()
        .expect("expected Ok");
    assert!(!result.quorum_reached);
}

#[then(regex = r"^le pourcentage de quorum vaut exactement (-?\d+(?:\.\d+)?)%$")]
fn then_percentage_exact(world: &mut MeetingHybridQuorumWorld, pct: String) {
    let result = world
        .quorum_result
        .as_ref()
        .expect("result must exist")
        .as_ref()
        .expect("expected Ok");
    assert_eq!(
        result.quorum_percentage,
        Decimal::from_str(&pct).expect("percentage must parse")
    );
}

#[then("le calcul est refusé pour authentification distancielle manquante")]
fn then_forbidden_auth(world: &mut MeetingHybridQuorumWorld) {
    let err = world
        .quorum_result
        .as_ref()
        .expect("result must exist")
        .as_ref()
        .expect_err("expected Err");
    assert!(matches!(err, AppError::Forbidden(_)));
}

#[then("la configuration du mode échoue avec URL de visioconférence manquante")]
fn then_mode_fails_missing_url(world: &mut MeetingHybridQuorumWorld) {
    let err = world
        .mode_result
        .as_ref()
        .expect("result must exist")
        .as_ref()
        .expect_err("expected Err");
    assert!(matches!(err, MeetingModeError::VideoconfUrlRequired { .. }));
}

#[then("la configuration du mode réussit")]
fn then_mode_succeeds(world: &mut MeetingHybridQuorumWorld) {
    let result = world.mode_result.as_ref().expect("result must exist");
    assert!(result.is_ok(), "expected Ok, got {:?}", result);
}

#[tokio::main]
async fn main() {
    MeetingHybridQuorumWorld::cucumber()
        .run_and_exit("tests/features/meeting_hybrid_quorum.feature")
        .await;
}
