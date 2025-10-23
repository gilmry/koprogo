# Issue #009 - Système de Notifications

**Priorité**: 🟡 IMPORTANT
**Estimation**: 8-10 heures
**Labels**: `enhancement`, `backend`, `frontend`, `important`, `notifications`

---

## 📋 Description

Implémenter un système complet de notifications multi-canal : email, push web, SMS (optionnel).

---

## 🎯 Objectifs

- [ ] Notifications email (SendGrid/SMTP)
- [ ] Push notifications web (Service Worker)
- [ ] Notifications in-app (cloche)
- [ ] Préférences utilisateur
- [ ] Templates personnalisables
- [ ] Queue asynchrone (Redis/Background jobs)

---

## 📐 Types de Notifications

| Événement | Canal | Destinataires |
|-----------|-------|---------------|
| Nouvelle AG planifiée | Email + Push | Tous copropriétaires |
| Appel de fonds | Email | Copropriétaires concernés |
| Paiement reçu | Email | Copropriétaire |
| Ticket créé | Email | Syndic |
| Ticket résolu | Email + In-app | Créateur ticket |
| Document ajouté | In-app | Concernés |
| Relance impayé | Email + SMS | Copropriétaire |

---

## 📐 Entité

```rust
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub title: String,
    pub body: String,
    pub link: Option<String>,
    pub read: bool,
    pub sent_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

pub enum NotificationType {
    Meeting,
    Payment,
    Ticket,
    Document,
    Work,
    General,
}
```

---

## 📝 User Stories

```gherkin
En tant que copropriétaire
Je veux être notifié des nouvelles AG
Afin de ne pas les manquer

Scénario: Notification AG
  Étant donné qu'une AG est créée
  Quand la date est confirmée
  Alors tous les copropriétaires reçoivent un email
  Et une notification push
  Et une notification in-app
```

---

## 🔧 Implémentation

### Email Service

```rust
use lettre::{Message, SmtpTransport, Transport};

pub struct EmailService {
    smtp: SmtpTransport,
}

impl EmailService {
    pub async fn send_meeting_notification(
        &self,
        to: &str,
        meeting: &Meeting,
    ) -> Result<(), String> {
        let email = Message::builder()
            .from("noreply@koprogo.com".parse().unwrap())
            .to(to.parse().unwrap())
            .subject("Nouvelle Assemblée Générale")
            .body(format!(
                "Une AG est planifiée le {}. Consultez l'ordre du jour sur KoproGo.",
                meeting.scheduled_at
            ))
            .unwrap();

        self.smtp.send(&email).map_err(|e| e.to_string())?;
        Ok(())
    }
}
```

### Web Push

```rust
use web_push::{WebPushClient, SubscriptionInfo};

pub async fn send_push_notification(
    subscription: &SubscriptionInfo,
    title: &str,
    body: &str,
) -> Result<(), String> {
    let client = WebPushClient::new()?;

    let payload = json!({
        "title": title,
        "body": body,
        "icon": "/icon-192.png",
    }).to_string();

    client.send(subscription, &payload).await?;
    Ok(())
}
```

---

## 🔧 Endpoints

| Méthode | Endpoint | Description |
|---------|----------|-------------|
| `GET` | `/api/v1/notifications` | Liste notifications user |
| `PUT` | `/api/v1/notifications/:id/read` | Marquer lue |
| `PUT` | `/api/v1/notifications/read-all` | Tout marquer lu |
| `GET` | `/api/v1/notifications/unread-count` | Nombre non lues |
| `POST` | `/api/v1/notifications/subscribe` | S'abonner push |
| `GET` | `/api/v1/notifications/preferences` | Préférences |
| `PUT` | `/api/v1/notifications/preferences` | Mettre à jour préfs |

---

## ✅ Critères d'Acceptation

- [ ] Emails envoyés via SMTP/SendGrid
- [ ] Push notifications web fonctionnelles
- [ ] Badge nombre non lues
- [ ] Préférences par type de notification
- [ ] Queue pour envois asynchrones
- [ ] Templates HTML personnalisables

---

## 🚀 Checklist

- [ ] Migration table notifications
- [ ] NotificationService
- [ ] EmailService (SMTP)
- [ ] WebPushService
- [ ] Handlers
- [ ] Service Worker frontend
- [ ] Composant NotificationBell.svelte
- [ ] Tests

---

**Créé le** : 2025-10-23
**Dépend de** : Issue #001 (meetings)
