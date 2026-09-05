use crate::application::ports::{
    MeetingRepository, ResolutionRepository, UnitOwnerRepository, VoteRepository,
};
use crate::domain::entities::{
    assert_voting_right_active, MajorityType, Resolution, ResolutionStatus, ResolutionType, Vote,
    VoteChoice,
};
use std::sync::Arc;
use uuid::Uuid;

pub struct ResolutionUseCases {
    resolution_repository: Arc<dyn ResolutionRepository>,
    vote_repository: Arc<dyn VoteRepository>,
    meeting_repository: Arc<dyn MeetingRepository>,
    // Story H17 — résolution du droit de vote du lot (Art. 3.87 §1).
    unit_owner_repository: Arc<dyn UnitOwnerRepository>,
}

impl ResolutionUseCases {
    pub fn new(
        resolution_repository: Arc<dyn ResolutionRepository>,
        vote_repository: Arc<dyn VoteRepository>,
        meeting_repository: Arc<dyn MeetingRepository>,
        unit_owner_repository: Arc<dyn UnitOwnerRepository>,
    ) -> Self {
        Self {
            resolution_repository,
            vote_repository,
            meeting_repository,
            unit_owner_repository,
        }
    }

    /// Create a new resolution for a meeting
    /// Enforces quorum validation per Art. 3.87 §5 CC before allowing resolution creation.
    /// Issue #310: Validates agenda_item_index if provided (Art. 3.87 CC - only agenda items can be voted on)
    pub async fn create_resolution(
        &self,
        meeting_id: Uuid,
        title: String,
        description: String,
        resolution_type: ResolutionType,
        majority_required: MajorityType,
        agenda_item_index: Option<usize>,
    ) -> Result<Resolution, String> {
        // Fetch the meeting and check quorum (Art. 3.87 §5 CC)
        let meeting = self
            .meeting_repository
            .find_by_id(meeting_id)
            .await?
            .ok_or_else(|| format!("Meeting not found: {}", meeting_id))?;

        // Enforce quorum validation before allowing resolution creation
        meeting.check_quorum_for_voting()?;

        // Issue #310: If agenda_item_index provided, validate it exists and is non-empty
        if let Some(index) = agenda_item_index {
            if index >= meeting.agenda.len() {
                return Err(
                    "Resolution must correspond to a valid agenda item (Art. 3.87 CC)".to_string(),
                );
            }
            let agenda_item = &meeting.agenda[index];
            if agenda_item.trim().is_empty() {
                return Err("Agenda item cannot be empty (Art. 3.87 CC)".to_string());
            }
        }

        let resolution = Resolution::new(
            meeting_id,
            title,
            description,
            resolution_type,
            majority_required,
            agenda_item_index,
        )?;

        self.resolution_repository.create(&resolution).await
    }

    /// Get a resolution by ID
    pub async fn get_resolution(&self, id: Uuid) -> Result<Option<Resolution>, String> {
        self.resolution_repository.find_by_id(id).await
    }

    /// Get all resolutions for a meeting
    pub async fn get_meeting_resolutions(
        &self,
        meeting_id: Uuid,
    ) -> Result<Vec<Resolution>, String> {
        self.resolution_repository
            .find_by_meeting_id(meeting_id)
            .await
    }

    /// Get resolutions by status
    pub async fn get_resolutions_by_status(
        &self,
        status: ResolutionStatus,
    ) -> Result<Vec<Resolution>, String> {
        self.resolution_repository.find_by_status(status).await
    }

    /// Update a resolution (only allowed if status is Pending)
    pub async fn update_resolution(&self, resolution: &Resolution) -> Result<Resolution, String> {
        if resolution.status != ResolutionStatus::Pending {
            return Err("Cannot update a resolution that is not pending".to_string());
        }

        self.resolution_repository.update(resolution).await
    }

    /// Delete a resolution (only allowed if no votes have been cast)
    pub async fn delete_resolution(&self, id: Uuid) -> Result<bool, String> {
        // Check if any votes exist
        let votes = self.vote_repository.find_by_resolution_id(id).await?;
        if !votes.is_empty() {
            return Err("Cannot delete a resolution with existing votes".to_string());
        }

        self.resolution_repository.delete(id).await
    }

    /// Cast a vote on a resolution
    pub async fn cast_vote(
        &self,
        resolution_id: Uuid,
        owner_id: Uuid,
        unit_id: Uuid,
        vote_choice: VoteChoice,
        voting_power: rust_decimal::Decimal,
        proxy_owner_id: Option<Uuid>,
    ) -> Result<Vote, String> {
        // Check if resolution exists and is pending
        let resolution = self
            .resolution_repository
            .find_by_id(resolution_id)
            .await?
            .ok_or_else(|| "Resolution not found".to_string())?;

        if resolution.status != ResolutionStatus::Pending {
            return Err("Cannot vote on a resolution that is not pending".to_string());
        }

        // Story H10 — Art. 3.87 §5 CC : un vote n'est valable que si le quorum de
        // l'AG est atteint (sauf 2e convocation). Gate défense-en-profondeur sur
        // le chemin VOTE (en plus du gate à la création de résolution, l.47) :
        // résout la réunion via `resolution.meeting_id` puis `check_quorum_for_voting`.
        let meeting = self
            .meeting_repository
            .find_by_id(resolution.meeting_id)
            .await?
            .ok_or_else(|| format!("Meeting not found: {}", resolution.meeting_id))?;
        meeting.check_quorum_for_voting()?;

        // Story H17 — Art. 3.87 §1 CC : un lot démembré (usufruit/nue-propriété,
        // emphytéose, superficie) ou en indivision a son droit de vote SUSPENDU
        // tant qu'un représentant unique n'est pas désigné. Gate → rejet
        // `VOTING_RIGHT_SUSPENDED`. Un lot suspendu ne peut donc pas voter : il
        // est de facto exclu du décompte/quorum (cohérence H9). Rétro-compat :
        // un lot sans titularité qualifiée reste `Active` (cf. domaine).
        let holders = self
            .unit_owner_repository
            .find_voting_holders_by_unit(unit_id)
            .await?;
        assert_voting_right_active(unit_id, &holders)?;

        // Check if unit has already voted
        if self
            .vote_repository
            .has_voted(resolution_id, unit_id)
            .await?
        {
            return Err("This unit has already voted on this resolution".to_string());
        }

        // Art. 3.87 §7 CC — un mandataire ne peut détenir plus de 3 procurations
        // Exception : si le total des voix de ses procurations < 10% du total général
        if let Some(proxy_id) = proxy_owner_id {
            self.validate_proxy_limit(resolution_id, proxy_id, voting_power)
                .await?;
        }

        // Create and save the vote
        let vote = Vote::new(
            resolution_id,
            owner_id,
            unit_id,
            vote_choice.clone(),
            voting_power,
            proxy_owner_id,
        )?;

        let created_vote = self.vote_repository.create(&vote).await?;

        // Update vote counts on the resolution
        self.update_resolution_vote_counts(resolution_id).await?;

        Ok(created_vote)
    }

