use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use image::Luma;
use qrcode::QrCode;
use rand::Rng;
use totp_lite::{totp_custom, Sha1};

/// TOTP Generator for Two-Factor Authentication
///
/// Provides cryptographic functions for TOTP secret generation, QR code creation,
/// code verification, backup code management, and AES-256-GCM encryption.
///
/// ## Security Features
/// - **TOTP Secret**: 32-byte random Base32-encoded secret (RFC 4648)
/// - **QR Code**: otpauth:// URI format for authenticator apps
/// - **Backup Codes**: 10 secure random 8-character alphanumeric codes
/// - **Encryption**: AES-256-GCM authenticated encryption for secrets
/// - **Hashing**: bcrypt (cost factor 12) for backup codes
/// - **Time Window**: ±30 seconds tolerance for clock skew (±1 step)
pub struct TotpGenerator;

impl TotpGenerator {
    /// Generate a random TOTP secret (Base32-encoded, 32 bytes)
    ///
    /// Returns a 52-character Base32 string suitable for TOTP generation.
    ///
    /// # Example
    /// ```
    /// let secret = TotpGenerator::generate_secret();
    /// assert_eq!(secret.len(), 52); // Base32 encoding of 32 bytes
    /// ```
    pub fn generate_secret() -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        Self::base32_encode(&bytes)
    }

    /// Generate QR code data URL for authenticator apps
    ///
    /// Creates an `otpauth://totp/` URI and encodes it as a PNG QR code data URL.
    ///
    /// # Arguments
    /// * `secret` - Base32-encoded TOTP secret
    /// * `issuer` - Service name (e.g., "KoproGo")
    /// * `account_name` - User identifier (e.g., email address)
    ///
    /// # Returns
    /// Base64-encoded PNG image data URL: `data:image/png;base64,...`
    ///
    /// # Errors
    /// Returns error if QR code generation or image encoding fails.
    pub fn generate_qr_code(
        secret: &str,
        issuer: &str,
        account_name: &str,
    ) -> Result<String, String> {
        // Create otpauth URI (RFC 6238 format)
        let uri = format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}",
            urlencoding::encode(issuer),
            urlencoding::encode(account_name),
            secret,
            urlencoding::encode(issuer)
        );

        // Generate QR code
        let code = QrCode::new(uri.as_bytes())
            .map_err(|e| format!("Failed to generate QR code: {}", e))?;

        // Render as PNG image
        let image = code.render::<Luma<u8>>().build();
        let mut png_bytes = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut png_bytes),
                image::ImageFormat::Png,
            )
            .map_err(|e| format!("Failed to encode PNG: {}", e))?;

        // Encode as data URL
        let base64_image = BASE64.encode(&png_bytes);
        Ok(format!("data:image/png;base64,{}", base64_image))
    }

    /// Verify TOTP code with ±1 time window tolerance
    ///
    /// Validates a 6-digit TOTP code against the secret with 30-second time steps.
    /// Accepts codes from current step, previous step (t-30s), and next step (t+30s).
    ///
    /// # Arguments
    /// * `secret` - Base32-encoded TOTP secret (unencrypted)
    /// * `code` - 6-digit TOTP code to verify
    ///
    /// # Returns
    /// `Ok(true)` if code is valid, `Ok(false)` if invalid format/value
    ///
    /// # Example
    /// ```
    /// let secret = TotpGenerator::generate_secret();
    /// let code = TotpGenerator::generate_current_code(&secret)?;
    /// assert!(TotpGenerator::verify_code(&secret, &code)?);
    /// ```
    pub fn verify_code(secret: &str, code: &str) -> Result<bool, String> {
        // Validate format (must be exactly 6 digits)
        if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
            return Ok(false);
        }

        // Decode Base32 secret
        let secret_bytes = Self::base32_decode(secret)?;

        // Get current Unix timestamp (seconds since epoch)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("System time error: {}", e))?
            .as_secs();

        // Check current time step and ±1 window (30-second steps)
        for time_offset in [-1, 0, 1] {
            let time_step = (now as i64 + time_offset * 30) as u64;
            let expected_code = totp_custom::<Sha1>(30, 6, &secret_bytes, time_step);

            if code == expected_code {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Generate 10 secure random backup codes (8 characters each)
    ///
    /// Each code is in format `XXXX-XXXX` with uppercase alphanumeric characters
    /// (excluding ambiguous chars: 0, O, 1, I, L).
    ///
    /// # Returns
    /// Vector of 10 unique backup codes
    ///
    /// # Example
    /// ```
    /// let codes = TotpGenerator::generate_backup_codes();
    /// assert_eq!(codes.len(), 10);
    /// assert!(codes[0].len() == 9); // "XXXX-XXXX"
    /// ```
    pub fn generate_backup_codes() -> Vec<String> {
        let mut rng = rand::thread_rng();
        const CHARSET: &[u8] = b"23456789ABCDEFGHJKMNPQRSTUVWXYZ"; // No 0,O,1,I,L
        const CODE_LENGTH: usize = 8;

        (0..10)
            .map(|_| {
                let code: String = (0..CODE_LENGTH)
                    .map(|_| {
                        let idx = rng.gen_range(0..CHARSET.len());
                        CHARSET[idx] as char
                    })
                    .collect();

                // Format as XXXX-XXXX
                format!("{}-{}", &code[0..4], &code[4..8])
            })
            .collect()
    }

    /// Hash backup code using bcrypt (cost factor 12)
    ///
    /// # Arguments
    /// * `code` - Plain text backup code (e.g., "ABCD-EFGH")
    ///
    /// # Returns
    /// Bcrypt hash string (60 characters)
    ///
    /// # Errors
    /// Returns error if bcrypt hashing fails
    pub fn hash_backup_code(code: &str) -> Result<String, String> {
        bcrypt::hash(code, 12).map_err(|e| format!("Failed to hash backup code: {}", e))
    }

    /// Verify backup code against bcrypt hash
    ///
    /// # Arguments
    /// * `code` - Plain text backup code
    /// * `hash` - Bcrypt hash to verify against
    ///
    /// # Returns
    /// `Ok(true)` if code matches hash, `Ok(false)` otherwise
    pub fn verify_backup_code(code: &str, hash: &str) -> Result<bool, String> {
        bcrypt::verify(code, hash).map_err(|e| format!("Failed to verify backup code: {}", e))
    }

    /// Encrypt TOTP secret using AES-256-GCM
    ///
    /// # Arguments
    /// * `secret` - Plain text Base32 secret
    /// * `key` - 32-byte AES-256 key (from environment variable)
    ///
    /// # Returns
    /// Base64-encoded ciphertext with prepended nonce (12 bytes + ciphertext)
    ///
    /// # Security
    /// - Uses AES-256-GCM for authenticated encryption
    /// - Random 96-bit nonce per encryption
    /// - Authenticated with GMAC tag
    ///
    /// # Errors
    /// Returns error if encryption fails or key is invalid
    pub fn encrypt_secret(secret: &str, key: &[u8; 32]) -> Result<String, String> {
        let cipher = Aes256Gcm::new(key.into());

        // Generate random 96-bit nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt secret
        let ciphertext = cipher
            .encrypt(nonce, secret.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        // Prepend nonce to ciphertext and encode as Base64
        let mut encrypted = nonce_bytes.to_vec();
        encrypted.extend_from_slice(&ciphertext);
        Ok(BASE64.encode(&encrypted))
    }

    /// Decrypt TOTP secret using AES-256-GCM
    ///
    /// # Arguments
    /// * `encrypted` - Base64-encoded ciphertext with prepended nonce
    /// * `key` - 32-byte AES-256 key (same as used for encryption)
    ///
    /// # Returns
    /// Plain text Base32 secret
    ///
    /// # Errors
    /// Returns error if decryption fails, key is invalid, or ciphertext is corrupted
    pub fn decrypt_secret(encrypted: &str, key: &[u8; 32]) -> Result<String, String> {
        let cipher = Aes256Gcm::new(key.into());

        // Decode Base64
        let encrypted_bytes = BASE64
            .decode(encrypted)
            .map_err(|e| format!("Invalid Base64: {}", e))?;

        if encrypted_bytes.len() < 12 {
            return Err("Encrypted data too short".to_string());
        }

        // Extract nonce (first 12 bytes) and ciphertext (rest)
        let (nonce_bytes, ciphertext) = encrypted_bytes.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        String::from_utf8(plaintext).map_err(|e| format!("Invalid UTF-8: {}", e))
    }

    // ========================================
    // Private helper methods
    // ========================================

    /// Base32 encode (RFC 4648)
    fn base32_encode(bytes: &[u8]) -> String {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
        let mut result = String::new();
        let mut bits = 0u32;
        let mut bit_count = 0;

        for &byte in bytes {
            bits = (bits << 8) | byte as u32;
            bit_count += 8;

            while bit_count >= 5 {
                bit_count -= 5;
                let index = ((bits >> bit_count) & 0x1F) as usize;
                result.push(ALPHABET[index] as char);
            }
        }

        if bit_count > 0 {
            let index = ((bits << (5 - bit_count)) & 0x1F) as usize;
            result.push(ALPHABET[index] as char);
        }

        result
    }

    /// Base32 decode (RFC 4648)
    fn base32_decode(encoded: &str) -> Result<Vec<u8>, String> {
        let encoded = encoded.to_uppercase();
        let mut result = Vec::new();
        let mut bits = 0u32;
        let mut bit_count = 0;

        for ch in encoded.chars() {
            if ch == '=' {
                break; // Padding
            }

            let value = match ch {
                'A'..='Z' => (ch as u32) - ('A' as u32),
                '2'..='7' => 26 + (ch as u32) - ('2' as u32),
                _ => return Err(format!("Invalid Base32 character: {}", ch)),
            };

            bits = (bits << 5) | value;
            bit_count += 5;

            if bit_count >= 8 {
                bit_count -= 8;
                result.push((bits >> bit_count) as u8);
                bits &= (1 << bit_count) - 1;
            }
        }

        Ok(result)
    }

    /// Generate current TOTP code (for testing)
    #[cfg(test)]
    pub fn generate_current_code(secret: &str) -> Result<String, String> {
        let secret_bytes = Self::base32_decode(secret)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Ok(totp_custom::<Sha1>(30, 6, &secret_bytes, now))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secret() {
        let secret = TotpGenerator::generate_secret();
        assert_eq!(secret.len(), 52); // Base32 encoding of 32 bytes
        assert!(secret.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
    }

    #[test]
    fn test_verify_code_valid() {
        let secret = TotpGenerator::generate_secret();
        let code = TotpGenerator::generate_current_code(&secret).unwrap();
        assert!(TotpGenerator::verify_code(&secret, &code).unwrap());
    }

    #[test]
    fn test_verify_code_invalid_format() {
        let secret = TotpGenerator::generate_secret();
        assert!(!TotpGenerator::verify_code(&secret, "12345").unwrap()); // 5 digits
        assert!(!TotpGenerator::verify_code(&secret, "1234567").unwrap()); // 7 digits
        assert!(!TotpGenerator::verify_code(&secret, "ABCDEF").unwrap()); // Non-digits
    }

    #[test]
    fn test_generate_backup_codes() {
        let codes = TotpGenerator::generate_backup_codes();
        assert_eq!(codes.len(), 10);

        for code in &codes {
            assert_eq!(code.len(), 9); // "XXXX-XXXX"
            assert!(code.contains('-'));
            let parts: Vec<&str> = code.split('-').collect();
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[0].len(), 4);
            assert_eq!(parts[1].len(), 4);

            // No ambiguous characters
            assert!(!code.contains('0'));
            assert!(!code.contains('O'));
            assert!(!code.contains('1'));
            assert!(!code.contains('I'));
            assert!(!code.contains('L'));
        }

        // All codes should be unique
        let unique_codes: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(unique_codes.len(), 10);
    }

    #[test]
    fn test_hash_and_verify_backup_code() {
        let code = "ABCD-EFGH";
        let hash = TotpGenerator::hash_backup_code(code).unwrap();
        assert_eq!(hash.len(), 60); // bcrypt hash length

        // Correct code
        assert!(TotpGenerator::verify_backup_code(code, &hash).unwrap());

        // Wrong code
        assert!(!TotpGenerator::verify_backup_code("WXYZ-1234", &hash).unwrap());
    }

    #[test]
    fn test_encrypt_decrypt_secret() {
        let secret = TotpGenerator::generate_secret();
        let key: [u8; 32] = rand::thread_rng().gen();

        let encrypted = TotpGenerator::encrypt_secret(&secret, &key).unwrap();
        assert_ne!(encrypted, secret); // Encrypted should differ

        let decrypted = TotpGenerator::decrypt_secret(&encrypted, &key).unwrap();
        assert_eq!(decrypted, secret);
    }

    #[test]
    fn test_decrypt_with_wrong_key() {
        let secret = TotpGenerator::generate_secret();
        let key1: [u8; 32] = rand::thread_rng().gen();
        let key2: [u8; 32] = rand::thread_rng().gen();

        let encrypted = TotpGenerator::encrypt_secret(&secret, &key1).unwrap();
        let result = TotpGenerator::decrypt_secret(&encrypted, &key2);
        assert!(result.is_err()); // Should fail with wrong key
    }

    #[test]
    fn test_base32_encode_decode() {
        let bytes = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello"
        let encoded = TotpGenerator::base32_encode(&bytes);
        assert_eq!(encoded, "JBSWY3DP");

        let decoded = TotpGenerator::base32_decode(&encoded).unwrap();
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn test_generate_qr_code() {
        let secret = TotpGenerator::generate_secret();
        let qr_data_url =
            TotpGenerator::generate_qr_code(&secret, "KoproGo", "user@example.com").unwrap();

        assert!(qr_data_url.starts_with("data:image/png;base64,"));
        assert!(qr_data_url.len() > 100); // Should be a non-trivial image
    }
}
