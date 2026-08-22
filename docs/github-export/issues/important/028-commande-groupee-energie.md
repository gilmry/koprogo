# Issue #028: Commande Groupée Énergie (Électricité + Gaz)

**Labels**: `energy`, `automation`, `belgian-market`, `cost-savings`
**Priority**: 🟡 High
**Track**: Software
**Phase**: Phase 2 (K3s - Automation & Community)
**Effort**: 10-12 heures

---

## 📋 Description

Système de commande groupée d'énergie (électricité + gaz) pour copropriétés belges, permettant négociation collective avec fournisseurs pour réduire coûts de 15-30%.

**Spécificité Belgique**: Marché libéralisé énergie depuis 2007, commandes groupées légales et encouragées par CWaPE/VREG/BRUGEL (régulateurs régionaux).

---

## 🎯 Objectifs Business

### Impact Économique
- **Économies**: 15-30% sur factures énergie via négociation groupée
- **Volume**: Plus de participants = meilleur prix négocié
- **Transparence**: Comparaison multi-fournisseurs avec scoring automatique

### Différenciateur Marché
- **Unique**: Peu de plateformes copropriété proposent cela en Belgique
- **Valeur ajoutée**: Service concret économies pour propriétaires
- **Écologie**: Possibilité privilégier fournisseurs verts (éolien, solaire)

---

## 🏗️ Architecture Technique

### 1. Entités Domain

#### `EnergyContract`
```rust
pub struct EnergyContract {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub contract_type: EnergyType,           // Electricity, Gas, Both
    pub provider: String,                    // Luminus, Engie, Mega, etc.
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub annual_consumption_kwh: f64,         // Consommation annuelle estimée
    pub price_per_kwh: f64,                  // Prix actuel
    pub status: ContractStatus,              // Active, Expired, PendingRenewal
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum EnergyType {
    Electricity,
    Gas,
    Both,
}

pub enum ContractStatus {
    Active,
    Expired,
    PendingRenewal,
    Cancelled,
}
```

#### `GroupPurchaseCampaign`
```rust
pub struct GroupPurchaseCampaign {
    pub id: Uuid,
    pub name: String,                        // "Campagne Énergie Hiver 2026"
    pub energy_type: EnergyType,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,             // Deadline inscription
    pub min_participants: i32,               // Minimum 10 copropriétés
    pub participants_count: i32,
    pub total_consumption_kwh: f64,          // Volume agrégé
    pub status: CampaignStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum CampaignStatus {
    Open,           // Inscriptions ouvertes
    NegotiatingQuotes,  // Collecte devis fournisseurs
    VotingPhase,    // Vote participants sur meilleures offres
    Closed,         // Contrats signés
    Cancelled,
}
```

#### `GroupPurchaseParticipant`
```rust
pub struct GroupPurchaseParticipant {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub annual_consumption_kwh: f64,
    pub current_price_per_kwh: f64,
    pub preferred_green_energy: bool,
    pub registered_at: DateTime<Utc>,
}
```

#### `EnergyQuote`
```rust
pub struct EnergyQuote {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub provider: String,                    // Luminus, Engie, Mega, etc.
    pub price_per_kwh: f64,
    pub contract_duration_months: i32,       // 12, 24, 36 mois
    pub green_energy_percentage: f32,        // 0-100%
    pub fixed_price: bool,                   // Prix fixe vs variable
    pub validity_date: DateTime<Utc>,
    pub conditions: String,                  // Conditions particulières
    pub status: QuoteStatus,
    pub created_at: DateTime<Utc>,
}

pub enum QuoteStatus {
    Pending,
    Accepted,
    Rejected,
}
```

---

### 2. Use Cases

#### `EnergyGroupPurchaseUseCases`
```rust
pub struct EnergyGroupPurchaseUseCases {
    contract_repo: Arc<dyn EnergyContractRepository>,
    campaign_repo: Arc<dyn GroupPurchaseCampaignRepository>,
    participant_repo: Arc<dyn GroupPurchaseParticipantRepository>,
    quote_repo: Arc<dyn EnergyQuoteRepository>,
}

impl EnergyGroupPurchaseUseCases {
    // Campagnes
    pub async fn create_campaign(&self, dto: CreateCampaignDto) -> Result<...>;
    pub async fn list_active_campaigns(&self) -> Result<Vec<...>>;
    pub async fn get_campaign_stats(&self, campaign_id: Uuid) -> Result<...>;

    // Participation
    pub async fn register_participant(&self, dto: RegisterParticipantDto) -> Result<...>;
    pub async fn calculate_potential_savings(&self, participant: ...) -> Result<f64>;

    // Devis fournisseurs
    pub async fn submit_provider_quote(&self, dto: CreateQuoteDto) -> Result<...>;
    pub async fn compare_quotes(&self, campaign_id: Uuid) -> Result<Vec<QuoteComparison>>;
    pub async fn score_quotes(&self, campaign_id: Uuid) -> Result<Vec<ScoredQuote>>;

    // Contrats
    pub async fn create_contract_from_quote(&self, quote_id: Uuid) -> Result<...>;
    pub async fn list_expiring_contracts(&self, days_before: i32) -> Result<Vec<...>>;
}
```

