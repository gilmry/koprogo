use crate::application::dto::age_request_dto::{
    AddCosignatoryDto, AgeRequestResponseDto, CreateAgeRequestDto, SyndicResponseDto,
};
use crate::application::ports::age_request_repository::AgeRequestRepository;
use crate::domain::entities::age_request::{AgeRequest, AgeRequestCosignatory};
use std::sync::Arc;
use uuid::Uuid;

pub struct AgeRequestUseCases {
    pub repo: Arc<dyn AgeRequestRepository>,
}

impl AgeRequestUseCases {
    pub fn new(repo: Arc<dyn AgeRequestRepository>) -> Self {
        Self { repo }
    }

    /// Crée une nouvelle demande d'AGE (B17-1)
    pub async fn create(
        &self,
        organization_id: Uuid,
        created_by: Uuid,
        dto: CreateAgeRequestDto,
    ) -> Result<AgeRequestResponseDto, String> {
        let age_request = AgeRequest::new(
            organization_id,
            dto.building_id,
            dto.title,
            dto.description,
            created_by,
        )?;
        let saved = self.repo.create(&age_request).await?;
        Ok(AgeRequestResponseDto::from(&saved))
    }

    /// Récupère une demande par son ID
    pub async fn get(
        &self,
        id: Uuid,
        organization_id: Uuid,
    ) -> Result<AgeRequestResponseDto, String> {
        let req = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Demande AGE {} introuvable", id))?;

        if req.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }

