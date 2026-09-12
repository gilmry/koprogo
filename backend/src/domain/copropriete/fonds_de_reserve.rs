//! L'obligation de constituer un fonds de réserve.
//!
//! Art. 3.86 § 3, alinéa 4 :
//!
//! > « L'association des copropriétaires doit constituer **au plus tard à
//! > l'issue d'une période de cinq ans suivant la date de la réception
//! > provisoire des parties communes** de l'immeuble, un fonds de réserve dont
//! > la **contribution annuelle ne peut être inférieure à cinq pour cent de la
//! > totalité des charges communes ordinaires de l'exercice précédent** ;
//! > l'association des copropriétaires peut décider **à une majorité de quatre
//! > cinquième des voix** de ne pas constituer ce fonds de réserve
//! > obligatoire. »
//!
//! Trois conditions, et il faut les tenir ensemble :
//!
//! 1. une **échéance** — cinq ans après la réception provisoire, pas après la
//!    constitution de l'ACP ni après la première AG ;
//! 2. un **plancher** — 5 % des charges communes **ordinaires** de l'exercice
//!    précédent. Les charges extraordinaires en sont exclues, et les y inclure
//!    gonflerait artificiellement l'obligation ;
//! 3. une **échappatoire** — la renonciation, mais seulement aux quatre
//!    cinquièmes des voix, ce qui n'est pas une majorité qu'on obtient
//!    distraitement.
//!
//! Ce que le domaine portait déjà : le taux (`RESERVE_FUND_RATE`), le solde et
//! le drapeau de renonciation. Ce qui manquait : le calcul, l'échéance, et le
//! fait de savoir dans quel état on se trouve.
//!
//! Voir issue #738 et ADR-0012.

use super::acp::RESERVE_FUND_RATE;
use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;

/// Où en est une ACP vis-à-vis de son obligation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatutFondsReserve {
    /// La réception provisoire n'est pas encodée : on ne peut pas dater
    /// l'échéance, donc ni exiger ni dispenser.
    ///
    /// Ce troisième état est délibéré. Répondre « pas encore exigible »
    /// dispenserait une ACP qui l'est peut-être depuis des années.
    ReceptionInconnue,
    /// Le délai de cinq ans court encore.
    PasEncoreExigible { exigible_le: NaiveDate },
    /// L'assemblée a renoncé aux quatre cinquièmes des voix.
    EcarteParLassemblee,
    /// L'obligation est due, avec son plancher annuel.
    Exigible { plancher_annuel: Decimal },
}

/// Ce que refuse un budget qui sous-dote le fonds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DotationInsuffisante {
    pub prevue: Decimal,
    pub plancher: Decimal,
    pub charges_ordinaires_n_moins_1: Decimal,
}

impl std::fmt::Display for DotationInsuffisante {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Art. 3.86 § 3 : la dotation au fonds de réserve prévue ({}) est inférieure au \
             plancher légal de {} — 5 % des {} de charges ordinaires de l'exercice précédent. \
             Seule une renonciation votée aux 4/5 en dispense.",
            self.prevue, self.plancher, self.charges_ordinaires_n_moins_1
        )
    }
}

/// Le plancher annuel : 5 % des charges communes **ordinaires** de l'exercice
/// précédent.
///
/// Les charges extraordinaires sont hors du calcul. Les y ajouter gonflerait
/// l'obligation d'une ACP qui a fait de gros travaux, alors que l'article vise
/// précisément le train de vie courant.
pub fn plancher_annuel(charges_ordinaires_n_moins_1: Decimal) -> Decimal {
    (charges_ordinaires_n_moins_1 * RESERVE_FUND_RATE).round_dp(2)
}

/// La date à laquelle l'obligation devient exigible.
///
/// « À l'issue d'une période de cinq ans suivant la réception provisoire ».
pub fn exigible_le(reception_provisoire: NaiveDate) -> Option<NaiveDate> {
    reception_provisoire.with_year(reception_provisoire.year() + 5)
}

/// L'état de l'obligation à une date donnée.
pub fn statut(
    reception_provisoire: Option<NaiveDate>,
    aujourdhui: NaiveDate,
    renonciation_votee: bool,
    charges_ordinaires_n_moins_1: Decimal,
) -> StatutFondsReserve {
    if renonciation_votee {
        return StatutFondsReserve::EcarteParLassemblee;
    }
    let Some(reception) = reception_provisoire else {
        return StatutFondsReserve::ReceptionInconnue;
    };
    let Some(echeance) = exigible_le(reception) else {
        return StatutFondsReserve::ReceptionInconnue;
    };
    if aujourdhui < echeance {
        return StatutFondsReserve::PasEncoreExigible {
            exigible_le: echeance,
        };
    }
    StatutFondsReserve::Exigible {
        plancher_annuel: plancher_annuel(charges_ordinaires_n_moins_1),
    }
}

