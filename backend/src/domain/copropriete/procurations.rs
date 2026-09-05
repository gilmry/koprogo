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

/// Les voix qu'une personne engage dans le vote, à quelque titre que ce soit.
///
/// Le texte vise « même comme **mandant** ou mandataire ». Une personne engage
/// donc : ses propres lots, même confiés à un mandataire, **et** les lots
/// qu'elle porte pour autrui.
///
/// Grouper par mandataire seul laissait passer un contournement documenté :
/// un copropriétaire majoritaire désignait un mandataire différent par lot,
/// de sorte qu'aucun d'eux ne dépassait le seuil pris isolément. La doctrine
/// belge le juge non conforme, le mandat étant lié à la personne du
/// copropriétaire et non au bien.
///
/// Pour une personne X donnée, chaque bulletin tombe d'un seul côté : soit X y
/// est engagée, soit non. Il n'y a donc pas de double compte dans la
/// comparaison entre X et le reste.
fn voix_engagees_par_personne(votes: &[Vote]) -> HashMap<Uuid, Decimal> {
    let mut engagees: HashMap<Uuid, Decimal> = HashMap::new();
    for vote in votes {
        *engagees.entry(vote.owner_id).or_insert(Decimal::ZERO) += vote.voting_power;
        if let Some(mandataire) = vote.proxy_owner_id {
            if mandataire != vote.owner_id {
                *engagees.entry(mandataire).or_insert(Decimal::ZERO) += vote.voting_power;
            }
        }
    }
    engagees
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

/// L'écart entre ce qu'un votant pesait et ce qui lui a été retenu.
///
/// Conservé pour que l'ACP puisse répondre de son décompte : si la décision
/// est attaquée, il faut pouvoir montrer que la règle a été appliquée, et de
/// combien. Un plafonnement silencieux serait indéfendable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcartDePlafond {
    pub votant: Uuid,
    /// Ce dont le votant disposait, procurations comprises.
    pub voix_brutes: Decimal,
    /// Ce qui a été retenu : la somme des voix des autres.
    pub voix_retenues: Decimal,
}

/// Le décompte d'une séance après application de l'Art. 3.87 § 7 al. 4.
#[derive(Debug, Clone, Default)]
pub struct DecompteDesVoix {
    retenues: HashMap<Uuid, Decimal>,
    ecarts: Vec<EcartDePlafond>,
}

impl DecompteDesVoix {
    /// Les voix retenues pour un votant, plafonnement compris.
    pub fn voix(&self, votant: Uuid) -> Decimal {
        self.retenues.get(&votant).copied().unwrap_or(Decimal::ZERO)
    }

    /// Les plafonnements appliqués. Vide quand personne n'était majoritaire.
    pub fn ecarts(&self) -> &[EcartDePlafond] {
        &self.ecarts
    }
}

/// Applique l'Art. 3.87 § 7 al. 4 : « Nul ne peut prendre part au vote, même
/// comme mandant ou mandataire, pour un nombre de voix supérieur à la somme
/// des voix dont disposent les autres copropriétaires présents ou
/// représentés. »
///
/// Le texte interdit de voter **pour** un nombre de voix supérieur ; il ne
/// frappe pas la séance de nullité. Le décompte du majoritaire est ramené à la
/// somme des autres, et l'assemblée délibère là-dessus. Refuser de clore
/// rendrait ingouvernable toute copropriété où un seul détient la majorité,
/// situation licite et fréquente.
///
/// Au plus un votant peut être plafonné : dépasser la somme des autres, c'est
/// dépasser la moitié du total, et deux personnes ne le peuvent pas ensemble.
///
/// Cas limite du votant unique : la somme des autres vaut zéro. Le ramener à
/// zéro viderait la séance de tout sens. C'est le quorum de l'Art. 3.87 § 5
/// qui traite ce cas, pas cet alinéa-ci.
pub fn plafonner_les_voix(votes: &[Vote]) -> DecompteDesVoix {
    let brutes = voix_engagees_par_personne(votes);
    // Le dénominateur est l'ensemble des voix présentes ou représentées, une
    // seule fois chacune — pas la somme des engagements, qui compte deux fois
    // un lot confié à un mandataire.
    let total: Decimal = votes.iter().map(|v| v.voting_power).sum();

    let mut retenues = HashMap::with_capacity(brutes.len());
    let mut ecarts = Vec::new();

    for (votant, poids) in brutes {
        let reste = total - poids;
        if poids > reste && !reste.is_zero() {
            ecarts.push(EcartDePlafond {
                votant,
                voix_brutes: poids,
                voix_retenues: reste,
            });
            retenues.insert(votant, reste);
        } else {
            retenues.insert(votant, poids);
        }
    }

    DecompteDesVoix { retenues, ecarts }
}