    /// Change a vote (if allowed by business rules)
    pub async fn change_vote(&self, vote_id: Uuid, new_choice: VoteChoice) -> Result<Vote, String> {
        let mut vote = self
            .vote_repository
            .find_by_id(vote_id)
            .await?
            .ok_or_else(|| "Vote not found".to_string())?;

        // Check if resolution is still pending
        let resolution = self
            .resolution_repository
            .find_by_id(vote.resolution_id)
            .await?
            .ok_or_else(|| "Resolution not found".to_string())?;

        if resolution.status != ResolutionStatus::Pending {
            return Err("Cannot change vote on a closed resolution".to_string());
        }

        // Update the vote
        vote.change_vote(new_choice)?;
        let updated_vote = self.vote_repository.update(&vote).await?;

        // Recalculate vote counts
        self.update_resolution_vote_counts(vote.resolution_id)
            .await?;

        Ok(updated_vote)
    }

    /// Get all votes for a resolution
    pub async fn get_resolution_votes(&self, resolution_id: Uuid) -> Result<Vec<Vote>, String> {
        self.vote_repository
            .find_by_resolution_id(resolution_id)
            .await
    }

    /// Get all votes by an owner
    pub async fn get_owner_votes(&self, owner_id: Uuid) -> Result<Vec<Vote>, String> {
        self.vote_repository.find_by_owner_id(owner_id).await
    }

    /// Close voting on a resolution and calculate final result
    pub async fn close_voting(
        &self,
        resolution_id: Uuid,
        total_voting_power: rust_decimal::Decimal,
    ) -> Result<Resolution, String> {
        let mut resolution = self
            .resolution_repository
            .find_by_id(resolution_id)
            .await?
            .ok_or_else(|| "Resolution not found".to_string())?;

        if resolution.status != ResolutionStatus::Pending {
            return Err("Resolution voting is already closed".to_string());
        }

        // Art. 3.87 § 7 — les plafonds de procuration se vérifient à la
        // CLÔTURE, pas à chaque vote : ils portent sur l'ensemble des voix
        // exprimées, et le dernier vote enregistré peut faire basculer une
        // séance qui était licite jusque-là.
        //
        // Le refus est bloquant. Une assemblée tenue en violation de ces
        // plafonds est attaquable, et ce sont ses décisions — travaux,
        // budgets, mandats — qui tombent avec elle. Mieux vaut refuser de
        // clore que proclamer un résultat annulable.
        let votes = self
            .vote_repository
            .find_by_resolution_id(resolution_id)
            .await?;
        crate::domain::copropriete::verifier_procurations(&votes, total_voting_power, None)
            .map_err(|refus| refus.to_string())?;

        // Art. 3.87 § 9 — le prestataire ne délibère pas sur sa propre
        // mission. Même moment, même raison : c'est l'ensemble des bulletins
        // qu'il faut regarder, pas le dernier déposé.
        crate::domain::copropriete::verifier_conflit_dinterets(
            &votes,
            resolution.prestataire_de_la_mission,
        )
        .map_err(|conflit| conflit.to_string())?;

        // Art. 3.87 § 7 al. 4 — le plafonnement des voix.
        //
        // Arbitrage humain du 2026-09-04 : le texte interdit de prendre part au
        // vote POUR un nombre de voix supérieur à la somme des autres, il ne
        // frappe pas la séance de nullité. On ramène donc le majoritaire au
        // poids des autres et on délibère sur ce décompte corrigé, au lieu de
        // refuser de clore — ce qui rendait ingouvernable toute copropriété où
        // un seul détient la majorité.
        let decompte = crate::domain::copropriete::plafonner_les_voix(&votes);
        let voix_plafonnees = if decompte.ecarts().is_empty() {
            None
        } else {
            let poids_retenus = crate::domain::copropriete::repartir_le_plafond(&votes, &decompte);
            resolution.recompter_avec(&votes, &poids_retenus);
            // Le décompte corrigé doit être PERSISTÉ, pas seulement servir au
            // calcul : sans cela la base garderait les voix brutes tandis que
            // le statut refléterait les voix plafonnées. Le procès-verbal
            // afficherait alors un décompte que son propre résultat contredit,
            // ce qui est indéfendable si la décision est attaquée.
            self.resolution_repository
                .update_vote_counts(
                    resolution_id,
                    resolution.vote_count_pour,
                    resolution.vote_count_contre,
                    resolution.vote_count_abstention,
                    resolution.total_voting_power_pour,
                    resolution.total_voting_power_contre,
                    resolution.total_voting_power_abstention,
                )
                .await?;
            Some(serde_json::json!(decompte
                .ecarts()
                .iter()
                .map(|e| serde_json::json!({
                    "votant": e.votant,
                    "voix_brutes": e.voix_brutes,
                    "voix_retenues": e.voix_retenues,
                }))
                .collect::<Vec<_>>()))
        };

        // Calculate final result
        resolution.close_voting(total_voting_power)?;

        // Update resolution with final status
        self.resolution_repository
            .close_voting(resolution_id, resolution.status.clone(), voix_plafonnees)
            .await?;

        // Fetch updated resolution
        self.resolution_repository
            .find_by_id(resolution_id)
            .await?
            .ok_or_else(|| "Resolution not found after closing".to_string())
    }

    /// Get vote summary for a meeting (all resolutions with their results)
    pub async fn get_meeting_vote_summary(
        &self,
        meeting_id: Uuid,
    ) -> Result<Vec<Resolution>, String> {
        self.resolution_repository
            .get_meeting_vote_summary(meeting_id)
            .await
    }

    /// Helper: Update vote counts for a resolution based on actual votes
    async fn update_resolution_vote_counts(&self, resolution_id: Uuid) -> Result<(), String> {
        // Get vote counts
        let (pour_count, contre_count, abstention_count) = self
            .vote_repository
            .count_by_resolution_and_choice(resolution_id)
            .await?;

        // Get voting power totals
        let (pour_power, contre_power, abstention_power) = self
            .vote_repository
            .sum_voting_power_by_resolution(resolution_id)
            .await?;

        // Update resolution
        self.resolution_repository
            .update_vote_counts(
                resolution_id,
                pour_count,
                contre_count,
                abstention_count,
                pour_power,
                contre_power,
                abstention_power,
            )
            .await
    }

