//! TechnicalSpec — cahier des charges versionnable + signatures multi-parties
//! (Story 3.8 — FR33).
//!
//! A [`TechnicalSpec`] materialises a syndic-produced specification for an
//! ACP (or a specific building inside the ACP). It is versionnable with a
//! strict SemVer-like triple ([`SemVer`]) and signed off by one or more
//! parties ([`SignatoryRole`]).
//!
//! # Workflow
//!
//! 1. The syndic creates a spec in [`TechnicalSpecStatus::Draft`].
//! 2. Once ready, the syndic submits it: the status moves to
//!    [`TechnicalSpecStatus::PendingSignatures`] (no further edits).
//! 3. Each `required_signatures` slot is filled via a
//!    [`TechnicalSpecSignature`] append-only row. When all required slots
//!    are filled, the use case promotes the spec to
//!    [`TechnicalSpecStatus::Approved`].
//! 4. A subsequent version bumps the spec ([`TechnicalSpec::bump`]) — if the
//!    bump is *major* (`requires_resignature` is true), the new draft must
//!    collect fresh signatures from every required signatory.
//!
//! # Invariants enforced at `new()` time
//!
//! - `title.len() in [5, 200]`
//! - `description.len() in [50, 10_000]`
//! - `deliverables` non empty, at most 50 entries, each non empty
//! - `required_signatures` non empty, at most 10
//! - `attachments` at most 20

use crate::application::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Bound constants
// ============================================================================

pub const MIN_TITLE_LEN: usize = 5;
pub const MAX_TITLE_LEN: usize = 200;
pub const MIN_DESCRIPTION_LEN: usize = 50;
pub const MAX_DESCRIPTION_LEN: usize = 10_000;
pub const MAX_DELIVERABLES: usize = 50;
pub const MAX_REQUIRED_SIGNATURES: usize = 10;
pub const MAX_ATTACHMENTS: usize = 20;

// ============================================================================
// SemVer — strict major.minor.patch (no v-prefix, no pre-release)
// ============================================================================

/// Strict semantic version triple (`major.minor.patch`).
///
/// Parsing is intentionally restrictive: no leading `v`, no pre-release
/// suffix, no build metadata. The codebase only needs the three numeric
/// components to decide whether a [`TechnicalSpec`] bump requires fresh
/// signatures (`major` increment) or not (`minor` / `patch`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl std::str::FromStr for SemVer {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Strict: no v-prefix, no pre-release, no build metadata.
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation("SemVer empty".to_string()));
        }
        if trimmed.starts_with('v') || trimmed.starts_with('V') {
            return Err(AppError::Validation(format!(
                "SemVer must not have a 'v' prefix: {}",
                trimmed
            )));
        }
        if trimmed.contains('-') || trimmed.contains('+') {
            return Err(AppError::Validation(format!(
                "SemVer must not carry pre-release or build metadata: {}",
                trimmed
            )));
        }
        let parts: Vec<&str> = trimmed.split('.').collect();
        if parts.len() != 3 {
            return Err(AppError::Validation(format!(
                "SemVer must be major.minor.patch (got {})",
                trimmed
            )));
        }
        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| AppError::Validation(format!("SemVer major not u32: {}", parts[0])))?;
        let minor = parts[1]
            .parse::<u32>()
            .map_err(|_| AppError::Validation(format!("SemVer minor not u32: {}", parts[1])))?;
        let patch = parts[2]
            .parse::<u32>()
            .map_err(|_| AppError::Validation(format!("SemVer patch not u32: {}", parts[2])))?;
        Ok(SemVer {
            major,
            minor,
            patch,
        })
    }
}

// ============================================================================
// Status — finite state machine for a TechnicalSpec
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TechnicalSpecStatus {
    /// Created but not yet submitted for signatures.
    Draft,
    /// Submitted — awaiting signatures from every required signatory.
    PendingSignatures,
    /// All required signatures collected.
    Approved,
    /// Replaced by a more recent version (bump chain).
    Superseded,
}

impl std::fmt::Display for TechnicalSpecStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TechnicalSpecStatus::Draft => write!(f, "draft"),
            TechnicalSpecStatus::PendingSignatures => write!(f, "pending_signatures"),
            TechnicalSpecStatus::Approved => write!(f, "approved"),
            TechnicalSpecStatus::Superseded => write!(f, "superseded"),
        }
    }
}