/// Répartit le plafonnement sur chaque bulletin, dans l'ordre des votes reçus.
///
/// Un mandataire peut voter « pour » son propre lot et « contre » celui d'un
/// mandant. Quand il est plafonné, il faut décider comment l'écart se répartit
/// entre ces sens. **La loi ne le dit pas.** Le choix retenu est la réduction
/// proportionnelle : chaque bulletin conserve la même part relative, donc
/// l'arbitrage du votant est préservé. Les deux autres lectures possibles —
/// retrancher d'abord des « pour », ou d'abord des « contre » — feraient
/// pencher le résultat dans un sens que rien ne justifie.
///
/// Rend un poids retenu par vote, dans le même ordre que `votes`.
pub fn repartir_le_plafond(votes: &[Vote], decompte: &DecompteDesVoix) -> Vec<Decimal> {
    let brutes = voix_engagees_par_personne(votes);

    // Le couple (retenu, brut) d'une personne. `None` si elle n'est pas
    // plafonnée.
    let plafond = |personne: Uuid| -> Option<(Decimal, Decimal)> {
        let brut = brutes.get(&personne).copied().unwrap_or(Decimal::ZERO);
        if brut.is_zero() {
            return None;
        }
        let retenu = decompte.voix(personne);
        if retenu == brut {
            None
        } else {
            Some((retenu, brut))
        }
    };

    votes
        .iter()
        .map(|v| {
            // Un bulletin engage son propriétaire ET son mandataire. Si les
            // deux sont plafonnés, c'est la réduction la plus forte qui
            // s'applique : retenir la plus douce laisserait l'un des deux
            // dépasser la somme des autres, ce que le texte interdit à chacun
            // pour son propre compte.
            let mut retenu_brut = plafond(v.owner_id);
            if let Some(mandataire) = v.proxy_owner_id {
                if let Some((r2, b2)) = plafond(mandataire) {
                    // Comparaison de deux fractions sans les évaluer :
                    // r2/b2 < r1/b1  ⟺  r2·b1 < r1·b2.
                    let plus_severe = match retenu_brut {
                        None => true,
                        Some((r1, b1)) => r2 * b1 < r1 * b2,
                    };
                    if plus_severe {
                        retenu_brut = Some((r2, b2));
                    }
                }
            }

            match retenu_brut {
                None => v.voting_power,
                // MULTIPLIER AVANT DE DIVISER.
                //
                // Calculer d'abord le ratio `retenu / brut` puis multiplier
                // laisse une traîne d'arrondi : 550 × (450/550) donne
                // 450,00000000000000000000000001. Ce résidu de 10⁻²⁶ suffit à
                // transformer une ÉGALITÉ en majorité — 450 contre 450
                // devenait « adopté » alors que l'Art. 3.88 § 1er exige PLUS
                // de la moitié des voix exprimées.
                //
                // Constaté en recette le 2026-09-04 sur une vraie résolution.
                // L'ADR-0008 impose `Decimal` pour éviter exactement cela ; le
                // type ne suffit pas, l'ordre des opérations compte aussi.
                Some((retenu, brut)) => v.voting_power * retenu / brut,
            }
        })
        .collect()
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

    // L'alinéa 4 — « nul ne prend part au vote pour un nombre de voix
    // supérieur à la somme des autres » — n'est PAS vérifié ici : ce n'est pas
    // un refus mais un plafonnement, appliqué par `plafonner_les_voix`.
    // Arbitrage humain du 2026-09-04 : le texte interdit de voter POUR un
    // nombre de voix supérieur, il ne frappe pas la séance de nullité.

    Ok(())
}

#[cfg(test)]
mod tests {
    // ── Art. 3.87 § 7 al. 4 — le plafonnement des voix ──────────────────────
    //
    // Le texte dit : « Nul ne peut prendre part au vote, même comme mandant ou
    // mandataire, pour un nombre de voix supérieur à la somme des voix dont
    // disposent les autres copropriétaires présents ou représentés. »
    //
    // Il interdit de VOTER POUR un nombre de voix supérieur ; il ne frappe pas
    // la séance de nullité. Le décompte du majoritaire est donc ramené à la
    // somme des autres, et l'assemblée délibère sur ce décompte corrigé. Une
    // copropriété où un seul détient la majorité est licite et courante :
    // refuser de clore la rendrait ingouvernable.
    //
    // L'écart est conservé. Si la décision est attaquée, l'ACP doit pouvoir
    // montrer qu'elle a appliqué la règle, et de combien.