    /// Check if a unit has voted on a resolution
    pub async fn has_unit_voted(&self, resolution_id: Uuid, unit_id: Uuid) -> Result<bool, String> {
        self.vote_repository.has_voted(resolution_id, unit_id).await
    }

    /// Valide la limite de procurations par mandataire (Art. 3.87 §7 CC).
    ///
    /// Règle: un mandataire ne peut détenir plus de 3 procurations.
    /// Exception: si le total des voix représentées < 10% du total général,
    /// la limite de 3 ne s'applique pas.
    ///
    /// Le `new_voting_power` est le pouvoir de vote de la nouvelle procuration
    /// envisagée (utilisé pour le calcul de l'exception 10%).
    async fn validate_proxy_limit(
        &self,
        resolution_id: Uuid,
        proxy_owner_id: Uuid,
        new_voting_power: rust_decimal::Decimal,
    ) -> Result<(), String> {
        let (existing_count, existing_power) = self
            .vote_repository
            .count_proxy_votes_for_mandataire(resolution_id, proxy_owner_id)
            .await?;

        // Total du pouvoir de vote représenté après ajout de la nouvelle procuration
        let total_proxy_power = existing_power + new_voting_power;

        // Exception 10% : si le total des procurations < 10% du total AG, pas de limite
        // On récupère le total de la résolution
        let resolution = self
            .resolution_repository
            .find_by_id(resolution_id)
            .await?
            .ok_or_else(|| "Resolution not found".to_string())?;

        // total_voting_power_* contient les votes déjà exprimés
        // Pour l'exception, on compare la puissance des procurations au total millièmes
        // En pratique, le syndic passe le total_quotas du meeting — on utilise une heuristique
        let total_all_votes = resolution.total_voting_power_pour
            + resolution.total_voting_power_contre
            + resolution.total_voting_power_abstention
            + new_voting_power; // inclure ce qu'on est en train d'ajouter

        // Exception: si total procurations < 10% du total général → pas de limite
        if total_all_votes > rust_decimal::Decimal::ZERO
            && (total_proxy_power / total_all_votes) < rust_decimal_macros::dec!(0.10)
        {
            return Ok(()); // Exception 10% s'applique
        }

        // Règle générale: max 3 procurations
        if existing_count >= 3 {
            return Err(format!(
                "Le mandataire détient déjà {} procurations. Maximum autorisé : 3 (Art. 3.87 §7 CC). \
                 Exception 10% non applicable (procurations représentent >{:.1}% des votes).",
                existing_count,
                if total_all_votes > rust_decimal::Decimal::ZERO {
                    (total_proxy_power / total_all_votes) * rust_decimal_macros::dec!(100)
                } else {
                    rust_decimal::Decimal::ZERO
                }
            ));
        }

        Ok(())
    }

    /// Get vote statistics for a resolution
    pub async fn get_vote_statistics(&self, resolution_id: Uuid) -> Result<VoteStatistics, String> {
        let resolution = self
            .resolution_repository
            .find_by_id(resolution_id)
            .await?
            .ok_or_else(|| "Resolution not found".to_string())?;

        Ok(VoteStatistics {
            total_votes: resolution.total_votes(),
            vote_count_pour: resolution.vote_count_pour,
            vote_count_contre: resolution.vote_count_contre,
            vote_count_abstention: resolution.vote_count_abstention,
            total_voting_power_pour: resolution.total_voting_power_pour,
            total_voting_power_contre: resolution.total_voting_power_contre,
            total_voting_power_abstention: resolution.total_voting_power_abstention,
            pour_percentage: resolution.pour_percentage(),
            contre_percentage: resolution.contre_percentage(),
            abstention_percentage: resolution.abstention_percentage(),
            status: resolution.status,
        })
    }
}

/// Vote statistics for a resolution
#[derive(Debug, Clone)]
pub struct VoteStatistics {
    pub total_votes: i32,
    pub vote_count_pour: i32,
    pub vote_count_contre: i32,
    pub vote_count_abstention: i32,
    pub total_voting_power_pour: rust_decimal::Decimal,
    pub total_voting_power_contre: rust_decimal::Decimal,
    pub total_voting_power_abstention: rust_decimal::Decimal,
    pub pour_percentage: f64,
    pub contre_percentage: f64,
    pub abstention_percentage: f64,
    pub status: ResolutionStatus,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::PageRequest;
    use crate::application::ports::{
        MeetingRepository, ResolutionRepository, UnitOwnerRepository, VoteRepository,
    };
    use crate::domain::entities::{
        LotHolder, Meeting, MeetingType, OwnershipType, UnitOwner, VoteChoice,
    };
    use async_trait::async_trait;
    use chrono::Utc;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Story H17 — mock UnitOwnerRepository : `find_voting_holders_by_unit`
    // configurable par lot (défaut vide → `voting_right_status` Active, donc
    // les tests existants ne changent pas de comportement).
    struct MockUnitOwnerRepository {
        holders: Mutex<HashMap<Uuid, Vec<LotHolder>>>,
    }

    impl MockUnitOwnerRepository {
        fn new() -> Self {
            Self {
                holders: Mutex::new(HashMap::new()),
            }
        }

        fn with_holders(unit_id: Uuid, holders: Vec<LotHolder>) -> Self {
            let m = Self::new();
            m.holders.lock().unwrap().insert(unit_id, holders);
            m
        }
    }

    #[async_trait]
    impl UnitOwnerRepository for MockUnitOwnerRepository {
        async fn create(&self, unit_owner: &UnitOwner) -> Result<UnitOwner, String> {
            Ok(unit_owner.clone())
        }
        async fn find_by_id(&self, _id: Uuid) -> Result<Option<UnitOwner>, String> {
            Ok(None)
        }
        async fn find_current_owners_by_unit(
            &self,
            _unit_id: Uuid,
        ) -> Result<Vec<UnitOwner>, String> {
            Ok(vec![])
        }
        async fn find_current_units_by_owner(
            &self,
            _owner_id: Uuid,
        ) -> Result<Vec<UnitOwner>, String> {
            Ok(vec![])
        }
        async fn find_all_owners_by_unit(&self, _unit_id: Uuid) -> Result<Vec<UnitOwner>, String> {
            Ok(vec![])
        }
        async fn find_all_units_by_owner(&self, _owner_id: Uuid) -> Result<Vec<UnitOwner>, String> {
            Ok(vec![])
        }
        async fn update(&self, unit_owner: &UnitOwner) -> Result<UnitOwner, String> {
            Ok(unit_owner.clone())
        }
        async fn delete(&self, _id: Uuid) -> Result<(), String> {
            Ok(())
        }
        async fn has_active_owners(&self, _unit_id: Uuid) -> Result<bool, String> {
            Ok(true)
        }
        async fn get_total_ownership_percentage(
            &self,
            _unit_id: Uuid,
        ) -> Result<rust_decimal::Decimal, String> {
            Ok(rust_decimal::Decimal::ONE)
        }
        async fn find_active_by_unit_and_owner(
            &self,
            _unit_id: Uuid,
            _owner_id: Uuid,
        ) -> Result<Option<UnitOwner>, String> {
            Ok(None)
        }
        async fn find_active_by_building(
            &self,
            _building_id: Uuid,
        ) -> Result<Vec<(Uuid, Uuid, rust_decimal::Decimal)>, String> {
            Ok(vec![])
        }

