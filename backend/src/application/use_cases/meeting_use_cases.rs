use crate::application::dto::{
    AddAgendaItemRequest, CompleteMeetingRequest, CreateMeetingRequest, MeetingResponse,
    PageRequest, UpdateMeetingRequest,
};
use crate::application::ports::{MeetingCompletionCheckerPort, MeetingRepository};
use crate::domain::entities::Meeting;
use chrono::Duration;
use rust_decimal::Decimal;
#[cfg(test)]
use rust_decimal_macros::dec;
use std::sync::Arc;
use uuid::Uuid;

pub struct MeetingUseCases {
    repository: Arc<dyn MeetingRepository>,
    /// Résolution de l'ACP dont c'est l'assemblée.
    ///
    /// Optionnel pour ne pas casser les constructeurs des tests, mais son
    /// absence fait échouer la création : une assemblée sans ACP n'est
    /// l'assemblée de personne (ADR-0045).
    building_repository: Option<Arc<dyn crate::application::ports::BuildingRepository>>,
    convocation_use_cases: Option<Arc<crate::application::use_cases::ConvocationUseCases>>,
    /// Track H Story H3 — Port DB pour construire la checklist Art. 3.87
    /// §3-5 CC avant `complete_meeting`. `None` signifie : pas de gate
    /// activée (tests legacy / fallback compat backward).
    completion_checker: Option<Arc<dyn MeetingCompletionCheckerPort>>,
}

impl MeetingUseCases {
    pub fn new(repository: Arc<dyn MeetingRepository>) -> Self {
        Self {
            repository,
            building_repository: None,
            convocation_use_cases: None,
            completion_checker: None,
        }
    }

    /// Create MeetingUseCases with ConvocationUseCases for automatic 2nd convocation scheduling
    pub fn new_with_convocation(
        repository: Arc<dyn MeetingRepository>,
        convocation_use_cases: Arc<crate::application::use_cases::ConvocationUseCases>,
    ) -> Self {
        Self {
            repository,
            building_repository: None,
            convocation_use_cases: Some(convocation_use_cases),
            completion_checker: None,
        }
    }

    /// Track H Story H3 — Configure le gate `assert_can_complete()` sur
    /// `complete_meeting()`. Builder pattern fluide pour ne pas multiplier
    /// les constructeurs et préserver la compat backward des tests existants.
    pub fn with_completion_checker(
        mut self,
        checker: Arc<dyn MeetingCompletionCheckerPort>,
    ) -> Self {
        self.completion_checker = Some(checker);
        self
    }

    /// Track H Story H3 — Construit la checklist Art. 3.87 §3-5 CC et liste
    /// les invariants manquants pour le FE (composant
    /// `<MissingInvariantsList>` + désactivation bouton « Clôturer »).
    ///
    /// Retourne `(checklist, missing[])` où `missing[]` est déjà sérialisé
    /// en `serde_json::Value` au format attendu par le frontend (i18n keys
    /// = `meeting.missing.<type>`).
    ///
    /// Erreurs :
    /// - `"Meeting not found"` → 404.
    /// - `"Completion checker not configured"` → 404 (état non câblé).
    pub async fn build_completion_checklist(
        &self,
        meeting_id: Uuid,
    ) -> Result<
        (
            crate::domain::entities::MeetingCompletionChecklist,
            Vec<serde_json::Value>,
        ),
        String,
    > {
        let meeting = self
            .repository
            .find_by_id(meeting_id)
            .await?
            .ok_or_else(|| "Meeting not found".to_string())?;

        let checker = self
            .completion_checker
            .as_ref()
            .ok_or_else(|| "Completion checker not configured".to_string())?;

        let checklist = checker.build_checklist(meeting_id).await?;

        // Calcule les invariants manquants côté domain (pas d'I/O).
        let missing_invariants = match meeting.assert_can_complete(&checklist) {
            Ok(()) => Vec::new(),
            Err(err) => err.missing,
        };

        // Sérialisation manuelle Decimal-as-string (mémoire `no-f64-in-money`).
        use crate::domain::entities::MissingInvariant;
        let missing_json: Vec<serde_json::Value> = missing_invariants
            .iter()
            .map(|m| match m {
                MissingInvariant::ConvocationsNotSent => {
                    serde_json::json!({"type": "ConvocationsNotSent"})
                }
                MissingInvariant::VotesNotClosed { open_resolutions } => serde_json::json!({
                    "type": "VotesNotClosed",
                    "open_resolutions": open_resolutions,
                }),
                MissingInvariant::AttendanceNotRecorded => {
                    serde_json::json!({"type": "AttendanceNotRecorded"})
                }
                MissingInvariant::QuorumNotReached {
                    attended_quotas,
                    total_quotas,
                } => serde_json::json!({
                    "type": "QuorumNotReached",
                    "attended_quotas": attended_quotas.to_string(),
                    "total_quotas": total_quotas.to_string(),
                }),
                // Story H9 — volet têtes du quorum double (Art. 3.87 §5).
                MissingInvariant::HeadCountQuorumNotReached {
                    present_owners_count,
                    total_owners_count,
                } => serde_json::json!({
                    "type": "HeadCountQuorumNotReached",
                    "present_owners_count": present_owners_count,
                    "total_owners_count": total_owners_count,
                }),
                MissingInvariant::MinutesDraftMissing => {
                    serde_json::json!({"type": "MinutesDraftMissing"})
                }
            })
            .collect();

        Ok((checklist, missing_json))
    }

