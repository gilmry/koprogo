//! Story 4.6 (#581) — `generate_ago_resolutions(meeting_id)`.
//!
//! Art. 3.89 § 5, 12° Code Civil belge impose que l'évaluation de
//! l'exécution des contrats en cours figure à l'ordre du jour de
//! l'assemblée ordinaire. La rendre optionnelle reviendrait à laisser le
//! syndic, qui y est évalué, décider lui-même s'il s'expose au vote — c'est
//! pourquoi ce point est ajouté d'office, jamais proposé.
//!
//! Restreint à l'AGO (`MeetingType::Ordinary`) : l'obligation légale porte
//! sur l'assemblée ordinaire, l'étendre à l'AGE ajouterait à la loi.

use crate::application::error::AppError;
use crate::application::ports::{MeetingRepository, ResolutionRepository};
use crate::domain::entities::{MeetingType, Resolution};
use std::sync::Arc;
use uuid::Uuid;

pub struct GenerateAgoResolutionsUseCase {
    meeting_repository: Arc<dyn MeetingRepository>,
    resolution_repository: Arc<dyn ResolutionRepository>,
}

impl GenerateAgoResolutionsUseCase {
    pub fn new(
        meeting_repository: Arc<dyn MeetingRepository>,
        resolution_repository: Arc<dyn ResolutionRepository>,
    ) -> Self {
        Self {
            meeting_repository,
            resolution_repository,
        }
    }

    /// Ajoute d'office la résolution `EvaluationContractorsAuto` à l'AGO
    /// désignée. Rend `Ok(vec![])` sans rien créer si la réunion est une AGE
    /// (@edge) — et rend la résolution déjà présente plutôt que d'en créer
    /// une seconde si l'appel est répété (idempotence : une convocation
    /// modifiée peut redéclencher la génération).
    pub async fn generate_ago_resolutions(
        &self,
        meeting_id: Uuid,
    ) -> Result<Vec<Resolution>, AppError> {
        let meeting = self
            .meeting_repository
            .find_by_id(meeting_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::NotFound(format!("meeting {}", meeting_id)))?;

        if meeting.meeting_type != MeetingType::Ordinary {
            return Ok(Vec::new());
        }

        let existantes = self
            .resolution_repository
            .find_by_meeting_id(meeting_id)
            .await
            .map_err(AppError::from)?;
        if let Some(deja_generee) = existantes.into_iter().find(|r| r.is_auto_generated()) {
            return Ok(vec![deja_generee]);
        }

        let resolution = Resolution::new_evaluation_contractors_auto(meeting_id);
        let created = self
            .resolution_repository
            .create(&resolution)
            .await
            .map_err(AppError::from)?;
        Ok(vec![created])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{Meeting, ResolutionStatus};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MockMeetingRepository {
        meetings: Mutex<HashMap<Uuid, Meeting>>,
    }

    impl MockMeetingRepository {
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
    impl MeetingRepository for MockMeetingRepository {
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
            _page_request: &crate::application::dto::PageRequest,
            _organization_id: Option<Uuid>,
        ) -> Result<(Vec<Meeting>, i64), String> {
            Ok((vec![], 0))
        }
    }

    struct MockResolutionRepository {
        resolutions: Mutex<HashMap<Uuid, Resolution>>,
    }

    impl MockResolutionRepository {
        fn new() -> Self {
            Self {
                resolutions: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl ResolutionRepository for MockResolutionRepository {
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
        async fn find_by_status(
            &self,
            status: ResolutionStatus,
        ) -> Result<Vec<Resolution>, String> {
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
            _total_voting_power_pour: rust_decimal::Decimal,
            _total_voting_power_contre: rust_decimal::Decimal,
            _total_voting_power_abstention: rust_decimal::Decimal,
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
        async fn get_meeting_vote_summary(
            &self,
            meeting_id: Uuid,
        ) -> Result<Vec<Resolution>, String> {
            self.find_by_meeting_id(meeting_id).await
        }
    }

    fn ago(id: Uuid) -> Meeting {
        Meeting::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            MeetingType::Ordinary,
            "AGO 2026".to_string(),
            None,
            chrono::Utc::now() + chrono::Duration::days(30),
            "Salle des fêtes".to_string(),
        )
        .map(|mut m| {
            m.id = id;
            m
        })
        .expect("meeting AGO de test")
    }

    fn age(id: Uuid) -> Meeting {
        Meeting::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            MeetingType::Extraordinary,
            "AGE 2026".to_string(),
            None,
            chrono::Utc::now() + chrono::Duration::days(30),
            "Salle des fêtes".to_string(),
        )
        .map(|mut m| {
            m.id = id;
            m
        })
        .expect("meeting AGE de test")
    }

    /// @happy — une AGO génère d'office sa résolution d'évaluation des
    /// prestataires, non éditable.
    #[tokio::test]
    async fn happy_ago_genere_la_resolution_auto() {
        let meeting_id = Uuid::new_v4();
        let meeting_repo = Arc::new(MockMeetingRepository::new());
        meeting_repo.pose(ago(meeting_id));
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let use_case = GenerateAgoResolutionsUseCase::new(meeting_repo, resolution_repo);

        let created = use_case
            .generate_ago_resolutions(meeting_id)
            .await
            .expect("génération réussie");

        assert_eq!(created.len(), 1);
        assert!(created[0].is_auto_generated());
        assert_eq!(created[0].meeting_id, meeting_id);
    }

    /// @edge — une AGE ne déclenche aucune génération : l'obligation légale
    /// (Art. 3.89 § 5, 12°) porte sur l'assemblée ordinaire.
    #[tokio::test]
    async fn edge_age_ne_genere_rien() {
        let meeting_id = Uuid::new_v4();
        let meeting_repo = Arc::new(MockMeetingRepository::new());
        meeting_repo.pose(age(meeting_id));
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let use_case = GenerateAgoResolutionsUseCase::new(meeting_repo, resolution_repo);

        let created = use_case
            .generate_ago_resolutions(meeting_id)
            .await
            .expect("pas d'erreur, juste rien à générer");

        assert!(created.is_empty());
    }

    /// @security — appeler la génération deux fois sur la même AGO ne crée
    /// pas une seconde résolution d'évaluation : un doublon donnerait au
    /// syndic une résolution à retirer sans toucher à celle qui l'évalue
    /// réellement, contournant de fait la garantie de la story.
    #[tokio::test]
    async fn security_generation_repetee_ne_duplique_pas() {
        let meeting_id = Uuid::new_v4();
        let meeting_repo = Arc::new(MockMeetingRepository::new());
        meeting_repo.pose(ago(meeting_id));
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let use_case = GenerateAgoResolutionsUseCase::new(meeting_repo, resolution_repo);

        let premiere = use_case.generate_ago_resolutions(meeting_id).await.unwrap();
        let seconde = use_case.generate_ago_resolutions(meeting_id).await.unwrap();

        assert_eq!(premiere.len(), 1);
        assert_eq!(seconde.len(), 1);
        assert_eq!(premiere[0].id, seconde[0].id);
    }

    /// @negative — une réunion introuvable rend une erreur typée 404, pas un
    /// panic ni une résolution fantôme.
    #[tokio::test]
    async fn negative_meeting_introuvable_rend_404() {
        let meeting_repo = Arc::new(MockMeetingRepository::new());
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let use_case = GenerateAgoResolutionsUseCase::new(meeting_repo, resolution_repo);

        let result = use_case.generate_ago_resolutions(Uuid::new_v4()).await;

        assert!(matches!(result, Err(AppError::NotFound(_))));
    }
}
