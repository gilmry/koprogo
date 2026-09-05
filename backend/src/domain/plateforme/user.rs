use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Roles métier KoproGo.
///
/// Story 3.1 — Sous-rôles métier (FR21 — séparation des pouvoirs comptables).
/// L'enum distingue :
/// - `Accountant` (générique, conservé pour compatibilité — encodeur + émetteur)
/// - `AccountantEncodeur` (saisie comptable amont : Invoice / Quote uniquement)
/// - `AccountantEmetteur` (sortie financière : Expense / CallForFunds uniquement)
/// - `CommunityModerator` (modération SEL, sondages, panneau d'affichage)
/// - Mandataires / spécialistes : Lawyer, Notary, Amo, Architect, Bet, Warden
///
/// INV-10 (séparation des pouvoirs) : un encodeur ne peut pas émettre, un émetteur
/// ne peut pas encoder. Le cumul des deux rôles passe par DEUX assignments
/// (cf. `UserRoleAssignment`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UserRole {
    SuperAdmin,
    Syndic,
    /// Comptable générique (rétrocompat — possède encodeur + émetteur).
    Accountant,
    /// Comptable encodeur — saisie comptable amont (facture, devis).
    AccountantEncodeur,
    /// Comptable émetteur — sortie financière (charges, appels de fonds).
    AccountantEmetteur,
    BoardMember, // Membre du conseil de copropriété
    Contractor,  // Prestataire externe (plombier, électricien, etc.)
    Owner,
    /// Modérateur communauté (SEL, sondages, notices).
    CommunityModerator,
    /// Avocat (conseil juridique de la copro).
    Lawyer,
    /// Notaire.
    Notary,
    /// Assistant Maître d'Ouvrage (AMO).
    Amo,
    /// Architecte.
    Architect,
    /// Bureau d'études techniques (BET).
    Bet,
    /// Concierge / gardien.
    Warden,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::SuperAdmin => write!(f, "superadmin"),
            UserRole::Syndic => write!(f, "syndic"),
            UserRole::Accountant => write!(f, "accountant"),
            UserRole::AccountantEncodeur => write!(f, "accountant.encodeur"),
            UserRole::AccountantEmetteur => write!(f, "accountant.emetteur"),
            UserRole::BoardMember => write!(f, "board_member"),
            UserRole::Contractor => write!(f, "contractor"),
            UserRole::Owner => write!(f, "owner"),
            UserRole::CommunityModerator => write!(f, "community.moderator"),
            UserRole::Lawyer => write!(f, "lawyer"),
            UserRole::Notary => write!(f, "notary"),
            UserRole::Amo => write!(f, "amo"),
            UserRole::Architect => write!(f, "architect"),
            UserRole::Bet => write!(f, "bet"),
            UserRole::Warden => write!(f, "warden"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Story 3.1 @edge: trim + lowercase, rejet caractères spéciaux non whitelist.
        let normalized = s.trim().to_lowercase();
        if normalized.is_empty() {
            return Err("Invalid user role: empty string".to_string());
        }
        // Whitelist stricte : seuls [a-z0-9_.] tolérés (refuse < > / etc. — @negative).
        if !normalized
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.')
        {
            return Err(format!("Invalid user role: invalid characters in {}", s));
        }
        match normalized.as_str() {
            "superadmin" => Ok(UserRole::SuperAdmin),
            "syndic" => Ok(UserRole::Syndic),
            "accountant" => Ok(UserRole::Accountant),
            "accountant.encodeur" => Ok(UserRole::AccountantEncodeur),
            "accountant.emetteur" => Ok(UserRole::AccountantEmetteur),
            "board_member" => Ok(UserRole::BoardMember),
            "contractor" => Ok(UserRole::Contractor),
            "owner" => Ok(UserRole::Owner),
            "community.moderator" => Ok(UserRole::CommunityModerator),
            "lawyer" => Ok(UserRole::Lawyer),
            "notary" => Ok(UserRole::Notary),
            "amo" => Ok(UserRole::Amo),
            "architect" => Ok(UserRole::Architect),
            "bet" => Ok(UserRole::Bet),
            "warden" => Ok(UserRole::Warden),
            _ => Err(format!("Invalid user role: {}", s)),
        }
    }
}

