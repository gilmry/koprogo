//! ContractorEvaluation — append-only rating of a contractor's prestation
//! on an approved [`TechnicalSpec`] (Story 3.9 — FR34 FR35 INV-21 INV-24).
//!
//! Distinct from [`ContractEvaluation`](super::contract_evaluation::ContractEvaluation),
//! which is the legacy marketplace-rating entity (Issue #276). The two live
//! side-by-side intentionally: Story 3.9 only introduces the new audit-grade
//! flow gated by a signed TechnicalSpec; the legacy free-form rating remains
//! for the marketplace use-case.
//!
//! # Workflow
//!
//! 1. A syndic (or a mandated owner) opens an evaluation against a
//!    contractor user once a [`TechnicalSpec`] has reached
//!    [`TechnicalSpecStatus::Approved`](super::technical_spec::TechnicalSpecStatus::Approved).
//! 2. The evaluator fills the 5 scores ([`EvaluationScores`]) and writes a
//!    comment.
//! 3. The row is persisted append-only — there are no public setters and a
//!    DB trigger blocks UPDATE / DELETE.
//!
//! # Invariants enforced at `new()` time
//!
//! - every score MUST be in `[1, 5]` (matches the SMALLINT `CHECK` in the
//!   migration);
//! - `linked_ticket_ids.len()` ≤ [`MAX_LINKED_TICKETS`], no duplicates;
//! - `comment.len()` ∈ `[MIN_COMMENT_LEN, MAX_COMMENT_LEN]` (after trim);
//! - `evaluator_user_id != contractor_user_id` (no self-evaluation —
//!   INV-21);
//! - no nil UUIDs (defensive — surface a 400 instead of an FK error at
//!   persistence time).
//!
//! Note: the *additional* invariant "the referenced TechnicalSpec MUST be
//! in `Approved` status" lives at the use-case boundary
//! ([`crate::application::use_cases::contractor_evaluation_use_cases`]) —
//! it is a workflow guard, not a structural one.

use crate::application::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

// ============================================================================
// Bound constants
// ============================================================================

pub const MIN_SCORE: u8 = 1;
pub const MAX_SCORE: u8 = 5;
pub const MIN_COMMENT_LEN: usize = 10;
pub const MAX_COMMENT_LEN: usize = 2000;
pub const MAX_LINKED_TICKETS: usize = 20;

// ============================================================================
// EvaluationScores
// ============================================================================

/// 5 dimensions rated on a 1..=5 Likert scale. The "overall" dimension is
/// stored explicitly (not computed) because the evaluator may weigh the
/// dimensions differently in their head — we don't want to surface a stale
/// arithmetic average as the headline number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationScores {
    /// Workmanship quality (1 = bad, 5 = excellent).
    pub quality: u8,
    /// Did they hit the agreed timeline?
    pub timeliness: u8,
    /// Were they reachable / responsive?
    pub communication: u8,
    /// Did the final invoice match the original quote?
    pub cost_compliance: u8,
    /// Headline opinion — what the evaluator would tell a peer.
    pub overall: u8,
}

impl EvaluationScores {
    /// True iff every dimension lies in `[MIN_SCORE, MAX_SCORE]`.
    fn is_within_bounds(&self) -> bool {
        let in_bounds = |s: u8| s >= MIN_SCORE && s <= MAX_SCORE;
        in_bounds(self.quality)
            && in_bounds(self.timeliness)
            && in_bounds(self.communication)
            && in_bounds(self.cost_compliance)
            && in_bounds(self.overall)
    }

    /// Arithmetic mean across the 5 dimensions. Useful for ranking but not
    /// authoritative — the evaluator's `overall` is the headline.
    pub fn average(&self) -> f64 {
        let sum = self.quality as u32
            + self.timeliness as u32
            + self.communication as u32
            + self.cost_compliance as u32
            + self.overall as u32;
        sum as f64 / 5.0
    }
}

