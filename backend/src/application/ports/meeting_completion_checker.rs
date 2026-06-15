//! Track H Story H3 — Port `MeetingCompletionCheckerPort`.
//!
//! Construit la `MeetingCompletionChecklist` (cf. domain) à partir de l'état
//! DB courant (convocations, résolutions, présences, units, minutes). Une
//! seule query SQL agrégée fait le travail (cf. adapter sqlx
//! `meeting_completion_checker_impl.rs`).
//!
//! **Hexagonal** : ce port permet à `Meeting::assert_can_complete()` de rester
//! pur (pas d'I/O). Le use-case `complete_meeting()` orchestre :
//!   1. `completion_checker.build_checklist(meeting_id)` (DB)
//!   2. `meeting.assert_can_complete(&checklist)` (pure domain)
//!   3. `meeting.complete_internal()` (state machine)

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::MeetingCompletionChecklist;

#[async_trait]
pub trait MeetingCompletionCheckerPort: Send + Sync {
    /// Construit la checklist Art. 3.87 §3-5 CC pour le meeting donné.
    ///
    /// Retourne `Err(String)` si :
    /// - le meeting n'existe pas (ou est soft-deleted).
    /// - la query DB échoue (timeout, contrainte, etc.).
    ///
    /// **Pas de side-effect** : pure read-only.
    async fn build_checklist(&self, meeting_id: Uuid)
        -> Result<MeetingCompletionChecklist, String>;
}