    /// Câble la résolution de l'ACP depuis l'immeuble.
    pub fn with_acp_resolution(
        mut self,
        building_repository: Arc<dyn crate::application::ports::BuildingRepository>,
    ) -> Self {
        self.building_repository = Some(building_repository);
        self
    }

    /// L'ACP dont c'est l'assemblée, résolue depuis l'immeuble.
    ///
    /// Art. 3.87 § 1er : l'assemblée générale est l'organe de l'association.
    /// Le syndic la tient (§ 2), il ne la possède pas.
    async fn resoudre_lacp(&self, building_id: Uuid) -> Result<Uuid, String> {
        let Some(building_repo) = &self.building_repository else {
            return Err(
                "Impossible de déterminer l'ACP dont c'est l'assemblée : dépôt d'immeubles \
                 non câblé"
                    .to_string(),
            );
        };
        let building = building_repo
            .find_by_id(building_id)
            .await?
            .ok_or_else(|| "Immeuble introuvable".to_string())?;
        Ok(building.acp_id)
    }

    pub async fn create_meeting(
        &self,
        request: CreateMeetingRequest,
    ) -> Result<MeetingResponse, String> {
        let acp_id = self.resoudre_lacp(request.building_id).await?;
        let mut meeting = Meeting::new(
            acp_id,
            request.organization_id,
            request.building_id,
            request.meeting_type,
            request.title,
            request.description,
            request.scheduled_date,
            request.location,
        )?;

        // Art. 3.87 §5 CC: 2nd convocation skips quorum requirement
        if request.is_second_convocation {
            meeting.is_second_convocation = true;
        }

        let created = self.repository.create(&meeting).await?;
        Ok(MeetingResponse::from(created))
    }

    pub async fn get_meeting(&self, id: Uuid) -> Result<Option<MeetingResponse>, String> {
        let meeting = self.repository.find_by_id(id).await?;
        Ok(meeting.map(MeetingResponse::from))
    }

    pub async fn list_meetings_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<MeetingResponse>, String> {
        let meetings = self.repository.find_by_building(building_id).await?;
        Ok(meetings.into_iter().map(MeetingResponse::from).collect())
    }

    pub async fn list_meetings_paginated(
        &self,
        page_request: &PageRequest,
        organization_id: Option<Uuid>,
    ) -> Result<(Vec<MeetingResponse>, i64), String> {
        let (meetings, total) = self
            .repository
            .find_all_paginated(page_request, organization_id)
            .await?;

        let dtos = meetings.into_iter().map(MeetingResponse::from).collect();
        Ok((dtos, total))
    }

    pub async fn update_meeting(
        &self,
        id: Uuid,
        request: UpdateMeetingRequest,
    ) -> Result<MeetingResponse, String> {
        let mut meeting = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Meeting not found".to_string())?;

        // Update fields if provided
        if let Some(title) = request.title {
            if title.is_empty() {
                return Err("Title cannot be empty".to_string());
            }
            meeting.title = title;
        }

        if let Some(description) = request.description {
            meeting.description = Some(description);
        }

        if let Some(scheduled_date) = request.scheduled_date {
            meeting.scheduled_date = scheduled_date;
        }

        if let Some(location) = request.location {
            if location.is_empty() {
                return Err("Location cannot be empty".to_string());
            }
            meeting.location = location;
        }

        meeting.updated_at = chrono::Utc::now();

        let updated = self.repository.update(&meeting).await?;
        Ok(MeetingResponse::from(updated))
    }