        /// Même source que ci-dessus dans les tests : les fixtures posent
        /// directement des quotes-parts déjà résolues.
        async fn find_active_quota_shares_by_building(
            &self,
            _building_id: Uuid,
        ) -> Result<Vec<(Uuid, Uuid, rust_decimal::Decimal)>, String> {
            Ok(vec![])
        }
        async fn find_voting_holders_by_unit(
            &self,
            unit_id: Uuid,
        ) -> Result<Vec<LotHolder>, String> {
            Ok(self
                .holders
                .lock()
                .unwrap()
                .get(&unit_id)
                .cloned()
                .unwrap_or_default())
        }
    }

    // Mock repositories for testing
    struct MockMeetingRepository {
        meetings: Mutex<HashMap<Uuid, Meeting>>,
    }

    impl MockMeetingRepository {
        fn new() -> Self {
            Self {
                meetings: Mutex::new(HashMap::new()),
            }
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

        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Meeting>, String> {
            Ok(self
                .meetings
                .lock()
                .unwrap()
                .values()
                .filter(|m| m.building_id == building_id)
                .cloned()
                .collect())
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
            _page_request: &PageRequest,
            _organization_id: Option<Uuid>,
        ) -> Result<(Vec<Meeting>, i64), String> {
            let meetings: Vec<_> = self.meetings.lock().unwrap().values().cloned().collect();
            let total = meetings.len() as i64;
            Ok((meetings, total))
        }
    }

    struct MockResolutionRepository {
        resolutions: Mutex<HashMap<Uuid, Resolution>>,
        /// Ce que la dernière clôture a consigné au titre de l'Art. 3.87 § 7
        /// al. 4. Sans ce champ, la trace du plafonnement serait invérifiable
        /// en test unitaire, et le seul moyen de la constater serait la base.
        dernier_plafonnement: Mutex<Option<serde_json::Value>>,
    }

    impl MockResolutionRepository {
        fn new() -> Self {
            Self {
                resolutions: Mutex::new(HashMap::new()),
                dernier_plafonnement: Mutex::new(None),
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
            resolution_id: Uuid,
            vote_count_pour: i32,
            vote_count_contre: i32,
            vote_count_abstention: i32,
            total_voting_power_pour: rust_decimal::Decimal,
            total_voting_power_contre: rust_decimal::Decimal,
            total_voting_power_abstention: rust_decimal::Decimal,
        ) -> Result<(), String> {
            if let Some(resolution) = self.resolutions.lock().unwrap().get_mut(&resolution_id) {
                resolution.vote_count_pour = vote_count_pour;
                resolution.vote_count_contre = vote_count_contre;
                resolution.vote_count_abstention = vote_count_abstention;
                resolution.total_voting_power_pour = total_voting_power_pour;
                resolution.total_voting_power_contre = total_voting_power_contre;
                resolution.total_voting_power_abstention = total_voting_power_abstention;
            }
            Ok(())
        }

        async fn close_voting(
            &self,
            resolution_id: Uuid,
            final_status: ResolutionStatus,
            voix_plafonnees: Option<serde_json::Value>,
        ) -> Result<(), String> {
            // La trace du plafonnement est conservée par le mock, sans quoi
            // aucun test unitaire ne pourrait vérifier qu'elle est bien écrite.
            *self.dernier_plafonnement.lock().unwrap() = voix_plafonnees;
            if let Some(resolution) = self.resolutions.lock().unwrap().get_mut(&resolution_id) {
                resolution.status = final_status;
                resolution.voted_at = Some(chrono::Utc::now());
            }
            Ok(())
        }

        async fn get_meeting_vote_summary(
            &self,
            meeting_id: Uuid,
        ) -> Result<Vec<Resolution>, String> {
            self.find_by_meeting_id(meeting_id).await
        }
    }

    struct MockVoteRepository {
        votes: Mutex<HashMap<Uuid, Vote>>,
    }

