//! BDD/Cucumber — Story 5.5 (#589) : comptable (encodeur ET émetteur) exclu
//! des routes communautaires, sauf cumul d'un rôle owner (ADR 0052, INV-6).
//!
//! Contrairement à `bdd_list_buildings_role_based.rs`, ce harnais n'a pas
//! besoin de testcontainers Postgres : la décision d'accès
//! (`community_access_denied`) et le filtrage de chemin (`is_community_path`)
//! sont des fonctions pures du middleware `community_access_guard`, exercées
//! ici directement sur des `UserRoleAssignment` construits en mémoire — la
//! même logique que celle appelée par le `Transform` Actix une fois les
//! rôles cumulés chargés depuis `UserUseCases::list_assignments_for_user`.

use cucumber::{given, then, when, World};
use koprogo_api::domain::entities::{UserRole, UserRoleAssignment};
use koprogo_api::infrastructure::web::middleware::community_access_guard::{
    community_access_denied, is_community_path,
};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Decision {
    Allowed,
    Denied,
    NotApplicable,
}

#[derive(Debug, Default, World)]
pub struct CommunityGuardWorld {
    assignments: Vec<UserRoleAssignment>,
    decision: Option<Decision>,
}

#[given("un middleware de garde d'accès communautaire")]
fn given_guard(_world: &mut CommunityGuardWorld) {
    // No-op : le garde est une fonction pure, rien à démarrer.
}

#[given(regex = r#"^\w+ a uniquement le rôle "([^"]+)"$"#)]
fn given_only_role(world: &mut CommunityGuardWorld, role: String) {
    let parsed = UserRole::from_str(&role).expect("rôle valide");
    world.assignments = vec![UserRoleAssignment::new(Uuid::new_v4(), parsed, None, true)];
}

#[given(regex = r#"^\w+ a le rôle "([^"]+)" et le rôle "([^"]+)"$"#)]
fn given_two_roles(world: &mut CommunityGuardWorld, role1: String, role2: String) {
    let user_id = Uuid::new_v4();
    let r1 = UserRole::from_str(&role1).expect("rôle 1 valide");
    let r2 = UserRole::from_str(&role2).expect("rôle 2 valide");
    world.assignments = vec![
        UserRoleAssignment::new(user_id, r1, None, true),
        UserRoleAssignment::new(user_id, r2, None, false),
    ];
}

#[when(regex = r#"^\w+ appelle une route communautaire$"#)]
fn when_calls_community_route(world: &mut CommunityGuardWorld) {
    let path = "/api/v1/exchanges";
    assert!(
        is_community_path(path),
        "chemin de test attendu comme route communautaire"
    );
    world.decision = Some(if community_access_denied(&world.assignments) {
        Decision::Denied
    } else {
        Decision::Allowed
    });
}

#[when(regex = r#"^\w+ appelle directement l'URL "([^"]+)" en contournant l'UI$"#)]
fn when_calls_url_directly(world: &mut CommunityGuardWorld, url: String) {
    assert!(
        is_community_path(&url),
        "l'URL {url} devrait être reconnue comme route communautaire"
    );
    world.decision = Some(if community_access_denied(&world.assignments) {
        Decision::Denied
    } else {
        Decision::Allowed
    });
}

#[when(regex = r#"^\w+ appelle l'URL "([^"]+)"$"#)]
fn when_calls_url(world: &mut CommunityGuardWorld, url: String) {
    world.decision = Some(if !is_community_path(&url) {
        Decision::NotApplicable
    } else if community_access_denied(&world.assignments) {
        Decision::Denied
    } else {
        Decision::Allowed
    });
}

#[then("l'accès est autorisé")]
fn then_allowed(world: &mut CommunityGuardWorld) {
    assert_eq!(world.decision, Some(Decision::Allowed));
}

#[then("l'accès est refusé avec INV-6")]
fn then_denied(world: &mut CommunityGuardWorld) {
    assert_eq!(world.decision, Some(Decision::Denied));
}

#[then("le garde ne s'applique pas à cette route")]
fn then_not_applicable(world: &mut CommunityGuardWorld) {
    assert_eq!(world.decision, Some(Decision::NotApplicable));
}

#[tokio::main]
async fn main() {
    use cucumber::writer::Stats as _;
    let writer = CommunityGuardWorld::cucumber()
        .run("tests/features/community_accountant_forbidden.feature")
        .await;
    if writer.execution_has_failed() {
        std::process::exit(1);
    }
}