// ============================================================================
// ContractorEvaluation entity
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractorEvaluation {
    pub id: Uuid,
    /// User id of the contractor being evaluated (role = Contractor).
    pub contractor_user_id: Uuid,
    /// REQUIRED — gating signed TechnicalSpec (Story 3.8). The application
    /// layer additionally checks the spec is in status `Approved`.
    pub technical_spec_id: Uuid,
    /// Tickets that motivated the evaluation (works carried out, complaints
    /// handled, etc.). Bounded to [`MAX_LINKED_TICKETS`] entries, no
    /// duplicates.
    pub linked_ticket_ids: Vec<Uuid>,
    /// Syndic or mandated owner who signed off the evaluation.
    pub evaluator_user_id: Uuid,
    pub scores: EvaluationScores,
    /// Free-form justification (`[10, 2000]` chars). Audit-grade — the
    /// contractor can request access to it via the GDPR data export
    /// endpoints.
    pub comment: String,
    pub created_at: DateTime<Utc>,
}

impl ContractorEvaluation {
    /// Build a new ContractorEvaluation. Enforces every structural invariant
    /// (cf. module-level docs). Workflow invariants (TechnicalSpec must be
    /// Approved, evaluator must hold a valid role) live in the use-case.
    pub fn new(
        contractor_user_id: Uuid,
        technical_spec_id: Uuid,
        linked_ticket_ids: Vec<Uuid>,
        evaluator_user_id: Uuid,
        scores: EvaluationScores,
        comment: String,
    ) -> Result<Self, AppError> {
        let trimmed_comment = comment.trim().to_string();
        Self::validate_invariants(
            contractor_user_id,
            technical_spec_id,
            &linked_ticket_ids,
            evaluator_user_id,
            &scores,
            &trimmed_comment,
        )?;

        Ok(Self {
            id: Uuid::new_v4(),
            contractor_user_id,
            technical_spec_id,
            linked_ticket_ids,
            evaluator_user_id,
            scores,
            comment: trimmed_comment,
            created_at: Utc::now(),
        })
    }

    fn validate_invariants(
        contractor_user_id: Uuid,
        technical_spec_id: Uuid,
        linked_ticket_ids: &[Uuid],
        evaluator_user_id: Uuid,
        scores: &EvaluationScores,
        comment: &str,
    ) -> Result<(), AppError> {
        // 1. Nil UUIDs — defensive guard.
        if contractor_user_id.is_nil() || technical_spec_id.is_nil() || evaluator_user_id.is_nil() {
            return Err(AppError::Validation(
                "ContractorEvaluation references must not be nil UUIDs".to_string(),
            ));
        }
        // 2. Self-evaluation — INV-21. Typed variant (not generic Validation)
        //    so handlers can surface a precise 422 + i18n message.
        if evaluator_user_id == contractor_user_id {
            return Err(AppError::EvaluatorIsContractor);
        }
        // 3. Scores within [1, 5].
        if !scores.is_within_bounds() {
            return Err(AppError::Validation(format!(
                "ContractorEvaluation scores must be in [{}, {}] for every dimension",
                MIN_SCORE, MAX_SCORE
            )));
        }
        // 4. linked_ticket_ids bound + uniqueness.
        if linked_ticket_ids.len() > MAX_LINKED_TICKETS {
            return Err(AppError::Validation(format!(
                "linked_ticket_ids must contain at most {} entries (got {})",
                MAX_LINKED_TICKETS,
                linked_ticket_ids.len()
            )));
        }
        let mut seen: HashSet<Uuid> = HashSet::with_capacity(linked_ticket_ids.len());
        for id in linked_ticket_ids {
            if id.is_nil() {
                return Err(AppError::Validation(
                    "linked_ticket_ids must not contain nil UUIDs".to_string(),
                ));
            }
            if !seen.insert(*id) {
                return Err(AppError::Validation(
                    "linked_ticket_ids must not contain duplicates".to_string(),
                ));
            }
        }
        // 5. Comment length (counted in chars, not bytes — matches the DB
        //    CHECK using length() which counts characters).
        let c_len = comment.chars().count();
        if c_len < MIN_COMMENT_LEN || c_len > MAX_COMMENT_LEN {
            return Err(AppError::Validation(format!(
                "comment length must be in [{}, {}] (got {})",
                MIN_COMMENT_LEN, MAX_COMMENT_LEN, c_len
            )));
        }
        Ok(())
    }

    /// Convenience: arithmetic mean of the 5 dimensions. Delegates to
    /// [`EvaluationScores::average`].
    pub fn average_score(&self) -> f64 {
        self.scores.average()
    }
}

