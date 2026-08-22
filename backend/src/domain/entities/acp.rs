//! `Acp` — Association des Copropriétaires (Art. 3.84-3.89 Code Civil belge).
//!
//! Story 1.1 — première brique de la refacto domaine
//! `Organization(0..1) → ACP(1..N) → Building(1..N)`
//! (cf. `docs/maury/refonte-ux-multi-role-acp/architecture.md` §2.1, ADR-0010).
//!
//! Une ACP est la **personne juridique** propriétaire collective de l'immeuble en
//! copropriété. Elle est distincte du cabinet syndic (`Organization`) qui la
//! gère : un cabinet peut gérer plusieurs ACPs, et une ACP peut être
//! auto-gérée (aucun cabinet).
//!
//! # Invariants
//!
//! - `name` non vide après trim, longueur ≥ 2 caractères (PRD FR1, FR3, INV-1).
//! - `slug` kebab-case dérivé du `name` (unicité scope = repository).
//! - `address_street`, `address_postal_code`, `address_city` non vides.
//! - `organization_id` est `Option<Uuid>` (NULL = ACP auto-gérée — ADR-0010).
//! - `legal_status` par défaut `"copropriete_belge"`.
//!
//! # Hexagonal
//!
//! Aucune dépendance `sqlx` / `actix_web`. Les erreurs de domaine
//! retournent `AcpError` (enum dédié), mappé vers `AppError::Validation`
//! côté application (cf. `application/error.rs`, pattern WP-A* #433).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Dénominateur par défaut de l'acte de base (millièmes belges classiques).
/// L'acte authentique peut fixer 10000 (dix-millièmes) ou une autre base —
/// cf. Art. 3.84 CC + ADR-0010. Jamais hard-codé ailleurs : toute logique de
/// conformité lit `Acp::total_tantiemes`.
pub const DEFAULT_TOTAL_TANTIEMES: i32 = 1000;

/// Story H13 — Taux minimal légal du fonds de réserve : **5 % des charges
/// ordinaires de l'exercice N-1** (Art. 3.86 §3 Code civil, loi du 18/06/2018
/// en vigueur depuis 2019). Obligatoire ; l'AG peut y renoncer à la majorité
/// des **4/5** (`reserve_fund_waived`). Cf. ADR-0012.
pub const RESERVE_FUND_RATE: Decimal = dec!(0.05);

/// Métriques agrégées d'une ACP (calculées par le repository via JOIN sur
/// tous les buildings de l'ACP — Story H6). Pureté : aucun I/O ici.
///
/// La conformité (Art. 3.84 CC, ADR-0010) s'évalue au **niveau ACP** :
/// `Σ units == Σ buildings.total_units` ET `Σ quota == acps.total_tantiemes`.
/// Volontairement non stockées (avoid stale state) — recalculées à la lecture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcpMetrics {
    /// Nombre réel de units de tous les blocs de l'ACP (`COUNT(units)`).
    pub units_count: i32,
    /// Somme des `buildings.total_units` (lots déclarés) de tous les blocs.
    pub declared_units_total: i32,
    /// Somme exacte des quotités générales (`SUM(units.quota::NUMERIC)`).
    pub quota_sum: Decimal,
    /// Nombre de buildings (blocs) rattachés à l'ACP.
    pub buildings_count: i32,
}

impl AcpMetrics {
    /// Métriques vides (ACP sans bloc / sans lot).
    pub fn empty() -> Self {
        Self {
            units_count: 0,
            declared_units_total: 0,
            quota_sum: Decimal::ZERO,
            buildings_count: 0,
        }
    }
}

/// Statut juridique d'une ACP. `Copropriete` correspond à
/// "copropriete_belge" en DB (encodage stable v0.1.0).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AcpLegalStatus {
    /// Copropriété belge ordinaire (Art. 3.84 CC).
    #[default]
    CoproprieteBelge,
}

impl AcpLegalStatus {
    /// Encodage DB stable.
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::CoproprieteBelge => "copropriete_belge",
        }
    }

    /// Décodage depuis la chaîne DB. Toute valeur inconnue est mappée
    /// volontairement vers `CoproprieteBelge` (mode `lenient`) plutôt que
    /// panic — un `legal_status` exotique en DB n'est pas une raison de
    /// faire crasher la lecture (audit + corrigé hors-bande).
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "copropriete_belge" => Self::CoproprieteBelge,
            _ => Self::CoproprieteBelge,
        }
    }
}