impl UserRole {
    // ===============================================================
    // === Story 3.1 — Permission helpers (FR21 séparation pouvoirs)
    // ===============================================================

    /// Peut saisir une facture / devis (entrée comptable amont).
    ///
    /// INV-10 : seuls encodeurs comptables + syndic + superadmin (les syndics
    /// gardent la pleine autorité en l'absence de comptable dédié).
    pub fn can_encode_invoices(&self) -> bool {
        matches!(
            self,
            UserRole::SuperAdmin
                | UserRole::Syndic
                | UserRole::Accountant
                | UserRole::AccountantEncodeur
        )
    }

    /// Peut émettre une charge (sortie financière).
    ///
    /// INV-10 : seuls émetteurs comptables + syndic + superadmin.
    /// Un `AccountantEncodeur` SEUL ne peut PAS émettre — il faut un
    /// `AccountantEmetteur` (ou cumul des deux assignments).
    pub fn can_emit_expenses(&self) -> bool {
        matches!(
            self,
            UserRole::SuperAdmin
                | UserRole::Syndic
                | UserRole::Accountant
                | UserRole::AccountantEmetteur
        )
    }

    /// Peut créer un appel de fonds.
    ///
    /// Même règle que `can_emit_expenses` : c'est une sortie financière.
    pub fn can_create_call_for_funds(&self) -> bool {
        matches!(
            self,
            UserRole::SuperAdmin
                | UserRole::Syndic
                | UserRole::Accountant
                | UserRole::AccountantEmetteur
        )
    }