    #[test]
    fn security_le_majoritaire_ne_contourne_pas_le_plafond_en_eclatant_ses_procurations() {
        // Le texte vise « même comme MANDANT ou mandataire ». Le mandant reste
        // donc plafonné pour ses propres voix, quel que soit le nombre de
        // mandataires entre lesquels il les répartit.
        //
        // Le contournement a été tenté en pratique : désigner un mandataire
        // distinct par lot, de sorte qu'aucun d'eux ne dépasse le seuil pris
        // isolément. La doctrine belge le juge non conforme — le mandat est
        // lié à la personne du copropriétaire, pas au bien.
        // Voir propertytoday.be, « Sens et non-sens de la réduction de vote de
        // l'art. 3.87 § 7 Cc », consulté le 2026-09-04.
        let majoritaire = Uuid::new_v4();
        let autre_a = Uuid::new_v4();
        let autre_b = Uuid::new_v4();
        let votes = vec![
            // 600 voix éclatées entre trois mandataires différents.
            vote(majoritaire, dec!(200), Some(Uuid::new_v4())),
            vote(majoritaire, dec!(200), Some(Uuid::new_v4())),
            vote(majoritaire, dec!(200), Some(Uuid::new_v4())),
            vote(autre_a, dec!(250), None),
            vote(autre_b, dec!(150), None),
        ];

        let decompte = plafonner_les_voix(&votes);

        let ecarts = decompte.ecarts();
        assert_eq!(
            ecarts.len(),
            1,
            "le mandant est plafonné, pas ses mandataires"
        );
        assert_eq!(ecarts[0].votant, majoritaire);
        assert_eq!(ecarts[0].voix_brutes, dec!(600));
        assert_eq!(ecarts[0].voix_retenues, dec!(400), "250 + 150");
    }

    #[test]
    fn le_plafond_se_repartit_proportionnellement_entre_les_sens() {
        // Le mandataire pèse 800 (200 pour lui, 600 portés), les autres 400.
        // Il est ramené à 400, soit la moitié. Chacun de ses bulletins est
        // réduit dans la même proportion : son arbitrage relatif entre
        // « pour » et « contre » reste intact.
        //
        // Le rapport est choisi exact (1/2) à dessein. Avec 2/3, la valeur
        // attendue et la valeur calculée diffèrent au dernier chiffre selon
        // l'ordre des opérations, et le test mesurerait l'arrondi de
        // `Decimal` plutôt que la règle de droit.
        let mandataire = Uuid::new_v4();
        let mandant = Uuid::new_v4();
        let autre = Uuid::new_v4();
        let votes = vec![
            vote(mandataire, dec!(200), None),
            vote(mandant, dec!(600), Some(mandataire)),
            vote(autre, dec!(400), None),
        ];

        let decompte = plafonner_les_voix(&votes);
        let retenus = repartir_le_plafond(&votes, &decompte);

        assert_eq!(retenus[0], dec!(100), "200 ramenés de moitié");
        assert_eq!(retenus[1], dec!(300), "600 ramenés de moitié");
        assert_eq!(retenus[2], dec!(400), "les autres ne bougent pas");
        assert_eq!(retenus[0] + retenus[1], dec!(400), "total ramené au reste");
    }

    #[test]
    fn le_majoritaire_est_ramene_a_la_somme_des_autres() {
        let majoritaire = Uuid::new_v4();
        let autre_a = Uuid::new_v4();
        let autre_b = Uuid::new_v4();
        let votes = vec![
            vote(majoritaire, dec!(600), None),
            vote(autre_a, dec!(250), None),
            vote(autre_b, dec!(150), None),
        ];

        let decompte = plafonner_les_voix(&votes);

        assert_eq!(decompte.voix(majoritaire), dec!(400), "ramené à 250 + 150");
        assert_eq!(decompte.voix(autre_a), dec!(250), "les autres sont intacts");
        assert_eq!(decompte.voix(autre_b), dec!(150));
    }