/// Erreurs métier produites par le domaine `Acp`.
///
/// Mappées vers `AppError::Validation` (HTTP 400/422) côté application via un
/// `impl From<AcpError> for AppError` (cf. `application/error.rs`).
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AcpError {
    #[error("ACP name cannot be empty")]
    NameEmpty,
    #[error("ACP name must be at least 2 characters long, got {0}")]
    NameTooShort(usize),
    #[error("ACP name must be at most 160 characters long, got {0}")]
    NameTooLong(usize),
    #[error("ACP address street cannot be empty")]
    AddressStreetEmpty,
    #[error("ACP postal code cannot be empty")]
    PostalCodeEmpty,
    #[error("ACP city cannot be empty")]
    CityEmpty,
    #[error("ACP total_tantiemes (acte de base) must be greater than 0, got {0}")]
    TotalTantiemesInvalid(i32),
    #[error("ACP fund balance cannot be negative, got {0}")]
    NegativeFundBalance(Decimal),
}

/// Représente une Association des Copropriétaires (ACP) — racine d'agrégat.
///
/// Cf. ADR-0010 (`docs/maury/refonte-ux-multi-role-acp/architecture.md` §4).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct Acp {
    pub id: Uuid,
    /// Cabinet syndic gestionnaire. `None` = ACP auto-gérée (ADR-0010).
    pub organization_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub legal_status: AcpLegalStatus,
    /// Dénominateur de l'acte de base (quotités). Source de vérité de la
    /// copropriété (Art. 3.84 CC, ADR-0010). 1000/10000/autre. Défaut
    /// `DEFAULT_TOTAL_TANTIEMES` ; modifiable via `with_total_tantiemes`.
    pub total_tantiemes: i32,
    /// Numéro BCE belge (optionnel — toutes les ACPs ne sont pas immatriculées).
    pub bce_number: Option<String>,
    pub address_street: String,
    pub address_postal_code: String,
    pub address_city: String,
    /// Story H13 — solde du **fonds de réserve** (compte distinct au nom de
    /// l'ACP, Art. 3.86 §3). Doit couvrir ≥ 5 % des charges ordinaires N-1
    /// sauf renonciation 4/5. Decimal exact (ADR-0007).
    #[serde(default)]
    pub reserve_fund_balance: Decimal,
    /// Story H13 — solde du **fonds de roulement** (compte distinct, dépenses
    /// courantes récurrentes — loi 2019).
    #[serde(default)]
    pub working_capital_balance: Decimal,
    /// Story H13 — l'AG a-t-elle renoncé au fonds de réserve obligatoire
    /// (vote 4/5, Art. 3.86 §3) ? Si `true`, la conformité réserve est levée.
    #[serde(default)]
    pub reserve_fund_waived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Acp {
    /// Constructeur validé.
    ///
    /// Invariants vérifiés :
    /// 1. `name.trim()` ≥ 2 chars, ≤ 160 chars
    /// 2. `address_street.trim()` non vide
    /// 3. `address_postal_code.trim()` non vide
    /// 4. `address_city.trim()` non vide
    ///
    /// Le `slug` est dérivé du `name` (kebab-case ASCII).
    pub fn new(
        organization_id: Option<Uuid>,
        name: String,
        address_street: String,
        address_postal_code: String,
        address_city: String,
        bce_number: Option<String>,
    ) -> Result<Self, AcpError> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(AcpError::NameEmpty);
        }
        let name_len = name.chars().count();
        if name_len < 2 {
            return Err(AcpError::NameTooShort(name_len));
        }
        if name_len > 160 {
            return Err(AcpError::NameTooLong(name_len));
        }

        let address_street = address_street.trim().to_string();
        if address_street.is_empty() {
            return Err(AcpError::AddressStreetEmpty);
        }
        let address_postal_code = address_postal_code.trim().to_string();
        if address_postal_code.is_empty() {
            return Err(AcpError::PostalCodeEmpty);
        }
        let address_city = address_city.trim().to_string();
        if address_city.is_empty() {
            return Err(AcpError::CityEmpty);
        }

        let slug = generate_slug(&name);
        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            name,
            slug,
            legal_status: AcpLegalStatus::default(),
            total_tantiemes: DEFAULT_TOTAL_TANTIEMES,
            bce_number,
            address_street,
            address_postal_code,
            address_city,
            reserve_fund_balance: Decimal::ZERO,
            working_capital_balance: Decimal::ZERO,
            reserve_fund_waived: false,
            created_at: now,
            updated_at: now,
        })
    }

    /// Rattache (ou détache, avec `None`) l'ACP à un cabinet syndic.
    /// Le sens "promote/demote" est interprété par l'application : ici on se
    /// limite à mettre à jour le champ + `updated_at`.
    pub fn set_organization(&mut self, organization_id: Option<Uuid>) {
        self.organization_id = organization_id;
        self.updated_at = Utc::now();
    }

    /// Mise à jour de l'identité de l'ACP (avec re-validation des invariants).
    pub fn update_info(
        &mut self,
        name: String,
        address_street: String,
        address_postal_code: String,
        address_city: String,
        bce_number: Option<String>,
    ) -> Result<(), AcpError> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(AcpError::NameEmpty);
        }
        let name_len = name.chars().count();
        if name_len < 2 {
            return Err(AcpError::NameTooShort(name_len));
        }
        if name_len > 160 {
            return Err(AcpError::NameTooLong(name_len));
        }
        let address_street = address_street.trim().to_string();
        if address_street.is_empty() {
            return Err(AcpError::AddressStreetEmpty);
        }
        let address_postal_code = address_postal_code.trim().to_string();
        if address_postal_code.is_empty() {
            return Err(AcpError::PostalCodeEmpty);
        }
        let address_city = address_city.trim().to_string();
        if address_city.is_empty() {
            return Err(AcpError::CityEmpty);
        }

        self.slug = generate_slug(&name);
        self.name = name;
        self.address_street = address_street;
        self.address_postal_code = address_postal_code;
        self.address_city = address_city;
        self.bce_number = bce_number;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// L'ACP est-elle auto-gérée (sans cabinet syndic) ?
    pub fn is_self_managed(&self) -> bool {
        self.organization_id.is_none()
    }

    /// Builder consommant : fixe le dénominateur de l'acte de base.
    ///
    /// Garde `new()` stable (les appelants existants conservent le défaut
    /// `DEFAULT_TOTAL_TANTIEMES`). Valide `value > 0` (Art. 3.84 CC — un acte
    /// de base sans tantièmes n'a pas de sens). Cf. ADR-0010.
    pub fn with_total_tantiemes(mut self, value: i32) -> Result<Self, AcpError> {
        if value <= 0 {
            return Err(AcpError::TotalTantiemesInvalid(value));
        }
        self.total_tantiemes = value;
        Ok(self)
    }

    /// Mise à jour du dénominateur de l'acte de base (re-validation).
    pub fn set_total_tantiemes(&mut self, value: i32) -> Result<(), AcpError> {
        if value <= 0 {
            return Err(AcpError::TotalTantiemesInvalid(value));
        }
        self.total_tantiemes = value;
        self.updated_at = Utc::now();
        Ok(())
    }

    // ========================================================================
    // Story H5 (CL1) — Conformité de la copropriété au niveau ACP (Art. 3.84 CC).
    //
    // Règle (ADR-0010, mémoires `admin-publishes-conform-buildings` +
    // `validate-before-compute`) : l'acte de base est porté par l'ACP. La
    // conformité s'évalue sur l'agrégat de TOUS les blocs :
    //   `Σ units_count == Σ buildings.total_units` ET
    //   `Σ units.quota == acps.total_tantiemes`
    // Decimal strict (ADR-0007) — aucune tolérance d'arrondi.
    //
    // Pureté hexagonale : reçoit `AcpMetrics` (calculé par le repository),
    // aucun I/O.
    // ========================================================================

    /// L'ACP est-elle conformante, étant données ses métriques agrégées ?
    pub fn is_conformant(&self, metrics: &AcpMetrics) -> bool {
        metrics.units_count == metrics.declared_units_total
            && metrics.quota_sum == Decimal::from(self.total_tantiemes)
    }

    /// Écart de quotités vs l'acte de base : `total_tantiemes - quota_sum`.
    /// Positif si l'ACP manque de quotités (drift), négatif si surplus.
    pub fn quota_delta(&self, metrics: &AcpMetrics) -> Decimal {
        Decimal::from(self.total_tantiemes) - metrics.quota_sum
    }

    /// Retourne `Err(AcpNotConformantError)` typée si l'ACP n'est pas conforme.
    /// Consommée par les use-cases (validate-before-compute, Story H7) et le
    /// frontend (banner/toast 422 narratif).
    pub fn assert_conformant(&self, metrics: &AcpMetrics) -> Result<(), AcpNotConformantError> {
        if !self.is_conformant(metrics) {
            return Err(AcpNotConformantError {
                acp_id: self.id,
                units_delta: metrics.declared_units_total - metrics.units_count,
                quota_delta: self.quota_delta(metrics),
                quota_basis: self.total_tantiemes,
            });
        }
        Ok(())
    }

    // ========================================================================
    // Story H13 (CL4) — Fonds de réserve & de roulement (Art. 3.86 §3, loi 2019).
    //
    // Le fonds de réserve doit représenter ≥ 5 % des charges ordinaires de
    // l'exercice N-1 (`RESERVE_FUND_RATE`), sauf renonciation 4/5 de l'AG
    // (`reserve_fund_waived`). Comptes distincts au nom de l'ACP (modélisés par
    // `reserve_fund_balance` + `working_capital_balance`). Decimal strict
    // (ADR-0007) — pas de tolérance d'arrondi. Cf. ADR-0012.
    // ========================================================================

    /// Fixe le solde du fonds de réserve (validé ≥ 0).
    pub fn set_reserve_fund_balance(&mut self, balance: Decimal) -> Result<(), AcpError> {
        if balance < Decimal::ZERO {
            return Err(AcpError::NegativeFundBalance(balance));
        }
        self.reserve_fund_balance = balance;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Fixe le solde du fonds de roulement (validé ≥ 0).
    pub fn set_working_capital_balance(&mut self, balance: Decimal) -> Result<(), AcpError> {
        if balance < Decimal::ZERO {
            return Err(AcpError::NegativeFundBalance(balance));
        }
        self.working_capital_balance = balance;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Enregistre la décision d'AG de renoncer (ou non) au fonds de réserve
    /// obligatoire (vote 4/5, Art. 3.86 §3). Le vote lui-même (quorum/majorité)
    /// relève de la gouvernance (CL3) ; ici on consigne l'issue.
    pub fn set_reserve_fund_waived(&mut self, waived: bool) {
        self.reserve_fund_waived = waived;
        self.updated_at = Utc::now();
    }

    /// Montant minimal légal du fonds de réserve = 5 % des charges ordinaires
    /// N-1 (`RESERVE_FUND_RATE`). Exact (Decimal).
    pub fn required_reserve_fund(&self, ordinary_charges_n1: Decimal) -> Decimal {
        ordinary_charges_n1 * RESERVE_FUND_RATE
    }

    /// La réserve est-elle conforme ? Vrai si renoncée (4/5) OU si le solde
    /// couvre le minimum légal (≥ 5 % charges N-1). Borne inclusive : exactement
    /// 5 % passe.
    pub fn is_reserve_fund_compliant(&self, ordinary_charges_n1: Decimal) -> bool {
        self.reserve_fund_waived
            || self.reserve_fund_balance >= self.required_reserve_fund(ordinary_charges_n1)
    }

    /// Retourne `Err(ReserveFundInsufficientError)` typée si la réserve est
    /// sous le seuil légal et non renoncée. Consommée par les gates de
    /// conformité (CL4) et le frontend (`<ReserveFundIndicator>`, différé #634).
    pub fn assert_reserve_fund_compliant(
        &self,
        ordinary_charges_n1: Decimal,
    ) -> Result<(), ReserveFundInsufficientError> {
        if self.is_reserve_fund_compliant(ordinary_charges_n1) {
            return Ok(());
        }
        Err(ReserveFundInsufficientError {
            acp_id: self.id,
            required: self.required_reserve_fund(ordinary_charges_n1),
            actual: self.reserve_fund_balance,
            ordinary_charges_n1,
        })
    }
}

/// Story H5 — Erreur typée de non-conformité d'une ACP (Art. 3.84 CC, INV-L3).
///
/// Exposée par `Acp::assert_conformant()`. Mappée vers
/// `AppError::AcpNotConformant` (HTTP 422 + payload `ACP_NOT_CONFORMANT`) par
/// `From<>` dans `application/error.rs`. Même convention que
/// `BuildingNotConformantError` (Story H1) : `quota_delta = total_tantiemes -
/// quota_sum`, `quota_basis` = acte de base (1000/10000/autre).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcpNotConformantError {
    pub acp_id: Uuid,
    pub units_delta: i32,
    pub quota_delta: Decimal,
    pub quota_basis: i32,
}