/// Un budget dote-t-il suffisamment le fonds de réserve ?
///
/// Ne refuse que dans l'état `Exigible` : une ACP encore dans ses cinq ans, ou
/// qui a voté la renonciation, peut doter ce qu'elle veut — y compris rien.
pub fn verifier_dotation(
    statut: &StatutFondsReserve,
    dotation_prevue: Decimal,
    charges_ordinaires_n_moins_1: Decimal,
) -> Result<(), DotationInsuffisante> {
    let StatutFondsReserve::Exigible { plancher_annuel } = statut else {
        return Ok(());
    };
    if dotation_prevue < *plancher_annuel {
        return Err(DotationInsuffisante {
            prevue: dotation_prevue,
            plancher: *plancher_annuel,
            charges_ordinaires_n_moins_1,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn le(annee: i32, mois: u32, jour: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(annee, mois, jour).expect("date valide")
    }

    #[test]
    fn happy_le_plancher_vaut_cinq_pourcents_des_charges_ordinaires() {
        assert_eq!(plancher_annuel(dec!(48000)), dec!(2400));
    }

    #[test]
    fn edge_le_plancher_sarrondit_au_centime() {
        // 5 % de 12 345,67 = 617,2835 → 617,28. Pas de dérive flottante
        // (ADR-0007) : c'est une somme opposable à des copropriétaires.
        assert_eq!(plancher_annuel(dec!(12345.67)), dec!(617.28));
    }

    #[test]
    fn happy_lecheance_court_cinq_ans_apres_la_reception_provisoire() {
        assert_eq!(exigible_le(le(2021, 3, 15)), Some(le(2026, 3, 15)));
    }

    #[test]
    fn happy_avant_cinq_ans_lobligation_nest_pas_due() {
        let statut = statut(Some(le(2023, 6, 1)), le(2026, 9, 3), false, dec!(48000));
        assert_eq!(
            statut,
            StatutFondsReserve::PasEncoreExigible {
                exigible_le: le(2028, 6, 1)
            }
        );
    }

    #[test]
    fn happy_passe_cinq_ans_le_plancher_sapplique() {
        let statut = statut(Some(le(2019, 6, 1)), le(2026, 9, 3), false, dec!(48000));
        assert_eq!(
            statut,
            StatutFondsReserve::Exigible {
                plancher_annuel: dec!(2400)
            }
        );
    }

    /// @edge — le jour même de l'échéance, l'obligation est due.
    ///
    /// « À l'issue d'une période de cinq ans » : la période est écoulée ce
    /// jour-là.
    #[test]
    fn edge_le_jour_de_lecheance_lobligation_est_due() {
        let statut = statut(Some(le(2021, 6, 1)), le(2026, 6, 1), false, dec!(48000));
        assert!(matches!(statut, StatutFondsReserve::Exigible { .. }));
    }

    #[test]
    fn happy_la_renonciation_aux_quatre_cinquiemes_dispense() {
        let statut = statut(Some(le(2015, 6, 1)), le(2026, 9, 3), true, dec!(48000));
        assert_eq!(statut, StatutFondsReserve::EcarteParLassemblee);
    }

    /// @security — sans réception provisoire encodée, on ne dispense pas.
    ///
    /// Répondre « pas encore exigible » dispenserait une ACP qui l'est
    /// peut-être depuis des années. Le troisième état est le seul honnête.
    #[test]
    fn security_sans_reception_provisoire_on_ne_dispense_pas() {
        let statut = statut(None, le(2026, 9, 3), false, dec!(48000));
        assert_eq!(statut, StatutFondsReserve::ReceptionInconnue);
        assert!(
            !matches!(statut, StatutFondsReserve::PasEncoreExigible { .. }),
            "l'inconnu ne vaut pas dispense"
        );
    }

    // ── La vérification d'un budget ────────────────────────────────────

    #[test]
    fn negative_un_budget_qui_sous_dote_est_refuse() {
        let statut = StatutFondsReserve::Exigible {
            plancher_annuel: dec!(2400),
        };
        let refus = verifier_dotation(&statut, dec!(1000), dec!(48000)).expect_err("doit refuser");
        assert_eq!(refus.plancher, dec!(2400));
        assert_eq!(refus.prevue, dec!(1000));
        assert!(
            refus.to_string().contains("4/5"),
            "le refus doit dire l'issue"
        );
    }

    #[test]
    fn edge_une_dotation_exactement_au_plancher_passe() {
        let statut = StatutFondsReserve::Exigible {
            plancher_annuel: dec!(2400),
        };
        assert!(verifier_dotation(&statut, dec!(2400), dec!(48000)).is_ok());
    }

    #[test]
    fn happy_avant_lecheance_une_dotation_nulle_est_licite() {
        let statut = StatutFondsReserve::PasEncoreExigible {
            exigible_le: le(2028, 6, 1),
        };
        assert!(verifier_dotation(&statut, Decimal::ZERO, dec!(48000)).is_ok());
    }

    #[test]
    fn happy_apres_renonciation_une_dotation_nulle_est_licite() {
        assert!(verifier_dotation(
            &StatutFondsReserve::EcarteParLassemblee,
            Decimal::ZERO,
            dec!(48000)
        )
        .is_ok());
    }

    /// Les charges extraordinaires ne gonflent pas le plancher.
    ///
    /// Une ACP qui a refait sa toiture ne doit pas voir son obligation de
    /// réserve exploser l'année suivante : l'article vise le train de vie
    /// courant, et c'est à l'appelant de ne passer que l'ordinaire.
    #[test]
    fn happy_le_plancher_ignore_ce_quon_ne_lui_donne_pas() {
        let ordinaire_seul = plancher_annuel(dec!(48000));
        let avec_extraordinaire = plancher_annuel(dec!(48000) + dec!(120000));
        assert_ne!(ordinaire_seul, avec_extraordinaire);
        assert_eq!(ordinaire_seul, dec!(2400));
    }
}
