//! Domain Entity: Fund (fonds de l'ACP — roulement / réserve / affecté)
//!
//! Prolonge le modèle binaire roulement/réserve (ADR-0012, Story H13a,
//! colonnes `acps.reserve_fund_balance` / `working_capital_balance`) par une
//! troisième nature : le fonds **affecté** (thésaurisation pluriannuelle
//! vers un chantier nommé). Voir ADR-0054, issue #635.
//!
//! Les trois natures restent distinctes et sur des comptes (soldes)
//! séparés — chaque `Fund` porte le sien :
//! - `WorkingCapital` (roulement) — dépenses périodiques.
//! - `Reserve` (réserve légale, Art. 3.86 §3 CC) — reste gouverné par
//!   `Acp::assert_reserve_fund_compliant` (ADR-0012) ; ce module ne
//!   réimplémente PAS cette obligation, il évite seulement de la confondre
//!   avec un fonds affecté facultatif.
//! - `Earmarked` (fonds affecté) — épargne vers un objet précis
//!   (`purpose`), votée à la majorité des 2/3 (Art. 3.88, gros travaux).
//!
//! **L'affectation n'est pas un verrou immuable** (décision PO @gilmry,
//! reprise ADR-0054) : une AG peut réaffecter un fonds affecté vers une
//! autre fin (`Fund::reassign`), à condition qu'une résolution ADOPTÉE le
//! décide. L'audit trail (`FundReassignment`) trace l'affectation d'origine.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::copropriete::resolution::{MajorityType, ResolutionStatus};

/// Nature d'un fonds — Art. 3.86 §3 CC (roulement/réserve) + thésaurisation
/// facultative (fonds affecté, pratique courante non prévue par la loi).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FundKind {
    /// Fonds de roulement — dépenses périodiques. Compte distinct.
    WorkingCapital,
    /// Fonds de réserve légal (Art. 3.86 §3, ADR-0012) — reste gouverné par
    /// `Acp::assert_reserve_fund_compliant`. Compte distinct.
    Reserve,
    /// Fonds affecté (thésaurisation) — épargne vers un chantier nommé
    /// (`purpose`), voté aux 2/3 (Art. 3.88).
    Earmarked,
}

/// Erreur typée du domaine `Fund` — jamais de `Result<_, String>`
/// (CRITICAL.md #4).
#[derive(Debug, Clone, PartialEq)]
pub enum FundError {
    /// Nom de fonds vide.
    EmptyName,
    /// Un fonds affecté doit porter un objet précis.
    EarmarkedRequiresPurpose,
    /// Un fonds affecté doit porter un objectif d'épargne strictement positif.
    EarmarkedRequiresPositiveTarget,
    /// Roulement/réserve ne portent pas d'objet (réservé aux fonds affectés).
    NonEarmarkedCannotHavePurpose,
    /// Roulement/réserve ne portent pas d'objectif d'épargne.
    NonEarmarkedCannotHaveTarget,
    /// Versement au fonds non strictement positif.
    NonPositiveContribution,
    /// Dépense imputée à un fonds affecté pour un autre objet que le sien.
    ExpensePurposeMismatch {
        fund_purpose: String,
        expense_work_ref: String,
    },
    /// La majorité obtenue pour créer un fonds affecté est sous le seuil des
    /// gros travaux (Art. 3.88, 2/3).
    InsufficientMajorityForCreation {
        required: MajorityType,
        used: MajorityType,
    },
    /// Seul un fonds affecté peut être réaffecté.
    OnlyEarmarkedFundsCanBeReassigned,
    /// La réaffectation exige une décision d'AG adoptée.
    ReassignmentRequiresAdoptedResolution,
    /// Montant réaffecté non strictement positif.
    NonPositiveReassignmentAmount,
    /// Montant réaffecté supérieur au solde disponible.
    ReassignmentAmountExceedsBalance,
}