impl std::str::FromStr for TechnicalSpecStatus {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "draft" => Ok(TechnicalSpecStatus::Draft),
            "pending_signatures" => Ok(TechnicalSpecStatus::PendingSignatures),
            "approved" => Ok(TechnicalSpecStatus::Approved),
            "superseded" => Ok(TechnicalSpecStatus::Superseded),
            other => Err(AppError::Validation(format!(
                "Invalid TechnicalSpecStatus: {}",
                other
            ))),
        }
    }
}

// ============================================================================
// SignatoryRole
// ============================================================================

/// Role authorised to sign a [`TechnicalSpec`]. Mandataire roles (AMO,
/// Lawyer, Architect) must additionally carry an active
/// [`crate::domain::entities::Mandate`] covering the spec's ACP — the use
/// case enforces this guard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignatoryRole {
    Syndic,
    Amo,
    Lawyer,
    Architect,
    AcpRepresentative,
}

impl SignatoryRole {
    /// Whether signing under this role requires an active Mandate (Story 3.4).
    pub fn requires_mandate(&self) -> bool {
        matches!(
            self,
            SignatoryRole::Amo | SignatoryRole::Lawyer | SignatoryRole::Architect
        )
    }
}

impl std::fmt::Display for SignatoryRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignatoryRole::Syndic => write!(f, "syndic"),
            SignatoryRole::Amo => write!(f, "amo"),
            SignatoryRole::Lawyer => write!(f, "lawyer"),
            SignatoryRole::Architect => write!(f, "architect"),
            SignatoryRole::AcpRepresentative => write!(f, "acp_representative"),
        }
    }
}

impl std::str::FromStr for SignatoryRole {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "syndic" => Ok(SignatoryRole::Syndic),
            "amo" => Ok(SignatoryRole::Amo),
            "lawyer" => Ok(SignatoryRole::Lawyer),
            "architect" => Ok(SignatoryRole::Architect),
            "acp_representative" => Ok(SignatoryRole::AcpRepresentative),
            other => Err(AppError::Validation(format!(
                "Invalid SignatoryRole: {}",
                other
            ))),
        }
    }
}