    impl MockVoteRepository {
        fn new() -> Self {
            Self {
                votes: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl VoteRepository for MockVoteRepository {
        async fn create(&self, vote: &Vote) -> Result<Vote, String> {
            self.votes.lock().unwrap().insert(vote.id, vote.clone());
            Ok(vote.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Vote>, String> {
            Ok(self.votes.lock().unwrap().get(&id).cloned())
        }

        async fn find_by_resolution_id(&self, resolution_id: Uuid) -> Result<Vec<Vote>, String> {
            Ok(self
                .votes
                .lock()
                .unwrap()
                .values()
                .filter(|v| v.resolution_id == resolution_id)
                .cloned()
                .collect())
        }

        async fn find_by_owner_id(&self, owner_id: Uuid) -> Result<Vec<Vote>, String> {
            Ok(self
                .votes
                .lock()
                .unwrap()
                .values()
                .filter(|v| v.owner_id == owner_id)
                .cloned()
                .collect())
        }

        async fn find_by_resolution_and_unit(
            &self,
            resolution_id: Uuid,
            unit_id: Uuid,
        ) -> Result<Option<Vote>, String> {
            Ok(self
                .votes
                .lock()
                .unwrap()
                .values()
                .find(|v| v.resolution_id == resolution_id && v.unit_id == unit_id)
                .cloned())
        }

        async fn has_voted(&self, resolution_id: Uuid, unit_id: Uuid) -> Result<bool, String> {
            Ok(self
                .find_by_resolution_and_unit(resolution_id, unit_id)
                .await?
                .is_some())
        }

        async fn update(&self, vote: &Vote) -> Result<Vote, String> {
            self.votes.lock().unwrap().insert(vote.id, vote.clone());
            Ok(vote.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            Ok(self.votes.lock().unwrap().remove(&id).is_some())
        }

        async fn count_by_resolution_and_choice(
            &self,
            resolution_id: Uuid,
        ) -> Result<(i32, i32, i32), String> {
            let votes = self.find_by_resolution_id(resolution_id).await?;
            let pour = votes
                .iter()
                .filter(|v| v.vote_choice == VoteChoice::Pour)
                .count() as i32;
            let contre = votes
                .iter()
                .filter(|v| v.vote_choice == VoteChoice::Contre)
                .count() as i32;
            let abstention = votes
                .iter()
                .filter(|v| v.vote_choice == VoteChoice::Abstention)
                .count() as i32;
            Ok((pour, contre, abstention))
        }

        async fn sum_voting_power_by_resolution(
            &self,
            resolution_id: Uuid,
        ) -> Result<
            (
                rust_decimal::Decimal,
                rust_decimal::Decimal,
                rust_decimal::Decimal,
            ),
            String,
        > {
            let votes = self.find_by_resolution_id(resolution_id).await?;
            let pour: rust_decimal::Decimal = votes
                .iter()
                .filter(|v| v.vote_choice == VoteChoice::Pour)
                .map(|v| v.voting_power)
                .sum();
            let contre: rust_decimal::Decimal = votes
                .iter()
                .filter(|v| v.vote_choice == VoteChoice::Contre)
                .map(|v| v.voting_power)
                .sum();
            let abstention: rust_decimal::Decimal = votes
                .iter()
                .filter(|v| v.vote_choice == VoteChoice::Abstention)
                .map(|v| v.voting_power)
                .sum();
            Ok((pour, contre, abstention))
        }

        async fn count_proxy_votes_for_mandataire(
            &self,
            resolution_id: Uuid,
            proxy_owner_id: Uuid,
        ) -> Result<(i64, rust_decimal::Decimal), String> {
            let votes = self.find_by_resolution_id(resolution_id).await?;
            let proxy_votes: Vec<_> = votes
                .iter()
                .filter(|v| v.proxy_owner_id == Some(proxy_owner_id))
                .collect();
            let count = proxy_votes.len() as i64;
            let power: rust_decimal::Decimal = proxy_votes.iter().map(|v| v.voting_power).sum();
            Ok((count, power))
        }
    }

    /// Art. 3.87 § 7 : la clôture refuse une séance où un votant pèse plus
    /// que tous les autres réunis : ses voix sont **ramenées** à la somme des
    /// leurs, et l'écart est consigné.
    ///
    /// Le contrôle est à la clôture et pas au vote, parce qu'il porte sur
    /// l'ensemble des voix : c'est le dernier bulletin déposé qui peut faire
    /// basculer une séance licite jusque-là.
    ///
    /// Anciennement `..._refuse_un_votant_plus_lourd_...`, qui attendait un
    /// refus de clore. Arbitrage humain du 2026-09-04, confirmé par la
    /// doctrine belge : l'Art. 3.87 § 7 al. 4 plafonne, il n'annule pas.
    #[tokio::test]
    async fn test_art_3_87_la_cloture_plafonne_un_votant_plus_lourd_que_tous_les_autres() {
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let vote_repo = Arc::new(MockVoteRepository::new());
        let use_cases = ResolutionUseCases::new(
            resolution_repo.clone(),
            vote_repo.clone(),
            Arc::new(MockMeetingRepository::new()),
            Arc::new(MockUnitOwnerRepository::new()),
        );

        let resolution = Resolution::new(
            Uuid::new_v4(),
            "Réfection de la toiture".to_string(),
            "Travaux de couverture votés en AGO".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Absolute,
            Some(1),
        )
        .expect("résolution valide");
        let resolution_id = resolution.id;
        resolution_repo
            .create(&resolution)
            .await
            .expect("résolution enregistrée");

        // 600 pour un seul, 399 pour tous les autres réunis.
        for (voix, _) in [(dec!(600), 0), (dec!(200), 1), (dec!(199), 2)] {
            vote_repo
                .create(
                    &Vote::new(
                        resolution_id,
                        Uuid::new_v4(),
                        Uuid::new_v4(),
                        VoteChoice::Pour,
                        voix,
                        None,
                    )
                    .expect("vote valide"),
                )
                .await
                .expect("vote enregistré");
        }

        // La séance se clôture : le poids d'un votant ne fait plus obstacle.
        use_cases
            .close_voting(resolution_id, dec!(1000))
            .await
            .expect("le plafonnement remplace le refus");

        // L'écart est consigné, sans quoi le procès-verbal afficherait un
        // décompte que rien dans les bulletins ne permettrait de retrouver.
        let trace = resolution_repo
            .dernier_plafonnement
            .lock()
            .unwrap()
            .clone()
            .expect("un plafonnement a eu lieu, il doit être tracé");
        let ecarts = trace.as_array().expect("un tableau d'écarts");
        assert_eq!(
            ecarts.len(),
            1,
            "un seul votant dépassait la somme des autres"
        );
        assert_eq!(ecarts[0]["voix_brutes"], serde_json::json!(dec!(600)));
        assert_eq!(
            ecarts[0]["voix_retenues"],
            serde_json::json!(dec!(399)),
            "ramené à 200 + 199"
        );
    }

    /// Reproduction du scénario de recette du 2026-09-04.
    ///
    /// Alice pèse 550 et vote « pour ». Bob (250) et Claire (200) votent
    /// « contre ». Après plafonnement, Alice est ramenée à 450 : le décompte
    /// devient 450 contre 450.
    ///
    /// La majorité absolue de l'Art. 3.88 § 1er exige **plus** de la moitié
    /// des voix exprimées. Une égalité n'est pas une majorité : la résolution
    /// doit être REJETÉE.
    #[tokio::test]
    async fn test_art_3_88_une_egalite_apres_plafonnement_nest_pas_une_majorite() {
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let vote_repo = Arc::new(MockVoteRepository::new());
        let use_cases = ResolutionUseCases::new(
            resolution_repo.clone(),
            vote_repo.clone(),
            Arc::new(MockMeetingRepository::new()),
            Arc::new(MockUnitOwnerRepository::new()),
        );

        let resolution = Resolution::new(
            Uuid::new_v4(),
            "Approbation des comptes".to_string(),
            "Comptes annuels".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Absolute,
            Some(1),
        )
        .expect("résolution valide");
        let resolution_id = resolution.id;
        resolution_repo
            .create(&resolution)
            .await
            .expect("enregistrée");

        for (voix, choix) in [
            (dec!(550), VoteChoice::Pour),
            (dec!(250), VoteChoice::Contre),
            (dec!(200), VoteChoice::Contre),
        ] {
            vote_repo
                .create(
                    &Vote::new(
                        resolution_id,
                        Uuid::new_v4(),
                        Uuid::new_v4(),
                        choix,
                        voix,
                        None,
                    )
                    .expect("vote valide"),
                )
                .await
                .expect("vote enregistré");
        }

        let close = use_cases
            .close_voting(resolution_id, dec!(1000))
            .await
            .expect("la clôture aboutit");

        assert_eq!(
            close.total_voting_power_pour,
            dec!(450),
            "Alice est ramenée à la somme des autres"
        );
        assert_eq!(close.total_voting_power_contre, dec!(450));
        assert_eq!(
            close.status,
            ResolutionStatus::Rejected,
            "450 contre 450 : égalité, donc pas de majorité absolue"
        );
    }

    /// La même séance, répartie normalement, se clôture sans obstacle.
    #[tokio::test]
    async fn test_art_3_87_une_seance_equilibree_se_cloture() {
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let vote_repo = Arc::new(MockVoteRepository::new());
        let use_cases = ResolutionUseCases::new(
            resolution_repo.clone(),
            vote_repo.clone(),
            Arc::new(MockMeetingRepository::new()),
            Arc::new(MockUnitOwnerRepository::new()),
        );

        let resolution = Resolution::new(
            Uuid::new_v4(),
            "Réfection de la toiture".to_string(),
            "Travaux de couverture votés en AGO".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Absolute,
            Some(1),
        )
        .expect("résolution valide");
        let resolution_id = resolution.id;
        resolution_repo
            .create(&resolution)
            .await
            .expect("résolution enregistrée");

        for voix in [dec!(400), dec!(300), dec!(300)] {
            vote_repo
                .create(
                    &Vote::new(
                        resolution_id,
                        Uuid::new_v4(),
                        Uuid::new_v4(),
                        VoteChoice::Pour,
                        voix,
                        None,
                    )
                    .expect("vote valide"),
                )
                .await
                .expect("vote enregistré");
        }

        assert!(use_cases
            .close_voting(resolution_id, dec!(1000))
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_create_resolution() {
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let vote_repo = Arc::new(MockVoteRepository::new());
        let meeting_repo = Arc::new(MockMeetingRepository::new());
        let use_cases = ResolutionUseCases::new(
            resolution_repo.clone(),
            vote_repo,
            meeting_repo.clone(),
            Arc::new(MockUnitOwnerRepository::new()),
        );

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        // Create a meeting with quorum reached
        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            Utc::now() + chrono::Duration::days(30),
            "Salle des fêtes".to_string(),
        )
        .unwrap();
        meeting.id = meeting_id;
        // Validate quorum (600/1000 = 60% > 50%)
        meeting
            .validate_quorum(
                rust_decimal_macros::dec!(600),
                rust_decimal_macros::dec!(1000),
            )
            .unwrap();
        meeting_repo.create(&meeting).await.unwrap();

        let result = use_cases
            .create_resolution(
                meeting_id,
                "Test Resolution".to_string(),
                "Description".to_string(),
                ResolutionType::Ordinary,
                MajorityType::Absolute,
                None,
            )
            .await;

        assert!(result.is_ok());
        let resolution = result.unwrap();
        assert_eq!(resolution.title, "Test Resolution");
        assert_eq!(resolution.status, ResolutionStatus::Pending);
    }

    #[tokio::test]
    async fn test_create_resolution_fails_without_quorum() {
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let vote_repo = Arc::new(MockVoteRepository::new());
        let meeting_repo = Arc::new(MockMeetingRepository::new());
        let use_cases = ResolutionUseCases::new(
            resolution_repo.clone(),
            vote_repo,
            meeting_repo.clone(),
            Arc::new(MockUnitOwnerRepository::new()),
        );

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        // Create a meeting with quorum NOT reached
        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            Utc::now() + chrono::Duration::days(30),
            "Salle des fêtes".to_string(),
        )
        .unwrap();
        meeting.id = meeting_id;
        // Validate quorum (400/1000 = 40% < 50%) — quorum NOT reached
        meeting
            .validate_quorum(
                rust_decimal_macros::dec!(400),
                rust_decimal_macros::dec!(1000),
            )
            .unwrap();
        meeting_repo.create(&meeting).await.unwrap();

        let result = use_cases
            .create_resolution(
                meeting_id,
                "Test Resolution".to_string(),
                "Description".to_string(),
                ResolutionType::Ordinary,
                MajorityType::Absolute,
                None,
            )
            .await;

        // Should fail because quorum not reached
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("second convocation"));
    }

    #[tokio::test]
    async fn test_cast_vote_updates_counts() {
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let vote_repo = Arc::new(MockVoteRepository::new());
        let meeting_repo = Arc::new(MockMeetingRepository::new());
        let use_cases = ResolutionUseCases::new(
            resolution_repo.clone(),
            vote_repo.clone(),
            meeting_repo.clone(),
            Arc::new(MockUnitOwnerRepository::new()),
        );

        // Create a resolution
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        // Create a meeting with quorum reached
        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            Utc::now() + chrono::Duration::days(30),
            "Salle des fêtes".to_string(),
        )
        .unwrap();
        meeting.id = meeting_id;
        meeting
            .validate_quorum(
                rust_decimal_macros::dec!(600),
                rust_decimal_macros::dec!(1000),
            )
            .unwrap();
        meeting_repo.create(&meeting).await.unwrap();

        let resolution = use_cases
            .create_resolution(
                meeting_id,
                "Test Resolution".to_string(),
                "Description".to_string(),
                ResolutionType::Ordinary,
                MajorityType::Absolute,
                None,
            )
            .await
            .unwrap();

        // Cast a vote
        let owner_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let result = use_cases
            .cast_vote(
                resolution.id,
                owner_id,
                unit_id,
                VoteChoice::Pour,
                rust_decimal_macros::dec!(100),
                None,
            )
            .await;

        assert!(result.is_ok());

        // Check that vote counts were updated
        let updated_resolution = use_cases
            .get_resolution(resolution.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_resolution.vote_count_pour, 1);
        assert_eq!(
            updated_resolution.total_voting_power_pour,
            rust_decimal_macros::dec!(100)
        );
    }

    #[tokio::test]
    async fn test_cannot_vote_twice() {
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let vote_repo = Arc::new(MockVoteRepository::new());
        let meeting_repo = Arc::new(MockMeetingRepository::new());
        let use_cases = ResolutionUseCases::new(
            resolution_repo.clone(),
            vote_repo,
            meeting_repo.clone(),
            Arc::new(MockUnitOwnerRepository::new()),
        );

        // Create resolution
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        // Create a meeting with quorum reached
        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            Utc::now() + chrono::Duration::days(30),
            "Salle des fêtes".to_string(),
        )
        .unwrap();
        meeting.id = meeting_id;
        meeting
            .validate_quorum(
                rust_decimal_macros::dec!(600),
                rust_decimal_macros::dec!(1000),
            )
            .unwrap();
        meeting_repo.create(&meeting).await.unwrap();

        let resolution = use_cases
            .create_resolution(
                meeting_id,
                "Test".to_string(),
                "Desc".to_string(),
                ResolutionType::Ordinary,
                MajorityType::Absolute,
                None,
            )
            .await
            .unwrap();

        let owner_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        // First vote succeeds
        let result1 = use_cases
            .cast_vote(
                resolution.id,
                owner_id,
                unit_id,
                VoteChoice::Pour,
                rust_decimal_macros::dec!(100),
                None,
            )
            .await;
        assert!(result1.is_ok());

        // Second vote from same unit fails
        let result2 = use_cases
            .cast_vote(
                resolution.id,
                owner_id,
                unit_id,
                VoteChoice::Contre,
                rust_decimal_macros::dec!(100),
                None,
            )
            .await;
        assert!(result2.is_err());
        assert!(result2.unwrap_err().contains("already voted"));
    }

    /// Art. 3.87 §7 CC — un mandataire ne peut pas détenir plus de 3 procurations
    #[tokio::test]
    async fn test_proxy_limit_max_3_enforced() {
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let vote_repo = Arc::new(MockVoteRepository::new());
        let meeting_repo = Arc::new(MockMeetingRepository::new());
        let use_cases = ResolutionUseCases::new(
            resolution_repo.clone(),
            vote_repo.clone(),
            meeting_repo.clone(),
            Arc::new(MockUnitOwnerRepository::new()),
        );

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        // Create a meeting with quorum reached
        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            Utc::now() + chrono::Duration::days(30),
            "Salle des fêtes".to_string(),
        )
        .unwrap();
        meeting.id = meeting_id;
        meeting
            .validate_quorum(
                rust_decimal_macros::dec!(600),
                rust_decimal_macros::dec!(1000),
            )
            .unwrap();
        meeting_repo.create(&meeting).await.unwrap();

        let resolution = use_cases
            .create_resolution(
                meeting_id,
                "Test procurations".to_string(),
                "Description".to_string(),
                ResolutionType::Ordinary,
                MajorityType::Absolute,
                None,
            )
            .await
            .unwrap();

        let mandataire_id = Uuid::new_v4();

        // Cast 3 proxy votes for the same mandataire (should all succeed)
        for i in 0..3 {
            let owner_id = Uuid::new_v4();
            let unit_id = Uuid::new_v4();
            let result = use_cases
                .cast_vote(
                    resolution.id,
                    owner_id,
                    unit_id,
                    VoteChoice::Pour,
                    rust_decimal_macros::dec!(100), // 100 millièmes chacun = 300 total
                    Some(mandataire_id),
                )
                .await;
            assert!(
                result.is_ok(),
                "Proxy vote {} should succeed, got: {:?}",
                i + 1,
                result.err()
            );
        }

        // 4th proxy vote for the same mandataire should fail (>3 proxies AND >10% of votes)
        let owner_id_4 = Uuid::new_v4();
        let unit_id_4 = Uuid::new_v4();
        let result4 = use_cases
            .cast_vote(
                resolution.id,
                owner_id_4,
                unit_id_4,
                VoteChoice::Pour,
                rust_decimal_macros::dec!(100),
                Some(mandataire_id),
            )
            .await;
        assert!(result4.is_err(), "4th proxy vote should be rejected");
        assert!(
            result4.unwrap_err().contains("3"),
            "Error should mention the 3-proxy limit"
        );
    }

    /// Art. 3.87 §7 CC — exception 10% : si total procurations < 10% → pas de limite
    #[tokio::test]
    async fn test_proxy_limit_10_percent_exception_allows_more() {
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let vote_repo = Arc::new(MockVoteRepository::new());
        let meeting_repo = Arc::new(MockMeetingRepository::new());
        let use_cases = ResolutionUseCases::new(
            resolution_repo.clone(),
            vote_repo.clone(),
            meeting_repo.clone(),
            Arc::new(MockUnitOwnerRepository::new()),
        );

        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        // Create a meeting with quorum reached
        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            Utc::now() + chrono::Duration::days(30),
            "Salle des fêtes".to_string(),
        )
        .unwrap();
        meeting.id = meeting_id;
        meeting
            .validate_quorum(
                rust_decimal_macros::dec!(600),
                rust_decimal_macros::dec!(1000),
            )
            .unwrap();
        meeting_repo.create(&meeting).await.unwrap();

        let resolution = use_cases
            .create_resolution(
                meeting_id,
                "Test exception 10%".to_string(),
                "Description".to_string(),
                ResolutionType::Ordinary,
                MajorityType::Absolute,
                None,
            )
            .await
            .unwrap();

        let mandataire_id = Uuid::new_v4();

        // First, add many direct votes to make the total large (900 millièmes direct)
        for _ in 0..9 {
            let _ = use_cases
                .cast_vote(
                    resolution.id,
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    VoteChoice::Pour,
                    rust_decimal_macros::dec!(100), // 100 millièmes each, 9 × 100 = 900 total direct
                    None,
                )
                .await;
        }

        // Now add 4 proxy votes of 5 millièmes each = 20 millièmes proxy
        // Total votes = 900 + 20 = 920 → 20/920 = 2.2% < 10% → exception applies
        for i in 0..4 {
            let result = use_cases
                .cast_vote(
                    resolution.id,
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    VoteChoice::Pour,
                    rust_decimal_macros::dec!(5), // 5 millièmes chacun
                    Some(mandataire_id),
                )
                .await;
            assert!(
                result.is_ok(),
                "Proxy vote {} (10% exception) should succeed, got: {:?}",
                i + 1,
                result.err()
            );
        }
    }