impl std::fmt::Display for FundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyName => write!(f, "Le nom du fonds ne peut pas être vide"),
            Self::EarmarkedRequiresPurpose => write!(
                f,
                "Un fonds affecté doit porter un objet précis (travail ciblé)"
            ),
            Self::EarmarkedRequiresPositiveTarget => write!(
                f,
                "Un fonds affecté doit porter un objectif d'épargne strictement positif"
            ),
            Self::NonEarmarkedCannotHavePurpose => write!(
                f,
                "Seul un fonds affecté peut porter un objet précis \
                 (roulement/réserve n'en portent pas)"
            ),
            Self::NonEarmarkedCannotHaveTarget => write!(
                f,
                "Seul un fonds affecté peut porter un objectif d'épargne \
                 (roulement/réserve n'en portent pas)"
            ),
            Self::NonPositiveContribution => {
                write!(f, "Un versement au fonds doit être strictement positif")
            }
            Self::ExpensePurposeMismatch {
                fund_purpose,
                expense_work_ref,
            } => write!(
                f,
                "Dépense refusée : le fonds est affecté à « {fund_purpose} », \
                 pas à « {expense_work_ref} »"
            ),
            Self::InsufficientMajorityForCreation { required, used } => write!(
                f,
                "La création d'un fonds affecté exige la majorité {required:?} \
                 (Art. 3.88, gros travaux) ; majorité obtenue : {used:?}"
            ),
            Self::OnlyEarmarkedFundsCanBeReassigned => write!(
                f,
                "Seul un fonds affecté peut être réaffecté (le fonds de réserve \
                 légal reste gouverné par sa propre règle de renonciation 4/5)"
            ),
            Self::ReassignmentRequiresAdoptedResolution => write!(
                f,
                "La réaffectation d'un fonds exige une décision d'AG adoptée"
            ),
            Self::NonPositiveReassignmentAmount => {
                write!(f, "Le montant réaffecté doit être strictement positif")
            }
            Self::ReassignmentAmountExceedsBalance => {
                write!(f, "Le montant réaffecté dépasse le solde du fonds")
            }
        }
    }
}

impl std::error::Error for FundError {}

/// Fonds de l'ACP — roulement, réserve légale, ou affecté (thésaurisation).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct Fund {
    pub id: Uuid,
    pub acp_id: Uuid,
    pub kind: FundKind,
    pub name: String,
    /// Objet précis du chantier — uniquement pour `FundKind::Earmarked`.
    pub purpose: Option<String>,
    /// Objectif d'épargne — uniquement pour `FundKind::Earmarked`.
    pub target_amount: Option<Decimal>,
    pub balance: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Audit trail d'une réaffectation de fonds — traçabilité comptable et
/// légale de l'affectation d'origine vers la nouvelle fin (DoD #635).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct FundReassignment {
    pub id: Uuid,
    pub fund_id: Uuid,
    pub previous_purpose: String,
    pub new_purpose: String,
    pub amount: Decimal,
    /// La décision d'AG qui autorise la réaffectation (Art. 3.88).
    pub resolution_id: Uuid,
    pub reassigned_at: DateTime<Utc>,
}