// ============================================================================
// Tests — taxonomie 4 catégories obligatoire (CRITICAL.md #3, Story 3.9)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_pair() -> (Uuid, Uuid) {
        (Uuid::new_v4(), Uuid::new_v4())
    }

    fn fixture_scores_all_top() -> EvaluationScores {
        EvaluationScores {
            quality: 5,
            timeliness: 5,
            communication: 5,
            cost_compliance: 5,
            overall: 5,
        }
    }

    fn fixture_scores_all_bottom() -> EvaluationScores {
        EvaluationScores {
            quality: 1,
            timeliness: 1,
            communication: 1,
            cost_compliance: 1,
            overall: 1,
        }
    }

    fn fixture_comment_min() -> String {
        // exactly MIN_COMMENT_LEN chars
        "X".repeat(MIN_COMMENT_LEN)
    }

    fn build_ok(scores: EvaluationScores, comment: String) -> ContractorEvaluation {
        let (contractor, evaluator) = fixture_pair();
        ContractorEvaluation::new(
            contractor,
            Uuid::new_v4(),
            Vec::new(),
            evaluator,
            scores,
            comment,
        )
        .expect("valid ContractorEvaluation must be created")
    }

    // ---- @happy -------------------------------------------------------------

    #[test]
    fn happy_minimal_evaluation_all_top_scores() {
        let e = build_ok(fixture_scores_all_top(), fixture_comment_min());
        assert_eq!(e.scores.quality, 5);
        assert_eq!(e.average_score(), 5.0);
        assert!(e.linked_ticket_ids.is_empty());
    }

    #[test]
    fn happy_all_bottom_scores_accepted() {
        let e = build_ok(fixture_scores_all_bottom(), fixture_comment_min());
        assert_eq!(e.average_score(), 1.0);
    }

    #[test]
    fn happy_average_score_mid_value() {
        // (3+3+3+3+3)/5 = 3.0
        let scores = EvaluationScores {
            quality: 3,
            timeliness: 3,
            communication: 3,
            cost_compliance: 3,
            overall: 3,
        };
        let e = build_ok(scores, fixture_comment_min());
        assert_eq!(e.average_score(), 3.0);
    }

    #[test]
    fn happy_evaluator_differs_from_contractor_persisted() {
        let e = build_ok(fixture_scores_all_top(), fixture_comment_min());
        assert_ne!(e.evaluator_user_id, e.contractor_user_id);
    }

    #[test]
    fn happy_comment_is_trimmed_before_storage() {
        let (contractor, evaluator) = fixture_pair();
        let e = ContractorEvaluation::new(
            contractor,
            Uuid::new_v4(),
            Vec::new(),
            evaluator,
            fixture_scores_all_top(),
            format!("  {}  ", fixture_comment_min()),
        )
        .unwrap();
        assert!(!e.comment.starts_with(' '));
        assert!(!e.comment.ends_with(' '));
    }

    // ---- @edge --------------------------------------------------------------

    #[test]
    fn edge_exactly_max_linked_tickets_accepted() {
        let (contractor, evaluator) = fixture_pair();
        let tickets: Vec<Uuid> = (0..MAX_LINKED_TICKETS).map(|_| Uuid::new_v4()).collect();
        let res = ContractorEvaluation::new(
            contractor,
            Uuid::new_v4(),
            tickets,
            evaluator,
            fixture_scores_all_top(),
            fixture_comment_min(),
        );
        assert!(res.is_ok(), "exactly MAX_LINKED_TICKETS must succeed");
    }

    #[test]
    fn edge_one_over_max_linked_tickets_rejected() {
        let (contractor, evaluator) = fixture_pair();
        let tickets: Vec<Uuid> = (0..=MAX_LINKED_TICKETS).map(|_| Uuid::new_v4()).collect();
        let err = ContractorEvaluation::new(
            contractor,
            Uuid::new_v4(),
            tickets,
            evaluator,
            fixture_scores_all_top(),
            fixture_comment_min(),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn edge_comment_exactly_min_len_accepted() {
        let (contractor, evaluator) = fixture_pair();
        let res = ContractorEvaluation::new(
            contractor,
            Uuid::new_v4(),
            Vec::new(),
            evaluator,
            fixture_scores_all_top(),
            "X".repeat(MIN_COMMENT_LEN),
        );
        assert!(res.is_ok());
    }

    #[test]
    fn edge_comment_one_under_min_rejected() {
        let (contractor, evaluator) = fixture_pair();
        let err = ContractorEvaluation::new(
            contractor,
            Uuid::new_v4(),
            Vec::new(),
            evaluator,
            fixture_scores_all_top(),
            "X".repeat(MIN_COMMENT_LEN - 1),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn edge_comment_exactly_max_len_accepted() {
        let (contractor, evaluator) = fixture_pair();
        let res = ContractorEvaluation::new(
            contractor,
            Uuid::new_v4(),
            Vec::new(),
            evaluator,
            fixture_scores_all_top(),
            "X".repeat(MAX_COMMENT_LEN),
        );
        assert!(res.is_ok());
    }

    // ---- @security ----------------------------------------------------------

    #[test]
    fn security_evaluator_equals_contractor_returns_typed_error() {
        let same = Uuid::new_v4();
        let err = ContractorEvaluation::new(
            same,
            Uuid::new_v4(),
            Vec::new(),
            same,
            fixture_scores_all_top(),
            fixture_comment_min(),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::EvaluatorIsContractor));
    }

    #[test]
    fn security_duplicate_linked_tickets_rejected() {
        let (contractor, evaluator) = fixture_pair();
        let t = Uuid::new_v4();
        let err = ContractorEvaluation::new(
            contractor,
            Uuid::new_v4(),
            vec![t, t],
            evaluator,
            fixture_scores_all_top(),
            fixture_comment_min(),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn security_nil_uuids_rejected() {
        let evaluator = Uuid::new_v4();
        // nil contractor
        let err1 = ContractorEvaluation::new(
            Uuid::nil(),
            Uuid::new_v4(),
            Vec::new(),
            evaluator,
            fixture_scores_all_top(),
            fixture_comment_min(),
        )
        .unwrap_err();
        assert!(matches!(err1, AppError::Validation(_)));
        // nil tech_spec
        let err2 = ContractorEvaluation::new(
            Uuid::new_v4(),
            Uuid::nil(),
            Vec::new(),
            evaluator,
            fixture_scores_all_top(),
            fixture_comment_min(),
        )
        .unwrap_err();
        assert!(matches!(err2, AppError::Validation(_)));
        // nil evaluator
        let err3 = ContractorEvaluation::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Vec::new(),
            Uuid::nil(),
            fixture_scores_all_top(),
            fixture_comment_min(),
        )
        .unwrap_err();
        assert!(matches!(err3, AppError::Validation(_)));
    }

    #[test]
    fn security_nil_linked_ticket_id_rejected() {
        let (contractor, evaluator) = fixture_pair();
        let err = ContractorEvaluation::new(
            contractor,
            Uuid::new_v4(),
            vec![Uuid::nil()],
            evaluator,
            fixture_scores_all_top(),
            fixture_comment_min(),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    // ---- @negative ----------------------------------------------------------

    #[test]
    fn negative_score_zero_rejected() {
        let (contractor, evaluator) = fixture_pair();
        let scores = EvaluationScores {
            quality: 0,
            timeliness: 5,
            communication: 5,
            cost_compliance: 5,
            overall: 5,
        };
        let err = ContractorEvaluation::new(
            contractor,
            Uuid::new_v4(),
            Vec::new(),
            evaluator,
            scores,
            fixture_comment_min(),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_score_six_rejected() {
        let (contractor, evaluator) = fixture_pair();
        let scores = EvaluationScores {
            quality: 5,
            timeliness: 5,
            communication: 5,
            cost_compliance: 5,
            overall: 6,
        };
        let err = ContractorEvaluation::new(
            contractor,
            Uuid::new_v4(),
            Vec::new(),
            evaluator,
            scores,
            fixture_comment_min(),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_comment_too_long_rejected() {
        let (contractor, evaluator) = fixture_pair();
        let err = ContractorEvaluation::new(
            contractor,
            Uuid::new_v4(),
            Vec::new(),
            evaluator,
            fixture_scores_all_top(),
            "X".repeat(MAX_COMMENT_LEN + 1),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_empty_comment_rejected() {
        let (contractor, evaluator) = fixture_pair();
        let err = ContractorEvaluation::new(
            contractor,
            Uuid::new_v4(),
            Vec::new(),
            evaluator,
            fixture_scores_all_top(),
            String::new(),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_whitespace_only_comment_rejected() {
        let (contractor, evaluator) = fixture_pair();
        let err = ContractorEvaluation::new(
            contractor,
            Uuid::new_v4(),
            Vec::new(),
            evaluator,
            fixture_scores_all_top(),
            "          ".to_string(), // 10 spaces, trimmed to 0
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
