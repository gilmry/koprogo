# Issue #019 - Convocations AG Automatiques

**Priorité**: 🟡 HIGH
**Estimation**: 5-7 heures
**Labels**: `enhancement`, `backend`, `frontend`, `notifications`, `legal-compliance`

---

## 📋 Description

Implémenter un système automatique de **convocation aux assemblées générales** avec génération de PDF et envoi email aux copropriétaires. La convocation doit être conforme aux exigences légales belges (délais, contenu obligatoire, ordre du jour).

**Contexte légal** : En Belgique, le syndic doit convoquer les copropriétaires en AG **au minimum 15 jours avant** la date de l'assemblée (pour AG ordinaire) et **8 jours** (pour AG extraordinaire urgente). La convocation doit contenir l'ordre du jour complet.

**Impact métier** : Automatiser ce processus fait gagner un temps considérable au syndic et garantit la conformité légale des délais.

---

## 🎯 Objectifs

- [ ] Générer automatiquement un PDF de convocation
- [ ] Envoyer par email à tous les copropriétaires
- [ ] Vérifier les délais légaux (15 jours AG ordinaire, 8 jours AG extraordinaire)
- [ ] Tracer les envois (audit trail)
- [ ] Permettre renvoi manuel si nécessaire
- [ ] Support multi-langue (FR/NL/DE/EN)
- [ ] Gestion procurations dans la convocation

---

## 📐 Spécifications Techniques

### Contenu Légal d'une Convocation

Une convocation d'AG belge doit contenir :

1. **En-tête** :
   - Nom de la copropriété
   - Adresse de l'immeuble
   - Logo (optionnel)

2. **Informations AG** :
   - Type d'assemblée (Ordinaire / Extraordinaire)
   - Date, heure, lieu de l'AG
   - Date limite de réponse/procuration

3. **Ordre du Jour** :
   - Liste numérotée des points à l'ordre du jour
   - Pour chaque point : description claire + vote si applicable

4. **Informations Pratiques** :
   - Instructions pour donner procuration
   - Formulaire de procuration (PDF séparé ou inclus)
   - Coordonnées syndic pour questions

5. **Pièces Jointes Mentionnées** :
   - Comptes annuels (si AGO)
   - Budget prévisionnel (si AGO)
   - Devis travaux (si vote travaux)

---

## 🔧 Détails d'Implémentation

### 1. Domain Layer - Extension Meeting Entity

**Fichier** : `backend/src/domain/entities/meeting.rs` (modifier existant)

```rust
impl Meeting {
    /// Génère les données pour la convocation
    pub fn prepare_convocation_data(&self) -> ConvocationData {
        ConvocationData {
            meeting_id: self.id,
            meeting_type: self.meeting_type.clone(),
            title: self.title.clone(),
            scheduled_date: self.scheduled_date,
            location: self.location.clone().unwrap_or_default(),
            agenda: self.agenda.clone(),
            deadline_for_proxy: self.scheduled_date - chrono::Duration::days(2),
        }
    }

    /// Vérifie si le délai légal est respecté
    pub fn check_legal_delay(&self, send_date: DateTime<Utc>) -> Result<(), String> {
        let days_until_meeting = (self.scheduled_date - send_date).num_days();

        let min_days = match self.meeting_type {
            MeetingType::Ordinary => 15,
            MeetingType::Extraordinary => 8,
        };

        if days_until_meeting < min_days {
            return Err(format!(
                "Insufficient delay: {} days. Minimum required: {} days for {:?} meeting",
                days_until_meeting, min_days, self.meeting_type
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConvocationData {
    pub meeting_id: Uuid,
    pub meeting_type: MeetingType,
    pub title: String,
    pub scheduled_date: DateTime<Utc>,
    pub location: String,
    pub agenda: serde_json::Value,
    pub deadline_for_proxy: DateTime<Utc>,
}
```

---

### 2. Application Layer - Convocation Use Cases

**Fichier** : `backend/src/application/use_cases/convocation_use_cases.rs`

