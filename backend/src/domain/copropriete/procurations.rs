//! Les plafonds de procuration à l'assemblée générale.
//!
//! Art. 3.87 § 7 énonce trois règles distinctes, souvent confondues :
//!
//! > « Nul ne peut prendre part au vote, même comme mandant ou mandataire,
//! > pour un nombre de voix **supérieur à la somme des voix dont disposent les
//! > autres copropriétaires** présents ou représentés. »
//!
//! > « Nul ne peut accepter **plus de trois procurations** de vote. Toutefois,
//! > un mandataire peut recevoir plus de trois procurations de vote si le total
//! > des voix dont il dispose lui-même et de celles de ses mandants **n'excède
//! > pas 10 %** du total des voix affectées à l'ensemble des lots de la
//! > copropriété. »
//!
//! > « Le syndic ne peut intervenir comme mandataire d'un copropriétaire à
//! > l'assemblée générale, nonobstant le droit pour lui, s'il est
//! > copropriétaire, de participer à ce titre aux délibérations. »
//!
//! Ces règles ne portent pas sur un vote isolé mais sur **l'ensemble des voix
//! exprimées** : on ne peut pas les vérifier en construisant un `Vote`, il
//! faut regarder la séance entière. D'où un service de domaine plutôt qu'une
//! validation d'entité.
//!
//! Elles ne sont pas décoratives. Une assemblée tenue en violation de l'une
//! d'elles est attaquable, et ce sont ses décisions — donc des travaux, des
//! budgets, des mandats — qui tombent avec elle.
//!
//! Voir issue #742.

use super::vote::Vote;
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

/// Ce que la loi refuse, avec de quoi le dire à celui qui préside.
#[derive(Debug, Clone, PartialEq)]
pub enum ProcurationRefusee {
    /// Plus de trois procurations, sans bénéficier de l'exception des 10 %.
    TropDeProcurations {
        mandataire: Uuid,
        recues: usize,
        part_des_voix: Decimal,
    },
    /// Un votant pèse plus que tous les autres réunis.
    PoidsSuperieurAuReste {
        votant: Uuid,
        voix: Decimal,
        voix_des_autres: Decimal,
    },
    /// Le syndic a voté comme mandataire d'un copropriétaire.
    SyndicMandataire { mandats: usize },
}

impl std::fmt::Display for ProcurationRefusee {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TropDeProcurations {
                mandataire,
                recues,
                part_des_voix,
            } => write!(
                f,
                "Art. 3.87 § 7 : le mandataire {mandataire} détient {recues} procurations \
                 et pèse {part_des_voix} % des voix. Au-delà de trois procurations, \
                 l'exception ne joue que sous 10 %."
            ),
            Self::PoidsSuperieurAuReste {
                votant,
                voix,
                voix_des_autres,
            } => write!(
                f,
                "Art. 3.87 § 7 : le votant {votant} pèse {voix} voix contre {voix_des_autres} \
                 pour tous les autres présents ou représentés réunis."
            ),
            Self::SyndicMandataire { mandats } => write!(
                f,
                "Art. 3.87 § 7 : le syndic ne peut être mandataire d'un copropriétaire \
                 ({mandats} mandat(s) détenu(s))."
            ),
        }
    }
}

/// Le plafond de procurations, hors exception.
const PROCURATIONS_MAX: usize = 3;

/// Le seuil de l'exception, en pourcentage du total des lots.
fn seuil_exception(total_des_lots: Decimal) -> Decimal {
    total_des_lots * Decimal::from(10) / Decimal::from(100)
}

/// Les voix rassemblées par chaque personne qui prend part au vote.
///
/// Un mandataire pèse ses propres voix **et** celles de ses mandants : c'est
/// bien « le total des voix dont il dispose lui-même et de celles de ses
/// mandants » que la loi vise.
fn voix_par_votant(votes: &[Vote]) -> HashMap<Uuid, Decimal> {
    let mut par_votant: HashMap<Uuid, Decimal> = HashMap::new();
    for vote in votes {
        *par_votant
            .entry(vote.effective_voter_id())
            .or_insert(Decimal::ZERO) += vote.voting_power;
    }
    par_votant
}

/// Le nombre de procurations acceptées par chaque mandataire.
fn procurations_par_mandataire(votes: &[Vote]) -> HashMap<Uuid, usize> {
    let mut par_mandataire: HashMap<Uuid, usize> = HashMap::new();
    for vote in votes.iter().filter(|v| v.is_proxy_vote()) {
        if let Some(mandataire) = vote.proxy_owner_id {
            *par_mandataire.entry(mandataire).or_insert(0) += 1;
        }
    }
    par_mandataire
}

