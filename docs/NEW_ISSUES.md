# Issues Manquantes - Features Critiques Non Planifiées

**Date**: 2025-10-23
**Source**: Analyse étude de marché + Business Plan Bootstrap
**Total**: 9 nouvelles issues identifiées

---

## 🔴 CRITIQUES (Bloquent Lancement Belgique)

### #016 - Conformité PCN Belge (Plan Comptable Normalisé)

**Besoin**: "La Belgique utilise le Plan Comptable Normalisé (PCN) pour la comptabilité des copropriétés. KoproGo doit générer des rapports conformes PCN."

**Priorité**: 🔴 CRITIQUE (Phase 1)
**Estimation**: 12-15h
**Bloque**: Lancement Belgique

**Specs**:
- Mapper catégories expenses → Comptes PCN (classe 6: charges, classe 7: produits)
- Export rapport PCN avec numérotation belge (60 Achats, 61 Services, 62 Rémunérations, etc.)
- Format PDF + Excel conforme normes belges
- Intégration avec `ExpenseCalculator` existant
- Tests conformité avec échantillon réel syndic belge

**Exemple Mapping**:
```rust
ExpenseCategory::Utilities -> PCN::Account("615") // Utilities services
ExpenseCategory::Maintenance -> PCN::Account("611") // Building maintenance
ExpenseCategory::Insurance -> PCN::Account("613") // Insurance premiums
```

**Dépendances**: Issue #003 (Financial Reports)

---

### #017 - CODA Import Bancaire (Format Belge)

**Besoin**: "Les banques belges utilisent le format CODA (COded Daily) pour les relevés bancaires. Permettre import automatique des paiements copropriétaires."

**Priorité**: 🔴 CRITIQUE (Phase 2)
**Estimation**: 15-20h

**Specs**:
- Parser fichiers CODA (.cod) - Format fixe 128 caractères/ligne
- Extraire: montant, date, référence communication structurée (+++xxx/xxxx/xxxxx+++)
- Matching automatique paiement → expense via référence
- Réconciliation bancaire automatique
- Gestion rejets (paiements non identifiés)
- Support multi-banques belges (BNP Paribas Fortis, ING, KBC, Belfius)

**Format CODA** (simplifié):
```
0000001001200001CODA FILE HEADER...
1...           // Transaction header
2...           // Transaction details (amount, date, reference)
8...           // File trailer
9...           // End record
```

**Endpoint**:
```
POST /api/v1/payments/coda-import
- Upload .cod file
- Parse + match expenses
- Return matched/unmatched list
- Validate before auto-marking paid
```

**ROI**: Gain temps énorme syndics belges (90% paiements auto-réconciliés)

---

### #019 - Internationalization FR/NL/EN

**Besoin**: "La Belgique est bilingue FR/NL. L'interface doit supporter français, néerlandais, anglais pour compétitivité."

**Priorité**: 🔴 CRITIQUE (Phase 1)
**Estimation**: 8-10h

**Specs**:
- Backend: Enums + messages erreur traduits (FR/NL/EN)
- Frontend: i18n avec `svelte-i18n` ou `@formatjs/intl`
- Fichiers locales: `locales/fr.json`, `locales/nl.json`, `locales/en.json`
- Détection langue automatique (header `Accept-Language`)
- Sélecteur langue UI
- Traduction complète:
  - Labels formulaires
  - Messages validation
  - Emails notifications
  - PDFs rapports
  - Documentation

**Fichier locale exemple** (`locales/fr.json`):
```json
{
  "dashboard.title": "Tableau de Bord",
  "buildings.create": "Créer un Immeuble",
  "expenses.status.paid": "Payé",
  "meetings.type.ordinary": "Assemblée Générale Ordinaire"
}
```

**Néerlandais prioritaire** pour marché belge flamand (60% population)

---

### #020 - Multi-Tenancy Parfait (Isolation Données Hosted)

**Besoin**: "Pour le modèle hosted 1€/mois, isolation totale des données par organisation (syndic). Sécurité + performance critiques."

**Priorité**: 🔴 CRITIQUE (Phase 1-2)
**Estimation**: 10-12h
**Bloque**: Lancement hosted

**Specs**:
- Table `organizations` (syndic, cabinet)
- Toutes entités ont `organization_id` (building, unit, expense, etc.)
- Row-Level Security PostgreSQL (RLS policies)
- Middleware Actix vérifie `organization_id` dans JWT
- Tests isolation: User org A ne peut JAMAIS voir données org B
- Indexes optimisés `(organization_id, created_at)`
- Signup self-service avec création organization auto