---

### 3. API Endpoints

```
POST   /api/v1/energy/campaigns                    # Créer campagne (SuperAdmin)
GET    /api/v1/energy/campaigns                    # Lister campagnes actives
GET    /api/v1/energy/campaigns/:id                # Détails campagne
POST   /api/v1/energy/campaigns/:id/register       # S'inscrire à campagne (Syndic)
GET    /api/v1/energy/campaigns/:id/participants   # Liste participants

POST   /api/v1/energy/quotes                       # Soumettre devis (Provider API key)
GET    /api/v1/energy/campaigns/:id/quotes         # Devis pour campagne
GET    /api/v1/energy/quotes/:id/compare           # Comparaison devis

POST   /api/v1/energy/contracts                    # Créer contrat depuis devis
GET    /api/v1/energy/contracts                    # Liste contrats (Syndic/Accountant)
GET    /api/v1/energy/contracts/expiring           # Contrats expirant < 90 jours
```

---

### 4. Frontend Components

#### Pages
- `/energy/campaigns` - Liste campagnes actives avec stats
- `/energy/campaigns/:id` - Détails campagne + inscription
- `/energy/campaigns/:id/quotes` - Comparaison devis fournisseurs
- `/energy/contracts` - Gestion contrats énergie
- `/energy/savings-calculator` - Calculateur économies potentielles

#### Composants Svelte
- `EnergyCampaignCard.svelte` - Card campagne avec stats
- `EnergyQuoteComparison.svelte` - Tableau comparatif devis
- `EnergySavingsChart.svelte` - Graphique économies projetées
- `EnergyProviderRating.svelte` - Notation fournisseurs
- `ContractExpiryAlert.svelte` - Alerte contrats expirants

---

## 🔄 Workflow Complet

### Phase 1: Lancement Campagne (SuperAdmin/Coordinateur)
1. **Création campagne** (ex: "Campagne Électricité Hiver 2026")
2. **Définition paramètres**:
   - Type énergie (élec, gaz, ou les deux)
   - Dates (inscription: 2 mois, négociation: 1 mois)
   - Minimum participants (ex: 10 copropriétés)
3. **Communication**: Email auto tous syndics inscrits plateforme

### Phase 2: Inscriptions Participants (Syndics)
1. **Copropriétés s'inscrivent** via formulaire:
   - Consommation annuelle estimée (kWh)
   - Prix actuel (€/kWh)
   - Préférence énergie verte (oui/non)
2. **Dashboard temps réel**:
   - Participants: 23/10 (minimum atteint ✅)
   - Volume agrégé: 4.2 GWh/an
   - Économies projetées: 18-25%

### Phase 3: Collecte Devis Fournisseurs (API Providers)
1. **Notification automatique fournisseurs** avec specs:
   - Volume total: 4.2 GWh/an
   - Profil consommation: résidentiel collectif
   - Durée souhaitée: 12-36 mois
2. **Fournisseurs soumettent devis** via API ou form web:
   - Prix/kWh fixe ou variable
   - % énergie verte
   - Conditions particulières
3. **Deadline réception**: 2 semaines

### Phase 4: Comparaison & Scoring (Automatique)
1. **Algorithme scoring**:
   ```
   Score = Prix (50%) + Énergie Verte (25%) + Durée Contrat (15%) + Réputation (10%)
   ```
2. **Tableau comparatif multi-critères**:
   ```
   | Fournisseur | €/kWh | Économies | Vert | Durée | Score | Rang |
   |-------------|-------|-----------|------|-------|-------|------|
   | Luminus     | 0.18  | 22%       | 100% | 24m   | 87/100| 🥇   |
   | Engie       | 0.19  | 18%       | 80%  | 36m   | 81/100| 🥈   |
   | Mega        | 0.17  | 26%       | 50%  | 12m   | 79/100| 🥉   |
   ```
