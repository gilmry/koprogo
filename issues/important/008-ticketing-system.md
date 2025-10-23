# Issue #008 - Système de Tickets de Maintenance

**Priorité**: 🟡 IMPORTANT
**Estimation**: 8-10 heures
**Labels**: `enhancement`, `backend`, `frontend`, `important`, `support`

---

## 📋 Description

Créer un système de ticketing pour gérer les demandes de maintenance et interventions dans les copropriétés.

---

## 🎯 Objectifs

- [ ] Entité `Ticket` avec statuts
- [ ] Déclaration par copropriétaires
- [ ] Affectation aux prestataires
- [ ] Upload photos problème
- [ ] Suivi résolution
- [ ] Historique interventions

---

## 📐 Entité

```rust
pub struct Ticket {
    pub id: Uuid,
    pub building_id: Uuid,
    pub unit_id: Option<Uuid>,
    pub created_by: Uuid, // Owner
    pub assigned_to: Option<Uuid>, // Service provider
    pub title: String,
    pub description: String,
    pub category: TicketCategory,
    pub priority: TicketPriority,
    pub status: TicketStatus,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

pub enum TicketCategory {
    Plumbing,
    Electrical,
    Heating,
    Cleaning,
    Security,
    Other,
}

pub enum TicketPriority {
    Low,
    Medium,
    High,
    Emergency,
}

pub enum TicketStatus {
    Open,
    InProgress,
    Resolved,
    Closed,
    Cancelled,
}
```

---

## 📝 User Stories

```gherkin
En tant que copropriétaire
Je veux signaler une fuite d'eau
Afin qu'elle soit réparée rapidement

Scénario: Création ticket urgence
  Quand je crée un ticket "Fuite au plafond"
  Et je définis priorité "Emergency"
  Et j'uploade une photo
  Alors le syndic reçoit une notification
  Et peut affecter un plombier
```

---

## 🔧 Endpoints

| Méthode | Endpoint | Description |
|---------|----------|-------------|
| `POST` | `/api/v1/tickets` | Créer ticket |
| `GET` | `/api/v1/tickets/:id` | Détails |
| `GET` | `/api/v1/buildings/:id/tickets` | Tickets immeuble |
| `PUT` | `/api/v1/tickets/:id/assign` | Affecter |
| `PUT` | `/api/v1/tickets/:id/status` | Changer statut |
| `POST` | `/api/v1/tickets/:id/comments` | Ajouter commentaire |

---

## ✅ Checklist

- [ ] Migration table tickets
- [ ] TicketUseCases
- [ ] Handlers
- [ ] Composant TicketForm.svelte
- [ ] Notifications email/SMS urgence
- [ ] Tests

---

**Créé le** : 2025-10-23