impl Fund {
    /// Crée un fonds de roulement ou de réserve (pas d'objet, pas
    /// d'objectif). Pour un fonds affecté, voir [`Fund::new_earmarked`].
    pub fn new(
        acp_id: Uuid,
        kind: FundKind,
        name: String,
        purpose: Option<String>,
        target_amount: Option<Decimal>,
    ) -> Result<Self, FundError> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(FundError::EmptyName);
        }

        match kind {
            FundKind::Earmarked => {
                let purpose_trimmed = purpose.as_deref().unwrap_or("").trim().to_string();
                if purpose_trimmed.is_empty() {
                    return Err(FundError::EarmarkedRequiresPurpose);
                }
                match target_amount {
                    Some(t) if t > Decimal::ZERO => {}
                    _ => return Err(FundError::EarmarkedRequiresPositiveTarget),
                }
            }
            FundKind::WorkingCapital | FundKind::Reserve => {
                if purpose.is_some() {
                    return Err(FundError::NonEarmarkedCannotHavePurpose);
                }
                if target_amount.is_some() {
                    return Err(FundError::NonEarmarkedCannotHaveTarget);
                }
            }
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            acp_id,
            kind,
            name,
            purpose,
            target_amount,
            balance: Decimal::ZERO,
            created_at: now,
            updated_at: now,
        })
    }

    /// Crée un fonds affecté, en exigeant la majorité des gros travaux (2/3,
    /// Art. 3.88 §1, 1°, b).
    ///
    /// @security (story #635) : créer un fonds fléché à la majorité simple
    /// contournerait la majorité qui protège les copropriétaires du
    /// chantier lui-même.
    pub fn new_earmarked(
        acp_id: Uuid,
        name: String,
        purpose: String,
        target_amount: Decimal,
        majority_used: MajorityType,
    ) -> Result<Self, FundError> {
        if !majority_used.satisfait(&MajorityType::TwoThirds) {
            return Err(FundError::InsufficientMajorityForCreation {
                required: MajorityType::TwoThirds,
                used: majority_used,
            });
        }
        Self::new(
            acp_id,
            FundKind::Earmarked,
            name,
            Some(purpose),
            Some(target_amount),
        )
    }

    /// Alimente le fonds. Le solde ne se confond avec aucun autre fonds :
    /// chaque `Fund` porte le sien.
    pub fn contribute(&mut self, amount: Decimal) -> Result<(), FundError> {
        if amount <= Decimal::ZERO {
            return Err(FundError::NonPositiveContribution);
        }
        self.balance += amount;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Progression de l'épargne vers l'objectif (`balance / target_amount`).
    /// `None` pour un fonds sans objectif (roulement/réserve) ou à objectif
    /// nul.
    pub fn progress(&self) -> Option<Decimal> {
        self.target_amount
            .filter(|t| *t > Decimal::ZERO)
            .map(|t| self.balance / t)
    }

    /// Le reliquat : ce que le fonds a accumulé au-delà de son objectif, une
    /// fois celui-ci atteint (chantier terminé sous budget).
    ///
    /// `None` tant que l'objectif n'est pas dépassé, ou pour un fonds sans
    /// objectif (roulement/réserve).
    pub fn reliquat(&self) -> Option<Decimal> {
        let target = self.target_amount?;
        (self.balance > target).then_some(self.balance - target)
    }

    /// Autorise (ou refuse) une dépense imputée à ce fonds pour l'objet
    /// `expense_work_ref`.
    ///
    /// Seul un fonds affecté restreint ainsi ses dépenses — roulement et
    /// réserve n'ont pas d'objet unique à comparer. `ag_override` lève la
    /// restriction quand il vaut `Some(ResolutionStatus::Adopted)` : « ne
    /// jamais bloquer une dépense au seul motif que le fonds était fléché
    /// ailleurs, dès lors qu'une décision d'AG l'autorise » (PO @gilmry).
    pub fn assert_expense_matches_purpose(
        &self,
        expense_work_ref: &str,
        ag_override: Option<ResolutionStatus>,
    ) -> Result<(), FundError> {
        if ag_override == Some(ResolutionStatus::Adopted) {
            return Ok(());
        }
        if self.kind != FundKind::Earmarked {
            return Ok(());
        }
        match &self.purpose {
            Some(purpose) if purpose == expense_work_ref => Ok(()),
            Some(purpose) => Err(FundError::ExpensePurposeMismatch {
                fund_purpose: purpose.clone(),
                expense_work_ref: expense_work_ref.to_string(),
            }),
            // Invariant : un fonds Earmarked porte toujours un purpose
            // (garanti par `Fund::new`).
            None => Ok(()),
        }
    }

    /// Réaffecte tout ou partie du solde de ce fonds affecté vers une autre
    /// fin, sur décision d'AG adoptée. Retourne l'audit trail
    /// (`FundReassignment`) tracant l'affectation d'origine.
    ///
    /// `amount = None` réaffecte la TOTALITÉ du solde disponible : le fonds
    /// sert alors intégralement la nouvelle fin (`purpose` change).
    /// `amount = Some(x)` avec `x` strictement inférieur au solde ne
    /// réaffecte qu'une partie : le reliquat continue de servir l'objet
    /// d'origine. Si `x` égale le solde entier, le résultat est le même que
    /// la totalité.
    pub fn reassign(
        &mut self,
        new_purpose: String,
        amount: Option<Decimal>,
        resolution_id: Uuid,
        resolution_status: ResolutionStatus,
    ) -> Result<FundReassignment, FundError> {
        if self.kind != FundKind::Earmarked {
            return Err(FundError::OnlyEarmarkedFundsCanBeReassigned);
        }
        if resolution_status != ResolutionStatus::Adopted {
            return Err(FundError::ReassignmentRequiresAdoptedResolution);
        }
        let moved = amount.unwrap_or(self.balance);
        if moved <= Decimal::ZERO {
            return Err(FundError::NonPositiveReassignmentAmount);
        }
        if moved > self.balance {
            return Err(FundError::ReassignmentAmountExceedsBalance);
        }

        let previous_purpose = self.purpose.clone().unwrap_or_default();
        self.balance -= moved;

        // Plus rien du solde d'origine : le fonds sert désormais
        // intégralement la nouvelle fin. Un reliquat partiel garde l'objet
        // d'origine pour ce qui reste.
        if self.balance == Decimal::ZERO {
            self.purpose = Some(new_purpose.clone());
        }
        self.updated_at = Utc::now();

        Ok(FundReassignment {
            id: Uuid::new_v4(),
            fund_id: self.id,
            previous_purpose,
            new_purpose,
            amount: moved,
            resolution_id,
            reassigned_at: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn earmarked(name: &str, purpose: &str, target: Decimal) -> Fund {
        Fund::new_earmarked(
            Uuid::new_v4(),
            name.to_string(),
            purpose.to_string(),
            target,
            MajorityType::TwoThirds,
        )
        .expect("fonds affecté valide")
    }

    // ── @happy ──────────────────────────────────────────────────────────

    /// Gherkin de la story #635 : un fonds affecté voté en AG progresse vers
    /// son objectif et reste distinct des deux autres fonds.
    #[test]
    fn happy_fonds_affecte_progresse_et_reste_distinct() {
        let acp_id = Uuid::new_v4();
        let mut toiture = Fund::new_earmarked(
            acp_id,
            "Réfection toiture".to_string(),
            "Réfection toiture".to_string(),
            dec!(50000),
            MajorityType::TwoThirds,
        )
        .expect("vote valide");
        let mut roulement = Fund::new(
            acp_id,
            FundKind::WorkingCapital,
            "Roulement".to_string(),
            None,
            None,
        )
        .unwrap();
        let mut reserve = Fund::new(
            acp_id,
            FundKind::Reserve,
            "Réserve légale".to_string(),
            None,
            None,
        )
        .unwrap();

        toiture.contribute(dec!(12500)).unwrap();
        roulement.contribute(dec!(3000)).unwrap();
        reserve.contribute(dec!(7000)).unwrap();

        assert_eq!(toiture.balance, dec!(12500));
        assert_eq!(toiture.progress(), Some(dec!(0.25)));
        // Les soldes ne se confondent avec aucun autre fonds.
        assert_eq!(roulement.balance, dec!(3000));
        assert_eq!(reserve.balance, dec!(7000));
        assert_ne!(toiture.balance, roulement.balance);
        assert_ne!(toiture.balance, reserve.balance);
    }

    /// AC checklist #635 — reliquat une fois l'objectif dépassé.
    #[test]
    fn happy_reliquat_apres_objectif_atteint() {
        let mut fonds = earmarked("Façade", "Ravalement façade", dec!(10000));
        fonds.contribute(dec!(11500)).unwrap();
        assert_eq!(fonds.reliquat(), Some(dec!(1500)));
    }

    /// AC checklist #635 — réaffectation par décision d'AG adoptée, avec
    /// audit trail.
    #[test]
    fn happy_reassignation_par_decision_ag_adoptee() {
        let mut fonds = earmarked("Ascenseur", "Remplacement ascenseur", dec!(20000));
        fonds.contribute(dec!(20000)).unwrap();
        let resolution_id = Uuid::new_v4();

        let audit = fonds
            .reassign(
                "Ravalement façade".to_string(),
                None,
                resolution_id,
                ResolutionStatus::Adopted,
            )
            .expect("réaffectation autorisée par l'AG");

        assert_eq!(audit.fund_id, fonds.id);
        assert_eq!(audit.previous_purpose, "Remplacement ascenseur");
        assert_eq!(audit.new_purpose, "Ravalement façade");
        assert_eq!(audit.amount, dec!(20000));
        assert_eq!(audit.resolution_id, resolution_id);
        assert_eq!(fonds.purpose, Some("Ravalement façade".to_string()));
        assert_eq!(fonds.balance, Decimal::ZERO);
    }

    // ── @negative ───────────────────────────────────────────────────────

    /// Gherkin @negative de la story #635 : une dépense imputée à un fonds
    /// affecté pour un autre objet que le sien est refusée.
    #[test]
    fn negative_depense_hors_objet_est_refusee() {
        let fonds = earmarked("Toiture", "Réfection toiture", dec!(50000));
        let erreur = fonds
            .assert_expense_matches_purpose("Ravalement façade", None)
            .expect_err("doit refuser");
        assert_eq!(
            erreur,
            FundError::ExpensePurposeMismatch {
                fund_purpose: "Réfection toiture".to_string(),
                expense_work_ref: "Ravalement façade".to_string(),
            }
        );
    }

    #[test]
    fn negative_reassignation_sans_resolution_adoptee_refusee() {
        let mut fonds = earmarked("Toiture", "Réfection toiture", dec!(50000));
        fonds.contribute(dec!(1000)).unwrap();

        for statut_non_adopte in [ResolutionStatus::Pending, ResolutionStatus::Rejected] {
            let erreur = fonds
                .clone()
                .reassign(
                    "Autre objet".to_string(),
                    None,
                    Uuid::new_v4(),
                    statut_non_adopte,
                )
                .expect_err("doit refuser sans décision adoptée");
            assert_eq!(erreur, FundError::ReassignmentRequiresAdoptedResolution);
        }
    }

    #[test]
    fn negative_contribution_non_positive_refusee() {
        let mut fonds = Fund::new(
            Uuid::new_v4(),
            FundKind::WorkingCapital,
            "Roulement".to_string(),
            None,
            None,
        )
        .unwrap();
        assert_eq!(
            fonds.contribute(Decimal::ZERO).unwrap_err(),
            FundError::NonPositiveContribution
        );
        assert_eq!(
            fonds.contribute(dec!(-1)).unwrap_err(),
            FundError::NonPositiveContribution
        );
    }

    // ── @edge ───────────────────────────────────────────────────────────

    /// Gherkin @edge de la story #635 : le fonds de réserve légal reste
    /// distinct, et l'obligation des 5 % (portée par ADR-0012, hors de ce
    /// module) n'est pas diluée dans le modèle facultatif — un `Fund::Reserve`
    /// ne peut pas être créé avec un objet/objectif de thésaurisation.
    #[test]
    fn edge_fonds_de_reserve_legal_reste_distinct_et_sans_objet() {
        let reserve = Fund::new(
            Uuid::new_v4(),
            FundKind::Reserve,
            "Réserve légale".to_string(),
            None,
            None,
        )
        .expect("fonds de réserve valide, sans vote qualifié requis ici");
        assert_eq!(reserve.kind, FundKind::Reserve);
        assert_eq!(reserve.purpose, None);
        assert_eq!(reserve.target_amount, None);

        let refuse_avec_objet = Fund::new(
            Uuid::new_v4(),
            FundKind::Reserve,
            "Réserve légale".to_string(),
            Some("Chantier".to_string()),
            None,
        )
        .expect_err("la réserve légale ne porte pas d'objet de thésaurisation");
        assert_eq!(refuse_avec_objet, FundError::NonEarmarkedCannotHavePurpose);
    }

    /// Une réaffectation partielle laisse un reliquat sous l'ancien objet.
    #[test]
    fn edge_reassignation_partielle_laisse_un_reliquat_sous_lancien_objet() {
        let mut fonds = earmarked("Toiture", "Réfection toiture", dec!(50000));
        fonds.contribute(dec!(30000)).unwrap();

        let audit = fonds
            .reassign(
                "Ravalement façade".to_string(),
                Some(dec!(10000)),
                Uuid::new_v4(),
                ResolutionStatus::Adopted,
            )
            .expect("réaffectation partielle autorisée");

        assert_eq!(audit.amount, dec!(10000));
        assert_eq!(fonds.balance, dec!(20000));
        // Le reliquat sert toujours l'objet d'origine.
        assert_eq!(fonds.purpose, Some("Réfection toiture".to_string()));
    }

    #[test]
    fn edge_reliquat_absent_avant_objectif_atteint() {
        let mut fonds = earmarked("Toiture", "Réfection toiture", dec!(50000));
        fonds.contribute(dec!(10000)).unwrap();
        assert_eq!(fonds.reliquat(), None);
    }

    // ── @security ───────────────────────────────────────────────────────

    /// Gherkin @security de la story #635 : la majorité exigée pour créer un
    /// fonds affecté est celle des gros travaux (2/3, Art. 3.88).
    #[test]
    fn security_creation_fonds_affecte_exige_majorite_des_deux_tiers() {
        let refus = Fund::new_earmarked(
            Uuid::new_v4(),
            "Toiture".to_string(),
            "Réfection toiture".to_string(),
            dec!(50000),
            MajorityType::Absolute,
        )
        .expect_err("la majorité simple ne suffit pas");
        assert_eq!(
            refus,
            FundError::InsufficientMajorityForCreation {
                required: MajorityType::TwoThirds,
                used: MajorityType::Absolute,
            }
        );

        for majorite_suffisante in [
            MajorityType::TwoThirds,
            MajorityType::FourFifths,
            MajorityType::Unanimity,
        ] {
            assert!(Fund::new_earmarked(
                Uuid::new_v4(),
                "Toiture".to_string(),
                "Réfection toiture".to_string(),
                dec!(50000),
                majorite_suffisante,
            )
            .is_ok());
        }
    }

    /// Une décision d'AG adoptée autorise une dépense hors objet — le
    /// fléchage par défaut ne doit jamais bloquer une décision d'AG.
    #[test]
    fn security_ag_override_autorise_une_depense_hors_objet() {
        let fonds = earmarked("Toiture", "Réfection toiture", dec!(50000));
        let resultat = fonds
            .assert_expense_matches_purpose("Ravalement façade", Some(ResolutionStatus::Adopted));
        assert!(resultat.is_ok());

        // Une résolution simplement en attente ne lève pas la restriction.
        let refuse = fonds
            .assert_expense_matches_purpose("Ravalement façade", Some(ResolutionStatus::Pending))
            .expect_err("doit refuser sans décision adoptée");
        assert!(matches!(refuse, FundError::ExpensePurposeMismatch { .. }));
    }

    /// Seul un fonds affecté peut être réaffecté : la réserve légale garde
    /// sa propre procédure de renonciation (4/5, ADR-0012), qu'une
    /// réaffectation générique contournerait sinon.
    #[test]
    fn security_seul_un_fonds_affecte_peut_etre_reassigne() {
        let mut roulement = Fund::new(
            Uuid::new_v4(),
            FundKind::WorkingCapital,
            "Roulement".to_string(),
            None,
            None,
        )
        .unwrap();
        let erreur = roulement
            .reassign(
                "Autre objet".to_string(),
                None,
                Uuid::new_v4(),
                ResolutionStatus::Adopted,
            )
            .expect_err("le roulement n'est pas réaffectable");
        assert_eq!(erreur, FundError::OnlyEarmarkedFundsCanBeReassigned);
    }
}