/// Vérifie les trois plafonds de l'Art. 3.87 § 7 sur une séance.
///
/// `total_des_lots` est le dénominateur de l'acte de base (Art. 3.85 § 1er),
/// pas la somme des voix présentes : l'exception des 10 % se calcule sur
/// « l'ensemble des lots de la copropriété », absents compris.
///
/// `syndic_owner_id` est l'identifiant du syndic **s'il est aussi
/// copropriétaire**. Il peut voter pour son propre lot ; il ne peut pas porter
/// celui d'un autre.
pub fn verifier_procurations(
    votes: &[Vote],
    total_des_lots: Decimal,
    syndic_owner_id: Option<Uuid>,
) -> Result<(), ProcurationRefusee> {
    let voix = voix_par_votant(votes);
    let seuil = seuil_exception(total_des_lots);

    // 1. Le syndic ne porte pas le lot d'un autre.
    if let Some(syndic) = syndic_owner_id {
        let mandats = votes
            .iter()
            .filter(|v| v.proxy_owner_id == Some(syndic) && v.owner_id != syndic)
            .count();
        if mandats > 0 {
            return Err(ProcurationRefusee::SyndicMandataire { mandats });
        }
    }

    // 2. Trois procurations, sauf à peser moins de 10 % du total des lots.
    for (mandataire, recues) in procurations_par_mandataire(votes) {
        if recues <= PROCURATIONS_MAX {
            continue;
        }
        let poids = voix.get(&mandataire).copied().unwrap_or(Decimal::ZERO);
        if poids > seuil {
            let part = if total_des_lots.is_zero() {
                Decimal::ZERO
            } else {
                poids * Decimal::from(100) / total_des_lots
            };
            return Err(ProcurationRefusee::TropDeProcurations {
                mandataire,
                recues,
                part_des_voix: part.round_dp(2),
            });
        }
    }

    // 3. Nul ne pèse plus que tous les autres réunis.
    let total_exprime: Decimal = voix.values().copied().sum();
    for (votant, poids) in &voix {
        let reste = total_exprime - poids;
        if *poids > reste {
            return Err(ProcurationRefusee::PoidsSuperieurAuReste {
                votant: *votant,
                voix: *poids,
                voix_des_autres: reste,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::copropriete::vote::VoteChoice;
    use rust_decimal_macros::dec;

    /// Mille millièmes, la convention la plus courante (Art. 3.85 § 1er).
    const TOTAL_DES_LOTS: Decimal = dec!(1000);

    fn vote(proprietaire: Uuid, voix: Decimal, mandataire: Option<Uuid>) -> Vote {
        Vote::new(
            Uuid::new_v4(),
            proprietaire,
            Uuid::new_v4(),
            VoteChoice::Pour,
            voix,
            mandataire,
        )
        .expect("vote valide")
    }

    // ── Le syndic n'est pas mandataire (Art. 3.87 § 7, dernier alinéa) ──

    #[test]
    fn security_le_syndic_ne_porte_pas_le_lot_dun_autre() {
        let syndic = Uuid::new_v4();
        let votes = vec![
            vote(Uuid::new_v4(), dec!(100), Some(syndic)),
            vote(Uuid::new_v4(), dec!(100), None),
        ];

        let refus =
            verifier_procurations(&votes, TOTAL_DES_LOTS, Some(syndic)).expect_err("doit refuser");
        assert_eq!(refus, ProcurationRefusee::SyndicMandataire { mandats: 1 });
    }

    #[test]
    fn happy_le_syndic_copropriétaire_vote_pour_son_propre_lot() {
        let syndic = Uuid::new_v4();
        // Il vote en son nom propre, sans procuration : `Vote::new` refuse
        // d'ailleurs qu'on soit son propre mandataire, et c'est juste — se
        // donner procuration à soi-même n'a pas de sens. La loi le prévoit
        // expressément : « nonobstant le droit pour lui, s'il est
        // copropriétaire, de participer à ce titre aux délibérations ».
        let votes = vec![
            vote(syndic, dec!(100), None),
            vote(Uuid::new_v4(), dec!(120), None),
            vote(Uuid::new_v4(), dec!(110), None),
        ];

        assert!(verifier_procurations(&votes, TOTAL_DES_LOTS, Some(syndic)).is_ok());
    }

    // ── Le plafond de trois procurations et son exception ──────────────

    #[test]
    fn happy_trois_procurations_passent_quel_que_soit_le_poids() {
        let mandataire = Uuid::new_v4();
        let mut votes: Vec<Vote> = (0..3)
            .map(|_| vote(Uuid::new_v4(), dec!(150), Some(mandataire)))
            .collect();
        // 450/1000 = 45 %, bien au-dessus des 10 %, et pourtant licite :
        // l'exception ne sert qu'À PARTIR de la quatrième procuration.
        //
        // Le reste est réparti sur trois votants plutôt que concentré sur un
        // seul : un copropriétaire pesant 550 sur 1000 tomberait sous la
        // troisième règle, celle du poids. Elle est plus mordante qu'il n'y
        // paraît — un majoritaire ne peut jamais emporter un vote seul.
        votes.push(vote(Uuid::new_v4(), dec!(200), None));
        votes.push(vote(Uuid::new_v4(), dec!(200), None));
        votes.push(vote(Uuid::new_v4(), dec!(150), None));

        assert!(verifier_procurations(&votes, TOTAL_DES_LOTS, None).is_ok());
    }

    #[test]
    fn negative_quatre_procurations_au_dessus_de_dix_pourcents_sont_refusees() {
        let mandataire = Uuid::new_v4();
        let mut votes: Vec<Vote> = (0..4)
            .map(|_| vote(Uuid::new_v4(), dec!(50), Some(mandataire)))
            .collect();
        votes.push(vote(Uuid::new_v4(), dec!(800), None));

        // 200/1000 = 20 % > 10 %.
        let refus = verifier_procurations(&votes, TOTAL_DES_LOTS, None).expect_err("doit refuser");
        match refus {
            ProcurationRefusee::TropDeProcurations {
                recues,
                part_des_voix,
                ..
            } => {
                assert_eq!(recues, 4);
                assert_eq!(part_des_voix, dec!(20.00));
            }
            autre => panic!("mauvais refus : {autre}"),
        }
    }

    #[test]
    fn happy_dix_procurations_sous_dix_pourcents_passent() {
        let mandataire = Uuid::new_v4();
        let mut votes: Vec<Vote> = (0..10)
            .map(|_| vote(Uuid::new_v4(), dec!(9), Some(mandataire)))
            .collect();
        // 90/1000 = 9 % : l'exception joue, malgré dix procurations.
        votes.push(vote(Uuid::new_v4(), dec!(310), None));
        votes.push(vote(Uuid::new_v4(), dec!(300), None));
        votes.push(vote(Uuid::new_v4(), dec!(300), None));

        assert!(verifier_procurations(&votes, TOTAL_DES_LOTS, None).is_ok());
    }

    #[test]
    fn edge_exactement_dix_pourcents_passe() {
        let mandataire = Uuid::new_v4();
        let mut votes: Vec<Vote> = (0..4)
            .map(|_| vote(Uuid::new_v4(), dec!(25), Some(mandataire)))
            .collect();
        // 100/1000 = 10 % PILE. La loi écrit « n'excède pas 10 % » : la borne
        // est inclusive, contrairement au quorum des trois quarts (Art. 3.87
        // § 5) où elle ne l'est pas.
        votes.push(vote(Uuid::new_v4(), dec!(300), None));
        votes.push(vote(Uuid::new_v4(), dec!(300), None));
        votes.push(vote(Uuid::new_v4(), dec!(300), None));

        assert!(verifier_procurations(&votes, TOTAL_DES_LOTS, None).is_ok());
    }

    // ── Nul ne pèse plus que tous les autres réunis ────────────────────

    #[test]
    fn negative_un_votant_ne_depasse_pas_la_somme_des_autres() {
        let dominant = Uuid::new_v4();
        let votes = vec![
            vote(dominant, dec!(600), None),
            vote(Uuid::new_v4(), dec!(200), None),
            vote(Uuid::new_v4(), dec!(199), None),
        ];

        let refus = verifier_procurations(&votes, TOTAL_DES_LOTS, None).expect_err("doit refuser");
        match refus {
            ProcurationRefusee::PoidsSuperieurAuReste {
                voix,
                voix_des_autres,
                ..
            } => {
                assert_eq!(voix, dec!(600));
                assert_eq!(voix_des_autres, dec!(399));
            }
            autre => panic!("mauvais refus : {autre}"),
        }
    }

    #[test]
    fn edge_une_egalite_parfaite_passe() {
        let votes = vec![
            vote(Uuid::new_v4(), dec!(500), None),
            vote(Uuid::new_v4(), dec!(300), None),
            vote(Uuid::new_v4(), dec!(200), None),
        ];
        // 500 contre 500 : « supérieur à » exclut l'égalité.
        assert!(verifier_procurations(&votes, TOTAL_DES_LOTS, None).is_ok());
    }

    #[test]
    fn security_un_mandataire_ne_contourne_pas_le_plafond_en_cumulant() {
        // Trois procurations seulement — le plafond des procurations passe —
        // mais le mandataire rassemble la majorité absolue des voix exprimées.
        // C'est la seconde règle qui l'arrête, et elle seule.
        let mandataire = Uuid::new_v4();
        let mut votes = vec![vote(mandataire, dec!(100), None)];
        votes.extend((0..3).map(|_| vote(Uuid::new_v4(), dec!(150), Some(mandataire))));
        votes.push(vote(Uuid::new_v4(), dec!(300), None));

        // 550 pour le mandataire, 300 pour le reste.
        let refus = verifier_procurations(&votes, TOTAL_DES_LOTS, None).expect_err("doit refuser");
        assert!(matches!(
            refus,
            ProcurationRefusee::PoidsSuperieurAuReste { .. }
        ));
    }

    #[test]
    fn happy_une_seance_ordinaire_ne_declenche_rien() {
        let votes = vec![
            vote(Uuid::new_v4(), dec!(250), None),
            vote(Uuid::new_v4(), dec!(250), None),
            vote(Uuid::new_v4(), dec!(250), None),
            vote(Uuid::new_v4(), dec!(150), Some(Uuid::new_v4())),
        ];
        assert!(verifier_procurations(&votes, TOTAL_DES_LOTS, None).is_ok());
    }
}
