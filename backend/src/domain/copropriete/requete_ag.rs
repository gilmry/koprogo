//! La convocation d'une assemblée sur requête des copropriétaires.
//!
//! Art. 3.87 § 2, alinéas 2 et 3 :
//!
//! > « le syndic tient une assemblée générale **sur requête d'un ou de
//! > plusieurs copropriétaires qui possèdent au moins un cinquième des parts
//! > dans les parties communes**. Cette requête est adressée au syndic **par
//! > envoi recommandé** et celui-ci adresse la convocation aux copropriétaires
//! > **dans les trente jours** de la réception de la requête. »
//!
//! > « Si le syndic ne donne pas suite à cette requête, **un des
//! > copropriétaires qui a cosigné la requête peut convoquer lui-même
//! > l'assemblée générale**. »
//!
//! C'est un contre-pouvoir, et le second alinéa en est la sanction : le syndic
//! ne peut pas enterrer une requête en la laissant sans réponse, puisque son
//! silence transfère le pouvoir de convoquer à ceux qui l'ont signée. Sans le
//! suivi du délai, ce transfert n'a aucun déclencheur — un copropriétaire ne
//! saurait pas à partir de quand il peut agir.
//!
//! Le seuil se compte **en quotités, pas en têtes** : « un cinquième des parts
//! dans les parties communes ». Dix-neuf copropriétaires détenant ensemble
//! 150 millièmes n'atteignent pas le seuil ; un seul en détenant 200 l'atteint.
//! Confondre les deux est l'erreur la plus naturelle, et elle prive de leur
//! droit ceux que l'article veut protéger.
//!
//! Voir issue #741.

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

/// Le délai laissé au syndic pour convoquer.
pub const DELAI_CONVOCATION_JOURS: i64 = 30;

/// Ce qui empêche une requête d'aboutir.
#[derive(Debug, Clone, PartialEq)]
pub enum RequeteIrrecevable {
    /// Les cosignataires ne réunissent pas le cinquième des parts.
    SeuilNonAtteint {
        quotites_reunies: Decimal,
        seuil_requis: Decimal,
    },
    /// Aucun cosignataire.
    SansSignataire,
}

impl std::fmt::Display for RequeteIrrecevable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SeuilNonAtteint {
                quotites_reunies,
                seuil_requis,
            } => write!(
                f,
                "Art. 3.87 § 2 : les cosignataires réunissent {quotites_reunies} quotités, \
                 le cinquième des parts en exige {seuil_requis}."
            ),
            Self::SansSignataire => write!(
                f,
                "Art. 3.87 § 2 : une requête d'assemblée doit émaner d'au moins un \
                 copropriétaire."
            ),
        }
    }
}

/// Une requête d'assemblée générale, recevable.
#[derive(Debug, Clone, PartialEq)]
pub struct RequeteAg {
    pub cosignataires: Vec<Uuid>,
    pub quotites_reunies: Decimal,
    /// Date de réception par le syndic — c'est elle qui fait courir le délai,
    /// pas la date d'envoi ni celle de signature.
    pub recue_le: DateTime<Utc>,
    pub echeance_convocation: DateTime<Utc>,
}

/// Le seuil : un cinquième des parts dans les parties communes.
pub fn seuil_du_cinquieme(total_des_lots: Decimal) -> Decimal {
    total_des_lots / Decimal::from(5)
}

/// Constitue une requête si le seuil est atteint.
///
/// `quotites_reunies` est la somme des quotités des cosignataires, pas leur
/// nombre : l'article compte en parts.
pub fn deposer(
    cosignataires: Vec<Uuid>,
    quotites_reunies: Decimal,
    total_des_lots: Decimal,
    recue_le: DateTime<Utc>,
) -> Result<RequeteAg, RequeteIrrecevable> {
    if cosignataires.is_empty() {
        return Err(RequeteIrrecevable::SansSignataire);
    }
    let seuil = seuil_du_cinquieme(total_des_lots);
    if quotites_reunies < seuil {
        return Err(RequeteIrrecevable::SeuilNonAtteint {
            quotites_reunies,
            seuil_requis: seuil,
        });
    }
    Ok(RequeteAg {
        cosignataires,
        quotites_reunies,
        recue_le,
        echeance_convocation: recue_le + Duration::days(DELAI_CONVOCATION_JOURS),
    })
}

impl RequeteAg {
    /// Le syndic a-t-il laissé passer le délai ?
    ///
    /// `convoquee_le` est la date à laquelle il a adressé la convocation, si
    /// il l'a fait.
    pub fn syndic_defaillant(
        &self,
        convoquee_le: Option<DateTime<Utc>>,
        moment: DateTime<Utc>,
    ) -> bool {
        match convoquee_le {
            Some(date) => date > self.echeance_convocation,
            None => moment > self.echeance_convocation,
        }
    }

