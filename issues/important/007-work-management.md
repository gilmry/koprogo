# Issue #007 - Gestion des Travaux

**Priorité**: 🟡 IMPORTANT
**Estimation**: 12-15 heures
**Labels**: `enhancement`, `backend`, `frontend`, `important`, `feature`

---

## 📋 Description

Créer un module complet de gestion des travaux en copropriété : planification, devis, votes en AG, suivi d'avancement, galerie photos.

**Contexte métier** : Les travaux importants (ravalement, toiture, ascenseur) nécessitent approbation en AG, comparaison de devis, appels de fonds exceptionnels.

---

## 🎯 Objectifs

- [ ] Créer entité `Work` (Travaux)
- [ ] Gestion multi-devis avec comparaison
- [ ] Workflow: Proposition → Vote AG → Planification → En cours → Terminé
- [ ] Galerie photos avant/après
- [ ] Appels de fonds spécifiques travaux
- [ ] Suivi garanties décennales

---

## 📐 Entité Domain

```rust
pub struct Work {
    pub id: Uuid,
    pub building_id: Uuid,
    pub title: String,
    pub description: String,
    pub work_type: WorkType,
    pub status: WorkStatus,
    pub estimated_cost: Decimal,
    pub actual_cost: Option<Decimal>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub warranty_end_date: Option<DateTime<Utc>>, // Garantie décennale
    pub voted_at_meeting_id: Option<Uuid>,
    pub selected_quote_id: Option<Uuid>,
}

pub enum WorkType {
    Roofing,
    Facade,
    Elevator,
    Plumbing,
    Electrical,
    Painting,
    Landscaping,
    Other,
}

pub enum WorkStatus {
    Proposed,
    VotedApproved,
    VotedRejected,
    InProgress,
    Completed,
    Cancelled,
}
```

**Entité `Quote`** :
```rust
pub struct Quote {
    pub id: Uuid,
    pub work_id: Uuid,
    pub provider_name: String,
    pub amount: Decimal,
    pub details: String,
    pub valid_until: DateTime<Utc>,
    pub document_id: Option<Uuid>, // PDF du devis
    pub selected: bool,
}
```

---

## 📝 User Stories

### US1 - Proposition travaux
```gherkin
En tant que syndic
Je veux proposer des travaux de ravalement
Afin de les soumettre à l'AG

Scénario: Création proposition
  Quand je crée un travail "Ravalement de façade"
  Et j'ajoute 3 devis
  Alors le statut est "Proposed"
  Et je peux l'ajouter à l'ordre du jour de l'AG
```

### US2 - Suivi avancement
```gherkin
En tant que copropriétaire
Je veux voir l'avancement des travaux en cours
Afin de suivre l'évolution

Scénario: Consultation timeline
  Étant donné des travaux en cours depuis 15 jours
  Quand je consulte la page travaux
  Alors je vois une timeline avec photos
  Et le pourcentage d'avancement
```

---

## 🔧 Endpoints

| Méthode | Endpoint | Description |
|---------|----------|-------------|
| `POST` | `/api/v1/works` | Créer travaux |
| `GET` | `/api/v1/works/:id` | Détails travaux |
| `GET` | `/api/v1/buildings/:id/works` | Travaux d'un immeuble |
| `PUT` | `/api/v1/works/:id/status` | Changer statut |
| `POST` | `/api/v1/works/:id/quotes` | Ajouter devis |
| `GET` | `/api/v1/works/:id/quotes` | Liste devis |
| `PUT` | `/api/v1/quotes/:id/select` | Sélectionner devis |
| `POST` | `/api/v1/works/:id/photos` | Ajouter photos |
| `GET` | `/api/v1/works/:id/timeline` | Timeline travaux |

---

## ✅ Critères d'Acceptation

- [ ] Création travaux avec multi-devis
- [ ] Comparaison visuelle devis (tableau)
- [ ] Upload photos avant/après
- [ ] Workflow statuts respecté
- [ ] Intégration avec meetings (vote)
- [ ] Export rapport travaux PDF
- [ ] Notifications changement statut

---

## 🚀 Checklist

- [ ] Migration tables works, quotes, work_photos
- [ ] Entités domain
- [ ] WorkUseCases + QuoteUseCases
- [ ] Handlers
- [ ] Composant WorkTimeline.svelte
- [ ] Tests

---

**Créé le** : 2025-10-23
**Dépend de** : Issue #001 (meetings), Issue #002 (documents)
