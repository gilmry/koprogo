# Issue #014 - Analytics et Business Intelligence

**Priorité**: 🟢 NICE-TO-HAVE
**Estimation**: 12-15 heures
**Labels**: `enhancement`, `backend`, `frontend`, `analytics`, `bi`

---

## 📋 Description

Système d'analytics avancé pour syndics gérant plusieurs copropriétés. Tableaux de bord, benchmarking, prédictions.

---

## 🎯 Objectifs

- [ ] Tableaux de bord multi-copropriétés
- [ ] KPIs en temps réel
- [ ] Benchmarking charges moyennes
- [ ] Prédictions financières
- [ ] Rapports clients automatiques
- [ ] Export BI pour cabinets

---

## 📐 Fonctionnalités

### 1. Dashboard Multi-Copropriétés

**Pour cabinets de syndic gérant 10-100 immeubles**

```rust
pub struct PortfolioMetrics {
    pub total_buildings: i32,
    pub total_units: i32,
    pub total_revenue_ytd: Decimal,
    pub avg_collection_rate: Decimal, // % paiements à temps
    pub buildings_by_status: HashMap<BuildingStatus, i32>,
    pub top_expenses_categories: Vec<(ExpenseCategory, Decimal)>,
}
```

**Widgets** :
- Revenus totaux par mois
- Taux de recouvrement moyen
- Top 5 immeubles par charges
- Alertes impayés > 30 jours

### 2. Benchmarking

**Comparer performances vs moyenne marché**

```rust
pub struct BenchmarkData {
    pub building_id: Uuid,
    pub metric: String,
    pub building_value: Decimal,
    pub market_average: Decimal,
    pub percentile: i32, // 0-100
}
```

**Exemples** :
- Charges/m² : Vous 45€, Moyenne 52€ → 86e percentile
- Impayés : Vous 5%, Moyenne 8% → Mieux
- Consommation énergie : Vous 180 kWh/m², Moyenne 200

### 3. Prédictions Financières

**ML-based forecasting**

```rust
pub struct FinancialForecast {
    pub building_id: Uuid,
    pub period: String, // "2026-Q1"
    pub predicted_revenue: Decimal,
    pub predicted_expenses: Decimal,
    pub confidence_interval: (Decimal, Decimal),
}
```

**Use case** : Aider budget prévisionnel AG

### 4. Rapports Clients Automatiques

**PDF mensuel auto-généré pour syndic**

Contenu :
- Résumé encaissements
- Nouveaux impayés
- Travaux planifiés
- Incidents (tickets)
- KPIs vs mois précédent

**Envoi** : Email automatique le 1er du mois

---

## 📝 User Stories

```gherkin
En tant que cabinet de syndic
Je veux voir mes KPIs sur tous mes immeubles
Afin d'identifier ceux nécessitant attention

Scénario: Dashboard portfolio
  Étant donné que je gère 25 immeubles
  Quand je consulte le dashboard BI
  Alors je vois :
    - Revenus totaux : 450 000€ YTD
    - Taux recouvrement moyen : 94%
    - 3 immeubles en alerte impayés
    - Graph tendances charges
```

---

## 🔧 Endpoints

| Méthode | Endpoint | Description |
|---------|----------|-------------|
| `GET` | `/api/v1/analytics/portfolio` | Métriques portfolio |
| `GET` | `/api/v1/analytics/benchmark/:building_id` | Benchmark immeuble |
| `GET` | `/api/v1/analytics/forecast/:building_id` | Prédictions |
| `POST` | `/api/v1/analytics/reports/generate` | Générer rapport |
| `GET` | `/api/v1/analytics/kpis/:building_id` | KPIs temps réel |

---

## 📊 KPIs à Tracker

| KPI | Formule | Benchmark |
|-----|---------|-----------|
| Taux de recouvrement | Paiements reçus / Appelés | > 95% |
| Charges/m²/an | Total charges / Surface | 40-60€ |
| Délai moyen paiement | Moyenne jours après échéance | < 10j |
| Taux occupation | Lots occupés / Total lots | > 95% |
| Tickets résolus | Fermés / Ouverts | > 80% |

---

## 🛠️ Technologies

**Frontend** :
- Chart.js / D3.js pour visualisations
- TanStack Table pour tableaux

**Backend** :
- PostgreSQL analytics queries optimisées
- Materialized views pour performance
- Cron jobs pour rapports auto

---

## ✅ Critères d'Acceptation

- [ ] Dashboard charge < 2s pour 100 immeubles
- [ ] Benchmarks basés sur données anonymisées
- [ ] Prédictions MAPE < 15%
- [ ] Rapports PDF générés en < 5s
- [ ] Exports Excel/CSV

---

## 🚀 Checklist

- [ ] Créer vues matérialisées PostgreSQL
- [ ] AnalyticsUseCases
- [ ] Handlers
- [ ] Composant AnalyticsDashboard.svelte
- [ ] Intégration Chart.js
- [ ] ML forecast service
- [ ] PDF report generator
- [ ] Tests performance
- [ ] Documentation

---

**Créé le** : 2025-10-23
**Business Value** : Premium feature (paywall possible)
