//! Le conflit d'intérêts à l'assemblée générale.
//!
//! Art. 3.87 § 9 :
//!
//! > « Aucune personne **mandatée ou employée** par l'association des
//! > copropriétaires, ou **prestant pour elle des services dans le cadre de
//! > tout autre contrat**, ne peut participer personnellement ou par
//! > procuration aux délibérations et aux votes **relatifs à la mission qui
//! > lui a été confiée**. »
//!
//! La règle est étroite et il faut la lire comme telle : elle n'écarte pas le
//! prestataire de toute l'assemblée, seulement des points qui **le
//! concernent**. Un entrepreneur copropriétaire vote normalement sur le
//! budget ; il ne vote pas sur l'attribution du marché qu'il brigue, ni sur la
//! réception de ses propres travaux.
//!
//! Elle vise aussi la procuration : donner son bulletin à un tiers ne
//! contourne rien, puisque c'est le mandant qui est écarté autant que le
//! mandataire. D'où l'usage de `effective_voter_id` **et** de `owner_id`.
//!
//! Le syndic en relève au premier chef — il est mandaté par l'association
//! (Art. 3.89) — et l'Art. 3.89 § 9 y ajoute qu'il ne peut être ni membre du
//! conseil de copropriété ni commissaire aux comptes dans la même ACP.
//!
//! Voir issue #743.

use super::vote::Vote;
use uuid::Uuid;

/// Un vote exprimé par une personne intéressée à la décision.
#[derive(Debug, Clone, PartialEq)]
pub struct ConflitDinterets {
    /// La personne dont la mission fait l'objet de la délibération.
    pub interesse: Uuid,
    /// Le nombre de bulletins qui lui reviennent, en son nom ou par mandat.
    pub bulletins: usize,
    /// A-t-il voté par l'intermédiaire d'un mandataire ?
    pub par_procuration: bool,
}

impl std::fmt::Display for ConflitDinterets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let voie = if self.par_procuration {
            " (dont par procuration, ce qui ne contourne rien)"
        } else {
            ""
        };
        write!(
            f,
            "Art. 3.87 § 9 : {} est le prestataire de la mission délibérée et a pris part \
             au vote pour {} bulletin(s){voie}. La délibération est à reprendre sans lui.",
            self.interesse, self.bulletins
        )
    }
}

/// Vérifie qu'aucun intéressé n'a pris part au vote sur sa propre mission.
///
/// `prestataire_de_la_mission` est l'identifiant du copropriétaire, du syndic
/// ou du prestataire dont la délibération traite la mission. `None` signifie
/// que la résolution ne porte sur la mission de personne — le cas courant, et
/// alors il n'y a rien à vérifier.
///
/// La vérification couvre les deux voies que la loi nomme :
///
/// - **personnellement** : l'intéressé figure comme `owner_id` d'un bulletin ;
/// - **par procuration** : il figure comme mandataire d'un autre, ou son
///   propre bulletin a été porté par un tiers. Les deux sont écartés, sinon la
///   règle se contournerait par un simple échange de pouvoirs.
pub fn verifier_conflit_dinterets(
    votes: &[Vote],
    prestataire_de_la_mission: Option<Uuid>,
) -> Result<(), ConflitDinterets> {
    let Some(interesse) = prestataire_de_la_mission else {
        return Ok(());
    };

    let bulletins: Vec<&Vote> = votes
        .iter()
        .filter(|v| v.owner_id == interesse || v.effective_voter_id() == interesse)
        .collect();

    if bulletins.is_empty() {
        return Ok(());
    }

    Err(ConflitDinterets {
        interesse,
        bulletins: bulletins.len(),
        par_procuration: bulletins.iter().any(|v| v.is_proxy_vote()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::copropriete::vote::VoteChoice;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

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

    #[test]
    fn happy_une_resolution_sans_prestataire_ne_declenche_rien() {
        let votes = vec![
            vote(Uuid::new_v4(), dec!(300), None),
            vote(Uuid::new_v4(), dec!(300), None),
        ];
        assert!(verifier_conflit_dinterets(&votes, None).is_ok());
    }

    #[test]
    fn happy_le_prestataire_absent_du_vote_ne_pose_pas_de_probleme() {
        let entrepreneur = Uuid::new_v4();
        let votes = vec![
            vote(Uuid::new_v4(), dec!(300), None),
            vote(Uuid::new_v4(), dec!(300), None),
        ];
        assert!(verifier_conflit_dinterets(&votes, Some(entrepreneur)).is_ok());
    }

    #[test]
    fn negative_le_prestataire_ne_vote_pas_sur_sa_propre_mission() {
        let entrepreneur = Uuid::new_v4();
        let votes = vec![
            vote(entrepreneur, dec!(150), None),
            vote(Uuid::new_v4(), dec!(300), None),
        ];

        let conflit =
            verifier_conflit_dinterets(&votes, Some(entrepreneur)).expect_err("doit refuser");
        assert_eq!(conflit.interesse, entrepreneur);
        assert_eq!(conflit.bulletins, 1);
        assert!(!conflit.par_procuration);
    }

    #[test]
    fn security_donner_procuration_ne_contourne_pas_la_regle() {
        // L'entrepreneur confie son bulletin à un tiers. La loi écarte le
        // vote « personnellement OU par procuration » : c'est bien la voix de
        // l'intéressé qui est écartée, pas seulement sa présence.
        let entrepreneur = Uuid::new_v4();
        let complaisant = Uuid::new_v4();
        let votes = vec![
            vote(entrepreneur, dec!(150), Some(complaisant)),
            vote(Uuid::new_v4(), dec!(300), None),
        ];

        let conflit =
            verifier_conflit_dinterets(&votes, Some(entrepreneur)).expect_err("doit refuser");
        assert_eq!(conflit.bulletins, 1);
        assert!(conflit.par_procuration);
    }

    #[test]
    fn security_recevoir_procuration_ne_contourne_pas_davantage() {
        // L'entrepreneur ne vote pas pour son lot mais porte celui d'un
        // autre : il « participe par procuration » aux termes de l'article.
        let entrepreneur = Uuid::new_v4();
        let votes = vec![
            vote(Uuid::new_v4(), dec!(150), Some(entrepreneur)),
            vote(Uuid::new_v4(), dec!(300), None),
        ];

        let conflit =
            verifier_conflit_dinterets(&votes, Some(entrepreneur)).expect_err("doit refuser");
        assert_eq!(conflit.bulletins, 1);
        assert!(conflit.par_procuration);
    }

    #[test]
    fn edge_les_deux_voies_cumulees_comptent_chaque_bulletin() {
        let entrepreneur = Uuid::new_v4();
        let votes = vec![
            vote(entrepreneur, dec!(150), None),
            vote(Uuid::new_v4(), dec!(100), Some(entrepreneur)),
            vote(Uuid::new_v4(), dec!(300), None),
        ];

        let conflit =
            verifier_conflit_dinterets(&votes, Some(entrepreneur)).expect_err("doit refuser");
        assert_eq!(conflit.bulletins, 2);
    }
}