    pub async fn add_agenda_item(
        &self,
        id: Uuid,
        request: AddAgendaItemRequest,
    ) -> Result<MeetingResponse, String> {
        let mut meeting = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Meeting not found".to_string())?;

        meeting.add_agenda_item(request.item)?;

        let updated = self.repository.update(&meeting).await?;
        Ok(MeetingResponse::from(updated))
    }

    /// Track H Story H3 — Clôture une AG après vérification stricte des
    /// invariants Art. 3.87 §3-5 CC :
    ///   1. Charge le meeting (404 si absent).
    ///   2. Charge la checklist via `completion_checker.build_checklist`.
    ///   3. `meeting.assert_can_complete(&checklist)?` — propage l'erreur
    ///      typée via `From<MeetingNotCompletableError> for String` (préfixe
    ///      `MEETING_NOT_COMPLETABLE:` reconnu par le handler → 422).
    ///   4. `meeting.complete_internal()?` — state machine.
    ///   5. Persistance.
    ///
    /// **DEPRECATED param** : `request.attendees_count` est conservé pour
    /// la compat backward de l'API ; la **source de vérité** depuis Story H3
    /// est `checklist.attended_quotas` (DB agrégat present_quotas). Si le
    /// checker n'est pas branché (`completion_checker = None`, cas legacy
    /// tests / hors-prod), on retombe sur l'ancien comportement
    /// `Meeting::complete(attendees_count)` pour ne pas casser la suite
    /// existante.
    pub async fn complete_meeting(
        &self,
        id: Uuid,
        request: CompleteMeetingRequest,
    ) -> Result<MeetingResponse, String> {
        let mut meeting = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Meeting not found".to_string())?;

        if let Some(checker) = &self.completion_checker {
            // Gate Art. 3.87 §3-5 CC — branché en prod via `with_completion_checker`.
            let checklist = checker.build_checklist(id).await?;
            // `?` invoque `From<MeetingNotCompletableError> for String` qui
            // produit le payload préfixé reconnaissable par le handler 422.
            meeting.assert_can_complete(&checklist)?;
            meeting.complete_internal()?;
            // `attendees_count` reste exposé sur le DTO pour compat — on le
            // dérive de la checklist (présents+représentés agrégés). Le param
            // entrant n'est PAS utilisé (defense-in-depth contre tampering).
            let _ = request.attendees_count; // explicitement ignoré
                                             // Pour préserver le retour DTO existant, on stocke un proxy
                                             // entier sur `attendees_count` quand il est dérivable. À défaut
                                             // on garde None.
                                             // NB : la perte de précision Decimal→i32 est acceptable ici car
                                             // c'est juste pour rétro-compat affichage — la vraie présence
                                             // vit dans `present_quotas` (Decimal, déjà sur l'entité Meeting).
            use rust_decimal::prelude::ToPrimitive;
            meeting.attendees_count = checklist.attended_quotas.to_i32();
        } else {
            // Path legacy (tests, environnements non câblés). Equivalent à
            // l'ancien `Meeting::complete(n)` — pour permettre une migration
            // progressive sans rupture de l'API publique.
            #[allow(deprecated)]
            meeting.complete(request.attendees_count)?;
        }

        let updated = self.repository.update(&meeting).await?;
        Ok(MeetingResponse::from(updated))
    }

    pub async fn cancel_meeting(&self, id: Uuid) -> Result<MeetingResponse, String> {
        let mut meeting = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Meeting not found".to_string())?;

        meeting.cancel()?;

        let updated = self.repository.update(&meeting).await?;
        Ok(MeetingResponse::from(updated))
    }