impl std::fmt::Display for AcpNotConformantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ACP {} not conformant: {} units missing, quota delta {} / {} (acte de base)",
            self.acp_id, self.units_delta, self.quota_delta, self.quota_basis
        )
    }
}

impl std::error::Error for AcpNotConformantError {}

/// Story H13 — Erreur typée : fonds de réserve sous le seuil légal des 5 %
/// (Art. 3.86 §3, loi 2019) et non renoncé par l'AG (4/5). Mappée vers
/// `AppError::ReserveFundInsufficient` (HTTP 422 + `RESERVE_FUND_INSUFFICIENT`)
/// par `From<>` dans `application/error.rs`. Cf. ADR-0012.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReserveFundInsufficientError {
    pub acp_id: Uuid,
    /// Minimum légal = 5 % des charges ordinaires N-1.
    pub required: Decimal,
    /// Solde actuel du fonds de réserve.
    pub actual: Decimal,
    /// Base de calcul : charges ordinaires de l'exercice N-1.
    pub ordinary_charges_n1: Decimal,
}

impl std::fmt::Display for ReserveFundInsufficientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ACP {} reserve fund insufficient: {} < required {} (5% of {} ordinary charges N-1)",
            self.acp_id, self.actual, self.required, self.ordinary_charges_n1
        )
    }
}

