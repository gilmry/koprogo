# Issue #024 - Module Devis et Comparaison Travaux

**Priorité**: 🟡 HIGH
**Estimation**: 8-10 heures
**Labels**: `enhancement`, `backend`, `frontend`, `finance`, `procurement`

---

## 📋 Description

Implémenter un système de gestion de devis pour travaux avec comparaison multi-prestataires. Obligation légale belge : le syndic doit présenter **plusieurs devis** (minimum 3) pour travaux importants avant vote en AG.

**Contexte légal** : Pour travaux >5000€, obligation de comparer plusieurs offres et présenter tableau comparatif en AG.

---

## 🎯 Objectifs

- [ ] Créer demandes de devis multi-prestataires
- [ ] Upload devis PDF par prestataire
- [ ] Tableau comparatif automatique
- [ ] Scoring devis (prix, délai, garanties)
- [ ] Vote AG sur devis choisi
- [ ] Tracking exécution travaux

---

## 📐 Implémentation

### Entities

```rust
struct QuoteRequest {
    id: Uuid,
    building_id: Uuid,
    work_description: String,
    min_quotes_required: i32, // Default 3
    deadline: NaiveDate,
    status: QuoteRequestStatus, // Draft, Sent, Quotes Received, Decided
}

struct Quote {
    id: Uuid,
    quote_request_id: Uuid,
    contractor_name: String,
    amount: f64,
    delivery_days: i32,
    warranty_years: i32,
    pdf_path: String,
    notes: String,
    score: f64, // Calcul automatique
}
```

### Calcul Score Automatique

```rust
fn calculate_quote_score(quote: &Quote, all_quotes: &[Quote]) -> f64 {
    let price_score = 1.0 - (quote.amount / max_price);
    let delivery_score = 1.0 - (quote.delivery_days / max_delivery);
    let warranty_score = quote.warranty_years / max_warranty;

    (price_score * 0.5) + (delivery_score * 0.3) + (warranty_score * 0.2)
}
```

---

## ✅ Critères

- [ ] Minimum 3 devis pour travaux >5000€
- [ ] Tableau comparatif avec scoring
- [ ] Export PDF comparatif pour AG
- [ ] Lien avec vote AG (#046)

---

**Créé le** : 2025-11-01
**Milestone** : v1.1 - Procurement
