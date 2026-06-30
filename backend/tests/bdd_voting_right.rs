//! Track H Story H17 — Runner BDD pour `voting_right_suspension.feature`.
//!
//! Teste la logique pure `voting_right_status(&[LotHolder])` et
//! `assert_single_voting_representative()` (Art. 3.87 §1 CC) sans dépendance
//! DB / testcontainers — les titularités du lot sont fabriquées par les
//! `Given` steps.
//!
//! Couvre la taxonomie 4-cat exigée par CRITICAL.md §3 :
//! `@happy` + `@edge` + `@security` + `@negative`.

use cucumber::{given, then, when, World};
use koprogo_api::domain::entities::{
    assert_single_voting_representative, voting_right_status, LotHolder, OwnershipType,
    VotingRightError, VotingRightStatus,
};
use uuid::Uuid;

#[derive(Debug, Default, World)]
pub struct VotingRightWorld {
    pub unit_id: Uuid,
    pub holders: Vec<LotHolder>,
    pub status: Option<VotingRightStatus>,
    pub single_rep_result: Option<Result<(), VotingRightError>>,
}

#[given("a lot held in full ownership by a single owner")]
fn given_full_owner(world: &mut VotingRightWorld) {
    world.unit_id = Uuid::new_v4();
    world.holders = vec![LotHolder::new(OwnershipType::FullOwner, false)];
}

#[given("a dismembered lot with the usufructuary designated as representative")]
fn given_dismembered_with_rep(world: &mut VotingRightWorld) {
    world.unit_id = Uuid::new_v4();
    world.holders = vec![
        LotHolder::new(OwnershipType::Usufruct, true),
        LotHolder::new(OwnershipType::BareOwner, false),
    ];
}

#[given("a dismembered lot without a designated representative")]
fn given_dismembered_without_rep(world: &mut VotingRightWorld) {
    world.unit_id = Uuid::new_v4();
    world.holders = vec![
        LotHolder::new(OwnershipType::Usufruct, false),
        LotHolder::new(OwnershipType::BareOwner, false),
    ];
}

#[given("a lot held in indivision without a designated representative")]
fn given_indivision_without_rep(world: &mut VotingRightWorld) {
    world.unit_id = Uuid::new_v4();
    world.holders = vec![
        LotHolder::new(OwnershipType::Indivisaire, false),
        LotHolder::new(OwnershipType::Indivisaire, false),
    ];
}

#[given("a lot in indivision with two designated representatives")]
fn given_indivision_two_reps(world: &mut VotingRightWorld) {
    world.unit_id = Uuid::new_v4();
    world.holders = vec![
        LotHolder::new(OwnershipType::Indivisaire, true),
        LotHolder::new(OwnershipType::Indivisaire, true),
    ];
}

#[when("the voting right status is evaluated")]
fn when_evaluate_status(world: &mut VotingRightWorld) {
    world.status = Some(voting_right_status(&world.holders));
}

#[when("the single representative rule is checked")]
fn when_check_single_rep(world: &mut VotingRightWorld) {
    world.single_rep_result = Some(assert_single_voting_representative(
        world.unit_id,
        &world.holders,
    ));
}

#[then("the lot can vote")]
fn then_can_vote(world: &mut VotingRightWorld) {
    assert_eq!(
        world.status.expect("status must be evaluated"),
        VotingRightStatus::Active,
        "the lot should be allowed to vote"
    );
}

#[then("the voting right is suspended")]
fn then_suspended(world: &mut VotingRightWorld) {
    assert_eq!(
        world.status.expect("status must be evaluated"),
        VotingRightStatus::Suspended,
        "the lot voting right should be suspended (Art. 3.87 §1 CC)"
    );
}

#[then("the designation is rejected")]
fn then_designation_rejected(world: &mut VotingRightWorld) {
    let result = world
        .single_rep_result
        .as_ref()
        .expect("the single representative rule must have been checked");
    assert!(
        matches!(
            result,
            Err(VotingRightError::MultipleRepresentatives { .. })
        ),
        "designating two representatives must be rejected, got {:?}",
        result
    );
}

#[tokio::main]
async fn main() {
    VotingRightWorld::cucumber()
        .run_and_exit("tests/features/voting_right_suspension.feature")
        .await;
}
