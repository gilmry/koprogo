use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Position du membre du conseil de copropriété
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BoardPosition {
    President, // Président du conseil
    Treasurer, // Trésorier du conseil
    Member,    // Membre simple
}

impl std::fmt::Display for BoardPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoardPosition::President => write!(f, "president"),
            BoardPosition::Treasurer => write!(f, "treasurer"),
            BoardPosition::Member => write!(f, "member"),
        }
    }
}

impl std::str::FromStr for BoardPosition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "president" => Ok(BoardPosition::President),
            "treasurer" => Ok(BoardPosition::Treasurer),
            "member" => Ok(BoardPosition::Member),
            _ => Err(format!("Invalid board position: {}", s)),
        }
    }
}

/// Membre du conseil de copropriété (Article 577-8/4 Code Civil belge)
/// Obligation légale pour immeubles >20 lots
/// Les membres doivent être des copropriétaires (Owner), pas nécessairement des utilisateurs de la plateforme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardMember {
    pub id: Uuid,
    pub owner_id: Uuid, // Référence à la table owners (copropriétaires)
    pub building_id: Uuid,
    pub position: BoardPosition,
    pub mandate_start: DateTime<Utc>,
    pub mandate_end: DateTime<Utc>,
    pub elected_by_meeting_id: Uuid, // ID de l'AG qui a élu ce membre
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BoardMember {
    /// Crée un nouveau membre du conseil avec validation.
    ///
    /// Validation per Art. 3.90 CC (conseil de copropriété) : durée du mandat
    /// ≈ 1 an. Fenêtre 330–395 jours, alignée sur la contrainte DB
    /// `mandate_duration_one_year`. NB : le plafond 3 ans est la règle *syndic*
    /// (Art. 3.89 CC) — un mandat distinct, non applicable au conseil
    /// (cf. `tests/features/legal_compliance.feature` §Art. 3.89 / Art. 3.90).
    pub fn new(
        owner_id: Uuid,
        building_id: Uuid,
        position: BoardPosition,
        mandate_start: DateTime<Utc>,
        mandate_end: DateTime<Utc>,
        elected_by_meeting_id: Uuid,
    ) -> Result<Self, String> {
        // Validation: mandate_start doit être avant mandate_end
        if mandate_start >= mandate_end {
            return Err("Mandate start date must be before end date".to_string());
        }

        // Durée du mandat ≈ 1 an (Art. 3.90 CC, conseil de copropriété).
        // Fenêtre 330–395 j alignée sur la contrainte DB `mandate_duration_one_year`.
        let duration_days = (mandate_end - mandate_start).num_days();
        if !(330..=395).contains(&duration_days) {
            return Err("Mandate duration must be approximately 1 year (Art. 3.90 CC)".to_string());
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            owner_id,
            building_id,
            position,
            mandate_start,
            mandate_end,
            elected_by_meeting_id,
            created_at: now,
            updated_at: now,
        })
    }

    /// Vérifie si le mandat est actuellement actif
    pub fn is_active(&self) -> bool {
        let now = Utc::now();
        now >= self.mandate_start && now <= self.mandate_end
    }

    /// Calcule le nombre de jours restants dans le mandat
    /// Retourne 0 si le mandat est expiré
    pub fn days_remaining(&self) -> i64 {
        let now = Utc::now();
        if now > self.mandate_end {
            return 0;
        }
        (self.mandate_end - now).num_days()
    }

    /// Vérifie si le mandat expire bientôt (< 60 jours)
    pub fn expires_soon(&self) -> bool {
        self.days_remaining() > 0 && self.days_remaining() < 60
    }

    /// Renouvelle le mandat (Art. 3.90 CC, conseil de copropriété : ≈ 1 an).
    /// mandate_duration_days: durée du nouveau mandat en jours (≈ 1 an, 330–395).
    pub fn extend_mandate(
        &mut self,
        mandate_duration_days: i64,
        new_elected_by_meeting_id: Uuid,
    ) -> Result<(), String> {
        if !self.expires_soon() && self.is_active() {
            return Err("Cannot extend mandate more than 60 days before expiration".to_string());
        }

        // Durée ≈ 1 an (Art. 3.90 CC) — fenêtre alignée sur la contrainte DB.
        if !(330..=395).contains(&mandate_duration_days) {
            return Err("Mandate duration must be approximately 1 year (Art. 3.90 CC)".to_string());
        }

        // Nouveau mandat commence à la fin de l'ancien
        self.mandate_start = self.mandate_end;
        self.mandate_end = self.mandate_start + Duration::days(mandate_duration_days);
        self.elected_by_meeting_id = new_elected_by_meeting_id;
        self.updated_at = Utc::now();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_board_member_success() {
        // Arrange
        let owner_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();
        let start = Utc::now();
        let end = start + Duration::days(365);

        // Act
        let result = BoardMember::new(
            owner_id,
            building_id,
            BoardPosition::President,
            start,
            end,
            meeting_id,
        );

        // Assert
        assert!(result.is_ok());
        let member = result.unwrap();
        assert_eq!(member.owner_id, owner_id);
        assert_eq!(member.building_id, building_id);
        assert_eq!(member.position, BoardPosition::President);
        assert_eq!(member.mandate_start, start);
        assert_eq!(member.mandate_end, end);
        assert_eq!(member.elected_by_meeting_id, meeting_id);
    }

    #[test]
    fn test_mandate_duration_one_year() {
        // Arrange
        let start = Utc::now();
        let end = start + Duration::days(365);

        // Act
        let result = BoardMember::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            BoardPosition::Member,
            start,
            end,
            Uuid::new_v4(),
        );

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_mandate_duration_too_short_fails() {
        // @negative — Arrange: 300 j < 330 (borne basse ≈ 1 an, Art. 3.90 CC)
        let start = Utc::now();
        let end = start + Duration::days(300);

        // Act
        let result = BoardMember::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            BoardPosition::Member,
            start,
            end,
            Uuid::new_v4(),
        );

        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Mandate duration must be approximately 1 year (Art. 3.90 CC)"
        );
    }

    #[test]
    fn test_mandate_duration_too_long_fails() {
        // @negative — 730 j (2 ans) > 395 : règle conseil ≈ 1 an (Art. 3.90 CC).
        // (Le plafond 3 ans est la règle *syndic* Art. 3.89, distincte.)
        let start = Utc::now();
        let end = start + Duration::days(730);

        // Act
        let result = BoardMember::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            BoardPosition::Member,
            start,
            end,
            Uuid::new_v4(),
        );

        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Mandate duration must be approximately 1 year (Art. 3.90 CC)"
        );
    }

    #[test]
    fn test_mandate_duration_boundaries() {
        // @edge — fenêtre 330–395 j (Art. 3.90 CC), alignée contrainte DB.
        let start = Utc::now();
        let mk = |days: i64| {
            BoardMember::new(
                Uuid::new_v4(),
                Uuid::new_v4(),
                BoardPosition::Member,
                start,
                start + Duration::days(days),
                Uuid::new_v4(),
            )
        };

        // Bornes incluses : 330 et 395 acceptées.
        assert!(mk(330).is_ok(), "330 days must be accepted (lower bound)");
        assert!(mk(395).is_ok(), "395 days must be accepted (upper bound)");
        // Juste hors bornes : 329 et 396 rejetées.
        assert!(mk(329).is_err(), "329 days must be rejected (< 330)");
        assert!(mk(396).is_err(), "396 days must be rejected (> 395)");
    }

    #[test]
    fn test_mandate_duration_far_too_long_fails() {
        // @negative — 1100 j (~3 ans, l'ancienne règle syndic mal appliquée)
        // doit désormais être rejeté pour le conseil (Art. 3.90 CC, ≈ 1 an).
        let start = Utc::now();
        let end = start + Duration::days(1100);

        // Act
        let result = BoardMember::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            BoardPosition::Member,
            start,
            end,
            Uuid::new_v4(),
        );

        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Mandate duration must be approximately 1 year (Art. 3.90 CC)"
        );
    }

    #[test]
    fn test_mandate_start_after_end_fails() {
        // Arrange
        let start = Utc::now();
        let end = start - Duration::days(10); // End avant start

        // Act
        let result = BoardMember::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            BoardPosition::President,
            start,
            end,
            Uuid::new_v4(),
        );

        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Mandate start date must be before end date"
        );
    }

    #[test]
    fn test_is_active_mandate() {
        // Arrange
        let start = Utc::now() - Duration::days(10); // Commencé il y a 10 jours
        let end = Utc::now() + Duration::days(355); // Expire dans 355 jours
        let member = BoardMember::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            BoardPosition::Member,
            start,
            end,
            Uuid::new_v4(),
        )
        .unwrap();

        // Act & Assert
        assert!(member.is_active());
    }

    #[test]
    fn test_is_not_active_future_mandate() {
        // Arrange
        let start = Utc::now() + Duration::days(10); // Commence dans 10 jours
        let end = start + Duration::days(365);
        let member = BoardMember::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            BoardPosition::Member,
            start,
            end,
            Uuid::new_v4(),
        )
        .unwrap();

        // Act & Assert
        assert!(!member.is_active());
    }

    #[test]
    fn test_days_remaining_calculation() {
        // Arrange
        let start = Utc::now() - Duration::days(300); // Commencé il y a 300 jours
        let end = start + Duration::days(365); // Expire dans 65 jours
        let member = BoardMember::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            BoardPosition::Treasurer,
            start,
            end,
            Uuid::new_v4(),
        )
        .unwrap();

        // Act
        let remaining = member.days_remaining();

        // Assert
        assert!((64..=66).contains(&remaining)); // ±1 jour de tolérance
    }

    #[test]
    fn test_days_remaining_expired_returns_zero() {
        // Arrange
        let start = Utc::now() - Duration::days(400); // Commencé il y a 400 jours
        let end = start + Duration::days(365); // Expiré il y a 35 jours
        let member = BoardMember::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            BoardPosition::Member,
            start,
            end,
            Uuid::new_v4(),
        )
        .unwrap();

        // Act
        let remaining = member.days_remaining();

        // Assert
        assert_eq!(remaining, 0);
    }

    #[test]
    fn test_expires_soon_true() {
        // Arrange
        let start = Utc::now() - Duration::days(320); // Commencé il y a 320 jours
        let end = start + Duration::days(365); // Expire dans 45 jours
        let member = BoardMember::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            BoardPosition::President,
            start,
            end,
            Uuid::new_v4(),
        )
        .unwrap();

        // Act & Assert
        assert!(member.expires_soon());
    }

    #[test]
    fn test_expires_soon_false_far_expiration() {
        // Arrange
        let start = Utc::now() - Duration::days(100); // Commencé il y a 100 jours
        let end = start + Duration::days(365); // Expire dans 265 jours
        let member = BoardMember::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            BoardPosition::Member,
            start,
            end,
            Uuid::new_v4(),
        )
        .unwrap();

        // Act & Assert
        assert!(!member.expires_soon());
    }

    #[test]
    fn test_extend_mandate_success() {
        // Arrange
        let start = Utc::now() - Duration::days(320); // Expire dans 45 jours
        let end = start + Duration::days(365);
        let new_meeting_id = Uuid::new_v4();
        let mut member = BoardMember::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            BoardPosition::President,
            start,
            end,
            Uuid::new_v4(),
        )
        .unwrap();

        let original_end = member.mandate_end;

        // Act: Extend for another 365 days
        let result = member.extend_mandate(365, new_meeting_id);

        // Assert
        assert!(result.is_ok());
        assert_eq!(member.mandate_start, original_end);
        assert_eq!(member.mandate_end, original_end + Duration::days(365));
        assert_eq!(member.elected_by_meeting_id, new_meeting_id);
    }

    #[test]
    fn test_extend_mandate_too_long_fails() {
        // @negative — renouveler pour 2 ans (730 j) > 395 : rejeté
        // (conseil ≈ 1 an, Art. 3.90 CC). L'ancien comportement (ok) appliquait
        // à tort la règle syndic 1–3 ans (Art. 3.89).
        let start = Utc::now() - Duration::days(320); // Expire dans 45 jours
        let end = start + Duration::days(365);
        let new_meeting_id = Uuid::new_v4();
        let mut member = BoardMember::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            BoardPosition::President,
            start,
            end,
            Uuid::new_v4(),
        )
        .unwrap();

        // Act: Extend for 2 years (730 days)
        let result = member.extend_mandate(730, new_meeting_id);

        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Mandate duration must be approximately 1 year (Art. 3.90 CC)"
        );
    }

    #[test]
    fn test_extend_mandate_far_too_long_fails() {
        // @negative — renouveler pour ~3 ans (1100 j) rejeté pour le conseil
        // (Art. 3.90 CC, ≈ 1 an).
        let start = Utc::now() - Duration::days(320); // Expire dans 45 jours
        let end = start + Duration::days(365);
        let new_meeting_id = Uuid::new_v4();
        let mut member = BoardMember::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            BoardPosition::President,
            start,
            end,
            Uuid::new_v4(),
        )
        .unwrap();

        // Act: Try to extend for ~3 years (1100 days)
        let result = member.extend_mandate(1100, new_meeting_id);

        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Mandate duration must be approximately 1 year (Art. 3.90 CC)"
        );
    }

    #[test]
    fn test_extend_mandate_fails_too_early() {
        // Arrange
        let start = Utc::now() - Duration::days(100); // Expire dans 265 jours
        let end = start + Duration::days(365);
        let mut member = BoardMember::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            BoardPosition::Member,
            start,
            end,
            Uuid::new_v4(),
        )
        .unwrap();

        // Act
        let result = member.extend_mandate(365, Uuid::new_v4());

        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Cannot extend mandate more than 60 days before expiration"
        );
    }

    #[test]
    fn test_board_position_display() {
        assert_eq!(BoardPosition::President.to_string(), "president");
        assert_eq!(BoardPosition::Treasurer.to_string(), "treasurer");
        assert_eq!(BoardPosition::Member.to_string(), "member");
    }

    #[test]
    fn test_board_position_from_str() {
        assert_eq!(
            "president".parse::<BoardPosition>().unwrap(),
            BoardPosition::President
        );
        assert_eq!(
            "President".parse::<BoardPosition>().unwrap(),
            BoardPosition::President
        );
        assert_eq!(
            "TREASURER".parse::<BoardPosition>().unwrap(),
            BoardPosition::Treasurer
        );
        assert_eq!(
            "member".parse::<BoardPosition>().unwrap(),
            BoardPosition::Member
        );

        assert!("invalid".parse::<BoardPosition>().is_err());
    }
}
