use crate::domain::entities::gdpr_art30::{ProcessingActivity, ProcessorAgreement};
use async_trait::async_trait;

/// Port pour le registre des traitements GDPR Art. 30
#[async_trait]
pub trait GdprArt30Repository: Send + Sync {
    async fn list_processing_activities(&self) -> Result<Vec<ProcessingActivity>, String>;
    async fn list_processor_agreements(&self) -> Result<Vec<ProcessorAgreement>, String>;
}
