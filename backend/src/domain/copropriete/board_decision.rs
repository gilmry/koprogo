use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Statut d'une décision du conseil de copropriété
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DecisionStatus {
    Pending,    // En attente d'exécution
    InProgress, // En cours d'exécution
    Completed,  // Terminée
    Overdue,    // En retard (deadline dépassée)
    Cancelled,  // Annulée
}

impl std::fmt::Display for DecisionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecisionStatus::Pending => write!(f, "pending"),
            DecisionStatus::InProgress => write!(f, "in_progress"),
            DecisionStatus::Completed => write!(f, "completed"),
            DecisionStatus::Overdue => write!(f, "overdue"),
            DecisionStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for DecisionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(DecisionStatus::Pending),
            "in_progress" => Ok(DecisionStatus::InProgress),
            "completed" => Ok(DecisionStatus::Completed),
            "overdue" => Ok(DecisionStatus::Overdue),
            "cancelled" => Ok(DecisionStatus::Cancelled),
            _ => Err(format!("Invalid decision status: {}", s)),
        }
    }
}

/// Décision prise par l'assemblée générale et suivie par le conseil de copropriété
/// Le conseil surveille l'exécution par le syndic des décisions votées en AG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardDecision {
    pub id: Uuid,
    pub building_id: Uuid,
    pub meeting_id: Uuid,                // AG qui a pris la décision
    pub subject: String,                 // Objet de la décision
    pub decision_text: String,           // Texte complet de la décision
    pub deadline: Option<DateTime<Utc>>, // Date limite d'exécution
    pub status: DecisionStatus,
    pub completed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>, // Notes du conseil sur le suivi
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BoardDecision {
    /// Crée une nouvelle décision à suivre
    pub fn new(
        building_id: Uuid,
        meeting_id: Uuid,
        subject: String,
        decision_text: String,
        deadline: Option<DateTime<Utc>>,
    ) -> Result<Self, String> {
        // Validation: subject ne peut pas être vide
        if subject.trim().is_empty() {
            return Err("Decision subject cannot be empty".to_string());
        }

        // Validation: decision_text ne peut pas être vide
        if decision_text.trim().is_empty() {
            return Err("Decision text cannot be empty".to_string());
        }

        // Validation: si deadline existe, elle doit être dans le futur
        if let Some(deadline_date) = deadline {
            if deadline_date <= Utc::now() {
                return Err("Deadline must be in the future".to_string());
            }
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            building_id,
            meeting_id,
            subject,
            decision_text,
            deadline,
            status: DecisionStatus::Pending,
            completed_at: None,
            notes: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Vérifie si la décision est en retard
    pub fn is_overdue(&self) -> bool {
        if self.status == DecisionStatus::Completed || self.status == DecisionStatus::Cancelled {
            return false;
        }

        if let Some(deadline) = self.deadline {
            Utc::now() > deadline
        } else {
            false
        }
    }

    /// Met à jour le statut de la décision
    /// Gère automatiquement les transitions valides et le timestamp completed_at
    pub fn update_status(&mut self, new_status: DecisionStatus) -> Result<(), String> {
        // Validation des transitions de statut
        match (&self.status, &new_status) {
            // On ne peut pas modifier une décision terminée ou annulée
            (DecisionStatus::Completed, _) => {
                return Err("Cannot change status of a completed decision".to_string());
            }
            (DecisionStatus::Cancelled, _) => {
                return Err("Cannot change status of a cancelled decision".to_string());
            }
            // Transitions valides
            (DecisionStatus::Pending, DecisionStatus::InProgress)
            | (DecisionStatus::Pending, DecisionStatus::Cancelled)
            | (DecisionStatus::InProgress, DecisionStatus::Completed)
            | (DecisionStatus::InProgress, DecisionStatus::Cancelled)
            | (DecisionStatus::Overdue, DecisionStatus::InProgress)
            | (DecisionStatus::Overdue, DecisionStatus::Completed)
            | (DecisionStatus::Overdue, DecisionStatus::Cancelled) => {}
            // Transition invalide
            _ => {
                return Err(format!(
                    "Invalid status transition from {} to {}",
                    self.status, new_status
                ));
            }
        }

        self.status = new_status.clone();
        self.updated_at = Utc::now();

        // Si on marque comme terminé, enregistrer la date
        if new_status == DecisionStatus::Completed {
            self.completed_at = Some(Utc::now());
        }

        Ok(())
    }

    /// Ajoute ou met à jour les notes de suivi
    pub fn add_notes(&mut self, notes: String) {
        self.notes = Some(notes);
        self.updated_at = Utc::now();
    }

    /// Vérifie le statut actuel et met à jour automatiquement si en retard
    pub fn check_and_update_overdue_status(&mut self) {
        if self.is_overdue() && self.status != DecisionStatus::Overdue {
            self.status = DecisionStatus::Overdue;
            self.updated_at = Utc::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_create_decision_success() {
        // Arrange
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();
        let subject = "Réparation ascenseur".to_string();
        let text = "Approuver les travaux de réparation de l'ascenseur pour un montant de 15000€"
            .to_string();
        let deadline = Some(Utc::now() + Duration::days(60));

        // Act
        let result = BoardDecision::new(
            building_id,
            meeting_id,
            subject.clone(),
            text.clone(),
            deadline,
        );

        // Assert
        assert!(result.is_ok());
        let decision = result.unwrap();
        assert_eq!(decision.building_id, building_id);
        assert_eq!(decision.meeting_id, meeting_id);
        assert_eq!(decision.subject, subject);
        assert_eq!(decision.decision_text, text);
        assert_eq!(decision.status, DecisionStatus::Pending);
        assert!(decision.completed_at.is_none());
        assert!(decision.deadline.is_some());
    }

    #[test]
    fn test_create_decision_empty_subject_fails() {
        // Arrange
        let subject = "   ".to_string(); // Seulement des espaces

        // Act
        let result = BoardDecision::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            subject,
            "Some text".to_string(),
            None,
        );

        // Assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Decision subject cannot be empty");
    }

    #[test]
    fn test_create_decision_empty_text_fails() {
        // Arrange
        let text = "".to_string();

        // Act
        let result = BoardDecision::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Subject".to_string(),
            text,
            None,
        );

        // Assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Decision text cannot be empty");
    }

    #[test]
    fn test_create_decision_past_deadline_fails() {
        // Arrange
        let deadline = Some(Utc::now() - Duration::days(1)); // Hier

        // Act
        let result = BoardDecision::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Subject".to_string(),
            "Text".to_string(),
            deadline,
        );

        // Assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Deadline must be in the future");
    }

    #[test]
    fn test_is_overdue_true() {
        // Arrange
        let deadline = Some(Utc::now() - Duration::days(1)); // Deadline dépassée
        let decision = BoardDecision {
            id: Uuid::new_v4(),
            building_id: Uuid::new_v4(),
            meeting_id: Uuid::new_v4(),
            subject: "Test".to_string(),
            decision_text: "Test text".to_string(),
            deadline,
            status: DecisionStatus::Pending,
            completed_at: None,
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Act & Assert
        assert!(decision.is_overdue());
    }

    #[test]
    fn test_is_overdue_false_no_deadline() {
        // Arrange
        let decision = BoardDecision {
            id: Uuid::new_v4(),
            building_id: Uuid::new_v4(),
            meeting_id: Uuid::new_v4(),
            subject: "Test".to_string(),
            decision_text: "Test text".to_string(),
            deadline: None, // Pas de deadline
            status: DecisionStatus::Pending,
            completed_at: None,
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Act & Assert
        assert!(!decision.is_overdue());
    }

    #[test]
    fn test_is_overdue_false_completed() {
        // Arrange
        let deadline = Some(Utc::now() - Duration::days(1)); // Deadline dépassée mais complété
        let decision = BoardDecision {
            id: Uuid::new_v4(),
            building_id: Uuid::new_v4(),
            meeting_id: Uuid::new_v4(),
            subject: "Test".to_string(),
            decision_text: "Test text".to_string(),
            deadline,
            status: DecisionStatus::Completed,
            completed_at: Some(Utc::now() - Duration::hours(2)),
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Act & Assert
        assert!(!decision.is_overdue()); // Pas overdue car déjà complété
    }

    #[test]
    fn test_update_status_valid_transitions() {
        // Arrange
        let mut decision = BoardDecision::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Test".to_string(),
            "Text".to_string(),
            Some(Utc::now() + Duration::days(30)),
        )
        .unwrap();

        // Act & Assert: Pending -> InProgress
        assert!(decision.update_status(DecisionStatus::InProgress).is_ok());
        assert_eq!(decision.status, DecisionStatus::InProgress);

        // Act & Assert: InProgress -> Completed
        assert!(decision.update_status(DecisionStatus::Completed).is_ok());
        assert_eq!(decision.status, DecisionStatus::Completed);
        assert!(decision.completed_at.is_some());
    }

    #[test]
    fn test_update_status_cannot_modify_completed() {
        // Arrange
        let mut decision = BoardDecision::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Test".to_string(),
            "Text".to_string(),
            None,
        )
        .unwrap();
        decision.update_status(DecisionStatus::InProgress).unwrap();
        decision.update_status(DecisionStatus::Completed).unwrap();

        // Act
        let result = decision.update_status(DecisionStatus::Pending);

        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Cannot change status of a completed decision"
        );
    }

    #[test]
    fn test_update_status_invalid_transition() {
        // Arrange
        let mut decision = BoardDecision::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Test".to_string(),
            "Text".to_string(),
            None,
        )
        .unwrap();

        // Act: Essayer Pending -> Completed (doit passer par InProgress)
        let result = decision.update_status(DecisionStatus::Completed);

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid status transition"));
    }

    #[test]
    fn test_add_notes() {
        // Arrange
        let mut decision = BoardDecision::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Test".to_string(),
            "Text".to_string(),
            None,
        )
        .unwrap();

        // Act
        decision.add_notes("Le syndic a confirmé le début des travaux".to_string());

        // Assert
        assert!(decision.notes.is_some());
        assert_eq!(
            decision.notes.unwrap(),
            "Le syndic a confirmé le début des travaux"
        );
    }

    #[test]
    fn test_check_and_update_overdue_status() {
        // Arrange
        let deadline = Some(Utc::now() - Duration::days(1)); // Déjà dépassée
        let mut decision = BoardDecision {
            id: Uuid::new_v4(),
            building_id: Uuid::new_v4(),
            meeting_id: Uuid::new_v4(),
            subject: "Test".to_string(),
            decision_text: "Test text".to_string(),
            deadline,
            status: DecisionStatus::Pending,
            completed_at: None,
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Act
        decision.check_and_update_overdue_status();

        // Assert
        assert_eq!(decision.status, DecisionStatus::Overdue);
    }

    #[test]
    fn test_decision_status_display() {
        assert_eq!(DecisionStatus::Pending.to_string(), "pending");
        assert_eq!(DecisionStatus::InProgress.to_string(), "in_progress");
        assert_eq!(DecisionStatus::Completed.to_string(), "completed");
        assert_eq!(DecisionStatus::Overdue.to_string(), "overdue");
        assert_eq!(DecisionStatus::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn test_decision_status_from_str() {
        assert_eq!(
            "pending".parse::<DecisionStatus>().unwrap(),
            DecisionStatus::Pending
        );
        assert_eq!(
            "COMPLETED".parse::<DecisionStatus>().unwrap(),
            DecisionStatus::Completed
        );
        assert_eq!(
            "in_progress".parse::<DecisionStatus>().unwrap(),
            DecisionStatus::InProgress
        );

        assert!("invalid".parse::<DecisionStatus>().is_err());
    }
}