    #[test]
    fn lecart_est_conserve_pour_pouvoir_en_repondre() {
        let majoritaire = Uuid::new_v4();
        let autre = Uuid::new_v4();
        let votes = vec![
            vote(majoritaire, dec!(600), None),
            vote(autre, dec!(400), None),
        ];

        let decompte = plafonner_les_voix(&votes);

        let ecarts = decompte.ecarts();
        assert_eq!(ecarts.len(), 1, "un seul votant plafonné");
        assert_eq!(ecarts[0].votant, majoritaire);
        assert_eq!(ecarts[0].voix_brutes, dec!(600));
        assert_eq!(ecarts[0].voix_retenues, dec!(400));
    }

    #[test]
    fn sans_majoritaire_rien_nest_plafonne() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let votes = vec![
            vote(a, dec!(400), None),
            vote(b, dec!(350), None),
            vote(c, dec!(250), None),
        ];

        let decompte = plafonner_les_voix(&votes);

        assert!(
            decompte.ecarts().is_empty(),
            "personne ne pèse plus que le reste"
        );
        assert_eq!(decompte.voix(a), dec!(400));
    }

    #[test]
    fn le_mandataire_pese_ses_voix_et_celles_de_ses_mandants() {
        // Un mandataire qui porte assez de procurations pour devenir
        // majoritaire est plafonné comme n'importe qui : la loi vise le
        // mandant COMME le mandataire.
        let mandataire = Uuid::new_v4();
        let mandant_a = Uuid::new_v4();
        let mandant_b = Uuid::new_v4();
        let isole = Uuid::new_v4();
        let votes = vec![
            vote(mandataire, dec!(200), None),
            vote(mandant_a, dec!(250), Some(mandataire)),
            vote(mandant_b, dec!(250), Some(mandataire)),
            vote(isole, dec!(300), None),
        ];

        let decompte = plafonner_les_voix(&votes);

        assert_eq!(decompte.voix(mandataire), dec!(300), "700 ramenés à 300");
        assert_eq!(decompte.voix(isole), dec!(300));
    }

    #[test]
    fn une_seule_voix_exprimee_nest_pas_plafonnee_a_zero() {
        // Cas limite : un unique votant présent. La somme des autres vaut
        // zéro. Le plafonner à zéro viderait la séance de tout sens ; le
        // quorum de l'Art. 3.87 § 5 est le garde-fou qui vaut ici, pas celui-ci.
        let seul = Uuid::new_v4();
        let votes = vec![vote(seul, dec!(500), None)];

        let decompte = plafonner_les_voix(&votes);

        assert_eq!(decompte.voix(seul), dec!(500));
        assert!(decompte.ecarts().is_empty());
    }

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
    fn un_votant_qui_depasse_la_somme_des_autres_est_plafonne_pas_refuse() {
        // Anciennement `negative_un_votant_ne_depasse_pas_la_somme_des_autres`,
        // qui attendait un refus. Arbitrage humain du 2026-09-04 : l'Art. 3.87
        // § 7 al. 4 plafonne, il n'annule pas. La séance reste valide.
        let dominant = Uuid::new_v4();
        let votes = vec![
            vote(dominant, dec!(600), None),
            vote(Uuid::new_v4(), dec!(200), None),
            vote(Uuid::new_v4(), dec!(199), None),
        ];

        verifier_procurations(&votes, TOTAL_DES_LOTS, None)
            .expect("le poids ne fait plus obstacle à la clôture");

        let decompte = plafonner_les_voix(&votes);
        assert_eq!(decompte.voix(dominant), dec!(399), "ramené à 200 + 199");
        assert_eq!(decompte.ecarts().len(), 1);
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
        // C'est le plafonnement de l'alinéa 4 qui le ramène à sa place.
        let mandataire = Uuid::new_v4();
        let mut votes = vec![vote(mandataire, dec!(100), None)];
        votes.extend((0..3).map(|_| vote(Uuid::new_v4(), dec!(150), Some(mandataire))));
        votes.push(vote(Uuid::new_v4(), dec!(300), None));

        // 550 pour le mandataire, 300 pour le reste. La propriété de sécurité
        // tient toujours : il ne peut pas emporter le vote à lui seul. Ce qui
        // change depuis l'arbitrage du 2026-09-04, c'est le mécanisme — il est
        // ramené au poids des autres, il n'est plus opposé un refus de clore.
        verifier_procurations(&votes, TOTAL_DES_LOTS, None)
            .expect("trois procurations : les plafonds de procuration passent");

        let decompte = plafonner_les_voix(&votes);
        assert_eq!(decompte.voix(mandataire), dec!(300), "550 ramenés à 300");
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