    // ------------------------------------------------------------------------
    // Story H10 (CL3) — Gate quorum sur le chemin VOTE (Art. 3.87 §5).
    // ------------------------------------------------------------------------

    /// @security — voter sur une AG sans quorum validé est rejeté (gate
    /// défense-en-profondeur sur `cast_vote`). On insère une résolution Pending
    /// directement (bypass `create_resolution`) avec une réunion SANS quorum.
    #[tokio::test]
    async fn security_cast_vote_rejected_without_quorum() {
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let vote_repo = Arc::new(MockVoteRepository::new());
        let meeting_repo = Arc::new(MockMeetingRepository::new());
        let use_cases = ResolutionUseCases::new(
            resolution_repo.clone(),
            vote_repo,
            meeting_repo.clone(),
            Arc::new(MockUnitOwnerRepository::new()),
        );

        let meeting_id = Uuid::new_v4();
        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            Uuid::new_v4(),
            Uuid::new_v4(),
            MeetingType::Ordinary,
            "AGO sans quorum".to_string(),
            None,
            Utc::now() + chrono::Duration::days(30),
            "Salle".to_string(),
        )
        .unwrap();
        meeting.id = meeting_id;
        // PAS de validate_quorum → quorum_percentage = None.
        meeting_repo.create(&meeting).await.unwrap();