    /// Peut modérer la communauté (SEL, sondages, panneau d'affichage).
    pub fn can_moderate_community(&self) -> bool {
        matches!(
            self,
            UserRole::SuperAdmin | UserRole::Syndic | UserRole::CommunityModerator
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct User {
    pub id: Uuid,

    #[validate(email(message = "Email must be valid"))]
    pub email: String,

    #[serde(skip_serializing)]
    pub password_hash: String,

    #[validate(length(min = 2, message = "First name must be at least 2 characters"))]
    pub first_name: String,

    #[validate(length(min = 2, message = "Last name must be at least 2 characters"))]
    pub last_name: String,

    pub role: UserRole,

    pub organization_id: Option<Uuid>,

    pub is_active: bool,

    // GDPR Article 18: Right to Restriction of Processing
    pub processing_restricted: bool,
    pub processing_restricted_at: Option<DateTime<Utc>>,

    // GDPR Article 21: Right to Object (Marketing opt-out)
    pub marketing_opt_out: bool,
    pub marketing_opt_out_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        email: String,
        password_hash: String,
        first_name: String,
        last_name: String,
        role: UserRole,
        organization_id: Option<Uuid>,
    ) -> Result<Self, String> {
        let user = Self {
            id: Uuid::new_v4(),
            email: email.to_lowercase().trim().to_string(),
            password_hash,
            first_name: first_name.trim().to_string(),
            last_name: last_name.trim().to_string(),
            role,
            organization_id,
            is_active: true,
            processing_restricted: false,
            processing_restricted_at: None,
            marketing_opt_out: false,
            marketing_opt_out_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        user.validate()
            .map_err(|e| format!("Validation error: {}", e))?;

        Ok(user)
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    pub fn update_profile(&mut self, first_name: String, last_name: String) -> Result<(), String> {
        self.first_name = first_name.trim().to_string();
        self.last_name = last_name.trim().to_string();
        self.updated_at = Utc::now();

        self.validate()
            .map_err(|e| format!("Validation error: {}", e))?;

        Ok(())
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    pub fn can_access_building(&self, building_org_id: Option<Uuid>) -> bool {
        match self.role {
            UserRole::SuperAdmin => true,
            _ => self.organization_id == building_org_id,
        }
    }

    // GDPR Article 16: Right to Rectification
    // Users can correct inaccurate personal data
    pub fn rectify_data(
        &mut self,
        email: Option<String>,
        first_name: Option<String>,
        last_name: Option<String>,
    ) -> Result<(), String> {
        // At least one field must be provided
        if email.is_none() && first_name.is_none() && last_name.is_none() {
            return Err("No fields provided for rectification".to_string());
        }

        // Validate email format BEFORE modifying anything
        if let Some(ref new_email) = email {
            let email_normalized = new_email.to_lowercase().trim().to_string();
            if !email_normalized.contains('@') || email_normalized.len() < 3 {
                return Err(format!("Invalid email format: {}", new_email));
            }
        }

        // Validate names are not empty BEFORE modifying
        if let Some(ref new_first_name) = first_name {
            if new_first_name.trim().is_empty() {
                return Err("First name cannot be empty".to_string());
            }
        }
        if let Some(ref new_last_name) = last_name {
            if new_last_name.trim().is_empty() {
                return Err("Last name cannot be empty".to_string());
            }
        }

        // Only apply changes after validation passes
        if let Some(new_email) = email {
            self.email = new_email.to_lowercase().trim().to_string();
        }
        if let Some(new_first_name) = first_name {
            self.first_name = new_first_name.trim().to_string();
        }
        if let Some(new_last_name) = last_name {
            self.last_name = new_last_name.trim().to_string();
        }

        self.updated_at = Utc::now();

        // Final validation with full validator
        self.validate()
            .map_err(|e| format!("Validation error: {}", e))?;

        Ok(())
    }

    // GDPR Article 18: Right to Restriction of Processing
    // Users can request temporary limitation of data processing
    pub fn restrict_processing(&mut self) -> Result<(), String> {
        if self.processing_restricted {
            return Err("Processing is already restricted for this user".to_string());
        }

        self.processing_restricted = true;
        self.processing_restricted_at = Some(Utc::now());
        self.updated_at = Utc::now();

        Ok(())
    }

    // GDPR Article 18: Unrestrict processing (admin action or legal requirement met)
    pub fn unrestrict_processing(&mut self) {
        self.processing_restricted = false;
        // Keep processing_restricted_at for audit trail
        self.updated_at = Utc::now();
    }

    // GDPR Article 21: Right to Object (Marketing opt-out)
    // Users can object to marketing communications and profiling
    pub fn set_marketing_opt_out(&mut self, opt_out: bool) {
        if opt_out && !self.marketing_opt_out {
            // User is opting out
            self.marketing_opt_out = true;
            self.marketing_opt_out_at = Some(Utc::now());
        } else if !opt_out && self.marketing_opt_out {
            // User is opting back in
            self.marketing_opt_out = false;
            // Keep marketing_opt_out_at for audit trail
        }

        self.updated_at = Utc::now();
    }

    // Helper to check if user data processing is allowed
    pub fn can_process_data(&self) -> bool {
        !self.processing_restricted
    }

    // Helper to check if marketing communications are allowed
    pub fn can_send_marketing(&self) -> bool {
        !self.marketing_opt_out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // ============================================================
    // === Story 3.1 — UserRole sous-rôles + helpers — 4 catégories
    // ============================================================

    // --- @happy : parsing canonique + helpers nominaux ---

    #[test]
    fn happy_parse_accountant_encodeur() {
        assert_eq!(
            UserRole::from_str("accountant.encodeur").unwrap(),
            UserRole::AccountantEncodeur
        );
    }

    #[test]
    fn happy_parse_accountant_emetteur() {
        assert_eq!(
            UserRole::from_str("accountant.emetteur").unwrap(),
            UserRole::AccountantEmetteur
        );
    }

    #[test]
    fn happy_parse_community_moderator() {
        assert_eq!(
            UserRole::from_str("community.moderator").unwrap(),
            UserRole::CommunityModerator
        );
    }

    #[test]
    fn happy_parse_mandataires() {
        assert_eq!(UserRole::from_str("lawyer").unwrap(), UserRole::Lawyer);
        assert_eq!(UserRole::from_str("notary").unwrap(), UserRole::Notary);
        assert_eq!(UserRole::from_str("amo").unwrap(), UserRole::Amo);
        assert_eq!(
            UserRole::from_str("architect").unwrap(),
            UserRole::Architect
        );
        assert_eq!(UserRole::from_str("bet").unwrap(), UserRole::Bet);
        assert_eq!(UserRole::from_str("warden").unwrap(), UserRole::Warden);
    }

    #[test]
    fn happy_encodeur_can_encode_invoices() {
        assert!(UserRole::AccountantEncodeur.can_encode_invoices());
    }

    #[test]
    fn happy_emetteur_can_emit_expenses() {
        assert!(UserRole::AccountantEmetteur.can_emit_expenses());
        assert!(UserRole::AccountantEmetteur.can_create_call_for_funds());
    }

    #[test]
    fn happy_community_moderator_can_moderate() {
        assert!(UserRole::CommunityModerator.can_moderate_community());
    }

    #[test]
    fn happy_display_round_trip() {
        for role in [
            UserRole::SuperAdmin,
            UserRole::Syndic,
            UserRole::Accountant,
            UserRole::AccountantEncodeur,
            UserRole::AccountantEmetteur,
            UserRole::BoardMember,
            UserRole::Contractor,
            UserRole::Owner,
            UserRole::CommunityModerator,
            UserRole::Lawyer,
            UserRole::Notary,
            UserRole::Amo,
            UserRole::Architect,
            UserRole::Bet,
            UserRole::Warden,
        ] {
            let s = role.to_string();
            let parsed = UserRole::from_str(&s)
                .unwrap_or_else(|e| panic!("Round-trip failed for {:?} -> {} : {}", role, s, e));
            assert_eq!(parsed, role);
        }
    }

    // --- @edge : trim/lowercase, syndic legacy still has all powers ---

    #[test]
    fn edge_parse_accountant_encodeur_trim_uppercase() {
        assert_eq!(
            UserRole::from_str("  ACCOUNTANT.ENCODEUR  ").unwrap(),
            UserRole::AccountantEncodeur
        );
    }

    #[test]
    fn edge_parse_mixed_case_community_moderator() {
        assert_eq!(
            UserRole::from_str("CoMmUnItY.MoDeRaToR").unwrap(),
            UserRole::CommunityModerator
        );
    }

    #[test]
    fn edge_syndic_keeps_all_finance_powers() {
        // Syndic = compétence pleine en l'absence de comptable dédié.
        assert!(UserRole::Syndic.can_encode_invoices());
        assert!(UserRole::Syndic.can_emit_expenses());
        assert!(UserRole::Syndic.can_create_call_for_funds());
        assert!(UserRole::Syndic.can_moderate_community());
    }

    #[test]
    fn edge_generic_accountant_keeps_both_powers() {
        // Rétrocompat : Accountant générique = encodeur + émetteur.
        assert!(UserRole::Accountant.can_encode_invoices());
        assert!(UserRole::Accountant.can_emit_expenses());
        assert!(UserRole::Accountant.can_create_call_for_funds());
    }

    #[test]
    fn edge_cumul_encodeur_et_emetteur_via_assignments() {
        // INV-10 : un user qui cumule les 2 assignments a les pleins droits comptables.
        // On simule le cumul en testant que chaque rôle apporte sa capacité.
        let encodeur = UserRole::AccountantEncodeur;
        let emetteur = UserRole::AccountantEmetteur;
        // Union des capacités (le call-site itère sur les assignments).
        let can_encode = encodeur.can_encode_invoices() || emetteur.can_encode_invoices();
        let can_emit = encodeur.can_emit_expenses() || emetteur.can_emit_expenses();
        let can_call = encodeur.can_create_call_for_funds() || emetteur.can_create_call_for_funds();
        assert!(
            can_encode && can_emit && can_call,
            "Encodeur+Emetteur cumul should grant all finance powers"
        );
    }

    // --- @security : INV-10 séparation des pouvoirs ---

    #[test]
    fn security_encodeur_cannot_emit_expenses() {
        // FR21 / INV-10 : un encodeur seul NE peut PAS émettre une charge.
        assert!(!UserRole::AccountantEncodeur.can_emit_expenses());
    }

    #[test]
    fn security_encodeur_cannot_create_call_for_funds() {
        assert!(!UserRole::AccountantEncodeur.can_create_call_for_funds());
    }

    #[test]
    fn security_emetteur_cannot_encode_invoices() {
        // Symétrique : un émetteur seul NE peut PAS saisir une facture.
        assert!(!UserRole::AccountantEmetteur.can_encode_invoices());
    }

    #[test]
    fn security_owner_has_no_finance_power() {
        assert!(!UserRole::Owner.can_encode_invoices());
        assert!(!UserRole::Owner.can_emit_expenses());
        assert!(!UserRole::Owner.can_create_call_for_funds());
        assert!(!UserRole::Owner.can_moderate_community());
    }

    #[test]
    fn security_community_moderator_has_no_finance_power() {
        assert!(!UserRole::CommunityModerator.can_emit_expenses());
        assert!(!UserRole::CommunityModerator.can_encode_invoices());
        assert!(!UserRole::CommunityModerator.can_create_call_for_funds());
    }

    #[test]
    fn security_mandataires_have_no_finance_power() {
        for role in [
            UserRole::Lawyer,
            UserRole::Notary,
            UserRole::Amo,
            UserRole::Architect,
            UserRole::Bet,
            UserRole::Warden,
            UserRole::Contractor,
            UserRole::BoardMember,
        ] {
            assert!(
                !role.can_emit_expenses(),
                "{} should not be able to emit expenses",
                role
            );
            assert!(
                !role.can_encode_invoices(),
                "{} should not be able to encode invoices",
                role
            );
            assert!(
                !role.can_create_call_for_funds(),
                "{} should not be able to create call for funds",
                role
            );
            assert!(
                !role.can_moderate_community(),
                "{} should not be able to moderate community",
                role
            );
        }
    }

    // --- @negative : rôles inconnus, vide, caractères spéciaux ---

    #[test]
    fn negative_unknown_role_rejected() {
        let err = UserRole::from_str("hackerman").unwrap_err();
        assert!(
            err.contains("Invalid user role"),
            "Unknown role should fail typed: got {}",
            err
        );
    }

    #[test]
    fn negative_empty_role_rejected() {
        let err = UserRole::from_str("").unwrap_err();
        assert!(
            err.contains("empty") || err.contains("Invalid"),
            "Empty role should fail: got {}",
            err
        );
    }

    #[test]
    fn negative_whitespace_only_role_rejected() {
        assert!(UserRole::from_str("   ").is_err());
    }

    #[test]
    fn negative_role_with_special_chars_rejected() {
        // Tentative d'injection : refusée AVANT le match sur la whitelist.
        assert!(UserRole::from_str("accountant.<script>").is_err());
        assert!(UserRole::from_str("accountant';drop").is_err());
        assert!(UserRole::from_str("accountant/encodeur").is_err());
    }

    #[test]
    fn negative_partial_subrole_rejected() {
        // "accountant.foo" n'est pas dans la whitelist.
        assert!(UserRole::from_str("accountant.foo").is_err());
        assert!(UserRole::from_str("community.spam").is_err());
    }

    // ============================================================
    // === Tests existants (rétrocompat User entity)
    // ============================================================

    #[test]
    fn test_create_user_success() {
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Syndic,
            Some(Uuid::new_v4()),
        );

        assert!(user.is_ok());
        let user = user.unwrap();
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.full_name(), "John Doe");
        assert!(user.is_active);
    }

    #[test]
    fn test_create_user_invalid_email() {
        let user = User::new(
            "invalid-email".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Syndic,
            None,
        );

        assert!(user.is_err());
    }

    #[test]
    fn test_update_profile() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Syndic,
            None,
        )
        .unwrap();

        let result = user.update_profile("Jane".to_string(), "Smith".to_string());
        assert!(result.is_ok());
        assert_eq!(user.full_name(), "Jane Smith");
    }

    #[test]
    fn test_deactivate_user() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Syndic,
            None,
        )
        .unwrap();

        user.deactivate();
        assert!(!user.is_active);
    }

