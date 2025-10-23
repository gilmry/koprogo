# Issue #013 - Écologie et Durabilité

**Priorité**: 🟢 NICE-TO-HAVE
**Estimation**: 12-15 heures
**Labels**: `enhancement`, `backend`, `frontend`, `sustainability`, `green`

---

## 📋 Description

Module écologique aligné avec l'objectif **< 0.5g CO2/requête**. Aider les copropriétés à réduire leur empreinte carbone et améliorer leur performance énergétique.

---

## 🎯 Objectifs

- [ ] Bilan carbone par immeuble
- [ ] Suivi consommations énergétiques
- [ ] Recommandations travaux isolation
- [ ] Tracking DPE (Diagnostic Performance Énergétique)
- [ ] Calculateur aides (MaPrimeRénov', CEE)
- [ ] Dashboard écologique

---

## 📐 Fonctionnalités

### 1. Bilan Carbone Immeuble
```rust
pub struct CarbonFootprint {
    pub building_id: Uuid,
    pub year: i32,
    pub heating_emissions: Decimal, // kgCO2
    pub electricity_emissions: Decimal,
    pub water_emissions: Decimal,
    pub waste_emissions: Decimal,
    pub total_emissions: Decimal,
    pub emissions_per_sqm: Decimal,
}
```

**Calculs** :
- Chauffage gaz : `consommation_kWh * 0.227 kgCO2/kWh`
- Électricité : `consommation_kWh * 0.055 kgCO2/kWh` (mix français)
- Eau : `m³ * 0.132 kgCO2/m³`

### 2. Suivi Consommations

```rust
pub struct EnergyConsumption {
    pub building_id: Uuid,
    pub period: String, // "2025-Q1"
    pub heating_kwh: Decimal,
    pub electricity_kwh: Decimal,
    pub water_m3: Decimal,
    pub cost: Decimal,
}
```

**Graph** : Évolution consommations année N vs N-1

### 3. DPE Tracker

```rust
pub enum DPEClass {
    A, B, C, D, E, F, G
}

pub struct DPEHistory {
    pub building_id: Uuid,
    pub dpe_class: DPEClass,
    pub score: i32, // kWhEP/m²/an
    pub greenhouse_gas_class: DPEClass,
    pub evaluated_at: DateTime<Utc>,
}
```

**Objectif** : Suivre amélioration DPE après travaux

### 4. Recommandations Travaux

**IA-powered** :
```rust
pub struct SustainabilityRecommendation {
    pub title: String,
    pub description: String,
    pub estimated_savings_eur: Decimal,
    pub estimated_savings_co2: Decimal,
    pub estimated_cost: Decimal,
    pub roi_years: Decimal,
    pub priority: Priority,
}
```

**Exemples** :
- Isolation combles : -30% chauffage, ROI 7 ans
- Pompe à chaleur : -50% émissions, Aide 5000€
- Panneaux solaires : Production 3000 kWh/an

### 5. Calculateur Aides

**API externe** : Scraper données MaPrimeRénov', CEE

```rust
pub async fn calculate_subsidies(
    building: &Building,
    work_type: WorkType,
) -> Vec<Subsidy> {
    // Appel API gouvernementale ou scraping
    // Retourne aides disponibles
}
```

---

## 📝 User Stories

```gherkin
En tant que syndic
Je veux voir l'empreinte carbone de mon immeuble
Afin de proposer des travaux d'amélioration en AG

Scénario: Consultation bilan
  Étant donné un immeuble chauffé au gaz
  Quand je consulte le dashboard écologie
  Alors je vois :
    - Émissions totales : 45 tonnes CO2/an
    - Consommation : 180 kWh/m²/an
    - DPE actuel : Classe D
    - Recommandations : Isolation + PAC
    - Aides disponibles : 25 000€
```

---

## 🔧 Endpoints

| Méthode | Endpoint | Description |
|---------|----------|-------------|
| `GET` | `/api/v1/buildings/:id/carbon-footprint` | Bilan carbone |
| `GET` | `/api/v1/buildings/:id/energy-consumption` | Consommations |
| `GET` | `/api/v1/buildings/:id/dpe-history` | Historique DPE |
| `POST` | `/api/v1/buildings/:id/energy-consumption` | Ajouter relevé |
| `GET` | `/api/v1/sustainability/recommendations/:id` | Recommandations |
| `GET` | `/api/v1/sustainability/subsidies` | Calcul aides |

---

## 📊 Dashboard Écologique

**Frontend** : Graphiques Chart.js

- Jauge émissions CO2 vs objectif
- Courbe consommations 12 derniers mois
- Évolution DPE (avant/après travaux)
- Top 3 recommandations
- Calculateur aides interactif

---

## ✅ Critères d'Acceptation

- [ ] Calcul bilan carbone précis (sources ADEME)
- [ ] Graphiques consommations année vs année
- [ ] DPE évolution trackable
- [ ] Recommandations IA pertinentes
- [ ] Calculateur aides à jour

---

## 🚀 Checklist

- [ ] Migrations tables
- [ ] SustainabilityUseCases
- [ ] Handlers
- [ ] Composant SustainabilityDashboard.svelte
- [ ] Intégration Chart.js
- [ ] Data ADEME / MaPrimeRénov'
- [ ] Tests calculs
- [ ] Documentation

---

## 🌍 Impact

**Marketing** :
- Certification "Green SaaS"
- Argument commercial RSE
- Partenariats organismes écologiques

---

**Créé le** : 2025-10-23
**Alignement** : Objectif < 0.5g CO2/req
