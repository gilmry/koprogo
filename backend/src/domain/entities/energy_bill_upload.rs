use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::energy_campaign::EnergyType;

/// Upload de facture énergie (données chiffrées)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnergyBillUpload {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub unit_id: Uuid,
    pub building_id: Uuid,
    pub organization_id: Uuid,

    // Données extraites (chiffrées)
    pub bill_period_start: DateTime<Utc>,
    pub bill_period_end: DateTime<Utc>,
    pub total_kwh_encrypted: Vec<u8>, // AES-256-GCM
    pub energy_type: EnergyType,
    pub provider: Option<String>,
    pub postal_code: String, // Pour tarifs régionaux CREG

    // Authentification facture
    pub file_hash: String,           // SHA-256
    pub file_path_encrypted: String, // S3 path chiffré
    pub ocr_confidence: f64,         // 0-100
    pub manually_verified: bool,

    // Upload metadata
    pub uploaded_by: Uuid,
    pub uploaded_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
    pub verified_by: Option<Uuid>,

    // GDPR Consent
    pub consent_timestamp: DateTime<Utc>,
    pub consent_ip: String,
    pub consent_user_agent: String,
    pub consent_signature_hash: String,

    // Privacy & Retention
    pub anonymized: bool,
    pub retention_until: DateTime<Utc>, // Auto-delete
    pub deleted_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl EnergyBillUpload {
    /// Créer nouvel upload avec consentement
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        campaign_id: Uuid,
        unit_id: Uuid,
        building_id: Uuid,
        organization_id: Uuid,
        bill_period_start: DateTime<Utc>,
        bill_period_end: DateTime<Utc>,
        total_kwh: f64,
        energy_type: EnergyType,
        postal_code: String,
        file_hash: String,
        file_path_encrypted: String,
        uploaded_by: Uuid,
        consent_ip: String,
        consent_user_agent: String,
        encryption_key: &[u8; 32],
    ) -> Result<Self, String> {
        // Validation
        if total_kwh <= 0.0 {
            return Err("Consumption must be positive".to_string());
        }

        if postal_code.len() != 4 {
            return Err("Invalid Belgian postal code".to_string());
        }

        if bill_period_start >= bill_period_end {
            return Err("Bill period start must be before end".to_string());
        }

        // Chiffrement AES-256-GCM de la consommation
        let total_kwh_encrypted = Self::encrypt_kwh(total_kwh, encryption_key)?;

        // Signature consentement
        let consent_data = format!("{}|{}|{}|{}", unit_id, total_kwh, consent_ip, Utc::now());
        let consent_signature_hash = format!("{:x}", md5::compute(consent_data.as_bytes()));

        // Rétention: 90 jours après fin campagne (GDPR)
        let retention_until = Utc::now() + chrono::Duration::days(90);

        Ok(Self {
            id: Uuid::new_v4(),
            campaign_id,
            unit_id,
            building_id,
            organization_id,
            bill_period_start,
            bill_period_end,
            total_kwh_encrypted,
            energy_type,
            provider: None,
            postal_code,
            file_hash,
            file_path_encrypted,
            ocr_confidence: 0.0,
            manually_verified: false,
            uploaded_by,
            uploaded_at: Utc::now(),
            verified_at: None,
            verified_by: None,
            consent_timestamp: Utc::now(),
            consent_ip,
            consent_user_agent,
            consent_signature_hash,
            anonymized: false,
            retention_until,
            deleted_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Chiffrer consommation kWh avec AES-256-GCM
    fn encrypt_kwh(kwh: f64, key: &[u8; 32]) -> Result<Vec<u8>, String> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };

        let cipher = Aes256Gcm::new(key.into());

        // Générer nonce aléatoire (12 bytes pour GCM)
        let nonce_bytes = Self::generate_nonce();
        let nonce = Nonce::from(nonce_bytes);

        let plaintext = kwh.to_string().as_bytes().to_vec();

        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        // Préfixer le ciphertext avec le nonce pour pouvoir déchiffrer plus tard
        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);
        Ok(result)
    }

    /// Déchiffrer consommation kWh (requiert consentement actif)
    pub fn decrypt_kwh(&self, key: &[u8; 32]) -> Result<f64, String> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };

        if self.deleted_at.is_some() {
            return Err("Bill has been deleted (GDPR)".to_string());
        }

        if self.total_kwh_encrypted.len() < 12 {
            return Err("Invalid encrypted data".to_string());
        }

        let cipher = Aes256Gcm::new(key.into());

        // Extraire nonce (12 premiers bytes)
        let nonce_array: [u8; 12] = self.total_kwh_encrypted[..12]
            .try_into()
            .map_err(|_| "Invalid nonce length".to_string())?;
        let nonce = Nonce::from(nonce_array);

        // Extraire ciphertext (reste des bytes)
        let ciphertext = &self.total_kwh_encrypted[12..];

        let plaintext = cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        let kwh_str = String::from_utf8(plaintext).map_err(|e| format!("UTF-8 error: {}", e))?;

        kwh_str
            .parse::<f64>()
            .map_err(|e| format!("Parse error: {}", e))
    }

    /// Générer nonce aléatoire de 12 bytes
    fn generate_nonce() -> [u8; 12] {
        use rand::Rng;
        let mut rng = rand::rng();
        let mut nonce = [0u8; 12];
        rng.fill(&mut nonce);
        nonce
    }

    /// Marquer comme vérifié (après OCR ou validation manuelle)
    pub fn mark_verified(&mut self, verified_by: Uuid) -> Result<(), String> {
        if self.verified_at.is_some() {
            return Err("Already verified".to_string());
        }

        self.manually_verified = true;
        self.verified_at = Some(Utc::now());
        self.verified_by = Some(verified_by);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Anonymiser (agréger au building)
    pub fn anonymize(&mut self) -> Result<(), String> {
        if self.anonymized {
            return Err("Already anonymized".to_string());
        }

        if !self.manually_verified && self.ocr_confidence < 95.0 {
            return Err("Must be verified before anonymization".to_string());
        }

        self.anonymized = true;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Supprimer données (GDPR Art. 17 - Droit à l'effacement)
    pub fn delete(&mut self) -> Result<(), String> {
        if self.deleted_at.is_some() {
            return Err("Already deleted".to_string());
        }

        self.deleted_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Vérifier si peut être supprimé automatiquement (rétention expirée)
    pub fn should_auto_delete(&self) -> bool {
        self.deleted_at.is_none() && self.retention_until < Utc::now()
    }

    /// Retirer consentement (GDPR Art. 7.3)
    pub fn withdraw_consent(&mut self) -> Result<(), String> {
        if self.deleted_at.is_some() {
            return Err("Already deleted".to_string());
        }

        // Supprimer immédiatement si consentement retiré
        self.delete()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_encryption_key() -> [u8; 32] {
        // 32 bytes exactly for AES-256
        *b"test_master_key_for_32bytes!##!!"
    }

    #[test]
    fn test_create_bill_upload_success() {
        let key = get_test_encryption_key();
        let bill = EnergyBillUpload::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Utc::now() - chrono::Duration::days(365),
            Utc::now(),
            2400.0,
            EnergyType::Electricity,
            "1050".to_string(),
            "abc123".to_string(),
            "/encrypted/path".to_string(),
            Uuid::new_v4(),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            &key,
        );

        assert!(bill.is_ok());
        let bill = bill.unwrap();
        assert_eq!(bill.energy_type, EnergyType::Electricity);
        assert!(!bill.anonymized);
        assert!(!bill.manually_verified);
    }

    #[test]
    fn test_create_bill_upload_invalid_postal_code() {
        let key = get_test_encryption_key();
        let result = EnergyBillUpload::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Utc::now() - chrono::Duration::days(365),
            Utc::now(),
            2400.0,
            EnergyType::Electricity,
            "123".to_string(), // Invalid: 3 digits instead of 4
            "abc123".to_string(),
            "/encrypted/path".to_string(),
            Uuid::new_v4(),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            &key,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid Belgian postal code");
    }

    #[test]
    fn test_create_bill_upload_negative_consumption() {
        let key = get_test_encryption_key();
        let result = EnergyBillUpload::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Utc::now() - chrono::Duration::days(365),
            Utc::now(),
            -100.0, // Invalid: negative
            EnergyType::Electricity,
            "1050".to_string(),
            "abc123".to_string(),
            "/encrypted/path".to_string(),
            Uuid::new_v4(),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            &key,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Consumption must be positive");
    }

    #[test]
    fn test_encrypt_decrypt_kwh() {
        let key = get_test_encryption_key();
        let original_kwh = 2400.5;

        let bill = EnergyBillUpload::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Utc::now() - chrono::Duration::days(365),
            Utc::now(),
            original_kwh,
            EnergyType::Electricity,
            "1050".to_string(),
            "abc123".to_string(),
            "/encrypted/path".to_string(),
            Uuid::new_v4(),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            &key,
        )
        .unwrap();

        // Déchiffrer
        let decrypted = bill.decrypt_kwh(&key);
        assert!(decrypted.is_ok());
        assert_eq!(decrypted.unwrap(), original_kwh);
    }

    #[test]
    fn test_decrypt_with_wrong_key() {
        let key = get_test_encryption_key();
        let wrong_key = *b"wrong_key_for_decryption_test!#!";

        let bill = EnergyBillUpload::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Utc::now() - chrono::Duration::days(365),
            Utc::now(),
            2400.0,
            EnergyType::Electricity,
            "1050".to_string(),
            "abc123".to_string(),
            "/encrypted/path".to_string(),
            Uuid::new_v4(),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            &key,
        )
        .unwrap();

        // Déchiffrer avec mauvaise clé
        let result = bill.decrypt_kwh(&wrong_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_mark_verified() {
        let key = get_test_encryption_key();
        let mut bill = EnergyBillUpload::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Utc::now() - chrono::Duration::days(365),
            Utc::now(),
            2400.0,
            EnergyType::Electricity,
            "1050".to_string(),
            "abc123".to_string(),
            "/encrypted/path".to_string(),
            Uuid::new_v4(),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            &key,
        )
        .unwrap();

        let verifier_id = Uuid::new_v4();
        assert!(bill.mark_verified(verifier_id).is_ok());
        assert!(bill.manually_verified);
        assert_eq!(bill.verified_by, Some(verifier_id));

        // Cannot verify twice
        assert!(bill.mark_verified(verifier_id).is_err());
    }

    #[test]
    fn test_anonymize() {
        let key = get_test_encryption_key();
        let mut bill = EnergyBillUpload::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Utc::now() - chrono::Duration::days(365),
            Utc::now(),
            2400.0,
            EnergyType::Electricity,
            "1050".to_string(),
            "abc123".to_string(),
            "/encrypted/path".to_string(),
            Uuid::new_v4(),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            &key,
        )
        .unwrap();

        // Cannot anonymize without verification
        bill.ocr_confidence = 90.0;
        assert!(bill.anonymize().is_err());

        // Mark as verified
        bill.mark_verified(Uuid::new_v4()).unwrap();

        // Now can anonymize
        assert!(bill.anonymize().is_ok());
        assert!(bill.anonymized);

        // Cannot anonymize twice
        assert!(bill.anonymize().is_err());
    }

    #[test]
    fn test_anonymize_high_ocr_confidence() {
        let key = get_test_encryption_key();
        let mut bill = EnergyBillUpload::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Utc::now() - chrono::Duration::days(365),
            Utc::now(),
            2400.0,
            EnergyType::Electricity,
            "1050".to_string(),
            "abc123".to_string(),
            "/encrypted/path".to_string(),
            Uuid::new_v4(),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            &key,
        )
        .unwrap();

        // High OCR confidence (≥95%) allows anonymization without manual verification
        bill.ocr_confidence = 98.0;
        assert!(bill.anonymize().is_ok());
        assert!(bill.anonymized);
    }

    #[test]
    fn test_delete() {
        let key = get_test_encryption_key();
        let mut bill = EnergyBillUpload::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Utc::now() - chrono::Duration::days(365),
            Utc::now(),
            2400.0,
            EnergyType::Electricity,
            "1050".to_string(),
            "abc123".to_string(),
            "/encrypted/path".to_string(),
            Uuid::new_v4(),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            &key,
        )
        .unwrap();

        assert!(bill.delete().is_ok());
        assert!(bill.deleted_at.is_some());

        // Cannot delete twice
        assert!(bill.delete().is_err());

        // Cannot decrypt deleted bill
        let result = bill.decrypt_kwh(&key);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Bill has been deleted (GDPR)");
    }

    #[test]
    fn test_should_auto_delete() {
        let key = get_test_encryption_key();
        let mut bill = EnergyBillUpload::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Utc::now() - chrono::Duration::days(365),
            Utc::now(),
            2400.0,
            EnergyType::Electricity,
            "1050".to_string(),
            "abc123".to_string(),
            "/encrypted/path".to_string(),
            Uuid::new_v4(),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            &key,
        )
        .unwrap();

        // Fresh bill should not be auto-deleted
        assert!(!bill.should_auto_delete());

        // Set retention in the past
        bill.retention_until = Utc::now() - chrono::Duration::days(1);
        assert!(bill.should_auto_delete());

        // Already deleted bill should not be auto-deleted again
        bill.delete().unwrap();
        assert!(!bill.should_auto_delete());
    }

    #[test]
    fn test_withdraw_consent() {
        let key = get_test_encryption_key();
        let mut bill = EnergyBillUpload::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Utc::now() - chrono::Duration::days(365),
            Utc::now(),
            2400.0,
            EnergyType::Electricity,
            "1050".to_string(),
            "abc123".to_string(),
            "/encrypted/path".to_string(),
            Uuid::new_v4(),
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
            &key,
        )
        .unwrap();

        assert!(bill.withdraw_consent().is_ok());
        assert!(bill.deleted_at.is_some());
    }
}
