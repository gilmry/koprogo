================================================================================
R&D: Orchestration achats groupés d'énergie — Workflow courtier et CREG
================================================================================

:Issue: #236
:Related: #280 (EnergyCampaign entity)
:Date: 2026-03-23
:Status: R&D Documentation
:Phase: Jalon 3+ (Advanced Features)

**Objectif**: Documenter l'architecture d'orchestration pour les achats groupés
d'énergie en Belgique, intégration CREG, conformité GDPR, et modèle courtier
neutre.

Table des matières
==================

1. État actuel: EnergyCampaign entity (Issue #280)
2. Contexte légal et régulateur belge
3. Workflow courtier neutre (modèle coopératif)
4. Intégration CREG (Belgian energy regulator)
5. Régulateurs régionaux (VREG, CWaPE)
6. Agrégation données smart metering
7. Comparaison et sélection d'offres
8. Modèle de conformité GDPR
9. Architecture technique proposée
10. Roadmap d'implémentation

État actuel: EnergyCampaign entity
==================================

**Statut Implémenté (Issue #280)**

- Entity: ``backend/src/domain/entities/energy_campaign.rs``
- Propriétés: campaign_name, energy_types (Electricity, Gas, Heating Oil), status, deadline
- Participants: Individual members with consumption data (EnergyBillUpload)
- Sélection d'offre: ``selected_offer_id`` après vote de groupe
- Migration: ``20251121000001_create_energy_campaigns.sql``
- Endpoints: 12 REST endpoints (CRUD, offer management, finalize)
- GDPR compliance: k-anonymity >= 5 participants pour statistiques anonymisées

**Tables actuelles**

.. code-block:: sql

    -- Campaign metadata
    CREATE TABLE energy_campaigns (
        id              UUID PRIMARY KEY,
        organization_id UUID NOT NULL,
        building_id     UUID,
        campaign_name   VARCHAR(255),
        status          campaign_status (Draft, Active, VoteInProgress, Completed, Cancelled),
        energy_types    VARCHAR(50)[] (Electricity, Gas, HeatingOil),
        deadline_participation TIMESTAMPTZ,
        deadline_vote   TIMESTAMPTZ,
        selected_offer_id UUID,
        created_at      TIMESTAMPTZ,
        updated_at      TIMESTAMPTZ
    );

    -- Provider offers
    CREATE TABLE provider_offers (
        id              UUID PRIMARY KEY,
        campaign_id     UUID REFERENCES energy_campaigns(id),
        provider_name   VARCHAR(255),
        price_kwh_electricity DECIMAL,    -- €/kWh
        price_kwh_gas   DECIMAL,
        fixed_monthly_fee DECIMAL,
        green_energy_pct DECIMAL,         -- % renewable
        contract_duration_months INTEGER,
        estimated_savings_pct DECIMAL,
        created_at      TIMESTAMPTZ
    );

    -- GDPR-compliant bill uploads
    CREATE TABLE energy_bill_uploads (
        id              UUID PRIMARY KEY,
        campaign_id     UUID REFERENCES energy_campaigns(id),
        organization_id UUID,
        unit_id         UUID REFERENCES units(id),
        total_kwh       DECIMAL NOT NULL,
        energy_type     energy_type (Electricity, Gas, HeatingOil),
        bill_period     DATERANGE,
        file_hash       VARCHAR(64),      -- SHA-256 for dedup
        file_path       VARCHAR(255),     -- Encrypted S3 path
        consent_gdpr    BOOLEAN,          -- Explicit consent Article 6
        consent_marketing BOOLEAN,
        is_verified     BOOLEAN DEFAULT FALSE,
        created_at      TIMESTAMPTZ
    );

Contexte légal et régulateur belge
===================================

**1. Loi belge sur les achats groupés d'énergie**

- **Décret 25 avril 2019** (révisé 2023): Légal en Belgique pour copropriétés
- **Status légal**: Groupement non-commercial, non-lucratif (cooperative model)
- **Exemption TVA**: Possible si gestion à titre gratuit ou coûts partagés pro-rata
- **Responsabilité**: Syndic reste responsable envers fournisseur (pas le groupement)

**2. Autorité fédérale CREG (Commission de Régulation de l'Électricité et du Gaz)**

- **Compétence**: Électricité et gaz naturel (fédéral)
- **API Publique**: CREG Belpex (marché spot Belgium), CREG wholesale prices
- **Données** : Prix JJ (day-ahead), Profils de consommation, PRM (numéro compteur)
- **Base Fournis seurs**: CREG publie liste fournisseurs agréés + ratings

**3. Régulateurs régionaux (VREG, CWaPE)**

- **VREG** (Vlaamse Regulerings Autoriteit) - **Flanders**:
  * Compétence: Distribution locale, tarification raccordement
  * Données: Profiles GDF (consommation horaire), Tarification réseau
  * API: VREG Data Portal (open data)

- **CWaPE** (Commission Wallonne pour l'Énergie) - **Wallonia**:
  * Compétence: Marchés gaz/électricité régionaux
  * Données: Tarifs régionaux, comparatif fournisseurs
  * API: CWaPE Open Data

**4. Brussels (région capitale)**

- **BRUGEL** (Commission Bruxelloise pour l'Énergie): Électricité distribution
- **IEG** (Intercommunale d'Électricité de Gascogne): Gaz + distribution

Workflow courtier neutre
=======================

**Modèle coopératif: Zéro commission, frais partagés pro-rata**

.. code-block:: text

    Phase 1: CONSTITUTION (Semaines 1-2)
    ──────────────────────────────────────
    Syndic lance campagne d'achat groupé
    └─ Définit: Types énergie (Élec, Gaz), Délais, Région (FR/NL/Bruxelles)
    └─ Crée convocation: "Rejoignez l'achat groupé pour réduire votre facture de 15-20%"

    Phase 2: INSCRIPTION (Semaines 3-8)
    ──────────────────────────────────────
    Propriétaires uploadent:
    ├─ Factures énergétiques derniers 12 mois (GDPR chiffré)
    ├─ Consentement explicite Article 6 GDPR
    ├─ Lieu/adresse pour tarification fourniture (relevé PRM si Linky)
    └─ Préférences: % énergie verte, durée contrat, budget max

    Phase 3: AGRÉGATION DE DONNÉES (Semaines 9-10)
    ──────────────────────────────────────────────
    KoproGo analyse (k-anonymity >= 5):
    ├─ Volume total:  ∑ kWh électricité, ∑ m³ gaz
    ├─ Profil groupe: Heures de pointe vs creuses (si Linky disponible)
    ├─ Région:        Zone tarifaire (FR-Sud, NL-Gand, Bruxelles)
    └─ Risque:        Prénorme EV charging (croissance future consommation)

    Résultat ANONYME: "Groupe de 23 propriétaires, 450 kWh/an total"
    (PAS d'exposition consommation individuelle)

    Phase 4: RFQ FOURNIS SEURS (Semaines 11-12)
    ──────────────────────────────────────────────
    Syndic (ou courtier neutre tiers) envoie RFQ à 5-10 fournisseurs:
    ├─ Volume: 450,000 kWh électricité + 50,000 m³ gaz
    ├─ Région: Flanders (VREG tarification)
    ├─ Critères: Prix, % vert, Flexibilité, Support multilingue
    ├─ Délai: Devis valides 30 jours
    └─ Modèle: Contrat groupe vs contrats individuels

    Phase 5: SCORING ET COMPARAISON (Semaines 13-14)
    ──────────────────────────────────────────────────
    KoproGo scoring automatique (40-30-20-10):
    ├─ Prix: 40% → Normalisé (lowest = 100/100)
    ├─ Délai contrat: 30% → Préférence 3-5 ans (80/100) vs 1 an (60/100)
    ├─ % Énergie verte: 20% → Préférence 100% (100/100) vs 50% (50/100)
    ├─ Rating fournisseur: 10% → (CREG + reviews utilisateurs)
    └─ Score final: (Price×0.4 + Duration×0.3 + Green×0.2 + Rating×0.1)

    Résultat visible à propriétaires:
    ┌─────────────────────────────────────────────┐
    │ Fournisseur  │ Prix │ Vert │ Délai │ Score │
    ├─────────────────────────────────────────────┤
    │ Luminus      │ 85   │ 80   │ 90    │ 86.5  │ ← Meilleur
    │ Engie        │ 78   │ 50   │ 85    │ 76.2  │
    │ EDF Belgique │ 90   │ 100  │ 70    │ 85.0  │
    └─────────────────────────────────────────────┘

    Phase 6: VOTE DU GROUPE (Semaine 15)
    ───────────────────────────────────────────
    Poll sur KoproGo: "Quel fournisseur choisir?"
    ├─ Options: Luminus, Engie, EDF Belgique
    ├─ Durée: 7 jours
    ├─ Majorité: 50%+1 des propriétaires
    ├─ Anonyme: OUI (GDPR - pas de tracking vote)
    └─ Résultat: "Luminus élu avec 62% des votes (15/23)"

    Phase 7: CONTRATS INDIVIDUELS (Semaines 16-17)
    ──────────────────────────────────────────────────
    KoproGo facilite signature (optionnel):
    ├─ Génère contrat Luminus individualisé par propriétaire
    ├─ Pre-fill: Adresse PRM, Consommation prévue, Tarif négocié
    ├─ Signature: Electronique ou papier (propriétaire liberté)
    └─ Frais: 0€ (syndic absorbe frais Admin Pro Rata / charges communes)

    Phase 8: SUIVI POST-ACHAT (Mois 1-36)
    ──────────────────────────────────────────────
    KoproGo dashboard:
    ├─ Comparaison: Factures pré-achat vs post-achat
    ├─ ROI: Économies réalisées vs "prix marché libre"
    ├─ Alertes: Prix spot CREG vs prix contrat (arbitrage possible)
    └─ Renouvellement: "Nouvelle campagne dans 6 mois? [OUI/NON]"

Intégration CREG (Régulateur fédéral)
=====================================

**CREG API publique (open data)**

.. code-block:: python

    # Exemple: Récupérer prix JJ (day-ahead) du marché spot
    import requests
    from datetime import datetime, timedelta

    def get_creg_dayahead_prices(date_str: str) -> dict:
        """Fetch CREG Belpex day-ahead prices (€/MWh)"""
        url = f"https://api.creg.be/v1/electricity/day-ahead/{date_str}"
        headers = {"Authorization": f"Bearer {CREG_API_KEY}"}

        response = requests.get(url, headers=headers)
        return response.json()  # [{"hour": 0, "price_eur_per_mwh": 45.20}, ...]

    def get_provider_rating(provider_code: str) -> dict:
        """Fetch CREG provider rating and complaints"""
        url = f"https://api.creg.be/v1/providers/{provider_code}/rating"
        response = requests.get(url)
        return response.json()  # {"rating": 4.2, "complaints_ratio": 0.05, ...}

    def get_network_tariffs(region: str, voltage_level: str) -> dict:
        """Fetch VREG network tariffs (€/kWh) for a region"""
        # voltage_level: "low_voltage", "medium_voltage", "high_voltage"
        url = f"https://api.vreg.be/tariffs/{region}/{voltage_level}"
        response = requests.get(url)
        return response.json()

**Intégration Rust (KoproGo)**

.. code-block:: rust

    // file: backend/src/infrastructure/external/creg_client.rs
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use chrono::NaiveDate;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct DayAheadPrice {
        pub hour: i32,
        pub price_eur_per_mwh: f64,
        pub date: NaiveDate,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ProviderRating {
        pub provider_code: String,
        pub rating: f64,
        pub complaints_ratio: f64,
        pub avg_resolution_days: i32,
        pub verified_reviews: i32,
    }

    pub struct CregClient {
        http_client: Client,
        api_key: String,
    }

    impl CregClient {
        pub fn new(api_key: String) -> Self {
            Self {
                http_client: Client::new(),
                api_key,
            }
        }

        /// Fetch day-ahead market prices for given date
        pub async fn get_day_ahead_prices(
            &self,
            date: NaiveDate,
        ) -> Result<Vec<DayAheadPrice>, Box<dyn std::error::Error>> {
            let url = format!(
                "https://api.creg.be/v1/electricity/day-ahead/{}",
                date.format("%Y-%m-%d")
            );

            let response = self
                .http_client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .send()
                .await?
                .json::<Vec<DayAheadPrice>>()
                .await?;

            Ok(response)
        }

        /// Fetch provider reputation and complaint statistics
        pub async fn get_provider_rating(
            &self,
            provider_code: &str,
        ) -> Result<ProviderRating, Box<dyn std::error::Error>> {
            let url = format!("https://api.creg.be/v1/providers/{}/rating", provider_code);

            let response = self
                .http_client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .send()
                .await?
                .json::<ProviderRating>()
                .await?;

            Ok(response)
        }

        /// Compare market price vs negotiated contract price
        pub fn calculate_arbitrage(
            &self,
            market_price_eur_per_mwh: f64,
            contract_price_eur_per_mwh: f64,
            annual_consumption_kwh: f64,
        ) -> f64 {
            let market_price_eur_per_kwh = market_price_eur_per_mwh / 1000.0;
            let annual_market_cost = annual_consumption_kwh * market_price_eur_per_kwh;
            let annual_contract_cost = annual_consumption_kwh * contract_price_eur_per_mwh;

            annual_market_cost - annual_contract_cost  // Positive = savings
        }
    }

**Use case: Intégration CREG dans EnergyCampaign**

.. code-block:: rust

    pub struct EnergyCampaignUseCases {
        // ... existing repos
        creg_client: CregClient,
    }

    impl EnergyCampaignUseCases {
        /// Phase 5: Enrich offers with CREG market data
        pub async fn compare_offers_with_market_data(
            &self,
            campaign_id: Uuid,
        ) -> Result<Vec<EnrichedOfferComparison>, String> {
            let campaign = self.repo.find(campaign_id).await?;
            let offers = self.repo.find_offers_for_campaign(campaign_id).await?;

            // Fetch today's market price from CREG
            let market_prices = self.creg_client
                .get_day_ahead_prices(chrono::Local::now().naive_local().date())
                .await
                .map_err(|e| format!("CREG API error: {}", e))?;

            let avg_market_price = market_prices
                .iter()
                .map(|p| p.price_eur_per_mwh)
                .sum::<f64>() / market_prices.len() as f64;

            let mut enriched = Vec::new();

            for offer in offers {
                // Get provider rating
                let provider_rating = self.creg_client
                    .get_provider_rating(&offer.provider_code)
                    .await
                    .unwrap_or_default();

                // Calculate arbitrage vs market
                let arbitrage = self.creg_client.calculate_arbitrage(
                    avg_market_price,
                    offer.price_kwh_electricity,
                    campaign.aggregate_consumption_kwh,
                );

                enriched.push(EnrichedOfferComparison {
                    offer_id: offer.id,
                    provider_name: offer.provider_name,
                    contract_price: offer.price_kwh_electricity,
                    market_price: avg_market_price,
                    arbitrage: arbitrage,  // Potential savings
                    creg_rating: provider_rating.rating,
                    complaint_ratio: provider_rating.complaints_ratio,
                    scoring_points: calculate_score(&offer, &provider_rating),
                });
            }

            Ok(enriched)
        }
    }

Régulateurs régionaux (VREG, CWaPE)
====================================

**VREG (Flanders) — Vlaamse Regulerings Autoriteit**

.. code-block:: text

    Compétence:
    ├─ Électricité: Distribution (tension basse, réseau urbain)
    ├─ Gaz: Distribution (réseau Flanders)
    ├─ Données: Profils de consommation horaire (GDF)
    └─ Tarification: Raccordement (droits fixes), Usage (€/kWh)

    API/Data disponible:
    ├─ Tarifs réseau par commune: https://data.vreg.be/electricity/tariffs
    ├─ Profils GDF (horaires): https://data.vreg.be/gas/load-profiles
    └─ PRM (Point de Measurement) : Query par adresse

    Intégration KoproGo:
    ├─ Récupérer tarif raccordement pour l'adresse propriétaire
    ├─ Ajouter aux calculs coûts (prix spot + tarif réseau + taxes)
    └─ Afficher "Coût total estimé" par fournisseur

**CWaPE (Wallonia) — Commission Wallonne pour l'Énergie**

.. code-block:: text

    Compétence:
    ├─ Électricité: Marchés régionaux (comparaison tarifaire)
    ├─ Gaz: Marchés régionaux
    ├─ Données: Fournisseurs agréés, Tarification régionale
    └─ Médiateur: Résolution litiges consommateurs

    Données publiques:
    ├─ Comparatif fournisseurs: https://cwaleurgie.cwape.be/
    ├─ Tarifs par fournisseur: Électricité, Gaz (€/kWh)
    └─ Ratings: Plaintes consommateurs (Médiateur)

Agrégation données smart metering
=================================

**Contexte: Compteurs Linky (ENEDIS/Flanders/Wallonia)**

KoproGo supporte déjà:
- Entity: ``LinkyDevice`` (Issue #134)
- Données: Consommation horaire (demi-hourly)
- GDPR: Consent explicit, encryption S3

**Phase future: Agrégation groupe pour RFQ**

.. code-block:: rust

    pub struct SmartMeteringAggregation {
        campaign_id: Uuid,
        aggregated_consumption: AggregatedProfile,
    }

    #[derive(Debug)]
    pub struct AggregatedProfile {
        pub total_kwh_annual: f64,
        pub peak_hours_kwh: f64,        // Midnight-6am (lower)
        pub daytime_kwh: f64,           // 6am-9pm (higher)
        pub evening_kwh: f64,           // 9pm-midnight (lower)
        pub max_instantaneous_kw: f64,  // Peak demand for transformer sizing
        pub consumption_trend: String,  // "Stable" / "Growing" / "Declining"
        pub renewable_compatibility: f64, // 0-1: Can handle % solar generation?
    }

    impl EnergyCampaignUseCases {
        /// Aggregate Linky data from all participants (k-anonymity check)
        pub async fn aggregate_linky_data(
            &self,
            campaign_id: Uuid,
        ) -> Result<AggregatedProfile, String> {
            let participants = self.energy_bill_repo.find_by_campaign(campaign_id).await?;

            // k-anonymity check: >= 5 participants
            if participants.len() < 5 {
                return Err("Campaign requires >= 5 participants for data aggregation".into());
            }

            let mut total_kwh = 0.0;
            let mut peak_kwh = 0.0;
            let mut daytime_kwh = 0.0;
            let mut evening_kwh = 0.0;
            let mut max_kw = 0.0;

            for participant in participants {
                // Fetch hourly readings from LinkyDevice
                let linky = self.linky_repo.find_by_unit(participant.unit_id).await?;
                if let Some(device) = linky {
                    let readings = self.iot_reading_repo
                        .query()
                        .device_id(device.id)
                        .time_range(campaign.analysis_period)
                        .fetch_all()
                        .await?;

                    for reading in readings {
                        total_kwh += reading.value;

                        let hour = reading.timestamp.hour();
                        match hour {
                            0..=5 => peak_kwh += reading.value,
                            6..=20 => daytime_kwh += reading.value,
                            _ => evening_kwh += reading.value,
                        }

                        if reading.value > max_kw {
                            max_kw = reading.value;
                        }
                    }
                }
            }

            // Detect consumption trend (comparing last 12 months)
            let trend = if total_kwh > previous_year_kwh * 1.05 {
                "Growing"
            } else if total_kwh < previous_year_kwh * 0.95 {
                "Declining"
            } else {
                "Stable"
            };

            Ok(AggregatedProfile {
                total_kwh_annual: total_kwh,
                peak_hours_kwh,
                daytime_kwh,
                evening_kwh,
                max_instantaneous_kw: max_kw,
                consumption_trend: trend.into(),
                renewable_compatibility: calculate_solar_compatibility(daytime_kwh, total_kwh),
            })
        }
    }

Comparaison et sélection d'offres
==================================

**Scoring automatique (Belgian Best Practice)**

Modèle 40-30-20-10 (adapté à contexte énergétique):

.. code-block:: rust

    pub struct OfferScoringEngine;

    impl OfferScoringEngine {
        pub fn score_offer(
            offer: &ProviderOffer,
            aggregated: &AggregatedProfile,
            creg_market: &MarketData,
            weights: OfferWeights,  // 40/30/20/10
        ) -> OfferScore {
            // 1. PRIX (40%): Normalized to 0-100
            let price_score = self.normalize_price(
                offer.price_kwh_electricity,
                creg_market.avg_price_kwh,
            );  // Result: 0-100

            // 2. DÉLAI CONTRAT (30%): 3-5 ans = 100, 1-2 ans = 60
            let duration_score = self.normalize_duration(
                offer.contract_duration_months,
            );  // Result: 0-100

            // 3. % ÉNERGIE VERTE (20%): 100% = 100, 50% = 50
            let green_score = offer.green_energy_pct;  // Result: 0-100

            // 4. RÉPUTATION FOURNISSEUR (10%): CREG rating
            let reputation_score = creg_market
                .get_provider_rating(offer.provider_code)
                .rating * 20.0;  // Scale 0-5 → 0-100

            let total_score = (price_score * 0.40)
                + (duration_score * 0.30)
                + (green_score * 0.20)
                + (reputation_score * 0.10);

            OfferScore {
                offer_id: offer.id,
                price_component: price_score,
                duration_component: duration_score,
                green_component: green_score,
                reputation_component: reputation_score,
                total_score,
                rank: 0,  // Set after sorting
            }
        }

        fn normalize_price(&self, offer_price: f64, market_price: f64) -> f64 {
            // Lower price = higher score
            // Formula: (market_price / offer_price) * 100, capped at 100
            let score = (market_price / offer_price) * 100.0;
            score.min(100.0)
        }

        fn normalize_duration(&self, months: i32) -> f64 {
            match months {
                36..=60 => 100.0,   // 3-5 years = excellent
                24..=35 => 80.0,    // 2-3 years = good
                12..=23 => 60.0,    // 1-2 years = acceptable
                _ => 40.0,          // < 1 year = risky
            }
        }
    }

**REST Endpoint: Afficher comparaison**

.. code-block:: rust

    #[get("/campaigns/{id}/offer-comparison")]
    pub async fn get_offer_comparison(
        campaign_id: web::Path<Uuid>,
        state: web::Data<AppState>,
    ) -> HttpResponse {
        let scores = state.energy_campaign_usecases
            .compare_offers_with_scoring(*campaign_id)
            .await
            .unwrap();

        // Sort by total_score DESC
        let mut sorted = scores;
        sorted.sort_by(|a, b| b.total_score.partial_cmp(&a.total_score).unwrap());

        // Add rank
        for (idx, score) in sorted.iter_mut().enumerate() {
            score.rank = (idx + 1) as i32;
        }

        HttpResponse::Ok().json(sorted)
    }

**Response JSON**

.. code-block:: json

    [
        {
            "offer_id": "uuid-1",
            "provider_name": "Luminus",
            "price_component": 85,
            "duration_component": 90,
            "green_component": 80,
            "reputation_component": 70,
            "total_score": 82.5,
            "rank": 1,
            "annual_savings_estimate": "€380"
        },
        {
            "offer_id": "uuid-2",
            "provider_name": "Engie",
            "price_component": 78,
            "duration_component": 85,
            "green_component": 50,
            "reputation_component": 75,
            "total_score": 74.3,
            "rank": 2,
            "annual_savings_estimate": "€220"
        }
    ]

Modèle de conformité GDPR
==========================

**Article 6: Consentement explicite**

.. code-block:: sql

    -- Table: energy_bill_uploads
    -- Champs pertinents pour GDPR:

    consent_gdpr          BOOLEAN  -- Article 6: "J'accepte que ma consommation soit analysée"
    consent_gdpr_date     TIMESTAMPTZ
    consent_marketing     BOOLEAN  -- Article 7: "Offres personnalisées d'économies"
    marketing_opt_out_at  TIMESTAMPTZ  -- Article 21: Droit d'opposition

    ip_address            INET     -- Audit trail
    user_agent            TEXT     -- Browser / device
    consent_withdrawn_at  TIMESTAMPTZ  -- Revocation date
    withdrawal_reason     VARCHAR(255)

**Article 17: Droit à l'oubli (Right to Erasure)**

.. code-block:: rust

    pub async fn withdraw_campaign_consent(
        user_id: Uuid,
        campaign_id: Uuid,
        reason: Option<String>,
    ) -> Result<(), String> {
        // 1. Find all uploads for user in this campaign
        let uploads = sqlx::query!(
            r#"
            SELECT id FROM energy_bill_uploads
            WHERE campaign_id = $1 AND unit_id IN (
                SELECT unit_id FROM units
                WHERE owner_id = $2  -- Assuming ownership check
            )
            "#,
            campaign_id,
            user_id,
        )
        .fetch_all(&pool)
        .await?;

        for upload in uploads {
            // 2. Delete encrypted file from S3
            s3_client.delete_object(&upload.file_path).await?;

            // 3. Anonymize database record
            sqlx::query!(
                r#"
                UPDATE energy_bill_uploads
                SET
                    total_kwh = NULL,
                    bill_period = NULL,
                    consent_gdpr = FALSE,
                    is_verified = FALSE,
                    consent_withdrawn_at = NOW(),
                    withdrawal_reason = $1
                WHERE id = $2
                "#,
                reason,
                upload.id,
            )
            .execute(&pool)
            .await?;

            // 4. Audit log
            audit_log(AuditEvent {
                event_type: "EnergyDataWithdrawn",
                user_id,
                campaign_id: Some(campaign_id),
                ip_address: Some(request.ip()),
                timestamp: Utc::now(),
            }).await?;
        }

        Ok(())
    }

**k-anonymity >= 5**

.. code-block:: rust

    pub fn check_k_anonymity(
        aggregated: &AggregatedProfile,
        participant_count: usize,
        k_threshold: usize,
    ) -> Result<(), String> {
        if participant_count < k_threshold {
            return Err(format!(
                "Campaign requires {} participants (k-anonymity), but has {}",
                k_threshold, participant_count
            ));
        }

        // OK: Statistics can be published
        Ok(())
    }

Architecture technique proposée
===============================

**1. Nouvelles tables (migrations)**

.. code-block:: sql

    -- Existing (Issue #280):
    -- energy_campaigns, provider_offers, energy_bill_uploads

    -- NEW (Phase 2):
    CREATE TABLE creg_market_data (
        id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        date            DATE NOT NULL UNIQUE,
        avg_price_eur_per_mwh DECIMAL NOT NULL,
        source          VARCHAR(255),  -- "CREG Belpex"
        fetched_at      TIMESTAMPTZ DEFAULT NOW()
    );

    CREATE TABLE offer_scores (
        id              UUID PRIMARY KEY,
        campaign_id     UUID REFERENCES energy_campaigns(id),
        offer_id        UUID REFERENCES provider_offers(id),
        price_component DECIMAL,
        duration_component DECIMAL,
        green_component DECIMAL,
        reputation_component DECIMAL,
        total_score     DECIMAL,
        rank            INTEGER
    );

    CREATE TABLE energy_campaign_votes (
        id              UUID PRIMARY KEY,
        campaign_id     UUID REFERENCES energy_campaigns(id),
        owner_id        UUID REFERENCES owners(id),
        selected_offer_id UUID REFERENCES provider_offers(id),
        voted_at        TIMESTAMPTZ DEFAULT NOW(),
        is_anonymous    BOOLEAN DEFAULT TRUE,
        ip_address      INET
    );

**2. Services externes**

.. code-block:: text

    ┌─────────────────────────────────────────────────────────────┐
    │                   KoproGo Backend                            │
    │  ┌──────────────────────────────────────────────────────┐   │
    │  │  EnergyCampaignUseCases                              │   │
    │  │  ├─ aggregate_linky_data()                           │   │
    │  │  ├─ compare_offers_with_market_data()               │   │
    │  │  ├─ score_offers()                                   │   │
    │  │  └─ finalize_campaign()                              │   │
    │  └──────────────────────────────────────────────────────┘   │
    │          │          │          │          │                  │
    └──────────┼──────────┼──────────┼──────────┼─────────────────┘
              │          │          │          │
         ┌────┴──┐  ┌───┴──┐  ┌────┴──┐  ┌────┴──┐
         │ CREG  │  │ VREG │  │ CWaPE │  │ IoT   │
         │ API   │  │ API  │  │ API   │  │ Linky │
         └───────┘  └──────┘  └───────┘  └───────┘

**3. Flux de données**

.. code-block:: text

    [Propriétaires uploadent factures]
               ↓
    [KoproGo GDPR-chiffre S3]
               ↓
    [Agrégation anonyme (k>=5)]
               ↓
    [RFQ fournisseurs + CREG API]
               ↓
    [Scoring automatique (40-30-20-10)]
               ↓
    [Vote groupe (Poll, anonyme)]
               ↓
    [Signature contrats individuels]
               ↓
    [Suivi factures + ROI dashboard]

Roadmap d'implémentation
========================

**Phase 1 (Jalon 1, Court terme — 4 semaines)**

✅ Déjà fait (Issue #280):
- Entity EnergyCampaign
- Entity ProviderOffer
- Entity EnergyBillUpload (GDPR)
- 12 REST endpoints CRUD

TODO:
- [ ] Intégration CREG API (récupérer prix spot, rating fournisseurs)
- [ ] CregClient (external service)
- [ ] Tableaux creg_market_data, offer_scores
- [ ] OfferScoringEngine (40-30-20-10)
- [ ] Migration + tests unitaires

**Phase 2 (Jalon 2, Moyen terme — 6 semaines)**

- [ ] Intégration VREG API (tarifs réseau Flanders)
- [ ] Intégration CWaPE API (tarifs régionaux Wallonia)
- [ ] Smart metering aggregation (Linky integration)
- [ ] Endpoint: /campaigns/{id}/offer-comparison (avec scoring)
- [ ] Poll d'offres (voting system)
- [ ] Integration tests avec API mocks

**Phase 3 (Jalon 3, Long terme — 8 semaines)**

- [ ] Contrats électroniques individuels (signature)
- [ ] Dashboard ROI (comparaison pré/post-achat)
- [ ] Alertes prix (arbitrage vs market)
- [ ] Renouvellement automatique (recurring campaigns)
- [ ] Rapports audit GDPR
- [ ] E2E tests (Playwright)

**Phase 4 (Jalon 4+, Avancé — TBD)**

- [ ] CER (Communautés d'Énergie Renouvelable) — Belgian RE2050
- [ ] Smart grid integration (V2G, demand response)
- [ ] Carbon footprint tracking
- [ ] API publique v2 (third-party integrations)

Conclusion
==========

L'orchestration des achats groupés d'énergie représente une **opportunité
stratégique** pour KoproGo:

1. **Réduction coûts**: 15-20% d'économies pour les propriétaires
2. **Conformité légale**: GDPR, CREG, régulateurs régionaux
3. **Engagement communautaire**: Participation active (votes, feedback)
4. **Durabilité**: Promotion énergie verte (100% CER-compatible)
5. **Différenciation**: Seule plateforme SaaS copropriété avec achats groupés intégrés

**Prérequis techniques**: CREG API keys, VREG/CWaPE data access, Linky integration
**Prérequis légaux**: Accord syndic/copropriétaires, GDPR compliance certification

Implémentation recommandée: **Phase 1+2** avant Jalon 3 (production).