```rust
use crate::domain::entities::meeting::*;
use crate::application::ports::meeting_repository::MeetingRepository;
use crate::application::ports::owner_repository::OwnerRepository;
use crate::application::ports::email_service::EmailService;
use crate::application::ports::pdf_generator::PdfGenerator;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

pub struct ConvocationUseCases {
    meeting_repo: Arc<dyn MeetingRepository>,
    owner_repo: Arc<dyn OwnerRepository>,
    email_service: Arc<dyn EmailService>,
    pdf_generator: Arc<dyn PdfGenerator>,
}

impl ConvocationUseCases {
    pub fn new(
        meeting_repo: Arc<dyn MeetingRepository>,
        owner_repo: Arc<dyn OwnerRepository>,
        email_service: Arc<dyn EmailService>,
        pdf_generator: Arc<dyn PdfGenerator>,
    ) -> Self {
        Self {
            meeting_repo,
            owner_repo,
            email_service,
            pdf_generator,
        }
    }

    pub async fn send_convocations(
        &self,
        meeting_id: Uuid,
    ) -> Result<ConvocationResult, String> {
        // 1. Récupérer le meeting
        let meeting = self
            .meeting_repo
            .find_by_id(meeting_id)
            .await?
            .ok_or("Meeting not found")?;

        // 2. Vérifier délai légal
        meeting.check_legal_delay(Utc::now())?;

        // 3. Récupérer tous les copropriétaires du building
        let owners = self
            .owner_repo
            .find_by_building(meeting.building_id)
            .await?;

        if owners.is_empty() {
            return Err("No owners found for this building".to_string());
        }

        // 4. Préparer données convocation
        let convocation_data = meeting.prepare_convocation_data();

        // 5. Générer PDF de convocation
        let pdf_path = self
            .pdf_generator
            .generate_convocation_pdf(&convocation_data)
            .await?;

        // 6. Envoyer à chaque copropriétaire
        let mut sent_count = 0;
        let mut failed_count = 0;
        let mut failed_emails = Vec::new();

        for owner in owners {
            let email_result = self
                .email_service
                .send_convocation_email(
                    &owner.email,
                    &owner.full_name(),
                    &convocation_data,
                    &pdf_path,
                )
                .await;

            match email_result {
                Ok(_) => sent_count += 1,
                Err(e) => {
                    failed_count += 1;
                    failed_emails.push((owner.email.clone(), e));
                }
            }
        }

        // 7. Créer audit log
        // TODO: Log convocation sent event

        Ok(ConvocationResult {
            meeting_id,
            total_owners: owners.len(),
            sent_count,
            failed_count,
            failed_emails,
            pdf_path,
        })
    }

    pub async fn resend_to_owner(
        &self,
        meeting_id: Uuid,
        owner_id: Uuid,
    ) -> Result<(), String> {
        let meeting = self
            .meeting_repo
            .find_by_id(meeting_id)
            .await?
            .ok_or("Meeting not found")?;

        let owner = self
            .owner_repo
            .find_by_id(owner_id)
            .await?
            .ok_or("Owner not found")?;

        let convocation_data = meeting.prepare_convocation_data();

        // Regénérer PDF ou utiliser existant
        let pdf_path = self
            .pdf_generator
            .generate_convocation_pdf(&convocation_data)
            .await?;

        self.email_service
            .send_convocation_email(&owner.email, &owner.full_name(), &convocation_data, &pdf_path)
            .await
    }
}

#[derive(Debug, serde::Serialize)]
pub struct ConvocationResult {
    pub meeting_id: Uuid,
    pub total_owners: usize,
    pub sent_count: usize,
    pub failed_count: usize,
    pub failed_emails: Vec<(String, String)>, // (email, error)
    pub pdf_path: String,
}
```

---

### 3. Infrastructure - Email Service

**Fichier** : `backend/src/infrastructure/email/smtp_email_service.rs`