// ============================================================================
// TechnicalSpec entity
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TechnicalSpec {
    pub id: Uuid,
    pub acp_id: Uuid,
    pub building_id: Option<Uuid>,
    pub title: String,
    pub description: String,
    pub version: SemVer,
    pub status: TechnicalSpecStatus,
    pub deliverables: Vec<String>,
    pub required_signatures: Vec<SignatoryRole>,
    pub attachments: Vec<String>,
    pub previous_version_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TechnicalSpec {
    /// Build a new TechnicalSpec in `Draft` status.
    ///
    /// Invariants (cf. module-level docs): title 5..=200 chars, description
    /// 50..=10_000 chars, deliverables non-empty (each non-empty) <= 50,
    /// required_signatures non-empty <= 10, attachments <= 20.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        acp_id: Uuid,
        building_id: Option<Uuid>,
        title: String,
        description: String,
        version: SemVer,
        deliverables: Vec<String>,
        required_signatures: Vec<SignatoryRole>,
        attachments: Vec<String>,
        previous_version_id: Option<Uuid>,
        created_by: Uuid,
    ) -> Result<Self, AppError> {
        let trimmed_title = title.trim().to_string();
        let trimmed_description = description.trim().to_string();
        Self::validate_invariants(
            acp_id,
            &trimmed_title,
            &trimmed_description,
            &deliverables,
            &required_signatures,
            &attachments,
            created_by,
        )?;

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            acp_id,
            building_id,
            title: trimmed_title,
            description: trimmed_description,
            version,
            status: TechnicalSpecStatus::Draft,
            deliverables: deliverables
                .into_iter()
                .map(|d| d.trim().to_string())
                .collect(),
            required_signatures,
            attachments,
            previous_version_id,
            created_by,
            created_at: now,
            updated_at: now,
        })
    }

    fn validate_invariants(
        acp_id: Uuid,
        title: &str,
        description: &str,
        deliverables: &[String],
        required_signatures: &[SignatoryRole],
        attachments: &[String],
        created_by: Uuid,
    ) -> Result<(), AppError> {
        if acp_id.is_nil() || created_by.is_nil() {
            return Err(AppError::Validation(
                "TechnicalSpec references must not be nil UUIDs".to_string(),
            ));
        }
        let t_len = title.chars().count();
        if t_len < MIN_TITLE_LEN || t_len > MAX_TITLE_LEN {
            return Err(AppError::Validation(format!(
                "title length must be in [{}, {}] (got {})",
                MIN_TITLE_LEN, MAX_TITLE_LEN, t_len
            )));
        }
        let d_len = description.chars().count();
        if d_len < MIN_DESCRIPTION_LEN || d_len > MAX_DESCRIPTION_LEN {
            return Err(AppError::Validation(format!(
                "description length must be in [{}, {}] (got {})",
                MIN_DESCRIPTION_LEN, MAX_DESCRIPTION_LEN, d_len
            )));
        }
        if deliverables.is_empty() {
            return Err(AppError::Validation(
                "deliverables must contain at least one entry".to_string(),
            ));
        }
        if deliverables.len() > MAX_DELIVERABLES {
            return Err(AppError::Validation(format!(
                "deliverables must contain at most {} entries (got {})",
                MAX_DELIVERABLES,
                deliverables.len()
            )));
        }
        if deliverables.iter().any(|d| d.trim().is_empty()) {
            return Err(AppError::Validation(
                "deliverables entries must not be empty".to_string(),
            ));
        }
        if required_signatures.is_empty() {
            return Err(AppError::Validation(
                "required_signatures must contain at least one role".to_string(),
            ));
        }
        if required_signatures.len() > MAX_REQUIRED_SIGNATURES {
            return Err(AppError::Validation(format!(
                "required_signatures must contain at most {} roles (got {})",
                MAX_REQUIRED_SIGNATURES,
                required_signatures.len()
            )));
        }
        if attachments.len() > MAX_ATTACHMENTS {
            return Err(AppError::Validation(format!(
                "attachments must contain at most {} entries (got {})",
                MAX_ATTACHMENTS,
                attachments.len()
            )));
        }
        Ok(())
    }

    /// Build the next version of this spec — keeps the same ACP / building /
    /// (optionally overridden) deliverables / signatures, increments the
    /// version and chains via `previous_version_id`. The result is always a
    /// fresh [`TechnicalSpecStatus::Draft`].
    ///
    /// Errors:
    /// - [`AppError::Validation`] if the new version is not strictly greater
    ///   than the current one.
    #[allow(clippy::too_many_arguments)]
    pub fn bump(
        &self,
        new_version: SemVer,
        new_title: Option<String>,
        new_description: Option<String>,
        new_deliverables: Option<Vec<String>>,
        new_required_signatures: Option<Vec<SignatoryRole>>,
        new_attachments: Option<Vec<String>>,
    ) -> Result<Self, AppError> {
        if !Self::is_strictly_greater(&new_version, &self.version) {
            return Err(AppError::Validation(format!(
                "new version {} must be strictly greater than {}",
                new_version, self.version
            )));
        }
        TechnicalSpec::new(
            self.acp_id,
            self.building_id,
            new_title.unwrap_or_else(|| self.title.clone()),
            new_description.unwrap_or_else(|| self.description.clone()),
            new_version,
            new_deliverables.unwrap_or_else(|| self.deliverables.clone()),
            new_required_signatures.unwrap_or_else(|| self.required_signatures.clone()),
            new_attachments.unwrap_or_else(|| self.attachments.clone()),
            Some(self.id),
            self.created_by,
        )
    }

    /// True iff `new` is strictly greater than `old` (lexicographic on the
    /// SemVer triple).
    pub fn is_strictly_greater(new: &SemVer, old: &SemVer) -> bool {
        (new.major, new.minor, new.patch) > (old.major, old.minor, old.patch)
    }

    /// True iff bumping from `self.version` to `new` requires every
    /// previously collected signature to be re-collected. Currently any
    /// `major` increment triggers this — `minor` / `patch` do not.
    pub fn requires_resignature(&self, new: &SemVer) -> bool {
        new.major > self.version.major
    }

    /// Transition Draft -> PendingSignatures. Idempotent on PendingSignatures.
    pub fn submit_for_signatures(&mut self) -> Result<(), AppError> {
        match self.status {
            TechnicalSpecStatus::Draft | TechnicalSpecStatus::PendingSignatures => {
                self.status = TechnicalSpecStatus::PendingSignatures;
                self.updated_at = Utc::now();
                Ok(())
            }
            TechnicalSpecStatus::Approved => Err(AppError::TechnicalSpecAlreadyApproved),
            TechnicalSpecStatus::Superseded => Err(AppError::Validation(
                "Cannot submit a superseded TechnicalSpec".to_string(),
            )),
        }
    }

    /// Promote to Approved once every required signature has been collected.
    /// Caller is responsible for verifying the signature set actually covers
    /// `required_signatures`.
    pub fn mark_approved(&mut self) {
        self.status = TechnicalSpecStatus::Approved;
        self.updated_at = Utc::now();
    }

    /// Whether all required signatures are present in `collected_roles`.
    pub fn has_all_required_signatures(&self, collected_roles: &[SignatoryRole]) -> bool {
        self.required_signatures
            .iter()
            .all(|r| collected_roles.contains(r))
    }
}