**Schema Example**:
```sql
CREATE TABLE organizations (
  id UUID PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  plan VARCHAR(50) DEFAULT 'free', -- free, starter, pro
  max_buildings INT DEFAULT 1,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

ALTER TABLE buildings ADD COLUMN organization_id UUID REFERENCES organizations(id);
CREATE INDEX idx_buildings_org ON buildings(organization_id);

-- Row-Level Security
ALTER TABLE buildings ENABLE ROW LEVEL SECURITY;
CREATE POLICY buildings_isolation ON buildings
  USING (organization_id = current_setting('app.current_organization_id')::UUID);
```

**JWT Claims**:
```json
{
  "user_id": "uuid",
  "organization_id": "uuid",
  "role": "admin"
}
```

**Critique** pour modèle hosted = fondation architecture SaaS

---

### #022 - Belgian Council Management (Conseil Copropriété >20 lots)

**Besoin**: "En Belgique, conseil de copropriété obligatoire si >20 lots. Gérer membres conseil, mandats, décisions."

**Priorité**: 🟡 IMPORTANT (Phase 2)
**Estimation**: 6-8h

**Specs**:
- Entité `Council` (conseil de copropriété)
- Entité `CouncilMember` (membre du conseil)
- Validation automatique: si building.total_units > 20 → council requis
- Gestion mandats (durée, renouvellement)
- Décisions conseil (distinctes des décisions AG)
- Intégration avec `Meeting` (conseil se réunit entre AG)

**Règle Métier Belgique**:
```rust
impl Building {
    pub fn requires_council(&self) -> bool {
        match self.country {
            Country::Belgium => self.total_units > 20,
            Country::France => self.total_units > 15, // Variable selon règlement
            _ => false,
        }
    }
}
```

**Endpoints**:
```
POST   /api/v1/buildings/:id/council
GET    /api/v1/buildings/:id/council/members
POST   /api/v1/councils/:id/decisions
```

---

## 🟡 IMPORTANTES (Phase 2-3)

### #018 - Exact Online Export (Logiciel Comptabilité BE #1)

**Besoin**: "Exact Online est le logiciel comptable le plus utilisé en Belgique. Export écritures comptables pour import Exact."

**Priorité**: 🟡 IMPORTANT (Phase 2)
**Estimation**: 10-12h

**Specs**:
- Export format CSV compatible Exact Online
- Colonnes: Journal, Date, Numéro pièce, Compte, Débit, Crédit, Libellé
- Mapping PCN → Comptes Exact
- Export périodique (mois, trimestre, année)
- Validation format avant export

**Endpoint**:
```
GET /api/v1/reports/exact-export/:building_id?period=2025-Q1
- Returns CSV file
- Headers: Journal,Date,Account,Debit,Credit,Description
```

**Alternatif**: Odoo, Sage export (ajoutables Phase 3)

---

### #021 - Stripe Billing 1€/Mois (Hosted Monetization)

**Besoin**: "Système de billing automatique pour abonnements hosted 1€/mois ou 10€/mois selon plan."

**Priorité**: 🟡 IMPORTANT (Phase 2)
**Estimation**: 6-8h

**Specs**:
- Intégration Stripe Billing (Subscriptions)
- Plans:
  - `free`: 1 copropriété, 0€
  - `starter`: 5 copropriétés, 10€/mois
  - `pro`: 20 copropriétés, 35€/mois
  - `enterprise`: Illimité, 150€/mois
- Webhook `invoice.paid` → activer organization
- Webhook `invoice.payment_failed` → suspendre accès
- Self-service upgrade/downgrade
- Facturation automatique

**Flow**:
1. User signup → Free plan
2. Ajoute 2ème building → Prompt upgrade
3. Choose plan → Stripe Checkout
4. Payment success → Unlock features
5. Recurring billing auto

**Différent de Issue #006** (paiements charges copropriétaires)

---

### #023 - Country Regulations Engine

**Besoin**: "Support multi-pays (BE/FR/ES/IT/TN) avec règles métier spécifiques par pays."

**Priorité**: 🟡 IMPORTANT (Phase 2-3)
**Estimation**: 12-15h

**Specs**:
- Trait `CountryRegulations`
- Implémentations: `BelgiumRegulations`, `FranceRegulations`, `TunisiaRegulations`, etc.
- Validations dynamiques selon `building.country`
- Exemples règles:
  - BE: Syndic obligatoire ≥2 lots
  - ES: Syndic obligatoire >4 lots
  - IT: Syndic obligatoire >4 lots
  - TN: Syndicat toujours obligatoire
- Config multi-pays pour:
  - Seuils syndic
  - Formats comptables (PCN BE, FEC FR)
  - Formats bancaires (CODA BE, SEPA EU)
  - Langues officielles

