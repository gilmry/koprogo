//! L'autorisation préalable des contrats liés au syndic.
//!
//! Art. 3.89 § 5, 13° — le syndic est chargé :
//!
//! > « de **solliciter l'autorisation préalable de l'assemblée générale** pour
//! > tout contrat entre l'association des copropriétaires et **le syndic, ses
//! > préposés, ses proches, parents ou alliés jusqu'au troisième degré
//! > inclus**, ou ceux de son conjoint jusqu'au même degré ; il en est de même
//! > des contrats entre l'association des copropriétaires et **une entreprise
//! > dont les personnes susvisées sont propriétaires ou dans le capital de
//! > laquelle elles détiennent une participation**. »
//!
//! Sans cette règle, un syndic peut faire signer à l'ACP un contrat avec sa
//! propre société sans qu'aucune trace de vote soit exigée. C'est le conflit
//! d'intérêts sous sa forme la plus directe — non plus un vote biaisé
//! (Art. 3.87 § 9) mais un contrat conclu hors de tout vote.
//!
//! **L'antériorité fait partie de l'invariant.** « Autorisation préalable » :
//! une résolution votée après la signature ne régularise rien. C'est ce que
//! vérifie `autorisation_valable`, et c'est le point qu'une implémentation
//! naïve manquerait — elle se contenterait de l'existence d'une résolution.
//!
//! Le lien reste attaché au contrat après coup, pour l'audit et pour le
//! successeur : un syndic entrant doit pouvoir voir quels engagements de l'ACP
//! ont été conclus avec le cabinet précédent.
//!
//! Voir issue #745 et [`super::conflit_dinterets`], qui traite l'autre face du
//! même problème.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// La nature du lien entre le cocontractant et le syndic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LienAvecLeSyndic {
    /// Le syndic lui-même.
    LeSyndic,
    /// Un préposé du syndic.
    Prepose,
    /// Parent ou allié jusqu'au troisième degré, du syndic ou de son conjoint.
    ///
    /// Le degré est porté pour l'audit : au-delà du troisième, il n'y a plus
    /// de lien au sens de l'article.
    Parente { degre: u8 },
    /// Une entreprise dont une des personnes ci-dessus est propriétaire ou
    /// détient une participation.
    ///
    /// L'article ne fixe **aucun seuil** de participation : détenir une part
    /// suffit. Ne pas en inventer un.
    EntrepriseLiee,
}

impl LienAvecLeSyndic {
    /// Ce lien déclenche-t-il l'obligation d'autorisation préalable ?
    ///
    /// La parenté ne compte que jusqu'au troisième degré inclus. Au-delà,
    /// l'article ne s'applique pas — et l'étendre serait inventer une règle.
    pub fn exige_autorisation(&self) -> bool {
        match self {
            Self::LeSyndic | Self::Prepose | Self::EntrepriseLiee => true,
            Self::Parente { degre } => *degre <= 3,
        }
    }
}

impl std::fmt::Display for LienAvecLeSyndic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeSyndic => write!(f, "le syndic lui-même"),
            Self::Prepose => write!(f, "un préposé du syndic"),
            Self::Parente { degre } => write!(f, "un parent ou allié au {degre}e degré"),
            Self::EntrepriseLiee => {
                write!(f, "une entreprise liée au syndic ou à ses proches")
            }
        }
    }
}

/// L'autorisation votée par l'assemblée, si elle existe.
#[derive(Debug, Clone, PartialEq)]
pub struct AutorisationAssemblee {
    pub resolution_id: Uuid,
    pub votee_le: DateTime<Utc>,
}

/// Pourquoi un contrat est refusé.
#[derive(Debug, Clone, PartialEq)]
pub enum ContratRefuse {
    /// Aucune résolution n'autorise ce contrat.
    AutorisationAbsente { lien: LienAvecLeSyndic },
    /// Une résolution existe, mais elle est postérieure à la signature.
    AutorisationTardive {
        lien: LienAvecLeSyndic,
        signe_le: DateTime<Utc>,
        votee_le: DateTime<Utc>,
    },
}

impl std::fmt::Display for ContratRefuse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AutorisationAbsente { lien } => write!(
                f,
                "Art. 3.89 § 5, 13° : ce contrat est conclu avec {lien} et exige \
                 l'autorisation préalable de l'assemblée générale."
            ),
            Self::AutorisationTardive {
                lien,
                signe_le,
                votee_le,
            } => write!(
                f,
                "Art. 3.89 § 5, 13° : contrat avec {lien} signé le {} et autorisé le {} — \
                 l'autorisation doit être PRÉALABLE, une régularisation après coup n'en \
                 est pas une.",
                signe_le.date_naive(),
                votee_le.date_naive()
            ),
        }
    }
}