        let mut resolution = Resolution::new(
            meeting_id,
            "R".to_string(),
            "D".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Absolute,
            None,
        )
        .unwrap();
        resolution.status = ResolutionStatus::Pending;
        resolution_repo.create(&resolution).await.unwrap();

        let err = use_cases
            .cast_vote(
                resolution.id,
                Uuid::new_v4(),
                Uuid::new_v4(),
                VoteChoice::Pour,
                rust_decimal_macros::dec!(100),
                None,
            )
            .await
            .unwrap_err();
        assert!(
            err.contains("Quorum"),
            "le vote sans quorum doit être rejeté, got: {}",
            err
        );
    }

    /// @negative — résolution dont la réunion n'existe plus (drift data) →
    /// vote rejeté proprement (pas de panic).
    #[tokio::test]
    async fn negative_cast_vote_meeting_not_found() {
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let vote_repo = Arc::new(MockVoteRepository::new());
        let meeting_repo = Arc::new(MockMeetingRepository::new()); // vide
        let use_cases = ResolutionUseCases::new(
            resolution_repo.clone(),
            vote_repo,
            meeting_repo,
            Arc::new(MockUnitOwnerRepository::new()),
        );

        let mut resolution = Resolution::new(
            Uuid::new_v4(), // meeting_id absent du repo
            "R".to_string(),
            "D".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Absolute,
            None,
        )
        .unwrap();
        resolution.status = ResolutionStatus::Pending;
        resolution_repo.create(&resolution).await.unwrap();

        let err = use_cases
            .cast_vote(
                resolution.id,
                Uuid::new_v4(),
                Uuid::new_v4(),
                VoteChoice::Pour,
                rust_decimal_macros::dec!(100),
                None,
            )
            .await
            .unwrap_err();
        assert!(
            err.contains("Meeting not found"),
            "réunion absente → erreur typée, got: {}",
            err
        );
    }