**Architecture**:
```rust
pub trait CountryRegulations {
    fn syndic_mandatory_threshold(&self) -> u32;
    fn council_mandatory_threshold(&self) -> Option<u32>;
    fn accounting_format(&self) -> AccountingFormat;
    fn supported_languages(&self) -> Vec<Language>;
}

pub struct BelgiumRegulations;
impl CountryRegulations for BelgiumRegulations {
    fn syndic_mandatory_threshold(&self) -> u32 { 2 }
    fn council_mandatory_threshold(&self) -> Option<u32> { Some(20) }
    fn accounting_format(&self) -> AccountingFormat { AccountingFormat::PCN }
    fn supported_languages(&self) -> Vec<Language> { vec![Language::FR, Language::NL] }
}
```

**Extensible** pour expansion géographique future

---

### #024 - Multi-Currency Support (EUR/TND)

**Besoin**: "Support Euro (Belgique/France) et Dinar Tunisien (Tunisie) pour expansion Afrique du Nord."

**Priorité**: 🟡 IMPORTANT (Phase 3)
**Estimation**: 6-8h

**Specs**:
- Champ `currency` dans `Building` (EUR, TND, USD, etc.)
- Calculs montants avec `rust_decimal` (précision)
- Affichage formaté selon locale (1 234,56 € vs 1,234.56 TND)
- Conversion taux de change (API externe ou manuel)
- Rapports multi-devises (dashboard portfolio)

**Frontend**:
```typescript
formatCurrency(amount: number, currency: string, locale: string): string {
  return new Intl.NumberFormat(locale, {
    style: 'currency',
    currency: currency
  }).format(amount);
}

// Usage
formatCurrency(1234.56, 'EUR', 'fr-BE') // "1 234,56 €"
formatCurrency(1234.56, 'TND', 'ar-TN') // "1234.560 د.ت"
```

---

## 🟢 NICE-TO-HAVE (Phase 4+)

### #025 - TLIS Integration (Tunisian Land Registry)

**Besoin**: "Intégration avec TLIS (Système de gestion de l'information foncière tunisien) pour vérification propriété."

**Priorité**: 🟢 NICE (Phase 4)
**Estimation**: 15-20h

**Specs**:
- API TLIS (si disponible publiquement)
- Vérification propriété immeuble/lot
- Import automatique données cadastrales
- Validation propriétaires

**Note**: Dépend de disponibilité API gouvernementale tunisienne

---

## 📊 Résumé Priorisation Nouvelles Issues

| Issue | Nom | Priorité | Phase | Estimation |
|-------|-----|----------|-------|------------|
| #016 | PCN Belge | 🔴 CRITIQUE | 1 | 12-15h |
| #019 | i18n FR/NL/EN | 🔴 CRITIQUE | 1 | 8-10h |
| #020 | Multi-tenancy | 🔴 CRITIQUE | 1 | 10-12h |
| #017 | CODA Import | 🔴 CRITIQUE | 2 | 15-20h |
| #022 | Belgian Council | 🟡 IMPORTANT | 2 | 6-8h |
| #018 | Exact Export | 🟡 IMPORTANT | 2 | 10-12h |
| #021 | Stripe Billing | 🟡 IMPORTANT | 2 | 6-8h |
| #023 | Regulations Engine | 🟡 IMPORTANT | 2-3 | 12-15h |
| #024 | Multi-currency | 🟡 IMPORTANT | 3 | 6-8h |
| #025 | TLIS Integration | 🟢 NICE | 4 | 15-20h |

**Total Effort**: ~86-128h (11-16 semaines à 20h/sem)

---

## 🚨 Impact sur Roadmap

Ces 9 nouvelles issues **doivent être intégrées** au planning:

**Phase 1 ajustée**:
- Issues existantes: #001-#005 (37-46h)
- Issues nouvelles: #016, #019, #020 (30-37h)
- **Total Phase 1**: 67-83h (8-10 semaines)

**Phase 2 ajustée**:
- Issues existantes: #006-#010 (53-64h)
- Issues nouvelles: #017, #018, #021, #022, #023 (49-63h)
- **Total Phase 2**: 102-127h (13-16 semaines)

**Sans ces issues**, KoproGo ne peut PAS:
- ✅ Lancer en Belgique (#016, #017, #019)
- ✅ Faire du hosted (#020, #021)
- ✅ Être compétitif vs Vilogi/Copriciel

---

**Prochaine étape**: Intégrer dans ROADMAP.md et PRIORITIES_TABLE.md

**Dernière mise à jour**: 2025-10-23
