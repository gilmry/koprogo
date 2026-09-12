//! Track H Story H3 — Runner BDD pour `meeting_complete.feature`.
//!
//! Teste la logique pure `Meeting::assert_can_complete(&checklist)` sans
//! dépendance DB / testcontainers (le port `MeetingCompletionCheckerPort`
//! est mocké via une checklist fabriquée pas-à-pas par les `Given` steps).
//!
//! Couvre la taxonomie 4-cat exigée par CRITICAL.md §3 :
//! `@happy` + `@edge` + `@security` + `@negative`.

use chrono::{Duration, Utc};
use cucumber::{given, then, when, World};
use koprogo_api::domain::entities::{
    Meeting, MeetingCompletionChecklist, MeetingNotCompletableError, MeetingType,
};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Default, World)]
pub struct MeetingCompleteWorld {
    pub meeting: Option<Meeting>,
    pub checklist: Option<MeetingCompletionChecklist>,
    pub last_result: Option<Result<(), MeetingNotCompletableError>>,
}

#[given("a coproperty management system")]
fn given_system(_world: &mut MeetingCompleteWorld) {
    // No-op : fixture context.
}

#[given(regex = r#"^a scheduled meeting "(.+)"$"#)]
fn given_scheduled_meeting(world: &mut MeetingCompleteWorld, title: String) {
    // Fixture pure : ni base ni immeuble réel, donc l'ACP est tirée au sort
    // comme l'organisation et l'immeuble. Ce scénario porte sur les invariants
    // de clôture (Art. 3.87 § 5), pas sur le rattachement.
    let acp_id = Uuid::new_v4();
    let org_id = Uuid::new_v4();
    let building_id = Uuid::new_v4();
    let future_date = Utc::now() + Duration::days(30);
    let meeting = Meeting::new(
        acp_id,
        org_id,
        building_id,
        MeetingType::Ordinary,
        title,
        None,
        future_date,
        "Salle des fêtes".to_string(),
    )
    .expect("meeting must build with valid fixtures");
    world.meeting = Some(meeting);
}

/// Parse une checklist depuis un step Gherkin textuel.
///
/// Format reconnu :
/// "convocations (sent|NOT sent), N open resolution(s)|no open resolutions,
/// attendance (recorded|NOT recorded), quorum A of T, minutes draft (present|NOT present)"
fn parse_checklist(spec: &str) -> MeetingCompletionChecklist {
    let convocations_sent = spec.contains("convocations sent");
    let attendance_recorded = spec.contains("attendance recorded");
    let minutes_draft_exists = spec.contains("minutes draft present");

    // Parse open resolutions.
    let open_resolutions: i32 = if spec.contains("no open resolutions") {
        0
    } else {
        // "N open resolution[s]"
        let mut n = 0;
        for word in spec.split_whitespace() {
            if let Ok(parsed) = word.parse::<i32>() {
                // Heuristique : le premier nombre rencontré juste avant le mot
                // "open" est le compte des résolutions. On garde le dernier
                // candidat valide trouvé avant "open".
                if spec
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .windows(2)
                    .any(|w| w[0] == word && w[1].starts_with("open"))
                {
                    n = parsed;
                }
            }
        }
        n
    };

    // Parse quorum "quorum A of T"
    let (attended, total) = {
        let parts: Vec<&str> = spec.split("quorum ").collect();
        if parts.len() >= 2 {
            let after = parts[1];
            let nums: Vec<&str> = after
                .split(',')
                .next()
                .unwrap_or("")
                .split_whitespace()
                .collect();
            // expected : ["A", "of", "T"]
            let a = nums
                .first()
                .and_then(|s| Decimal::from_str(s).ok())
                .unwrap_or(Decimal::ZERO);
            let t = nums
                .get(2)
                .and_then(|s| Decimal::from_str(s).ok())
                .unwrap_or(Decimal::ZERO);
            (a, t)
        } else {
            (Decimal::ZERO, Decimal::ZERO)
        }
    };

    // Story H9 — parse "heads P of T" (têtes présentes / total copropriétaires).
    // Défaut : têtes OK (10/10) quand non spécifié, pour que les scénarios
    // « quotités seules » exercent le volet quotités du quorum double.
    let (present_heads, total_heads) = {
        let parts: Vec<&str> = spec.split("heads ").collect();
        if parts.len() >= 2 {
            let after = parts[1];
            let nums: Vec<&str> = after
                .split(',')
                .next()
                .unwrap_or("")
                .split_whitespace()
                .collect();
            let p = nums
                .first()
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            let t = nums.get(2).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
            (p, t)
        } else {
            (10, 10) // défaut : volet têtes satisfait
        }
    };

    MeetingCompletionChecklist {
        convocations_sent,
        open_resolutions,
        attendance_recorded,
        attended_quotas: attended,
        total_quotas: total,
        present_owners_count: present_heads,
        total_owners_count: total_heads,
        minutes_draft_exists,
    }
}