3. **Analyse détaillée**:
   - Graphiques économies sur durée contrat
   - Comparaison avec prix marché actuel
   - Projection factures annuelles

### Phase 5: Vote Participants (Syndics + Propriétaires)
1. **Notification vote** (email + notification app)
2. **Page vote** avec authentification forte (optionnel: itsme®)
3. **3 meilleures offres** présentées avec analyse
4. **Durée vote**: 2 semaines
5. **Résultats**: Majorité simple (50%+1)

### Phase 6: Attribution & Contractualisation
1. **Fournisseur gagnant notifié** automatiquement
2. **Contrats individuels générés** (1 par copropriété)
3. **Workflow signature**:
   - PDF contrat envoyé à chaque syndic
   - Signature électronique (ex: DocuSign, eIDAS)
4. **Activation contrats**: Date coordonnée (ex: 1er janvier)

### Phase 7: Suivi & Renouvellement
1. **Alertes automatiques**:
   - J-90: "Votre contrat expire dans 3 mois"
   - J-60: "Nouvelle campagne ouverte pour renouvellement"
2. **Dashboard suivi**:
   - Consommation réelle vs estimée
   - Économies réalisées (€)
   - Impact CO2 si énergie verte

---

## 📊 Business Logic Complexe

### Calcul Économies Potentielles
```rust
pub fn calculate_potential_savings(
    current_price: f64,
    annual_consumption_kwh: f64,
    group_quotes: Vec<EnergyQuote>,
) -> SavingsProjection {
    let current_annual_cost = current_price * annual_consumption_kwh;

    let mut projections = vec![];
    for quote in group_quotes {
        let new_annual_cost = quote.price_per_kwh * annual_consumption_kwh;
        let savings_eur = current_annual_cost - new_annual_cost;
        let savings_percentage = (savings_eur / current_annual_cost) * 100.0;

        projections.push(QuoteSavings {
            provider: quote.provider,
            savings_eur,
            savings_percentage,
            green_energy: quote.green_energy_percentage,
        });
    }

    projections.sort_by(|a, b| b.savings_eur.partial_cmp(&a.savings_eur).unwrap());

    SavingsProjection {
        current_cost: current_annual_cost,
        best_saving: projections.first(),
        all_options: projections,
    }
}
```

### Scoring Devis Multi-Critères
```rust
pub fn score_quote(quote: &EnergyQuote, campaign_stats: &CampaignStats) -> f32 {
    // Critère 1: Prix (50%)
    let market_avg_price = campaign_stats.average_current_price;
    let price_savings = ((market_avg_price - quote.price_per_kwh) / market_avg_price) * 100.0;
    let price_score = (price_savings.max(0.0).min(30.0) / 30.0) * 50.0;

    // Critère 2: Énergie Verte (25%)
    let green_score = (quote.green_energy_percentage / 100.0) * 25.0;

    // Critère 3: Durée Contrat (15% - préférence 24 mois)
    let ideal_duration = 24;
    let duration_diff = (quote.contract_duration_months - ideal_duration).abs();
    let duration_score = (1.0 - (duration_diff as f32 / 24.0).min(1.0)) * 15.0;

    // Critère 4: Réputation Provider (10% - hardcoded pour MVP)
    let reputation_score = get_provider_reputation(&quote.provider) * 10.0;

    price_score + green_score + duration_score + reputation_score
}
```

---

## 🔧 Intégrations Externes

### 1. API Fournisseurs (Optionnel Phase 2)
- **Luminus API**: Automatisation devis
- **Engie B2B API**: Prix temps réel
- **Mega API**: Soumission automatique

**Fallback MVP**: Form web manuel pour fournisseurs (no-code)

### 2. Régulateurs Belges (Informatif)
- **CWaPE** (Wallonie): Tarifs régulés
- **VREG** (Flandre): Prix marché
- **BRUGEL** (Bruxelles): Comparaisons officielles

**Usage**: Afficher liens ressources légales

---

## 🧪 Tests & Validations