```rust
use crate::application::ports::email_service::EmailService;
use crate::domain::entities::meeting::ConvocationData;
use async_trait::async_trait;
use lettre::{
    message::{header, Attachment, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};

pub struct SmtpEmailService {
    smtp_server: String,
    smtp_username: String,
    smtp_password: String,
    from_address: String,
}

impl SmtpEmailService {
    pub fn new(
        smtp_server: String,
        smtp_username: String,
        smtp_password: String,
        from_address: String,
    ) -> Self {
        Self {
            smtp_server,
            smtp_username,
            smtp_password,
            from_address,
        }
    }
}

#[async_trait]
impl EmailService for SmtpEmailService {
    async fn send_convocation_email(
        &self,
        to_email: &str,
        to_name: &str,
        convocation: &ConvocationData,
        pdf_path: &str,
    ) -> Result<(), String> {
        // Lire le PDF
        let pdf_content = std::fs::read(pdf_path)
            .map_err(|e| format!("Failed to read PDF: {}", e))?;

        // Créer le body HTML
        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
            </head>
            <body style="font-family: Arial, sans-serif; line-height: 1.6;">
                <h2>Convocation Assemblée Générale</h2>
                <p>Cher/Chère {name},</p>
                <p>Vous êtes convoqué(e) à l'Assemblée Générale suivante :</p>
                <ul>
                    <li><strong>Type :</strong> {meeting_type:?}</li>
                    <li><strong>Date :</strong> {date}</li>
                    <li><strong>Lieu :</strong> {location}</li>
                </ul>
                <p>Vous trouverez en pièce jointe la convocation officielle avec l'ordre du jour complet.</p>
                <p><strong>Date limite pour donner procuration :</strong> {proxy_deadline}</p>
                <p>Cordialement,<br>Le Syndic</p>
            </body>
            </html>
            "#,
            name = to_name,
            meeting_type = convocation.meeting_type,
            date = convocation.scheduled_date.format("%d/%m/%Y à %H:%M"),
            location = convocation.location,
            proxy_deadline = convocation.deadline_for_proxy.format("%d/%m/%Y"),
        );

        // Créer l'attachment PDF
        let attachment = Attachment::new("convocation.pdf".to_string())
            .body(pdf_content, header::ContentType::parse("application/pdf").unwrap());

        // Construire l'email
        let email = Message::builder()
            .from(self.from_address.parse().unwrap())
            .to(to_email.parse().unwrap())
            .subject(format!(
                "Convocation AG - {}",
                convocation.scheduled_date.format("%d/%m/%Y")
            ))
            .multipart(
                MultiPart::mixed()
                    .singlepart(SinglePart::html(html_body))
                    .singlepart(attachment),
            )
            .map_err(|e| format!("Failed to build email: {}", e))?;

        // Envoyer via SMTP
        let creds = Credentials::new(
            self.smtp_username.clone(),
            self.smtp_password.clone(),
        );

        let mailer = SmtpTransport::relay(&self.smtp_server)
            .map_err(|e| format!("SMTP relay error: {}", e))?
            .credentials(creds)
            .build();

        mailer
            .send(&email)
            .map_err(|e| format!("Failed to send email: {}", e))?;

        Ok(())
    }
}
```

---

### 4. Infrastructure - PDF Generator

**Fichier** : `backend/src/infrastructure/pdf/convocation_pdf_generator.rs`