    /// Un cosignataire peut-il convoquer lui-même ?
    ///
    /// C'est la sanction du silence (Art. 3.87 § 2, alinéa 3), et elle ne
    /// s'ouvre qu'aux **cosignataires** : un copropriétaire qui n'a pas signé
    /// la requête ne récupère pas ce pouvoir.
    pub fn peut_convoquer_lui_meme(
        &self,
        copropietaire: Uuid,
        convoquee_le: Option<DateTime<Utc>>,
        moment: DateTime<Utc>,
    ) -> bool {
        self.cosignataires.contains(&copropietaire)
            && self.syndic_defaillant(convoquee_le, moment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    const TOTAL: Decimal = dec!(1000);

    fn il_y_a(jours: i64) -> DateTime<Utc> {
        Utc::now() - Duration::days(jours)
    }

    #[test]
    fn happy_le_seuil_vaut_un_cinquieme_des_parts() {
        assert_eq!(seuil_du_cinquieme(TOTAL), dec!(200));
    }

    #[test]
    fn happy_une_requete_au_seuil_est_recevable() {
        let requete = deposer(vec![Uuid::new_v4()], dec!(200), TOTAL, il_y_a(5))
            .expect("recevable");
        assert_eq!(requete.quotites_reunies, dec!(200));
    }

    /// Le seuil se compte en quotités, pas en têtes.
    ///
    /// C'est l'erreur la plus naturelle, et elle prive de leur droit ceux que
    /// l'article veut protéger.
    #[test]
    fn negative_dix_neuf_petits_porteurs_natteignent_pas_le_seuil() {
        let cosignataires: Vec<Uuid> = (0..19).map(|_| Uuid::new_v4()).collect();
        let refus = deposer(cosignataires, dec!(150), TOTAL, il_y_a(5))
            .expect_err("doit refuser");

        assert_eq!(
            refus,
            RequeteIrrecevable::SeuilNonAtteint {
                quotites_reunies: dec!(150),
                seuil_requis: dec!(200)
            }
        );
    }

    #[test]
    fn happy_un_seul_copropietaire_suffisamment_dote_atteint_le_seuil() {
        assert!(deposer(vec![Uuid::new_v4()], dec!(250), TOTAL, il_y_a(5)).is_ok());
    }

    #[test]
    fn negative_une_requete_sans_signataire_est_irrecevable() {
        assert_eq!(
            deposer(vec![], dec!(500), TOTAL, il_y_a(5)),
            Err(RequeteIrrecevable::SansSignataire)
        );
    }

    // ── Le délai et sa sanction ────────────────────────────────────────

    #[test]
    fn happy_le_delai_court_depuis_la_reception_par_le_syndic() {
        let recue = il_y_a(10);
        let requete = deposer(vec![Uuid::new_v4()], dec!(200), TOTAL, recue).unwrap();
        assert_eq!(requete.echeance_convocation, recue + Duration::days(30));
    }

    #[test]
    fn happy_un_syndic_qui_convoque_a_temps_nest_pas_defaillant() {
        let requete = deposer(vec![Uuid::new_v4()], dec!(200), TOTAL, il_y_a(40)).unwrap();
        assert!(!requete.syndic_defaillant(Some(il_y_a(20)), Utc::now()));
    }

    #[test]
    fn negative_un_syndic_silencieux_devient_defaillant_passe_trente_jours() {
        let requete = deposer(vec![Uuid::new_v4()], dec!(200), TOTAL, il_y_a(40)).unwrap();
        assert!(requete.syndic_defaillant(None, Utc::now()));
    }

    #[test]
    fn happy_avant_lecheance_le_silence_nest_pas_une_defaillance() {
        let requete = deposer(vec![Uuid::new_v4()], dec!(200), TOTAL, il_y_a(10)).unwrap();
        assert!(!requete.syndic_defaillant(None, Utc::now()));
    }

    /// La sanction du silence : le cosignataire convoque lui-même.
    #[test]
    fn happy_le_cosignataire_recupere_le_pouvoir_de_convoquer() {
        let signataire = Uuid::new_v4();
        let requete = deposer(vec![signataire], dec!(200), TOTAL, il_y_a(40)).unwrap();

        assert!(requete.peut_convoquer_lui_meme(signataire, None, Utc::now()));
    }

    /// @security — mais seulement lui.
    ///
    /// Un copropriétaire qui n'a pas signé la requête ne récupère pas ce
    /// pouvoir : l'article le réserve à « un des copropriétaires qui a cosigné
    /// la requête ».
    #[test]
    fn security_un_non_signataire_ne_recupere_pas_ce_pouvoir() {
        let signataire = Uuid::new_v4();
        let passant = Uuid::new_v4();
        let requete = deposer(vec![signataire], dec!(200), TOTAL, il_y_a(40)).unwrap();

        assert!(!requete.peut_convoquer_lui_meme(passant, None, Utc::now()));
    }

    /// @edge — et pas avant que le délai soit écoulé.
    #[test]
    fn edge_le_cosignataire_ne_peut_pas_devancer_lecheance() {
        let signataire = Uuid::new_v4();
        let requete = deposer(vec![signataire], dec!(200), TOTAL, il_y_a(10)).unwrap();

        assert!(!requete.peut_convoquer_lui_meme(signataire, None, Utc::now()));
    }

    /// @edge — une convocation tardive reste une défaillance.
    ///
    /// Le syndic qui convoque le trente-cinquième jour n'efface pas son
    /// retard : le cosignataire pouvait déjà agir au trente-et-unième.
    #[test]
    fn edge_une_convocation_tardive_reste_une_defaillance() {
        let requete = deposer(vec![Uuid::new_v4()], dec!(200), TOTAL, il_y_a(40)).unwrap();
        assert!(requete.syndic_defaillant(Some(il_y_a(3)), Utc::now()));
    }
}