### Tests Unitaires
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_savings_positive() {
        let current_price = 0.25; // €/kWh
        let consumption = 50000.0; // kWh/an
        let quote = EnergyQuote {
            price_per_kwh: 0.20,
            ..Default::default()
        };

        let savings = calculate_savings(current_price, consumption, &quote);
        assert_eq!(savings.eur, 2500.0); // (0.25 - 0.20) * 50000
        assert_eq!(savings.percentage, 20.0);
    }

    #[test]
    fn test_score_quote_perfect() {
        let quote = EnergyQuote {
            price_per_kwh: 0.15, // 30% cheaper than market
            green_energy_percentage: 100.0,
            contract_duration_months: 24, // Ideal
            provider: "Luminus".to_string(),
            ..Default::default()
        };

        let stats = CampaignStats {
            average_current_price: 0.22,
            ..Default::default()
        };

        let score = score_quote(&quote, &stats);
        assert!(score >= 90.0); // Near-perfect score
    }
}
```

### Tests E2E (Cucumber)
```gherkin
Feature: Commande Groupée Énergie

  Scenario: Syndic s'inscrit à campagne active
    Given une campagne "Hiver 2026" est ouverte
    And le syndic gère un immeuble consommant 50000 kWh/an
    When le syndic s'inscrit à la campagne
    Then la participation est confirmée
    And le dashboard affiche "Économies projetées: 18-25%"

  Scenario: Comparaison devis multi-fournisseurs
    Given une campagne avec 3 devis soumis:
      | Provider | Prix/kWh | Vert | Score |
      | Luminus  | 0.18     | 100% | 87    |
      | Engie    | 0.19     | 80%  | 81    |
      | Mega     | 0.17     | 50%  | 79    |
    When le syndic accède à la comparaison
    Then les devis sont triés par score descendant
    And le meilleur devis est "Luminus" avec badge 🥇
```

---

## 📚 Documentation Utilisateur

### Guide Syndic: "Participer à une Commande Groupée"
1. **Prérequis**: Contrat énergie actuel (copie facture)
2. **Inscription**: Formulaire avec consommation annuelle
3. **Vote**: Notification quand devis disponibles (2 semaines)
4. **Signature**: PDF contrat envoyé par email
5. **Activation**: Changement fournisseur automatique à date convenue

### FAQ
**Q: La participation est-elle engageante?**
R: Non, vous pouvez vous retirer jusqu'à la phase de vote.

**Q: Qui négocie avec les fournisseurs?**
R: La plateforme automatise l'agrégation. Pas de négociation manuelle.

**Q: Que se passe-t-il si minimum participants non atteint?**
R: La campagne est annulée automatiquement, aucun engagement.

---

## 🚀 Livrables Phase 2 (K3s)

### Semaine 1-2 (5-6h)
- ✅ 4 nouvelles entités domain + repositories
- ✅ Migrations SQL (tables + indexes)
- ✅ UseCases core (création campagne, inscription, devis)
- ✅ Tests unitaires (15+ tests)

### Semaine 3 (4-5h)
- ✅ 8 API endpoints
- ✅ Handlers avec authorization (SuperAdmin, Syndic, Provider)
- ✅ Validation business rules (min participants, deadlines)

### Semaine 4 (3-4h)
- ✅ 5 pages frontend + 5 composants Svelte
- ✅ Dashboard campagne avec stats temps réel
- ✅ Tableau comparatif devis interactif
- ✅ Tests E2E Cucumber (3 scenarios)

### Documentation
- ✅ Guide utilisateur syndic (FR/NL)
- ✅ API docs pour fournisseurs
- ✅ Cahier des charges intégration fournisseurs (Phase 3)

---

## 🔗 Dépendances

**Dépend de**:
- ✅ #016 (Plan Comptable Belge) - Pour lier factures énergie aux comptes
- ✅ #001 (Meeting API) - Pour votes AG si validation requise
- ✅ #042 (GDPR) - Données consommation sensibles

**Bloque** (Nice-to-Have Phase 3):
- 🔮 Integration API fournisseurs automatisée
- 🔮 Dashboard analytics consommations temps réel

---

## 📊 Métriques Succès

**KPIs Business**:
- **Adoption**: >30% copropriétés inscrites à première campagne
- **Économies**: Moyenne 20% économies participants
- **Satisfaction**: NPS >8/10 participants

**KPIs Techniques**:
- **Performance**: API < 200ms (calculs scoring)
- **Uptime**: 99.9% durant phases critiques (votes, deadlines)
- **Sécurité**: 0 breach données consommation (GDPR compliant)

---

**Version**: 1.0
**Créé le**: 2024-11-07
**Auteur**: KoproGo Roadmap Team
**Statut**: 🟡 Proposé (Phase 2 - Q2 2026)