/// Un contrat conclu par l'ACP peut-il l'être ?
///
/// `lien` est `None` pour un cocontractant ordinaire : l'immense majorité des
/// contrats d'une ACP, et rien n'est exigé d'eux.
pub fn autorisation_valable(
    lien: Option<LienAvecLeSyndic>,
    signe_le: DateTime<Utc>,
    autorisation: Option<&AutorisationAssemblee>,
) -> Result<(), ContratRefuse> {
    let Some(lien) = lien else {
        return Ok(());
    };
    if !lien.exige_autorisation() {
        return Ok(());
    }
    let Some(autorisation) = autorisation else {
        return Err(ContratRefuse::AutorisationAbsente { lien });
    };
    if autorisation.votee_le > signe_le {
        return Err(ContratRefuse::AutorisationTardive {
            lien,
            signe_le,
            votee_le: autorisation.votee_le,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn il_y_a(jours: i64) -> DateTime<Utc> {
        Utc::now() - Duration::days(jours)
    }

    fn autorisation(votee_il_y_a: i64) -> AutorisationAssemblee {
        AutorisationAssemblee {
            resolution_id: Uuid::new_v4(),
            votee_le: il_y_a(votee_il_y_a),
        }
    }

    #[test]
    fn happy_un_contrat_ordinaire_nexige_rien() {
        assert!(autorisation_valable(None, il_y_a(10), None).is_ok());
    }

    #[test]
    fn happy_un_contrat_lie_autorise_avant_signature_passe() {
        assert!(autorisation_valable(
            Some(LienAvecLeSyndic::EntrepriseLiee),
            il_y_a(10),
            Some(&autorisation(30)),
        )
        .is_ok());
    }

    /// Le cas que la règle vise : le syndic fait signer à l'ACP un contrat
    /// avec sa propre société.
    #[test]
    fn security_un_contrat_avec_la_societe_du_syndic_sans_vote_est_refuse() {
        let refus = autorisation_valable(
            Some(LienAvecLeSyndic::EntrepriseLiee),
            il_y_a(10),
            None,
        )
        .expect_err("doit refuser");

        assert_eq!(
            refus,
            ContratRefuse::AutorisationAbsente {
                lien: LienAvecLeSyndic::EntrepriseLiee
            }
        );
    }

    /// **L'antériorité fait partie de l'invariant.**
    ///
    /// Une implémentation naïve se contenterait de l'existence d'une
    /// résolution. « Autorisation préalable » : une régularisation après coup
    /// n'en est pas une.
    #[test]
    fn security_une_autorisation_posterieure_ne_regularise_rien() {
        let refus = autorisation_valable(
            Some(LienAvecLeSyndic::LeSyndic),
            il_y_a(30),
            Some(&autorisation(10)),
        )
        .expect_err("doit refuser");

        assert!(matches!(refus, ContratRefuse::AutorisationTardive { .. }));
        assert!(
            refus.to_string().contains("PRÉALABLE"),
            "le refus doit dire pourquoi : {refus}"
        );
    }

    /// @edge — autorisée le jour même de la signature, elle passe.
    #[test]
    fn edge_une_autorisation_le_jour_meme_passe() {
        let moment = il_y_a(10);
        let autorisation = AutorisationAssemblee {
            resolution_id: Uuid::new_v4(),
            votee_le: moment,
        };
        assert!(
            autorisation_valable(Some(LienAvecLeSyndic::Prepose), moment, Some(&autorisation))
                .is_ok()
        );
    }

    // ── Le périmètre du lien ───────────────────────────────────────────

    #[test]
    fn happy_la_parente_compte_jusquau_troisieme_degre() {
        for degre in 1..=3 {
            assert!(
                LienAvecLeSyndic::Parente { degre }.exige_autorisation(),
                "le {degre}e degré est inclus"
            );
        }
    }

    /// @edge — au-delà du troisième degré, l'article ne s'applique plus.
    ///
    /// L'étendre serait inventer une règle, et bloquer des contrats que la loi
    /// autorise.
    #[test]
    fn edge_au_dela_du_troisieme_degre_larticle_ne_sapplique_plus() {
        let lien = LienAvecLeSyndic::Parente { degre: 4 };
        assert!(!lien.exige_autorisation());
        assert!(autorisation_valable(Some(lien), il_y_a(10), None).is_ok());
    }

    /// L'article ne fixe aucun seuil de participation : détenir une part
    /// suffit. Ne pas en inventer un.
    #[test]
    fn security_une_participation_meme_minime_declenche_lobligation() {
        assert!(LienAvecLeSyndic::EntrepriseLiee.exige_autorisation());
    }

    #[test]
    fn happy_le_syndic_et_ses_preposes_sont_vises() {
        assert!(LienAvecLeSyndic::LeSyndic.exige_autorisation());
        assert!(LienAvecLeSyndic::Prepose.exige_autorisation());
    }
}