    #[test]
    fn test_superadmin_can_access_all_buildings() {
        let user = User::new(
            "admin@example.com".to_string(),
            "hashed_password".to_string(),
            "Admin".to_string(),
            "User".to_string(),
            UserRole::SuperAdmin,
            None,
        )
        .unwrap();

        assert!(user.can_access_building(Some(Uuid::new_v4())));
        assert!(user.can_access_building(None));
    }

    #[test]
    fn test_regular_user_access_control() {
        let org_id = Uuid::new_v4();
        let user = User::new(
            "syndic@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Syndic".to_string(),
            UserRole::Syndic,
            Some(org_id),
        )
        .unwrap();

        assert!(user.can_access_building(Some(org_id)));
        assert!(!user.can_access_building(Some(Uuid::new_v4())));
    }

    // GDPR Article 16 Tests
    #[test]
    fn test_rectify_data_success() {
        let mut user = User::new(
            "old@example.com".to_string(),
            "hashed_password".to_string(),
            "OldFirst".to_string(),
            "OldLast".to_string(),
            UserRole::Owner,
            None,
        )
        .unwrap();

        let result = user.rectify_data(
            Some("new@example.com".to_string()),
            Some("NewFirst".to_string()),
            Some("NewLast".to_string()),
        );

        assert!(result.is_ok());
        assert_eq!(user.email, "new@example.com");
        assert_eq!(user.first_name, "NewFirst");
        assert_eq!(user.last_name, "NewLast");
    }

