# Issue #012 - Marketplace de Prestataires

**Priorité**: 🟢 NICE-TO-HAVE
**Estimation**: 20-25 heures
**Labels**: `enhancement`, `backend`, `frontend`, `marketplace`

---

## 📋 Description

Créer une marketplace intégrée de prestataires de services pour copropriétés : plombiers, électriciens, jardiniers, assureurs, etc.

---

## 🎯 Objectifs

- [ ] Annuaire prestataires vérifiés
- [ ] Notation et avis copropriétaires
- [ ] Demande de devis en ligne
- [ ] Comparateur de prix
- [ ] Contrats cadre copropriété
- [ ] Certification qualité (RGE, etc.)

---

## 📐 Entités

```rust
pub struct ServiceProvider {
    pub id: Uuid,
    pub company_name: String,
    pub category: ServiceCategory,
    pub description: String,
    pub phone: String,
    pub email: String,
    pub address: String,
    pub verified: bool,
    pub certifications: Vec<String>, // RGE, Qualibat, etc.
    pub average_rating: Decimal,
    pub total_reviews: i32,
}

pub enum ServiceCategory {
    Plumbing,
    Electrical,
    Gardening,
    Cleaning,
    Security,
    Insurance,
    Legal,
    Accounting,
}

pub struct Review {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub building_id: Uuid,
    pub author_id: Uuid,
    pub rating: i32, // 1-5
    pub comment: String,
    pub work_quality: i32,
    pub responsiveness: i32,
    pub value_for_money: i32,
}

pub struct QuoteRequest {
    pub id: Uuid,
    pub building_id: Uuid,
    pub category: ServiceCategory,
    pub description: String,
    pub status: QuoteRequestStatus,
    pub quotes_received: Vec<Uuid>,
}
```

---

## 📝 User Stories

```gherkin
En tant que syndic
Je veux demander 3 devis pour ravalement
Afin de comparer les prix

Scénario: Demande devis
  Quand je crée une demande "Ravalement 500m²"
  Et je sélectionne catégorie "Facade"
  Alors la demande est envoyée aux prestataires
  Et je reçois les devis sous 48h
  Et je peux comparer côte à côte
```

---

## 🔧 Endpoints

| Méthode | Endpoint | Description |
|---------|----------|-------------|
| `GET` | `/api/v1/providers?category=` | Liste prestataires |
| `GET` | `/api/v1/providers/:id` | Détails prestataire |
| `POST` | `/api/v1/providers/:id/reviews` | Ajouter avis |
| `POST` | `/api/v1/quote-requests` | Demander devis |
| `GET` | `/api/v1/quote-requests/:id/quotes` | Devis reçus |

---

## ✅ Critères d'Acceptation

- [ ] Recherche prestataires par catégorie + localisation
- [ ] Notation moyenne calculée automatiquement
- [ ] Avis modérés (anti-spam)
- [ ] Demande devis envoyée par email aux prestataires
- [ ] Comparateur visuel 3 devis max
- [ ] Badge "Prestataire vérifié"

---

## 🚀 Checklist

- [ ] Migrations tables
- [ ] ProviderUseCases + ReviewUseCases
- [ ] Handlers
- [ ] Composant ProviderSearch.svelte
- [ ] Système modération avis
- [ ] Tests

---

**Créé le** : 2025-10-23
**Business Model** : Commission sur contrats signés
