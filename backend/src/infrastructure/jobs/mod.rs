//! Background jobs (cron / scheduled work).
//!
//! Phase A intentionally ships minimal stubs (no scheduler wiring). Real
//! cron driving (tokio interval task, schedlock, etc.) is a Phase B
//! follow-up tracked in the Story 3.7 acceptance notes.

pub mod sla_escalation_job;

pub use sla_escalation_job::SlaEscalationJob;
