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