```rust
use crate::application::ports::pdf_generator::PdfGenerator;
use crate::domain::entities::meeting::ConvocationData;
use async_trait::async_trait;
use std::fs;
use uuid::Uuid;

pub struct ConvocationPdfGenerator {
    output_dir: String,
}

impl ConvocationPdfGenerator {
    pub fn new(output_dir: String) -> Self {
        Self { output_dir }
    }

    fn generate_html(&self, data: &ConvocationData) -> String {
        format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <style>
                    body {{
                        font-family: Arial, sans-serif;
                        margin: 40px;
                        line-height: 1.6;
                    }}
                    h1 {{ color: #2c3e50; }}
                    .header {{ text-align: center; margin-bottom: 40px; }}
                    .info-box {{ background: #ecf0f1; padding: 15px; border-radius: 5px; }}
                    .agenda {{ margin-top: 30px; }}
                    .agenda-item {{ margin-bottom: 15px; }}
                </style>
            </head>
            <body>
                <div class="header">
                    <h1>CONVOCATION<br>ASSEMBLÉE GÉNÉRALE {meeting_type:?}</h1>
                </div>

                <div class="info-box">
                    <p><strong>Date :</strong> {date}</p>
                    <p><strong>Heure :</strong> {time}</p>
                    <p><strong>Lieu :</strong> {location}</p>
                </div>

                <div class="agenda">
                    <h2>ORDRE DU JOUR</h2>
                    {agenda_html}
                </div>

                <div style="margin-top: 40px;">
                    <p><strong>Date limite pour procuration :</strong> {proxy_deadline}</p>
                    <p><em>En cas d'absence, merci de donner procuration avant cette date.</em></p>
                </div>

                <div style="margin-top: 60px; text-align: right;">
                    <p>Le Syndic</p>
                </div>
            </body>
            </html>
            "#,
            meeting_type = data.meeting_type,
            date = data.scheduled_date.format("%d/%m/%Y"),
            time = data.scheduled_date.format("%H:%M"),
            location = data.location,
            agenda_html = self.format_agenda(&data.agenda),
            proxy_deadline = data.deadline_for_proxy.format("%d/%m/%Y"),
        )
    }

    fn format_agenda(&self, agenda: &serde_json::Value) -> String {
        // TODO: Parser le JSON agenda et formater en HTML
        // Exemple si agenda est un array de strings:
        if let Some(items) = agenda.as_array() {
            items
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    format!(
                        "<div class=\"agenda-item\"><strong>{}.</strong> {}</div>",
                        i + 1,
                        item.as_str().unwrap_or(""),
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            "<p>Ordre du jour à définir</p>".to_string()
        }
    }
}

#[async_trait]
impl PdfGenerator for ConvocationPdfGenerator {
    async fn generate_convocation_pdf(
        &self,
        data: &ConvocationData,
    ) -> Result<String, String> {
        let html = self.generate_html(data);
        let filename = format!("convocation_{}.pdf", Uuid::new_v4());
        let output_path = format!("{}/{}", self.output_dir, filename);

        // Option 1: Utiliser wkhtmltopdf via command
        // std::process::Command::new("wkhtmltopdf")
        //     .arg("-")
        //     .arg(&output_path)
        //     .stdin(std::process::Stdio::piped())
        //     .spawn()
        //     .unwrap()
        //     .stdin.unwrap().write_all(html.as_bytes()).unwrap();

        // Option 2: Utiliser printpdf (TODO)
        // Pour l'instant, sauvegarder HTML (TODO: vraie génération PDF)
        fs::write(&output_path, html)
            .map_err(|e| format!("Failed to write PDF: {}", e))?;

        Ok(output_path)
    }
}
```

---

## ✅ Critères d'Acceptation

### Fonctionnels
- [ ] Convocation générée automatiquement avec ordre du jour
- [ ] PDF conforme visuellement (header, sections, footer)
- [ ] Email envoyé à tous les copropriétaires
- [ ] Délais légaux vérifiés (erreur si < 15j)
- [ ] Possibilité de renvoi manuel à un copropriétaire
- [ ] Audit log de tous les envois

### Techniques
- [ ] PDF généré en < 2 secondes
- [ ] Emails envoyés en parallèle (async)
- [ ] Tests E2E complets
- [ ] Support multi-langue (templates FR/NL/DE/EN)

---

## 🧪 Plan de Tests

### Tests Unitaires
```rust
#[test]
fn test_legal_delay_ordinary_meeting() {
    let meeting = create_test_meeting(MeetingType::Ordinary, 20); // 20 jours
    assert!(meeting.check_legal_delay(Utc::now()).is_ok());
}

#[test]
fn test_legal_delay_insufficient() {
    let meeting = create_test_meeting(MeetingType::Ordinary, 10); // 10 jours
    assert!(meeting.check_legal_delay(Utc::now()).is_err());
}
```

---

## 🔗 Dépendances

### Bloquantes
- ✅ Meeting entity exists
- ✅ Owner entity exists

### Recommandées
- Issue #009 : Notifications System (infrastructure email)
- Issue #047 : PDF Generation Extended (templates)

---

## 🚀 Checklist

- [ ] 1. Modifier `domain/entities/meeting.rs`
- [ ] 2. Créer `convocation_use_cases.rs`
- [ ] 3. Créer `smtp_email_service.rs`
- [ ] 4. Créer `convocation_pdf_generator.rs`
- [ ] 5. Créer handlers HTTP
- [ ] 6. Tests (10+ tests)
- [ ] 7. Frontend: bouton "Envoyer convocations"
- [ ] 8. Commit : `feat: implement automatic AG convocations with email/PDF`

---

**Créé le** : 2025-11-01
**Milestone** : v1.1 - Automation Features