    // ------------------------------------------------------------------------
    // Story H17 (CL3) — Gate droit de vote sur `cast_vote` (Art. 3.87 §1).
    // ------------------------------------------------------------------------

    /// Helper — réunion avec quorum validé + résolution Pending prête à voter.
    async fn meeting_with_quorum_and_resolution(
        resolution_repo: &Arc<MockResolutionRepository>,
        meeting_repo: &Arc<MockMeetingRepository>,
    ) -> Resolution {
        let meeting_id = Uuid::new_v4();
        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            Uuid::new_v4(),
            Uuid::new_v4(),
            MeetingType::Ordinary,
            "AGO".to_string(),
            None,
            Utc::now() + chrono::Duration::days(30),
            "Salle".to_string(),
        )
        .unwrap();
        meeting.id = meeting_id;
        meeting
            .validate_quorum(
                rust_decimal_macros::dec!(600),
                rust_decimal_macros::dec!(1000),
            )
            .unwrap();
        meeting_repo.create(&meeting).await.unwrap();

        let mut resolution = Resolution::new(
            meeting_id,
            "R".to_string(),
            "D".to_string(),
            ResolutionType::Ordinary,
            MajorityType::Absolute,
            None,
        )
        .unwrap();
        resolution.status = ResolutionStatus::Pending;
        resolution_repo.create(&resolution).await.unwrap();
        resolution
    }

    /// @security — un lot en indivision SANS représentant unique désigné a son
    /// droit de vote suspendu (Art. 3.87 §1) : `cast_vote` est rejeté avec
    /// `VOTING_RIGHT_SUSPENDED`, et le lot ne contribue donc pas au quorum.
    #[tokio::test]
    async fn security_cast_vote_rejected_when_voting_right_suspended() {
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let vote_repo = Arc::new(MockVoteRepository::new());
        let meeting_repo = Arc::new(MockMeetingRepository::new());
        let resolution = meeting_with_quorum_and_resolution(&resolution_repo, &meeting_repo).await;

        let unit_id = Uuid::new_v4();
        // Lot indivis (2 indivisaires), aucun représentant désigné → suspendu.
        let unit_owner_repo = Arc::new(MockUnitOwnerRepository::with_holders(
            unit_id,
            vec![
                LotHolder::new(OwnershipType::Indivisaire, false),
                LotHolder::new(OwnershipType::Indivisaire, false),
            ],
        ));
        let use_cases = ResolutionUseCases::new(
            resolution_repo.clone(),
            vote_repo,
            meeting_repo,
            unit_owner_repo,
        );

        let err = use_cases
            .cast_vote(
                resolution.id,
                Uuid::new_v4(),
                unit_id,
                VoteChoice::Pour,
                rust_decimal_macros::dec!(100),
                None,
            )
            .await
            .unwrap_err();
        assert!(
            err.contains("VOTING_RIGHT_SUSPENDED"),
            "lot suspendu → vote rejeté typé, got: {}",
            err
        );
    }

    /// @happy — un lot démembré AVEC représentant unique désigné peut voter :
    /// le gate H17 laisse passer (Art. 3.87 §1).
    #[tokio::test]
    async fn happy_cast_vote_allowed_with_designated_representative() {
        let resolution_repo = Arc::new(MockResolutionRepository::new());
        let vote_repo = Arc::new(MockVoteRepository::new());
        let meeting_repo = Arc::new(MockMeetingRepository::new());
        let resolution = meeting_with_quorum_and_resolution(&resolution_repo, &meeting_repo).await;

        let unit_id = Uuid::new_v4();
        // Usufruit + nue-propriété, l'usufruitier est le représentant → actif.
        let unit_owner_repo = Arc::new(MockUnitOwnerRepository::with_holders(
            unit_id,
            vec![
                LotHolder::new(OwnershipType::Usufruct, true),
                LotHolder::new(OwnershipType::BareOwner, false),
            ],
        ));
        let use_cases = ResolutionUseCases::new(
            resolution_repo.clone(),
            vote_repo,
            meeting_repo,
            unit_owner_repo,
        );

        let result = use_cases
            .cast_vote(
                resolution.id,
                Uuid::new_v4(),
                unit_id,
                VoteChoice::Pour,
                rust_decimal_macros::dec!(100),
                None,
            )
            .await;
        assert!(
            result.is_ok(),
            "lot avec représentant désigné doit pouvoir voter, got: {:?}",
            result.err()
        );
    }
}