#[given(regex = r"^the completion checklist has (.+)$")]
fn given_checklist(world: &mut MeetingCompleteWorld, spec: String) {
    world.checklist = Some(parse_checklist(&spec));
}

#[when("the syndic asserts completion")]
fn when_assert(world: &mut MeetingCompleteWorld) {
    let meeting = world.meeting.as_ref().expect("meeting must exist");
    let checklist = world.checklist.as_ref().expect("checklist must exist");
    world.last_result = Some(meeting.assert_can_complete(checklist));
}

#[then("the assertion is Ok")]
fn then_ok(world: &mut MeetingCompleteWorld) {
    let r = world.last_result.as_ref().expect("result must exist");
    assert!(r.is_ok(), "expected Ok, got {:?}", r);
}

#[then(regex = r"^the assertion fails with (\d+) missing invariant[s]?$")]
fn then_fails_with_n(world: &mut MeetingCompleteWorld, n: i32) {
    let err = world
        .last_result
        .as_ref()
        .expect("result must exist")
        .as_ref()
        .expect_err("expected Err");
    assert_eq!(
        err.missing.len() as i32,
        n,
        "expected {} missing, got {} ({:?})",
        n,
        err.missing.len(),
        err.missing
    );
}

#[then(regex = r"^the assertion fails with at least (\d+) missing invariant[s]?$")]
fn then_fails_with_at_least(world: &mut MeetingCompleteWorld, n: i32) {
    let err = world
        .last_result
        .as_ref()
        .expect("result must exist")
        .as_ref()
        .expect_err("expected Err");
    assert!(
        (err.missing.len() as i32) >= n,
        "expected at least {} missing, got {} ({:?})",
        n,
        err.missing.len(),
        err.missing
    );
}

#[then(regex = r#"^the missing invariant contains "(.+)"$"#)]
fn then_contains_type(world: &mut MeetingCompleteWorld, type_name: String) {
    use koprogo_api::domain::entities::MissingInvariant;
    let err = world
        .last_result
        .as_ref()
        .expect("result must exist")
        .as_ref()
        .expect_err("expected Err");
    let found = err.missing.iter().any(|m| {
        matches!(
            (m, type_name.as_str()),
            (MissingInvariant::ConvocationsNotSent, "ConvocationsNotSent")
                | (MissingInvariant::VotesNotClosed { .. }, "VotesNotClosed")
                | (
                    MissingInvariant::AttendanceNotRecorded,
                    "AttendanceNotRecorded"
                )
                | (
                    MissingInvariant::QuorumNotReached { .. },
                    "QuorumNotReached"
                )
                | (
                    MissingInvariant::HeadCountQuorumNotReached { .. },
                    "HeadCountQuorumNotReached"
                )
                | (MissingInvariant::MinutesDraftMissing, "MinutesDraftMissing")
        )
    });
    assert!(
        found,
        "expected to find {} in missing list, got {:?}",
        type_name, err.missing
    );
}

#[tokio::main]
async fn main() {
    MeetingCompleteWorld::cucumber()
        .run_and_exit("tests/features/meeting_complete.feature")
        .await;
}