    pub async fn reschedule_meeting(
        &self,
        id: Uuid,
        new_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<MeetingResponse, String> {
        let mut meeting = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Meeting not found".to_string())?;

        meeting.reschedule(new_date)?;

        let updated = self.repository.update(&meeting).await?;
        Ok(MeetingResponse::from(updated))
    }

    pub async fn delete_meeting(&self, id: Uuid) -> Result<bool, String> {
        self.repository.delete(id).await
    }

    /// Attach minutes document to a completed meeting (Issue #313)
    pub async fn attach_minutes(
        &self,
        id: Uuid,
        document_id: Uuid,
    ) -> Result<MeetingResponse, String> {
        let mut meeting = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Meeting not found".to_string())?;

        meeting.set_minutes_sent(document_id)?;

        let updated = self.repository.update(&meeting).await?;
        Ok(MeetingResponse::from(updated))
    }

    /// Valide le quorum d'une AG (Art. 3.87 §5 CC).
    /// Doit être appelé AVANT que les votes soient ouverts.
    /// Si quorum non atteint, déclenche automatiquement la création d'une 2e convocation
    /// pour le même bâtiment (si ConvocationUseCases disponible).
    /// Retourne Ok(true) si quorum atteint, Ok(false) si 2e convocation requise.
    pub async fn validate_quorum(
        &self,
        meeting_id: Uuid,
        present_quotas: Decimal,
        total_quotas: Decimal,
    ) -> Result<(bool, MeetingResponse), String> {
        let mut meeting = self
            .repository
            .find_by_id(meeting_id)
            .await?
            .ok_or_else(|| "Meeting not found".to_string())?;

        let quorum_reached = meeting.validate_quorum(present_quotas, total_quotas)?;
        let updated = self.repository.update(&meeting).await?;

        // Art. 3.87 §5 CC: Si quorum non atteint, déclencher une 2e convocation
        if !quorum_reached {
            if let Some(convocation_uc) = &self.convocation_use_cases {
                // Create second meeting (15 days after first)
                let second_meeting_date = meeting.scheduled_date + Duration::days(15);
                let second_meeting_id = Uuid::new_v4();

                // Schedule second convocation (language defaults to FR)
                let _result = convocation_uc
                    .schedule_second_convocation(
                        meeting.organization_id,
                        meeting.building_id,
                        meeting_id,
                        second_meeting_id,
                        second_meeting_date,
                        "FR".to_string(),
                        Uuid::nil(), // system-created convocation
                    )
                    .await;
                // Note: We don't fail if second convocation scheduling fails
                // (could log the error, but don't block the quorum validation result)
            }
        }

        Ok((quorum_reached, MeetingResponse::from(updated)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::PageRequest;
    use crate::application::ports::MeetingRepository;
    use crate::domain::entities::{MeetingStatus, MeetingType};
    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use mockall::mock;
    use std::sync::Arc;

    use crate::domain::entities::Building;

    /// Dépôt d'immeubles minimal : un immeuble rattaché à une ACP quelconque.
    ///
    /// L'assemblée cherche désormais l'ACP dont elle est l'assemblée
    /// (Art. 3.87 § 1er, ADR-0045). Les tests qui vérifient le rattachement
    /// nomment l'ACP ; les autres n'ont besoin que d'un immeuble qui existe.
    fn mock_building_repo() -> Arc<dyn crate::application::ports::BuildingRepository> {
        mock_building_repo_pour(Uuid::new_v4())
    }

    fn mock_building_repo_pour(
        acp_id: Uuid,
    ) -> Arc<dyn crate::application::ports::BuildingRepository> {
        struct Depot {
            acp_id: Uuid,
        }
        impl Depot {
            fn immeuble(&self) -> Building {
                Building::new(
                    self.acp_id,
                    "Résidence du Parc".to_string(),
                    "12 Rue de la Loi".to_string(),
                    "Brussels".to_string(),
                    "1000".to_string(),
                    "Belgium".to_string(),
                    10,
                    1000,
                    Some(2015),
                )
                .expect("immeuble valide")
            }
        }
        #[async_trait]
        impl crate::application::ports::BuildingRepository for Depot {
            async fn create(&self, b: &Building) -> Result<Building, String> {
                Ok(b.clone())
            }
            async fn find_by_id(&self, _id: Uuid) -> Result<Option<Building>, String> {
                Ok(Some(self.immeuble()))
            }
            async fn find_all(&self) -> Result<Vec<Building>, String> {
                Ok(vec![self.immeuble()])
            }
            async fn find_all_paginated(
                &self,
                _p: &PageRequest,
                _f: &crate::application::dto::BuildingFilters,
            ) -> Result<(Vec<Building>, i64), String> {
                Ok((vec![self.immeuble()], 1))
            }
            async fn update(&self, b: &Building) -> Result<Building, String> {
                Ok(b.clone())
            }
            async fn delete(&self, _id: Uuid) -> Result<bool, String> {
                Ok(true)
            }
            async fn find_by_slug(&self, _s: &str) -> Result<Option<Building>, String> {
                Ok(Some(self.immeuble()))
            }
            async fn find_by_id_with_metrics(
                &self,
                _id: Uuid,
            ) -> Result<Option<(Building, crate::domain::entities::BuildingMetrics)>, String>
            {
                Ok(None)
            }
        }
        Arc::new(Depot { acp_id })
    }

    mock! {
        MeetingRepo {}

        #[async_trait]
        impl MeetingRepository for MeetingRepo {
            async fn create(&self, meeting: &Meeting) -> Result<Meeting, String>;
            async fn find_by_id(&self, id: Uuid) -> Result<Option<Meeting>, String>;
            async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Meeting>, String>;
            async fn update(&self, meeting: &Meeting) -> Result<Meeting, String>;
            async fn delete(&self, id: Uuid) -> Result<bool, String>;
            async fn find_all_paginated(
                &self,
                page_request: &PageRequest,
                organization_id: Option<Uuid>,
            ) -> Result<(Vec<Meeting>, i64), String>;
        }
    }

    /// Helper to build a valid Meeting for testing purposes.
    fn make_meeting(building_id: Uuid, org_id: Uuid) -> Meeting {
        Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            Some("Annual general assembly".to_string()),
            Utc::now() + Duration::days(30),
            "Salle des fêtes".to_string(),
        )
        .unwrap()
    }

    // ---------------------------------------------------------------
    // 1. Create meeting success
    // ---------------------------------------------------------------
    /// Art. 3.87 § 1er et ADR-0045 : l'assemblée est l'organe de l'ACP.
    ///
    /// L'ACP se lit sur l'immeuble, jamais sur l'appelant. Un changement de
    /// syndic ne rend pas les assemblées passées invisibles.
    #[tokio::test]
    async fn test_lassemblee_est_celle_de_lacp_pas_du_syndic() {
        let acp_de_limmeuble = Uuid::new_v4();
        let cabinet_qui_la_tient = Uuid::new_v4();

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo.expect_create().returning(|m| Ok(m.clone()));

        let use_cases = MeetingUseCases::new(Arc::new(mock_repo))
            .with_acp_resolution(mock_building_repo_pour(acp_de_limmeuble));

        let assemblee = use_cases
            .create_meeting(CreateMeetingRequest {
                organization_id: cabinet_qui_la_tient,
                building_id: Uuid::new_v4(),
                meeting_type: MeetingType::Ordinary,
                title: "AGO 2026".to_string(),
                description: None,
                scheduled_date: Utc::now() + Duration::days(30),
                location: "Salle communale".to_string(),
                is_second_convocation: false,
            })
            .await
            .expect("création valide");

        assert_eq!(
            assemblee.acp_id, acp_de_limmeuble,
            "l'assemblée est celle de l'ACP de l'immeuble"
        );
        // La réponse n'expose pas le syndic, et c'est voulu : ce qui compte
        // pour un client est de quelle copropriété relève l'assemblée. La
        // trace d'auteur reste en base, elle n'est pas un droit d'accès.
        assert_ne!(
            assemblee.acp_id, cabinet_qui_la_tient,
            "l'ACP et le syndic sont deux entités distinctes"
        );
    }

    /// Sans dépôt d'immeubles, on ne sait pas de quelle ACP c'est l'assemblée.
    #[tokio::test]
    async fn test_pas_dassemblee_sans_acp_identifiable() {
        let mut mock_repo = MockMeetingRepo::new();
        mock_repo.expect_create().returning(|m| Ok(m.clone()));

        let use_cases = MeetingUseCases::new(Arc::new(mock_repo)); // pas de résolution

        let erreur = use_cases
            .create_meeting(CreateMeetingRequest {
                organization_id: Uuid::new_v4(),
                building_id: Uuid::new_v4(),
                meeting_type: MeetingType::Ordinary,
                title: "AGO 2026".to_string(),
                description: None,
                scheduled_date: Utc::now() + Duration::days(30),
                location: "Salle communale".to_string(),
                is_second_convocation: false,
            })
            .await
            .expect_err("doit refuser");

        assert!(
            erreur.contains("ACP"),
            "le refus doit nommer ce qui manque : {erreur}"
        );
    }

    #[tokio::test]
    async fn test_create_meeting_success() {
        let mut mock_repo = MockMeetingRepo::new();

        mock_repo.expect_create().returning(|m| Ok(m.clone()));

        let use_cases =
            MeetingUseCases::new(Arc::new(mock_repo)).with_acp_resolution(mock_building_repo());

        let request = CreateMeetingRequest {
            organization_id: Uuid::new_v4(),
            building_id: Uuid::new_v4(),
            meeting_type: MeetingType::Ordinary,
            title: "AGO 2024".to_string(),
            description: Some("Annual assembly".to_string()),
            scheduled_date: Utc::now() + Duration::days(30),
            location: "Salle communale".to_string(),
            is_second_convocation: false,
        };

        let result = use_cases.create_meeting(request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.title, "AGO 2024");
        assert_eq!(response.status, MeetingStatus::Scheduled);
    }

    // ---------------------------------------------------------------
    // 2. Create meeting with invalid data (empty title)
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_create_meeting_empty_title_fails() {
        let mock_repo = MockMeetingRepo::new();
        let use_cases =
            MeetingUseCases::new(Arc::new(mock_repo)).with_acp_resolution(mock_building_repo());

        let request = CreateMeetingRequest {
            organization_id: Uuid::new_v4(),
            building_id: Uuid::new_v4(),
            meeting_type: MeetingType::Ordinary,
            title: "".to_string(),
            description: None,
            scheduled_date: Utc::now() + Duration::days(30),
            location: "Salle communale".to_string(),
            is_second_convocation: false,
        };

        let result = use_cases.create_meeting(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Title cannot be empty"));
    }

    // ---------------------------------------------------------------
    // 3. Create meeting with empty location
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_create_meeting_empty_location_fails() {
        let mock_repo = MockMeetingRepo::new();
        let use_cases =
            MeetingUseCases::new(Arc::new(mock_repo)).with_acp_resolution(mock_building_repo());

        let request = CreateMeetingRequest {
            organization_id: Uuid::new_v4(),
            building_id: Uuid::new_v4(),
            meeting_type: MeetingType::Extraordinary,
            title: "AGE 2024".to_string(),
            description: None,
            scheduled_date: Utc::now() + Duration::days(15),
            location: "".to_string(),
            is_second_convocation: false,
        };

        let result = use_cases.create_meeting(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Location cannot be empty"));
    }

    // ---------------------------------------------------------------
    // 4. Get meeting by ID — found
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_get_meeting_found() {
        let building_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let meeting = make_meeting(building_id, org_id);
        let meeting_id = meeting.id;

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == meeting_id)
            .returning(move |_| Ok(Some(make_meeting(building_id, org_id))));

        let use_cases =
            MeetingUseCases::new(Arc::new(mock_repo)).with_acp_resolution(mock_building_repo());

        let result = use_cases.get_meeting(meeting_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    // ---------------------------------------------------------------
    // 5. Get meeting by ID — not found
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_get_meeting_not_found() {
        let mut mock_repo = MockMeetingRepo::new();
        mock_repo.expect_find_by_id().returning(|_| Ok(None));

        let use_cases =
            MeetingUseCases::new(Arc::new(mock_repo)).with_acp_resolution(mock_building_repo());

        let result = use_cases.get_meeting(Uuid::new_v4()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ---------------------------------------------------------------
    // 6. List meetings by building
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_list_meetings_by_building() {
        let building_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_find_by_building()
            .withf(move |id| *id == building_id)
            .returning(move |_| {
                Ok(vec![
                    make_meeting(building_id, org_id),
                    make_meeting(building_id, org_id),
                ])
            });

        let use_cases =
            MeetingUseCases::new(Arc::new(mock_repo)).with_acp_resolution(mock_building_repo());

        let result = use_cases.list_meetings_by_building(building_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    // ---------------------------------------------------------------
    // 7. Update meeting — success
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_update_meeting_success() {
        let building_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let meeting = make_meeting(building_id, org_id);
        let meeting_id = meeting.id;
        let meeting_clone = meeting.clone();

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == meeting_id)
            .returning(move |_| Ok(Some(meeting_clone.clone())));

        mock_repo.expect_update().returning(|m| Ok(m.clone()));

        let use_cases =
            MeetingUseCases::new(Arc::new(mock_repo)).with_acp_resolution(mock_building_repo());

        let request = UpdateMeetingRequest {
            title: Some("Renamed AGO".to_string()),
            description: Some("Updated description".to_string()),
            scheduled_date: None,
            location: None,
        };

        let result = use_cases.update_meeting(meeting_id, request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.title, "Renamed AGO");
    }

    // ---------------------------------------------------------------
    // 8. Update meeting — not found
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_update_meeting_not_found() {
        let mut mock_repo = MockMeetingRepo::new();
        mock_repo.expect_find_by_id().returning(|_| Ok(None));

        let use_cases =
            MeetingUseCases::new(Arc::new(mock_repo)).with_acp_resolution(mock_building_repo());

        let request = UpdateMeetingRequest {
            title: Some("New title".to_string()),
            description: None,
            scheduled_date: None,
            location: None,
        };

        let result = use_cases.update_meeting(Uuid::new_v4(), request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Meeting not found"));
    }

    // ---------------------------------------------------------------
    // 9. Update meeting — empty title rejected
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_update_meeting_empty_title_rejected() {
        let building_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let meeting = make_meeting(building_id, org_id);
        let meeting_id = meeting.id;
        let meeting_clone = meeting.clone();

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_find_by_id()
            .returning(move |_| Ok(Some(meeting_clone.clone())));

        let use_cases =
            MeetingUseCases::new(Arc::new(mock_repo)).with_acp_resolution(mock_building_repo());

        let request = UpdateMeetingRequest {
            title: Some("".to_string()),
            description: None,
            scheduled_date: None,
            location: None,
        };

        let result = use_cases.update_meeting(meeting_id, request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Title cannot be empty"));
    }

    // ---------------------------------------------------------------
    // 10. Delete meeting
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_delete_meeting_success() {
        let meeting_id = Uuid::new_v4();

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_delete()
            .withf(move |id| *id == meeting_id)
            .returning(|_| Ok(true));

        let use_cases =
            MeetingUseCases::new(Arc::new(mock_repo)).with_acp_resolution(mock_building_repo());

        let result = use_cases.delete_meeting(meeting_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    // ---------------------------------------------------------------
    // 11. Validate quorum — reached (>50%)
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_validate_quorum_reached() {
        let building_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let meeting = make_meeting(building_id, org_id);
        let meeting_id = meeting.id;
        let meeting_clone = meeting.clone();

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == meeting_id)
            .returning(move |_| Ok(Some(meeting_clone.clone())));

        mock_repo.expect_update().returning(|m| Ok(m.clone()));

        let use_cases =
            MeetingUseCases::new(Arc::new(mock_repo)).with_acp_resolution(mock_building_repo());

        // 600/1000 = 60% → quorum reached
        let result = use_cases
            .validate_quorum(meeting_id, dec!(600), dec!(1000))
            .await;
        assert!(result.is_ok());
        let (reached, response) = result.unwrap();
        assert!(reached);
        assert!(response.quorum_validated);
        assert!((response.quorum_percentage.unwrap() - 60.0).abs() < 0.01);
    }

    // ---------------------------------------------------------------
    // 12. Validate quorum — not reached (<=50%)
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_validate_quorum_not_reached() {
        let building_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let meeting = make_meeting(building_id, org_id);
        let meeting_id = meeting.id;
        let meeting_clone = meeting.clone();

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == meeting_id)
            .returning(move |_| Ok(Some(meeting_clone.clone())));

        mock_repo.expect_update().returning(|m| Ok(m.clone()));

        // No convocation_use_cases set, so second convocation won't be triggered
        let use_cases =
            MeetingUseCases::new(Arc::new(mock_repo)).with_acp_resolution(mock_building_repo());

        // 400/1000 = 40% → quorum NOT reached
        let result = use_cases
            .validate_quorum(meeting_id, dec!(400), dec!(1000))
            .await;
        assert!(result.is_ok());
        let (reached, response) = result.unwrap();
        assert!(!reached);
        assert!(!response.quorum_validated);
        assert!((response.quorum_percentage.unwrap() - 40.0).abs() < 0.01);
    }

    // ---------------------------------------------------------------
    // 13. Validate quorum — meeting not found
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_validate_quorum_meeting_not_found() {
        let mut mock_repo = MockMeetingRepo::new();
        mock_repo.expect_find_by_id().returning(|_| Ok(None));

        let use_cases =
            MeetingUseCases::new(Arc::new(mock_repo)).with_acp_resolution(mock_building_repo());

        let result = use_cases
            .validate_quorum(Uuid::new_v4(), dec!(600), dec!(1000))
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Meeting not found"));
    }

    // ---------------------------------------------------------------
    // 14. Complete meeting via use case
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_complete_meeting_success() {
        let building_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let meeting = make_meeting(building_id, org_id);
        let meeting_id = meeting.id;
        let meeting_clone = meeting.clone();

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == meeting_id)
            .returning(move |_| Ok(Some(meeting_clone.clone())));

        mock_repo.expect_update().returning(|m| Ok(m.clone()));

        let use_cases =
            MeetingUseCases::new(Arc::new(mock_repo)).with_acp_resolution(mock_building_repo());

        let request = CompleteMeetingRequest {
            attendees_count: 42,
        };
        let result = use_cases.complete_meeting(meeting_id, request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, MeetingStatus::Completed);
        assert_eq!(response.attendees_count, Some(42));
    }

    // ---------------------------------------------------------------
    // 15. Cancel meeting via use case
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_cancel_meeting_success() {
        let building_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let meeting = make_meeting(building_id, org_id);
        let meeting_id = meeting.id;
        let meeting_clone = meeting.clone();

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == meeting_id)
            .returning(move |_| Ok(Some(meeting_clone.clone())));

        mock_repo.expect_update().returning(|m| Ok(m.clone()));

        let use_cases =
            MeetingUseCases::new(Arc::new(mock_repo)).with_acp_resolution(mock_building_repo());

        let result = use_cases.cancel_meeting(meeting_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, MeetingStatus::Cancelled);
    }

    // ---------------------------------------------------------------
    // 16. Add agenda item via use case
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_add_agenda_item_success() {
        let building_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let meeting = make_meeting(building_id, org_id);
        let meeting_id = meeting.id;
        let meeting_clone = meeting.clone();

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == meeting_id)
            .returning(move |_| Ok(Some(meeting_clone.clone())));

        mock_repo.expect_update().returning(|m| Ok(m.clone()));

        let use_cases =
            MeetingUseCases::new(Arc::new(mock_repo)).with_acp_resolution(mock_building_repo());

        let request = AddAgendaItemRequest {
            item: "Approbation des comptes".to_string(),
        };
        let result = use_cases.add_agenda_item(meeting_id, request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.agenda.len(), 1);
        assert_eq!(response.agenda[0], "Approbation des comptes");
    }

    // ---------------------------------------------------------------
    // 17. Validate quorum — exact 50% NOT reached (Art. 3.87 §5: >50% strict)
    // ---------------------------------------------------------------
    #[tokio::test]
    async fn test_validate_quorum_exact_50_percent_not_reached() {
        let building_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let meeting = make_meeting(building_id, org_id);
        let meeting_id = meeting.id;
        let meeting_clone = meeting.clone();

        let mut mock_repo = MockMeetingRepo::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == meeting_id)
            .returning(move |_| Ok(Some(meeting_clone.clone())));

        mock_repo.expect_update().returning(|m| Ok(m.clone()));

        let use_cases =
            MeetingUseCases::new(Arc::new(mock_repo)).with_acp_resolution(mock_building_repo());

        // 500/1000 = exactly 50% → quorum NOT reached (Art. 3.87 §5: strictly >50%)
        let result = use_cases
            .validate_quorum(meeting_id, dec!(500), dec!(1000))
            .await;
        assert!(result.is_ok());
        let (reached, response) = result.unwrap();
        assert!(!reached);
        assert!(!response.quorum_validated);
        assert!((response.quorum_percentage.unwrap() - 50.0).abs() < 0.01);
    }
}