impl std::error::Error for ReserveFundInsufficientError {}

/// Génération du slug kebab-case (déaccentué, alphanum + tirets).
///
/// Exemple : `"Résidence Les Tilleuls"` → `"residence-les-tilleuls"`.
fn generate_slug(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'à' | 'á' | 'â' | 'ã' | 'ä' => 'a',
            'È' | 'É' | 'Ê' | 'Ë' | 'è' | 'é' | 'ê' | 'ë' => 'e',
            'Ì' | 'Í' | 'Î' | 'Ï' | 'ì' | 'í' | 'î' | 'ï' => 'i',
            'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
            'Ù' | 'Ú' | 'Û' | 'Ü' | 'ù' | 'ú' | 'û' | 'ü' => 'u',
            'Ç' | 'ç' => 'c',
            'Ñ' | 'ñ' => 'n',
            _ if c.is_alphanumeric() => c.to_ascii_lowercase(),
            _ if c.is_whitespace() || c == '-' => '-',
            _ => '-',
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

// ============================================================================
// Tests — taxonomie 4 catégories (CRITICAL.md règle #3, #427).
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ----- @happy --------------------------------------------------------------

    #[test]
    fn happy_new_acp_with_organization_succeeds() {
        let org_id = Uuid::new_v4();
        let acp = Acp::new(
            Some(org_id),
            "Residence Les Tilleuls".to_string(),
            "Rue de la Paix 12".to_string(),
            "1000".to_string(),
            "Bruxelles".to_string(),
            None,
        )
        .expect("constructor must accept valid inputs");

        assert_eq!(acp.organization_id, Some(org_id));
        assert_eq!(acp.name, "Residence Les Tilleuls");
        assert_eq!(acp.slug, "residence-les-tilleuls");
        assert_eq!(acp.legal_status, AcpLegalStatus::CoproprieteBelge);
        assert_eq!(acp.address_city, "Bruxelles");
        assert!(!acp.is_self_managed());
    }

    #[test]
    fn happy_new_acp_without_organization_is_self_managed() {
        let acp = Acp::new(
            None,
            "Copro Autogeree".to_string(),
            "Rue X 1".to_string(),
            "1000".to_string(),
            "Bruxelles".to_string(),
            None,
        )
        .unwrap();

        assert!(acp.is_self_managed());
        assert_eq!(acp.organization_id, None);
    }

    #[test]
    fn happy_set_organization_attaches_and_detaches() {
        let mut acp = Acp::new(
            None,
            "Test".to_string(),
            "Rue X".to_string(),
            "1000".to_string(),
            "Bruxelles".to_string(),
            None,
        )
        .unwrap();
        let original_updated = acp.updated_at;

        let org_id = Uuid::new_v4();
        acp.set_organization(Some(org_id));
        assert_eq!(acp.organization_id, Some(org_id));
        assert!(acp.updated_at >= original_updated);

        acp.set_organization(None);
        assert_eq!(acp.organization_id, None);
        assert!(acp.is_self_managed());
    }

    #[test]
    fn happy_update_info_regenerates_slug() {
        let mut acp = Acp::new(
            None,
            "Old Name".to_string(),
            "Rue X".to_string(),
            "1000".to_string(),
            "Bruxelles".to_string(),
            None,
        )
        .unwrap();
        assert_eq!(acp.slug, "old-name");

        acp.update_info(
            "New Name".to_string(),
            "Rue X".to_string(),
            "1000".to_string(),
            "Bruxelles".to_string(),
            None,
        )
        .unwrap();
        assert_eq!(acp.name, "New Name");
        assert_eq!(acp.slug, "new-name");
    }

    // ----- total_tantiemes (acte de base, ADR-0010) — 4-cat ------------------

    fn sample_acp() -> Acp {
        Acp::new(
            None,
            "Acte Base Test".to_string(),
            "Rue X 1".to_string(),
            "1000".to_string(),
            "Bruxelles".to_string(),
            None,
        )
        .unwrap()
    }

    #[test]
    fn happy_total_tantiemes_defaults_to_1000() {
        assert_eq!(sample_acp().total_tantiemes, DEFAULT_TOTAL_TANTIEMES);
        assert_eq!(sample_acp().total_tantiemes, 1000);
    }

    #[test]
    fn happy_with_total_tantiemes_10000_acte_dix_millemes() {
        let acp = sample_acp().with_total_tantiemes(10000).unwrap();
        assert_eq!(acp.total_tantiemes, 10000);
    }

    #[test]
    fn edge_with_total_tantiemes_1_accepted() {
        let acp = sample_acp().with_total_tantiemes(1).unwrap();
        assert_eq!(acp.total_tantiemes, 1);
    }

    #[test]
    fn edge_set_total_tantiemes_updates_timestamp() {
        let mut acp = sample_acp();
        let before = acp.updated_at;
        acp.set_total_tantiemes(10000).unwrap();
        assert_eq!(acp.total_tantiemes, 10000);
        assert!(acp.updated_at >= before);
    }

    #[test]
    fn security_total_tantiemes_must_be_explicit_to_change() {
        // Le défaut ne peut PAS être 0 ou négatif silencieusement : seul
        // `with_total_tantiemes`/`set_total_tantiemes` (validés) le modifient.
        assert!(sample_acp().total_tantiemes > 0);
    }

    #[test]
    fn negative_with_total_tantiemes_zero_rejected() {
        let err = sample_acp().with_total_tantiemes(0).unwrap_err();
        assert_eq!(err, AcpError::TotalTantiemesInvalid(0));
    }

    #[test]
    fn negative_with_total_tantiemes_negative_rejected() {
        let err = sample_acp().with_total_tantiemes(-5).unwrap_err();
        assert_eq!(err, AcpError::TotalTantiemesInvalid(-5));
    }

    #[test]
    fn negative_set_total_tantiemes_zero_rejected_and_unchanged() {
        let mut acp = sample_acp().with_total_tantiemes(10000).unwrap();
        let err = acp.set_total_tantiemes(0).unwrap_err();
        assert_eq!(err, AcpError::TotalTantiemesInvalid(0));
        assert_eq!(acp.total_tantiemes, 10000); // inchangé
    }

    // ----- assert_conformant (Story H5, ADR-0010) — 4-cat --------------------

    fn metrics(units: i32, declared: i32, quota: Decimal, blocs: i32) -> AcpMetrics {
        AcpMetrics {
            units_count: units,
            declared_units_total: declared,
            quota_sum: quota,
            buildings_count: blocs,
        }
    }

    #[test]
    fn happy_acp_conformant_base_1000_mono_bloc() {
        let acp = sample_acp(); // total_tantiemes = 1000
        let m = metrics(10, 10, Decimal::from(1000), 1);
        assert!(acp.is_conformant(&m));
        assert!(acp.assert_conformant(&m).is_ok());
    }

    #[test]
    fn happy_acp_conformant_base_10000_multi_blocs() {
        let acp = sample_acp().with_total_tantiemes(10000).unwrap();
        // 3 blocs, 182 lots au total, Σ quotités = 10000.
        let m = metrics(182, 182, Decimal::from(10000), 3);
        assert!(acp.assert_conformant(&m).is_ok());
    }

    #[test]
    fn edge_acp_quota_drift_one_tenth_base_10000() {
        let acp = sample_acp().with_total_tantiemes(10000).unwrap();
        let m = metrics(182, 182, Decimal::from(9999) + Decimal::new(9, 1), 3); // 9999.9
        let err = acp.assert_conformant(&m).unwrap_err();
        assert_eq!(err.acp_id, acp.id);
        assert_eq!(err.quota_delta, Decimal::new(1, 1)); // 0.1
        assert_eq!(err.quota_basis, 10000);
        assert_eq!(err.units_delta, 0);
    }

    #[test]
    fn edge_acp_units_drift_quota_ok() {
        let acp = sample_acp(); // 1000
                                // 9 lots réels mais 10 déclarés ; quotités OK à 1000.
        let m = metrics(9, 10, Decimal::from(1000), 1);
        let err = acp.assert_conformant(&m).unwrap_err();
        assert_eq!(err.units_delta, 1);
        assert_eq!(err.quota_delta, Decimal::ZERO);
    }

    #[test]
    fn security_acp_metrics_tampering_detected() {
        // Métriques forgées « conformes-mais-fausses » : le domaine reflète
        // fidèlement les metrics reçues (la source de vérité = la query SQL,
        // testée séparément). Ici un quota_sum tronqué est bien détecté.
        let acp = sample_acp().with_total_tantiemes(10000).unwrap();
        let m = metrics(182, 182, Decimal::from(5000), 3); // moitié manquante
        let err = acp.assert_conformant(&m).unwrap_err();
        assert_eq!(err.quota_delta, Decimal::from(5000));
        assert_eq!(err.quota_basis, 10000);
    }

    #[test]
    fn negative_acp_empty_metrics_is_not_conformant() {
        let acp = sample_acp(); // 1000
        let m = AcpMetrics::empty();
        let err = acp.assert_conformant(&m).unwrap_err();
        assert_eq!(err.quota_delta, Decimal::from(1000));
        assert_eq!(err.quota_basis, 1000);
        assert_eq!(err.units_delta, 0);
    }

    #[test]
    fn negative_acp_not_conformant_error_display_is_narrative() {
        let acp = sample_acp().with_total_tantiemes(10000).unwrap();
        let err = acp
            .assert_conformant(&metrics(181, 182, Decimal::from(9975), 3))
            .unwrap_err();
        let s = format!("{}", err);
        assert!(s.contains("not conformant"));
        assert!(s.contains("10000"));
    }

    // ----- Fonds de réserve (Story H13, Art. 3.86 §3, loi 2019) — 4-cat ------

    #[test]
    fn happy_reserve_fund_meets_5pct_threshold() {
        // Charges ordinaires N-1 = 100 000 € → réserve requise = 5 000 €.
        let charges = Decimal::from(100_000);
        let mut acp = sample_acp();
        acp.set_reserve_fund_balance(Decimal::from(5000)).unwrap();
        assert_eq!(acp.required_reserve_fund(charges), Decimal::from(5000));
        assert!(acp.is_reserve_fund_compliant(charges));
        assert!(acp.assert_reserve_fund_compliant(charges).is_ok());
        // Un solde supérieur reste conforme.
        acp.set_reserve_fund_balance(Decimal::from(8000)).unwrap();
        assert!(acp.assert_reserve_fund_compliant(charges).is_ok());
    }

    #[test]
    fn edge_reserve_fund_exactly_5pct_ok_below_ko_waived_ok() {
        let charges = Decimal::from(100_000); // requis = 5000
        let mut acp = sample_acp();
        // Exactement 5 % → OK (borne inclusive).
        acp.set_reserve_fund_balance(Decimal::from(5000)).unwrap();
        assert!(acp.is_reserve_fund_compliant(charges));
        // 4 990 (< 5 %) → KO.
        acp.set_reserve_fund_balance(Decimal::from(4990)).unwrap();
        assert!(!acp.is_reserve_fund_compliant(charges));
        // Renonciation 4/5 → conforme même sous le seuil.
        acp.set_reserve_fund_waived(true);
        assert!(acp.is_reserve_fund_compliant(charges));
        assert!(acp.assert_reserve_fund_compliant(charges).is_ok());
    }

    #[test]
    fn security_reserve_fund_threshold_not_bypassable() {
        // Sans renonciation, un solde sous le seuil ne peut être déclaré
        // conforme (pas de contournement silencieux).
        let charges = Decimal::from(200_000); // requis = 10000
        let mut acp = sample_acp();
        acp.set_reserve_fund_balance(Decimal::from(9999)).unwrap();
        assert!(!acp.reserve_fund_waived);
        assert!(!acp.is_reserve_fund_compliant(charges));
        let err = acp.assert_reserve_fund_compliant(charges).unwrap_err();
        assert_eq!(err.required, Decimal::from(10000));
        assert_eq!(err.actual, Decimal::from(9999));
    }

    #[test]
    fn negative_reserve_fund_insufficient_typed_and_negative_balance_rejected() {
        let charges = Decimal::from(100_000);
        let acp = sample_acp(); // réserve = 0
        let err = acp.assert_reserve_fund_compliant(charges).unwrap_err();
        assert_eq!(err.acp_id, acp.id);
        assert_eq!(err.required, Decimal::from(5000));
        assert_eq!(err.actual, Decimal::ZERO);
        assert_eq!(err.ordinary_charges_n1, charges);
        assert!(format!("{}", err).contains("reserve fund insufficient"));
        // Solde négatif rejeté (erreur typée, état inchangé).
        let mut acp2 = sample_acp();
        let e2 = acp2
            .set_reserve_fund_balance(Decimal::from(-1))
            .unwrap_err();
        assert_eq!(e2, AcpError::NegativeFundBalance(Decimal::from(-1)));
        assert_eq!(acp2.reserve_fund_balance, Decimal::ZERO);
    }

    // ----- @edge ---------------------------------------------------------------

    #[test]
    fn edge_minimum_name_length_2_accepted() {
        let acp = Acp::new(
            None,
            "Ab".to_string(),
            "Rue X 1".to_string(),
            "1000".to_string(),
            "Bruxelles".to_string(),
            None,
        );
        assert!(acp.is_ok());
    }

    #[test]
    fn edge_name_is_trimmed_before_validation() {
        let acp = Acp::new(
            None,
            "   Trimmed Acp   ".to_string(),
            "Rue X 1".to_string(),
            "1000".to_string(),
            "Bruxelles".to_string(),
            None,
        )
        .unwrap();
        assert_eq!(acp.name, "Trimmed Acp");
        assert_eq!(acp.slug, "trimmed-acp");
    }

    #[test]
    fn edge_address_fields_are_trimmed() {
        let acp = Acp::new(
            None,
            "Some Name".to_string(),
            "  Rue X 1  ".to_string(),
            "  1000  ".to_string(),
            "  Bruxelles  ".to_string(),
            None,
        )
        .unwrap();
        assert_eq!(acp.address_street, "Rue X 1");
        assert_eq!(acp.address_postal_code, "1000");
        assert_eq!(acp.address_city, "Bruxelles");
    }

    #[test]
    fn edge_legal_status_default_is_copropriete_belge() {
        let acp = Acp::new(
            None,
            "Some Name".to_string(),
            "Rue X 1".to_string(),
            "1000".to_string(),
            "Bruxelles".to_string(),
            None,
        )
        .unwrap();
        assert_eq!(acp.legal_status.as_db_str(), "copropriete_belge");
    }

    #[test]
    fn edge_unknown_legal_status_db_string_decodes_to_default() {
        assert_eq!(
            AcpLegalStatus::from_db_str("totally_unknown_value"),
            AcpLegalStatus::CoproprieteBelge
        );
    }

    // ----- @security ----------------------------------------------------------

    // L'agrégat lui-même ne porte pas la logique RBAC (qui vit dans les use-cases
    // — `acp_use_cases.rs`). Mais on s'assure que les invariants empêchent au
    // moins l'invariant structurel : `organization_id` est explicitement
    // optionnel et NE peut PAS être inféré silencieusement.

    #[test]
    fn security_organization_id_is_required_to_be_explicit() {
        // Compile-time guarantee : la signature impose `Option<Uuid>`,
        // pas de fallback "current org" implicite.
        #[allow(clippy::type_complexity)]
        let _: fn(
            Option<Uuid>,
            String,
            String,
            String,
            String,
            Option<String>,
        ) -> Result<Acp, AcpError> = Acp::new;
    }

    // ----- @negative ----------------------------------------------------------

    #[test]
    fn negative_empty_name_is_rejected() {
        let err = Acp::new(
            None,
            "".to_string(),
            "Rue X 1".to_string(),
            "1000".to_string(),
            "Bruxelles".to_string(),
            None,
        )
        .unwrap_err();
        assert_eq!(err, AcpError::NameEmpty);
    }

    #[test]
    fn negative_whitespace_only_name_is_rejected_as_empty() {
        let err = Acp::new(
            None,
            "    ".to_string(),
            "Rue X 1".to_string(),
            "1000".to_string(),
            "Bruxelles".to_string(),
            None,
        )
        .unwrap_err();
        assert_eq!(err, AcpError::NameEmpty);
    }

    #[test]
    fn negative_single_char_name_is_too_short() {
        let err = Acp::new(
            None,
            "A".to_string(),
            "Rue X 1".to_string(),
            "1000".to_string(),
            "Bruxelles".to_string(),
            None,
        )
        .unwrap_err();
        assert_eq!(err, AcpError::NameTooShort(1));
    }

    #[test]
    fn negative_name_too_long_is_rejected() {
        let long_name = "A".repeat(161);
        let err = Acp::new(
            None,
            long_name,
            "Rue X 1".to_string(),
            "1000".to_string(),
            "Bruxelles".to_string(),
            None,
        )
        .unwrap_err();
        assert_eq!(err, AcpError::NameTooLong(161));
    }

    #[test]
    fn negative_empty_street_is_rejected() {
        let err = Acp::new(
            None,
            "Some Name".to_string(),
            "".to_string(),
            "1000".to_string(),
            "Bruxelles".to_string(),
            None,
        )
        .unwrap_err();
        assert_eq!(err, AcpError::AddressStreetEmpty);
    }

    #[test]
    fn negative_empty_postal_code_is_rejected() {
        let err = Acp::new(
            None,
            "Some Name".to_string(),
            "Rue X 1".to_string(),
            "".to_string(),
            "Bruxelles".to_string(),
            None,
        )
        .unwrap_err();
        assert_eq!(err, AcpError::PostalCodeEmpty);
    }

    #[test]
    fn negative_empty_city_is_rejected() {
        let err = Acp::new(
            None,
            "Some Name".to_string(),
            "Rue X 1".to_string(),
            "1000".to_string(),
            "".to_string(),
            None,
        )
        .unwrap_err();
        assert_eq!(err, AcpError::CityEmpty);
    }

    #[test]
    fn negative_update_info_re_validates_invariants() {
        let mut acp = Acp::new(
            None,
            "Valid".to_string(),
            "Rue X 1".to_string(),
            "1000".to_string(),
            "Bruxelles".to_string(),
            None,
        )
        .unwrap();
        let err = acp
            .update_info(
                "".to_string(),
                "Rue X 1".to_string(),
                "1000".to_string(),
                "Bruxelles".to_string(),
                None,
            )
            .unwrap_err();
        assert_eq!(err, AcpError::NameEmpty);
        // Name unchanged because update failed.
        assert_eq!(acp.name, "Valid");
    }
}
