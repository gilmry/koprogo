use lettre::message::{header::ContentType, Mailbox};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::env;
use uuid::Uuid;

/// Email service for sending GDPR-related notifications
#[derive(Clone)]
pub struct EmailService {
    smtp_host: String,
    smtp_port: u16,
    smtp_username: String,
    smtp_password: String,
    from_email: String,
    from_name: String,
    enabled: bool,
}

impl EmailService {
    /// Create a new email service from environment variables
    pub fn from_env() -> Result<Self, String> {
        let enabled = env::var("SMTP_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            .parse::<bool>()
            .unwrap_or(false);

        if !enabled {
            log::info!("Email service disabled (SMTP_ENABLED=false)");
            return Ok(Self {
                smtp_host: String::new(),
                smtp_port: 0,
                smtp_username: String::new(),
                smtp_password: String::new(),
                from_email: String::new(),
                from_name: String::new(),
                enabled: false,
            });
        }

        let smtp_host = env::var("SMTP_HOST").map_err(|_| "SMTP_HOST not set".to_string())?;
        let smtp_port = env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse::<u16>()
            .map_err(|_| "SMTP_PORT must be a valid port number".to_string())?;
        let smtp_username =
            env::var("SMTP_USERNAME").map_err(|_| "SMTP_USERNAME not set".to_string())?;
        let smtp_password =
            env::var("SMTP_PASSWORD").map_err(|_| "SMTP_PASSWORD not set".to_string())?;
        let from_email =
            env::var("SMTP_FROM_EMAIL").map_err(|_| "SMTP_FROM_EMAIL not set".to_string())?;
        let from_name = env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "KoproGo".to_string());

        log::info!("Email service enabled: {}:{}", smtp_host, smtp_port);

        Ok(Self {
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            from_email,
            from_name,
            enabled: true,
        })
    }

    /// Send GDPR data export notification
    pub async fn send_gdpr_export_notification(
        &self,
        user_email: &str,
        user_name: &str,
        export_id: Uuid,
    ) -> Result<(), String> {
        if !self.enabled {
            log::debug!(
                "Email disabled - would send export notification to {}",
                user_email
            );
            return Ok(());
        }

        let subject = "Your GDPR Data Export is Ready";
        let body = format!(
            r#"Dear {},

Your request to export your personal data has been completed.

Export ID: {}
Export Date: {}

This export contains all personal information we hold about you, in compliance with GDPR Article 15 (Right to Access).

Important Security Notice:
- This export contains sensitive personal information
- Please store it securely and do not share it
- The export will be available for download for 30 days

If you did not request this export, please contact our Data Protection Officer immediately.

Best regards,
The KoproGo Team

---
This is an automated message. Please do not reply to this email.
For questions, contact: dpo@koprogo.com
"#,
            user_name,
            export_id,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        self.send_email(user_email, subject, &body).await
    }

    /// Send GDPR data erasure confirmation
    pub async fn send_gdpr_erasure_notification(
        &self,
        user_email: &str,
        user_name: &str,
        owners_anonymized: usize,
    ) -> Result<(), String> {
        if !self.enabled {
            log::debug!(
                "Email disabled - would send erasure notification to {}",
                user_email
            );
            return Ok(());
        }

        let subject = "GDPR Data Erasure Confirmation";
        let body = format!(
            r#"Dear {},

Your request to erase your personal data has been completed.

Erasure Summary:
- User account: Anonymized
- Owner profiles: {} anonymized
- Erasure Date: {}

In compliance with GDPR Article 17 (Right to Erasure), we have anonymized your personal information.

Important Information:
- Your account can no longer be recovered
- Some data may be retained for legal compliance (e.g., financial records for 7 years)
- Anonymous data may be retained for statistical purposes

If you did not request this erasure, please contact us immediately as this action cannot be reversed.

Best regards,
The KoproGo Team

---
This is an automated message. Please do not reply to this email.
For questions, contact: dpo@koprogo.com
"#,
            user_name,
            owners_anonymized,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        self.send_email(user_email, subject, &body).await
    }

    /// Send admin-initiated GDPR operation notification
    pub async fn send_admin_gdpr_notification(
        &self,
        user_email: &str,
        user_name: &str,
        operation: &str,
        admin_email: &str,
    ) -> Result<(), String> {
        if !self.enabled {
            log::debug!(
                "Email disabled - would send admin notification to {}",
                user_email
            );
            return Ok(());
        }

        let subject = format!("GDPR {} Performed by Administrator", operation);
        let body_text = format!(
            r#"Dear {},

An administrator has performed a GDPR {} operation on your account.

Operation: {}
Performed by: {}
Date: {}

This operation was performed by a KoproGo administrator, typically in response to:
- A compliance request
- Legal obligation
- Account cleanup
- Data subject request via alternative channel

If you have questions about this operation, please contact our Data Protection Officer.

Best regards,
The KoproGo Team

---
This is an automated message. Please do not reply to this email.
For questions, contact: dpo@koprogo.com
"#,
            user_name,
            operation,
            operation,
            admin_email,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        self.send_email(user_email, &subject, &body_text).await
    }

    /// Generic email sending function
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        let from = Mailbox::new(
            Some(self.from_name.clone()),
            self.from_email
                .parse()
                .map_err(|e| format!("Invalid from email: {}", e))?,
        );

        let to_mailbox = Mailbox::new(
            None,
            to.parse()
                .map_err(|e| format!("Invalid recipient email: {}", e))?,
        );

        let email = Message::builder()
            .from(from)
            .to(to_mailbox)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())
            .map_err(|e| format!("Failed to build email: {}", e))?;

        let creds = Credentials::new(self.smtp_username.clone(), self.smtp_password.clone());

        let mailer = SmtpTransport::relay(&self.smtp_host)
            .map_err(|e| format!("Failed to create SMTP transport: {}", e))?
            .port(self.smtp_port)
            .credentials(creds)
            .build();

        // Send email in blocking thread to avoid blocking async runtime
        let send_result = tokio::task::spawn_blocking(move || mailer.send(&email))
            .await
            .map_err(|e| format!("Failed to spawn blocking task: {}", e))?;

        send_result.map_err(|e| format!("Failed to send email: {}", e))?;

        log::info!("Email sent successfully to {}: {}", to, subject);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_service_disabled() {
        std::env::set_var("SMTP_ENABLED", "false");
        let service = EmailService::from_env().unwrap();
        assert!(!service.enabled);
    }

    #[tokio::test]
    async fn test_send_export_notification_disabled() {
        std::env::set_var("SMTP_ENABLED", "false");
        let service = EmailService::from_env().unwrap();
        let result = service
            .send_gdpr_export_notification("test@example.com", "Test User", Uuid::new_v4())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_erasure_notification_disabled() {
        std::env::set_var("SMTP_ENABLED", "false");
        let service = EmailService::from_env().unwrap();
        let result = service
            .send_gdpr_erasure_notification("test@example.com", "Test User", 2)
            .await;
        assert!(result.is_ok());
    }
}