// ============================================================================
// TechnicalSpecSignature — append-only (pattern Story 3.7)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TechnicalSpecSignature {
    pub id: Uuid,
    pub technical_spec_id: Uuid,
    pub signatory_user_id: Uuid,
    pub role: SignatoryRole,
    pub mandate_id: Option<Uuid>,
    pub signed_at: DateTime<Utc>,
}

impl TechnicalSpecSignature {
    pub fn new(
        technical_spec_id: Uuid,
        signatory_user_id: Uuid,
        role: SignatoryRole,
        mandate_id: Option<Uuid>,
    ) -> Result<Self, AppError> {
        if technical_spec_id.is_nil() || signatory_user_id.is_nil() {
            return Err(AppError::Validation(
                "TechnicalSpecSignature references must not be nil UUIDs".to_string(),
            ));
        }
        // Mandataire roles require a Mandate (caller verifies the Mandate is
        // active; we only check the presence of the link here).
        if role.requires_mandate() && mandate_id.is_none() {
            return Err(AppError::SignatoryNotAuthorized);
        }
        Ok(Self {
            id: Uuid::new_v4(),
            technical_spec_id,
            signatory_user_id,
            role,
            mandate_id,
            signed_at: Utc::now(),
        })
    }
}

// ============================================================================
// Tests — taxonomie 4 categories obligatoire (CRITICAL.md #3, Story 3.8)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn fixture_acp_user() -> (Uuid, Uuid) {
        (Uuid::new_v4(), Uuid::new_v4())
    }

    fn fixture_description() -> String {
        // 60+ chars to be safely above MIN_DESCRIPTION_LEN.
        "Renovation toiture bat. A : etancheite, isolation 18 cm laine de roche.".to_string()
    }

    fn fixture_deliverables() -> Vec<String> {
        vec![
            "Plan d'execution".to_string(),
            "Cahier des charges detaille".to_string(),
        ]
    }

    fn fixture_required_sigs() -> Vec<SignatoryRole> {
        vec![SignatoryRole::Syndic, SignatoryRole::Architect]
    }

    // ---- @happy -------------------------------------------------------------

    #[test]
    fn happy_semver_from_str_simple_triple() {
        let v = SemVer::from_str("1.2.3").expect("valid SemVer must parse");
        assert_eq!(v, SemVer::new(1, 2, 3));
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn happy_semver_roundtrip_via_display_and_from_str() {
        let v = SemVer::new(0, 1, 0);
        let parsed = SemVer::from_str(&v.to_string()).unwrap();
        assert_eq!(parsed, v);
    }

    #[test]
    fn happy_create_minimal_draft_spec() {
        let (acp, user) = fixture_acp_user();
        let spec = TechnicalSpec::new(
            acp,
            None,
            "Toiture".to_string(),
            fixture_description(),
            SemVer::new(0, 1, 0),
            fixture_deliverables(),
            fixture_required_sigs(),
            Vec::new(),
            None,
            user,
        )
        .expect("valid TechnicalSpec must be created");
        assert_eq!(spec.status, TechnicalSpecStatus::Draft);
        assert_eq!(spec.acp_id, acp);
        assert_eq!(spec.created_by, user);
        assert_eq!(spec.version, SemVer::new(0, 1, 0));
        assert!(spec.previous_version_id.is_none());
    }

    #[test]
    fn happy_major_bump_requires_resignature_minor_does_not() {
        let (acp, user) = fixture_acp_user();
        let spec = TechnicalSpec::new(
            acp,
            None,
            "Toiture".to_string(),
            fixture_description(),
            SemVer::new(1, 5, 7),
            fixture_deliverables(),
            fixture_required_sigs(),
            Vec::new(),
            None,
            user,
        )
        .unwrap();
        assert!(spec.requires_resignature(&SemVer::new(2, 0, 0)));
        assert!(!spec.requires_resignature(&SemVer::new(1, 6, 0)));
        assert!(!spec.requires_resignature(&SemVer::new(1, 5, 8)));
    }

    #[test]
    fn happy_has_all_required_signatures_truthy() {
        let (acp, user) = fixture_acp_user();
        let spec = TechnicalSpec::new(
            acp,
            None,
            "Facade".to_string(),
            fixture_description(),
            SemVer::new(1, 0, 0),
            fixture_deliverables(),
            vec![SignatoryRole::Syndic, SignatoryRole::Architect],
            Vec::new(),
            None,
            user,
        )
        .unwrap();
        assert!(
            spec.has_all_required_signatures(&[SignatoryRole::Syndic, SignatoryRole::Architect,])
        );
        assert!(!spec.has_all_required_signatures(&[SignatoryRole::Syndic]));
    }

    #[test]
    fn happy_signatory_role_roundtrips_via_display_and_from_str() {
        for r in [
            SignatoryRole::Syndic,
            SignatoryRole::Amo,
            SignatoryRole::Lawyer,
            SignatoryRole::Architect,
            SignatoryRole::AcpRepresentative,
        ] {
            let s = r.to_string();
            assert_eq!(SignatoryRole::from_str(&s).unwrap(), r);
        }
    }

    #[test]
    fn happy_submit_for_signatures_from_draft() {
        let (acp, user) = fixture_acp_user();
        let mut spec = TechnicalSpec::new(
            acp,
            None,
            "Toiture".to_string(),
            fixture_description(),
            SemVer::new(0, 1, 0),
            fixture_deliverables(),
            fixture_required_sigs(),
            Vec::new(),
            None,
            user,
        )
        .unwrap();
        spec.submit_for_signatures().expect("draft submits");
        assert_eq!(spec.status, TechnicalSpecStatus::PendingSignatures);
    }

    #[test]
    fn happy_signatory_role_requires_mandate_for_external_pros_only() {
        assert!(SignatoryRole::Amo.requires_mandate());
        assert!(SignatoryRole::Lawyer.requires_mandate());
        assert!(SignatoryRole::Architect.requires_mandate());
        assert!(!SignatoryRole::Syndic.requires_mandate());
        assert!(!SignatoryRole::AcpRepresentative.requires_mandate());
    }

    // ---- @edge --------------------------------------------------------------

    #[test]
    fn edge_title_at_min_len_is_accepted() {
        let (acp, user) = fixture_acp_user();
        let title = "X".repeat(MIN_TITLE_LEN);
        let res = TechnicalSpec::new(
            acp,
            None,
            title,
            fixture_description(),
            SemVer::new(1, 0, 0),
            fixture_deliverables(),
            fixture_required_sigs(),
            Vec::new(),
            None,
            user,
        );
        assert!(res.is_ok(), "title exactly MIN must succeed");
    }

    #[test]
    fn edge_title_one_under_min_is_rejected() {
        let (acp, user) = fixture_acp_user();
        let title = "X".repeat(MIN_TITLE_LEN - 1);
        let err = TechnicalSpec::new(
            acp,
            None,
            title,
            fixture_description(),
            SemVer::new(1, 0, 0),
            fixture_deliverables(),
            fixture_required_sigs(),
            Vec::new(),
            None,
            user,
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn edge_exactly_50_deliverables_is_accepted_51_rejected() {
        let (acp, user) = fixture_acp_user();
        let exactly = (0..MAX_DELIVERABLES)
            .map(|i| format!("livrable-{}", i))
            .collect::<Vec<_>>();
        let ok = TechnicalSpec::new(
            acp,
            None,
            "Toiture".to_string(),
            fixture_description(),
            SemVer::new(1, 0, 0),
            exactly,
            fixture_required_sigs(),
            Vec::new(),
            None,
            user,
        );
        assert!(ok.is_ok(), "50 deliverables must succeed");

        let too_many = (0..=MAX_DELIVERABLES)
            .map(|i| format!("livrable-{}", i))
            .collect::<Vec<_>>();
        let err = TechnicalSpec::new(
            acp,
            None,
            "Toiture".to_string(),
            fixture_description(),
            SemVer::new(1, 0, 0),
            too_many,
            fixture_required_sigs(),
            Vec::new(),
            None,
            user,
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn edge_requires_resignature_only_on_major_bump() {
        let (acp, user) = fixture_acp_user();
        let spec = TechnicalSpec::new(
            acp,
            None,
            "Toiture".to_string(),
            fixture_description(),
            SemVer::new(1, 5, 7),
            fixture_deliverables(),
            fixture_required_sigs(),
            Vec::new(),
            None,
            user,
        )
        .unwrap();
        // minor bump
        assert!(!spec.requires_resignature(&SemVer::new(1, 6, 0)));
        // patch bump
        assert!(!spec.requires_resignature(&SemVer::new(1, 5, 8)));
        // major bump
        assert!(spec.requires_resignature(&SemVer::new(2, 0, 0)));
    }

    #[test]
    fn edge_bump_increments_version_and_resets_to_draft() {
        let (acp, user) = fixture_acp_user();
        let v1 = TechnicalSpec::new(
            acp,
            None,
            "Toiture".to_string(),
            fixture_description(),
            SemVer::new(1, 0, 0),
            fixture_deliverables(),
            fixture_required_sigs(),
            Vec::new(),
            None,
            user,
        )
        .unwrap();
        // Force the spec to Approved to check the bump still resets.
        let mut v1_approved = v1.clone();
        v1_approved.mark_approved();
        let v2 = v1_approved
            .bump(SemVer::new(1, 1, 0), None, None, None, None, None)
            .expect("minor bump must succeed");
        assert_eq!(v2.status, TechnicalSpecStatus::Draft);
        assert_eq!(v2.previous_version_id, Some(v1_approved.id));
        assert_eq!(v2.version, SemVer::new(1, 1, 0));
    }

    // ---- @security ----------------------------------------------------------

    #[test]
    fn security_semver_rejects_v_prefix() {
        let err = SemVer::from_str("v1.2.3").unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn security_semver_rejects_pre_release_suffix() {
        let err = SemVer::from_str("1.2.3-rc1").unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn security_semver_rejects_build_metadata() {
        let err = SemVer::from_str("1.2.3+build5").unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn security_semver_rejects_two_components_only() {
        let err = SemVer::from_str("1.2").unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn security_semver_rejects_negative_components() {
        // Negative numbers cannot parse as u32 -> Validation.
        let err = SemVer::from_str("-1.0.0").unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn security_nil_uuids_rejected_in_constructor() {
        let (acp, user) = fixture_acp_user();
        let err_acp = TechnicalSpec::new(
            Uuid::nil(),
            None,
            "Toiture".to_string(),
            fixture_description(),
            SemVer::new(1, 0, 0),
            fixture_deliverables(),
            fixture_required_sigs(),
            Vec::new(),
            None,
            user,
        )
        .unwrap_err();
        assert!(matches!(err_acp, AppError::Validation(_)));

        let err_user = TechnicalSpec::new(
            acp,
            None,
            "Toiture".to_string(),
            fixture_description(),
            SemVer::new(1, 0, 0),
            fixture_deliverables(),
            fixture_required_sigs(),
            Vec::new(),
            None,
            Uuid::nil(),
        )
        .unwrap_err();
        assert!(matches!(err_user, AppError::Validation(_)));
    }

    #[test]
    fn security_signature_for_mandataire_role_without_mandate_rejected() {
        let err = TechnicalSpecSignature::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            SignatoryRole::Lawyer,
            None,
        )
        .unwrap_err();
        assert!(matches!(err, AppError::SignatoryNotAuthorized));
    }

    // ---- @negative ----------------------------------------------------------

    #[test]
    fn negative_empty_title_rejected() {
        let (acp, user) = fixture_acp_user();
        let err = TechnicalSpec::new(
            acp,
            None,
            String::new(),
            fixture_description(),
            SemVer::new(1, 0, 0),
            fixture_deliverables(),
            fixture_required_sigs(),
            Vec::new(),
            None,
            user,
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_description_too_short_rejected() {
        let (acp, user) = fixture_acp_user();
        let err = TechnicalSpec::new(
            acp,
            None,
            "Toiture".to_string(),
            "trop court".to_string(),
            SemVer::new(1, 0, 0),
            fixture_deliverables(),
            fixture_required_sigs(),
            Vec::new(),
            None,
            user,
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_empty_deliverables_rejected() {
        let (acp, user) = fixture_acp_user();
        let err = TechnicalSpec::new(
            acp,
            None,
            "Toiture".to_string(),
            fixture_description(),
            SemVer::new(1, 0, 0),
            Vec::new(),
            fixture_required_sigs(),
            Vec::new(),
            None,
            user,
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_blank_deliverable_entry_rejected() {
        let (acp, user) = fixture_acp_user();
        let err = TechnicalSpec::new(
            acp,
            None,
            "Toiture".to_string(),
            fixture_description(),
            SemVer::new(1, 0, 0),
            vec!["valid".to_string(), "   ".to_string()],
            fixture_required_sigs(),
            Vec::new(),
            None,
            user,
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_empty_required_signatures_rejected() {
        let (acp, user) = fixture_acp_user();
        let err = TechnicalSpec::new(
            acp,
            None,
            "Toiture".to_string(),
            fixture_description(),
            SemVer::new(1, 0, 0),
            fixture_deliverables(),
            Vec::new(),
            Vec::new(),
            None,
            user,
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_too_many_attachments_rejected() {
        let (acp, user) = fixture_acp_user();
        let too_many: Vec<String> = (0..=MAX_ATTACHMENTS)
            .map(|i| format!("s3://x/{}", i))
            .collect();
        let err = TechnicalSpec::new(
            acp,
            None,
            "Toiture".to_string(),
            fixture_description(),
            SemVer::new(1, 0, 0),
            fixture_deliverables(),
            fixture_required_sigs(),
            too_many,
            None,
            user,
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_bump_to_equal_or_lower_version_rejected() {
        let (acp, user) = fixture_acp_user();
        let spec = TechnicalSpec::new(
            acp,
            None,
            "Toiture".to_string(),
            fixture_description(),
            SemVer::new(1, 2, 3),
            fixture_deliverables(),
            fixture_required_sigs(),
            Vec::new(),
            None,
            user,
        )
        .unwrap();
        let err_equal = spec
            .bump(SemVer::new(1, 2, 3), None, None, None, None, None)
            .unwrap_err();
        assert!(matches!(err_equal, AppError::Validation(_)));
        let err_lower = spec
            .bump(SemVer::new(1, 2, 2), None, None, None, None, None)
            .unwrap_err();
        assert!(matches!(err_lower, AppError::Validation(_)));
    }

    #[test]
    fn negative_submit_approved_spec_rejected() {
        let (acp, user) = fixture_acp_user();
        let mut spec = TechnicalSpec::new(
            acp,
            None,
            "Toiture".to_string(),
            fixture_description(),
            SemVer::new(1, 0, 0),
            fixture_deliverables(),
            fixture_required_sigs(),
            Vec::new(),
            None,
            user,
        )
        .unwrap();
        spec.mark_approved();
        let err = spec.submit_for_signatures().unwrap_err();
        assert!(matches!(err, AppError::TechnicalSpecAlreadyApproved));
    }

    #[test]
    fn negative_invalid_status_string_rejected() {
        let err = TechnicalSpecStatus::from_str("voted").unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_invalid_signatory_role_string_rejected() {
        let err = SignatoryRole::from_str("plombier").unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