        Ok(AgeRequestResponseDto::from(&req))
    }

    /// Liste les demandes d'un bâtiment
    pub async fn list_by_building(
        &self,
        building_id: Uuid,
        organization_id: Uuid,
    ) -> Result<Vec<AgeRequestResponseDto>, String> {
        let requests = self.repo.find_by_building(building_id).await?;
        // Filtrer par organisation (sécurité)
        let filtered: Vec<_> = requests
            .iter()
            .filter(|r| r.organization_id == organization_id)
            .map(AgeRequestResponseDto::from)
            .collect();
        Ok(filtered)
    }

    /// Ouvre une demande pour signatures publiques (Draft → Open)
    pub async fn open(
        &self,
        id: Uuid,
        organization_id: Uuid,
        requester_id: Uuid,
    ) -> Result<AgeRequestResponseDto, String> {
        let mut req = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Demande AGE {} introuvable", id))?;

        if req.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }
        if req.created_by != requester_id {
            return Err("Seul l'initiateur peut ouvrir cette demande".to_string());
        }

        req.open()?;
        let updated = self.repo.update(&req).await?;
        Ok(AgeRequestResponseDto::from(&updated))
    }

    /// Ajoute un cosignataire à la demande (B17-2)
    /// Calcule automatiquement si le seuil 1/5 est atteint
    pub async fn add_cosignatory(
        &self,
        id: Uuid,
        organization_id: Uuid,
        dto: AddCosignatoryDto,
    ) -> Result<AgeRequestResponseDto, String> {
        let mut req = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Demande AGE {} introuvable", id))?;

        if req.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }

        let _newly_reached = req.add_cosignatory(dto.owner_id, dto.shares_pct)?;

        // Persister la mise à jour du total + status
        let updated = self.repo.update(&req).await?;

        // Persister le cosignataire en BDD
        let cosignatory = AgeRequestCosignatory::new(id, dto.owner_id, dto.shares_pct)?;
        self.repo.add_cosignatory(&cosignatory).await?;

        // Recharger avec tous les cosignataires (pour le DTO complet)
        let full = self.repo.find_by_id(id).await?.unwrap_or(updated);

        Ok(AgeRequestResponseDto::from(&full))
    }

    /// Retire un cosignataire
    pub async fn remove_cosignatory(
        &self,
        id: Uuid,
        owner_id: Uuid,
        organization_id: Uuid,
    ) -> Result<AgeRequestResponseDto, String> {
        let mut req = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Demande AGE {} introuvable", id))?;

        if req.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }

        req.remove_cosignatory(owner_id)?;

        // Supprimer de la BDD
        self.repo.remove_cosignatory(id, owner_id).await?;

        // Persister les changements de status / total
        let updated = self.repo.update(&req).await?;
        let full = self.repo.find_by_id(id).await?.unwrap_or(updated);
        Ok(AgeRequestResponseDto::from(&full))
    }

    /// Soumet la demande au syndic (Reached → Submitted) - B17-3
    /// Démarre le délai de 15j (Art. 3.87 §2 CC)
    pub async fn submit_to_syndic(
        &self,
        id: Uuid,
        organization_id: Uuid,
        requester_id: Uuid,
    ) -> Result<AgeRequestResponseDto, String> {
        let mut req = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Demande AGE {} introuvable", id))?;

        if req.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }
        if req.created_by != requester_id {
            return Err("Seul l'initiateur peut soumettre cette demande au syndic".to_string());
        }

        req.submit_to_syndic()?;
        let updated = self.repo.update(&req).await?;
        Ok(AgeRequestResponseDto::from(&updated))
    }

    /// Le syndic répond à la demande (accept ou reject) - B17-3
    pub async fn syndic_response(
        &self,
        id: Uuid,
        organization_id: Uuid,
        dto: SyndicResponseDto,
    ) -> Result<AgeRequestResponseDto, String> {
        let mut req = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Demande AGE {} introuvable", id))?;

        if req.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }

        if dto.accepted {
            req.accept_by_syndic(dto.notes)?;
        } else {
            let reason = dto
                .notes
                .ok_or_else(|| "Un motif de refus est obligatoire".to_string())?;
            req.reject_by_syndic(reason)?;
        }

        let updated = self.repo.update(&req).await?;
        Ok(AgeRequestResponseDto::from(&updated))
    }

    /// Déclenche l'auto-convocation si le délai syndic est dépassé (B17-3)
    /// Peut être appelé manuellement ou par un job de fond
    pub async fn trigger_auto_convocation(
        &self,
        id: Uuid,
        organization_id: Uuid,
    ) -> Result<AgeRequestResponseDto, String> {
        let mut req = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Demande AGE {} introuvable", id))?;

        if req.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }

        req.trigger_auto_convocation()?;
        let updated = self.repo.update(&req).await?;
        Ok(AgeRequestResponseDto::from(&updated))
    }

    /// Retire la demande
    pub async fn withdraw(
        &self,
        id: Uuid,
        organization_id: Uuid,
        requester_id: Uuid,
    ) -> Result<AgeRequestResponseDto, String> {
        let mut req = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Demande AGE {} introuvable", id))?;

        if req.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }

        req.withdraw(requester_id)?;
        let updated = self.repo.update(&req).await?;
        Ok(AgeRequestResponseDto::from(&updated))
    }

    /// Job de fond : vérifie et expire les demandes dont le délai syndic est dépassé (B17-3)
    pub async fn process_expired_deadlines(&self) -> Result<usize, String> {
        let expired = self.repo.find_expired_deadlines().await?;
        let count = expired.len();

        for mut req in expired {
            if let Err(e) = req.trigger_auto_convocation() {
                // Logguer mais continuer
                eprintln!(
                    "Erreur trigger_auto_convocation pour demande {}: {}",
                    req.id, e
                );
                continue;
            }
            if let Err(e) = self.repo.update(&req).await {
                eprintln!(
                    "Erreur update demande {} lors de l'auto-convocation: {}",
                    req.id, e
                );
            }
        }

        Ok(count)
    }

    /// Supprime une demande (Draft/Withdrawn seulement)
    pub async fn delete(
        &self,
        id: Uuid,
        organization_id: Uuid,
        requester_id: Uuid,
    ) -> Result<(), String> {
        let req = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| format!("Demande AGE {} introuvable", id))?;

        if req.organization_id != organization_id {
            return Err("Accès refusé".to_string());
        }
        if req.created_by != requester_id {
            return Err("Seul l'initiateur peut supprimer cette demande".to_string());
        }

        use crate::domain::entities::age_request::AgeRequestStatus;
        if req.status != AgeRequestStatus::Draft && req.status != AgeRequestStatus::Withdrawn {
            return Err(format!(
                "Impossible de supprimer une demande en statut {:?}. \
                 Seules les demandes Draft ou Withdrawn peuvent être supprimées.",
                req.status
            ));
        }

        self.repo.delete(id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::age_request_dto::{
        AddCosignatoryDto, CreateAgeRequestDto, SyndicResponseDto,
    };
    use crate::application::ports::age_request_repository::AgeRequestRepository;
    use crate::domain::entities::age_request::{
        AgeRequest, AgeRequestCosignatory, AgeRequestStatus,
    };
    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use std::collections::HashMap;
    use std::sync::Mutex;
    use uuid::Uuid;

    // ---------------------------------------------------------------------------
    // Mock repository using Mutex<HashMap> (same pattern as resolution_use_cases)
    // ---------------------------------------------------------------------------

    struct MockAgeRequestRepository {
        requests: Mutex<HashMap<Uuid, AgeRequest>>,
        cosignatories: Mutex<HashMap<Uuid, Vec<AgeRequestCosignatory>>>,
    }

    impl MockAgeRequestRepository {
        fn new() -> Self {
            Self {
                requests: Mutex::new(HashMap::new()),
                cosignatories: Mutex::new(HashMap::new()),
            }
        }

        /// Seed a pre-built AgeRequest into the mock store (for tests that need
        /// a request in a specific state before calling use-case methods).
        fn seed(&self, req: AgeRequest) {
            self.requests.lock().unwrap().insert(req.id, req);
        }
    }

    #[async_trait]
    impl AgeRequestRepository for MockAgeRequestRepository {
        async fn create(&self, age_request: &AgeRequest) -> Result<AgeRequest, String> {
            self.requests
                .lock()
                .unwrap()
                .insert(age_request.id, age_request.clone());
            Ok(age_request.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<AgeRequest>, String> {
            Ok(self.requests.lock().unwrap().get(&id).cloned())
        }

        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<AgeRequest>, String> {
            Ok(self
                .requests
                .lock()
                .unwrap()
                .values()
                .filter(|r| r.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn find_by_organization(
            &self,
            organization_id: Uuid,
        ) -> Result<Vec<AgeRequest>, String> {
            Ok(self
                .requests
                .lock()
                .unwrap()
                .values()
                .filter(|r| r.organization_id == organization_id)
                .cloned()
                .collect())
        }

        async fn update(&self, age_request: &AgeRequest) -> Result<AgeRequest, String> {
            self.requests
                .lock()
                .unwrap()
                .insert(age_request.id, age_request.clone());
            Ok(age_request.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            Ok(self.requests.lock().unwrap().remove(&id).is_some())
        }

        async fn add_cosignatory(&self, cosignatory: &AgeRequestCosignatory) -> Result<(), String> {
            self.cosignatories
                .lock()
                .unwrap()
                .entry(cosignatory.age_request_id)
                .or_default()
                .push(cosignatory.clone());
            Ok(())
        }

        async fn remove_cosignatory(
            &self,
            age_request_id: Uuid,
            owner_id: Uuid,
        ) -> Result<bool, String> {
            let mut map = self.cosignatories.lock().unwrap();
            if let Some(list) = map.get_mut(&age_request_id) {
                let before = list.len();
                list.retain(|c| c.owner_id != owner_id);
                Ok(list.len() < before)
            } else {
                Ok(false)
            }
        }

        async fn find_cosignatories(
            &self,
            age_request_id: Uuid,
        ) -> Result<Vec<AgeRequestCosignatory>, String> {
            Ok(self
                .cosignatories
                .lock()
                .unwrap()
                .get(&age_request_id)
                .cloned()
                .unwrap_or_default())
        }

        async fn find_expired_deadlines(&self) -> Result<Vec<AgeRequest>, String> {
            let now = Utc::now();
            Ok(self
                .requests
                .lock()
                .unwrap()
                .values()
                .filter(|r| {
                    r.status == AgeRequestStatus::Submitted
                        && r.syndic_deadline_at.map(|d| now > d).unwrap_or(false)
                })
                .cloned()
                .collect())
        }
    }

    // ---------------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------------

    fn make_use_cases(repo: Arc<MockAgeRequestRepository>) -> AgeRequestUseCases {
        AgeRequestUseCases::new(repo)
    }

    fn org_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }

    fn building_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
    }

    fn creator_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap()
    }

    /// Helper: create a Draft AGE request through the use case layer and return its ID.
    async fn create_draft(uc: &AgeRequestUseCases) -> Uuid {
        let dto = CreateAgeRequestDto {
            building_id: building_id(),
            title: "Remplacement toiture".to_string(),
            description: Some("Infiltrations importantes".to_string()),
        };
        let resp = uc.create(org_id(), creator_id(), dto).await.unwrap();
        resp.id
    }

    /// Helper: create a Draft request, open it, add enough cosignatories to reach
    /// the 1/5 threshold, and return the request ID.
    async fn create_reached(uc: &AgeRequestUseCases, repo: &MockAgeRequestRepository) -> Uuid {
        let id = create_draft(uc).await;

        // Open
        uc.open(id, org_id(), creator_id()).await.unwrap();

        // Add cosignatories totalling >= 20%
        let owner1 = Uuid::new_v4();
        let owner2 = Uuid::new_v4();
        uc.add_cosignatory(
            id,
            org_id(),
            AddCosignatoryDto {
                owner_id: owner1,
                shares_pct: 0.12,
            },
        )
        .await
        .unwrap();
        uc.add_cosignatory(
            id,
            org_id(),
            AddCosignatoryDto {
                owner_id: owner2,
                shares_pct: 0.10,
            },
        )
        .await
        .unwrap();

        // Verify state is Reached
        let req = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(req.status, AgeRequestStatus::Reached);

        id
    }

    /// Helper: create a Submitted request (threshold reached, then submitted to syndic).
    async fn create_submitted(uc: &AgeRequestUseCases, repo: &MockAgeRequestRepository) -> Uuid {
        let id = create_reached(uc, repo).await;
        uc.submit_to_syndic(id, org_id(), creator_id())
            .await
            .unwrap();

        let req = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(req.status, AgeRequestStatus::Submitted);
        id
    }

    // ---------------------------------------------------------------------------
    // Tests
    // ---------------------------------------------------------------------------

    #[tokio::test]
    async fn test_create_age_request_success() {
        let repo = Arc::new(MockAgeRequestRepository::new());
        let uc = make_use_cases(repo.clone());

        let dto = CreateAgeRequestDto {
            building_id: building_id(),
            title: "Remplacement toiture".to_string(),
            description: Some("Infiltrations importantes".to_string()),
        };

        let resp = uc.create(org_id(), creator_id(), dto).await.unwrap();

        assert_eq!(resp.status, "draft");
        assert_eq!(resp.title, "Remplacement toiture");
        assert_eq!(resp.organization_id, org_id());
        assert_eq!(resp.building_id, building_id());
        assert_eq!(resp.created_by, creator_id());
        assert_eq!(resp.total_shares_pct, 0.0);
        assert!(!resp.threshold_reached);
        assert_eq!(resp.threshold_pct, AgeRequest::DEFAULT_THRESHOLD_PCT);
        assert!(resp.cosignatories.is_empty());

        // Verify persisted in mock repo
        let stored = repo.find_by_id(resp.id).await.unwrap();
        assert!(stored.is_some());
    }

    #[tokio::test]
    async fn test_add_cosignatory_threshold_logic() {
        let repo = Arc::new(MockAgeRequestRepository::new());
        let uc = make_use_cases(repo.clone());

        let id = create_draft(&uc).await;
        uc.open(id, org_id(), creator_id()).await.unwrap();

        // First cosignatory: 10% -- not enough for 1/5 threshold
        let owner1 = Uuid::new_v4();
        let resp = uc
            .add_cosignatory(
                id,
                org_id(),
                AddCosignatoryDto {
                    owner_id: owner1,
                    shares_pct: 0.10,
                },
            )
            .await
            .unwrap();
        assert_eq!(resp.status, "open");
        assert!(!resp.threshold_reached);
        assert!((resp.total_shares_pct - 0.10).abs() < 1e-9);

        // Second cosignatory: 12% -- total 22% >= 20% threshold
        let owner2 = Uuid::new_v4();
        let resp = uc
            .add_cosignatory(
                id,
                org_id(),
                AddCosignatoryDto {
                    owner_id: owner2,
                    shares_pct: 0.12,
                },
            )
            .await
            .unwrap();
        assert_eq!(resp.status, "reached");
        assert!(resp.threshold_reached);
        assert!(resp.threshold_reached_at.is_some());
        assert!((resp.total_shares_pct - 0.22).abs() < 1e-9);
        assert_eq!(resp.shares_pct_missing, 0.0);
    }

    #[tokio::test]
    async fn test_submit_to_syndic_when_threshold_reached() {
        let repo = Arc::new(MockAgeRequestRepository::new());
        let uc = make_use_cases(repo.clone());

        let id = create_reached(&uc, &repo).await;

        let resp = uc
            .submit_to_syndic(id, org_id(), creator_id())
            .await
            .unwrap();

        assert_eq!(resp.status, "submitted");
        assert!(resp.submitted_to_syndic_at.is_some());
        assert!(resp.syndic_deadline_at.is_some());

        // Deadline = submitted + 15 days
        let submitted = resp.submitted_to_syndic_at.unwrap();
        let deadline = resp.syndic_deadline_at.unwrap();
        let diff = deadline - submitted;
        assert_eq!(diff.num_days(), AgeRequest::SYNDIC_DEADLINE_DAYS);
    }

    #[tokio::test]
    async fn test_submit_to_syndic_rejected_when_threshold_not_reached() {
        let repo = Arc::new(MockAgeRequestRepository::new());
        let uc = make_use_cases(repo.clone());

        let id = create_draft(&uc).await;
        uc.open(id, org_id(), creator_id()).await.unwrap();

        // Add a single small cosignatory (5% < 20%)
        uc.add_cosignatory(
            id,
            org_id(),
            AddCosignatoryDto {
                owner_id: Uuid::new_v4(),
                shares_pct: 0.05,
            },
        )
        .await
        .unwrap();

        // Attempt submit -- should fail because status is Open, not Reached
        let result = uc.submit_to_syndic(id, org_id(), creator_id()).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Reached"),
            "Error should mention Reached status requirement, got: {}",
            err
        );

        // Verify status unchanged
        let req = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(req.status, AgeRequestStatus::Open);
    }

    #[tokio::test]
    async fn test_syndic_accepts_request() {
        let repo = Arc::new(MockAgeRequestRepository::new());
        let uc = make_use_cases(repo.clone());

        let id = create_submitted(&uc, &repo).await;

        let resp = uc
            .syndic_response(
                id,
                org_id(),
                SyndicResponseDto {
                    accepted: true,
                    notes: Some("Convocation prévue le 15/04".to_string()),
                },
            )
            .await
            .unwrap();

        assert_eq!(resp.status, "accepted");
        assert!(resp.syndic_response_at.is_some());
        assert_eq!(
            resp.syndic_notes.as_deref(),
            Some("Convocation prévue le 15/04")
        );
    }

    #[tokio::test]
    async fn test_syndic_rejects_request_with_reason() {
        let repo = Arc::new(MockAgeRequestRepository::new());
        let uc = make_use_cases(repo.clone());

        let id = create_submitted(&uc, &repo).await;

        let resp = uc
            .syndic_response(
                id,
                org_id(),
                SyndicResponseDto {
                    accepted: false,
                    notes: Some("Demande insuffisamment motivée".to_string()),
                },
            )
            .await
            .unwrap();

        assert_eq!(resp.status, "rejected");
        assert!(resp.syndic_response_at.is_some());
        assert_eq!(
            resp.syndic_notes.as_deref(),
            Some("Demande insuffisamment motivée")
        );
    }

    #[tokio::test]
    async fn test_syndic_rejects_without_reason_fails() {
        let repo = Arc::new(MockAgeRequestRepository::new());
        let uc = make_use_cases(repo.clone());

        let id = create_submitted(&uc, &repo).await;

        // Reject with notes = None => should fail (reason is mandatory)
        let result = uc
            .syndic_response(
                id,
                org_id(),
                SyndicResponseDto {
                    accepted: false,
                    notes: None,
                },
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("motif") || err.contains("obligatoire"),
            "Error should mention mandatory reason, got: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_withdraw_request_by_initiator() {
        let repo = Arc::new(MockAgeRequestRepository::new());
        let uc = make_use_cases(repo.clone());

        let id = create_draft(&uc).await;
        uc.open(id, org_id(), creator_id()).await.unwrap();

        // Withdraw by initiator
        let resp = uc.withdraw(id, org_id(), creator_id()).await.unwrap();
        assert_eq!(resp.status, "withdrawn");

        // Verify persisted
        let req = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(req.status, AgeRequestStatus::Withdrawn);
    }

    #[tokio::test]
    async fn test_deadline_15_days_logic() {
        let repo = Arc::new(MockAgeRequestRepository::new());
        let uc = make_use_cases(repo.clone());

        // Create a request that is already Submitted with an expired deadline
        // by seeding directly into the mock.
        let mut req = AgeRequest::new(
            org_id(),
            building_id(),
            "Toiture urgente".to_string(),
            None,
            creator_id(),
        )
        .unwrap();
        req.open().unwrap();
        req.add_cosignatory(Uuid::new_v4(), 0.25).unwrap();
        req.submit_to_syndic().unwrap();

        // Manually set the deadline to 16 days in the past (expired)
        let past_submitted = Utc::now() - Duration::days(16);
        req.submitted_to_syndic_at = Some(past_submitted);
        req.syndic_deadline_at =
            Some(past_submitted + Duration::days(AgeRequest::SYNDIC_DEADLINE_DAYS));

        let request_id = req.id;
        repo.seed(req);

        // trigger_auto_convocation should succeed because deadline is past
        let resp = uc
            .trigger_auto_convocation(request_id, org_id())
            .await
            .unwrap();

        assert_eq!(resp.status, "expired");
        assert!(resp.auto_convocation_triggered);

        // Also verify find_expired_deadlines picks it up (it won't anymore
        // since status is now Expired, but verify the flow was correct)
        let req = repo.find_by_id(request_id).await.unwrap().unwrap();
        assert_eq!(req.status, AgeRequestStatus::Expired);
        assert!(req.auto_convocation_triggered);
    }
}