    #[test]
    fn test_rectify_data_partial() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Owner,
            None,
        )
        .unwrap();

        let result = user.rectify_data(None, Some("Jane".to_string()), None);

        assert!(result.is_ok());
        assert_eq!(user.email, "test@example.com"); // unchanged
        assert_eq!(user.first_name, "Jane"); // changed
        assert_eq!(user.last_name, "Doe"); // unchanged
    }

    #[test]
    fn test_rectify_data_invalid_email() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Owner,
            None,
        )
        .unwrap();

        let result = user.rectify_data(Some("invalid-email".to_string()), None, None);

        assert!(result.is_err());
        assert_eq!(user.email, "test@example.com"); // unchanged on error
    }

    // GDPR Article 18 Tests
    #[test]
    fn test_restrict_processing_success() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Owner,
            None,
        )
        .unwrap();

        assert!(!user.processing_restricted);
        assert!(user.can_process_data());

        let result = user.restrict_processing();

        assert!(result.is_ok());
        assert!(user.processing_restricted);
        assert!(user.processing_restricted_at.is_some());
        assert!(!user.can_process_data());
    }

    #[test]
    fn test_restrict_processing_already_restricted() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Owner,
            None,
        )
        .unwrap();

        user.restrict_processing().unwrap();

        let result = user.restrict_processing();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Processing is already restricted"));
    }

    #[test]
    fn test_unrestrict_processing() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Owner,
            None,
        )
        .unwrap();

        user.restrict_processing().unwrap();
        assert!(!user.can_process_data());

        let restriction_timestamp = user.processing_restricted_at;

        user.unrestrict_processing();

        assert!(!user.processing_restricted);
        assert!(user.can_process_data());
        assert_eq!(user.processing_restricted_at, restriction_timestamp); // Audit trail preserved
    }

    // GDPR Article 21 Tests
    #[test]
    fn test_set_marketing_opt_out() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Owner,
            None,
        )
        .unwrap();

        assert!(!user.marketing_opt_out);
        assert!(user.can_send_marketing());

        user.set_marketing_opt_out(true);

        assert!(user.marketing_opt_out);
        assert!(user.marketing_opt_out_at.is_some());
        assert!(!user.can_send_marketing());
    }

    #[test]
    fn test_set_marketing_opt_in_after_opt_out() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Owner,
            None,
        )
        .unwrap();

        user.set_marketing_opt_out(true);
        assert!(!user.can_send_marketing());

        let opt_out_timestamp = user.marketing_opt_out_at;

        user.set_marketing_opt_out(false);

        assert!(!user.marketing_opt_out);
        assert!(user.can_send_marketing());
        assert_eq!(user.marketing_opt_out_at, opt_out_timestamp); // Audit trail preserved
    }

    #[test]
    fn test_gdpr_defaults_on_new_user() {
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            UserRole::Owner,
            None,
        )
        .unwrap();

        // GDPR defaults
        assert!(!user.processing_restricted);
        assert!(user.processing_restricted_at.is_none());
        assert!(!user.marketing_opt_out);
        assert!(user.marketing_opt_out_at.is_none());

        // Helper methods
        assert!(user.can_process_data());
        assert!(user.can_send_marketing());
    }
}
