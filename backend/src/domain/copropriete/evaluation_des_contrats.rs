//! Le rapport annuel d'évaluation des contrats de fournitures.
//!
//! Art. 3.89 § 5, 12° — le syndic est chargé :
//!
//! > « de soumettre à l'**assemblée générale ordinaire** un **rapport
//! > d'évaluation** des **contrats de fournitures régulières**. »
//!
//! Une phrase, trois contraintes, et chacune se perd facilement :
//!
//! 1. **à l'assemblée générale ordinaire** — pas à une extraordinaire, pas au
//!    conseil de copropriété. C'est devant l'assemblée annuelle que les
//!    copropriétaires jugent les contrats qu'ils paient ;
//! 2. **annuellement** — chaque AGO, sans reconduction ni report. Un rapport
//!    présenté en 2025 ne couvre pas l'exercice 2026 ;
//! 3. **les contrats de fournitures régulières** — l'entretien de l'ascenseur,
//!    le nettoyage, le chauffage, l'assurance. Pas les marchés ponctuels : une
//!    réfection de toiture n'est pas une fourniture régulière, et l'y inclure
//!    noierait le rapport sous des lignes qui n'appellent aucune décision de
//!    reconduction.
//!
//! L'obligation existe parce que ces contrats se reconduisent tacitement.
//! Sans un point annuel, une copropriété paie pendant dix ans un contrat
//! d'entretien que personne n'a rediscuté — et c'est exactement ce que
//! l'article veut empêcher.
//!
//! Elle prolonge l'Art. 3.88 § 1er, 1°, c), qui fait voter le seuil au-delà
//! duquel la mise en concurrence est obligatoire : l'un fixe la règle, l'autre
//! oblige à regarder chaque année si elle est tenue.
//!
//! Voir issue #581.

use uuid::Uuid;

/// La nature d'un engagement de l'ACP, du point de vue de cette obligation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NatureDuContrat {
    /// Fourniture régulière : entretien, nettoyage, chauffage, assurance.
    /// C'est ce que le rapport doit couvrir.
    FournitureReguliere,
    /// Marché ponctuel : des travaux, une expertise. Hors du rapport.
    MarchePonctuel,
}

/// Un contrat de l'ACP, tel que l'évaluation le voit.
#[derive(Debug, Clone, PartialEq)]
pub struct ContratDeLacp {
    pub id: Uuid,
    pub objet: String,
    pub nature: NatureDuContrat,
    /// Le fournisseur, pour que le rapport soit nominatif.
    pub fournisseur: String,
}

/// Ce qui manque au rapport d'un exercice.
#[derive(Debug, Clone, PartialEq)]
pub enum EvaluationManquante {
    /// Aucun rapport n'a été soumis pour cet exercice.
    RapportAbsent { exercice: i32 },
    /// Des contrats de fournitures régulières n'y figurent pas.
    ContratsNonEvalues { objets: Vec<String> },
}

impl std::fmt::Display for EvaluationManquante {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RapportAbsent { exercice } => write!(
                f,
                "Art. 3.89 § 5, 12° : aucun rapport d'évaluation des contrats de fournitures \
                 régulières soumis à l'AGO de l'exercice {exercice}."
            ),
            Self::ContratsNonEvalues { objets } => write!(
                f,
                "Art. 3.89 § 5, 12° : {} contrat(s) de fournitures régulières absent(s) du \
                 rapport — {}. Ces contrats se reconduisent tacitement ; sans point annuel, \
                 personne ne les rediscute.",
                objets.len(),
                objets.join(", ")
            ),
        }
    }
}

/// Le rapport soumis à une assemblée générale ordinaire.
#[derive(Debug, Clone, PartialEq)]
pub struct RapportDevaluation {
    pub exercice: i32,
    /// L'AGO devant laquelle il a été soumis.
    ///
    /// Obligatoire : le soumettre au conseil de copropriété ou à une
    /// extraordinaire ne satisfait pas l'article.
    pub ago_id: Uuid,
    /// Les contrats effectivement évalués.
    pub contrats_evalues: Vec<Uuid>,
}

/// Vérifie qu'un exercice a bien son rapport, et qu'il est complet.
///
/// `contrats` sont tous les engagements en cours de l'ACP ; seules les
/// fournitures régulières sont attendues au rapport.
pub fn verifier_exercice(
    exercice: i32,
    contrats: &[ContratDeLacp],
    rapports: &[RapportDevaluation],
) -> Result<(), EvaluationManquante> {
    let Some(rapport) = rapports.iter().find(|r| r.exercice == exercice) else {
        return Err(EvaluationManquante::RapportAbsent { exercice });
    };

    let oublies: Vec<String> = contrats
        .iter()
        .filter(|c| c.nature == NatureDuContrat::FournitureReguliere)
        .filter(|c| !rapport.contrats_evalues.contains(&c.id))
        .map(|c| format!("{} ({})", c.objet, c.fournisseur))
        .collect();

    if oublies.is_empty() {
        Ok(())
    } else {
        Err(EvaluationManquante::ContratsNonEvalues { objets: oublies })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contrat(objet: &str, nature: NatureDuContrat) -> ContratDeLacp {
        ContratDeLacp {
            id: Uuid::new_v4(),
            objet: objet.to_string(),
            nature,
            fournisseur: "Kone SA".to_string(),
        }
    }

    #[test]
    fn happy_un_rapport_complet_satisfait_lobligation() {
        let ascenseur = contrat("Entretien ascenseur", NatureDuContrat::FournitureReguliere);
        let nettoyage = contrat("Nettoyage communs", NatureDuContrat::FournitureReguliere);
        let rapports = vec![RapportDevaluation {
            exercice: 2026,
            ago_id: Uuid::new_v4(),
            contrats_evalues: vec![ascenseur.id, nettoyage.id],
        }];

        assert!(verifier_exercice(2026, &[ascenseur, nettoyage], &rapports).is_ok());
    }

    #[test]
    fn negative_sans_rapport_lexercice_est_en_defaut() {
        assert_eq!(
            verifier_exercice(2026, &[], &[]),
            Err(EvaluationManquante::RapportAbsent { exercice: 2026 })
        );
    }

    /// @security — un contrat oublié est un contrat que personne ne rediscute.
    ///
    /// C'est précisément ce que l'article veut empêcher : la reconduction
    /// tacite pendant dix ans d'un entretien que plus personne n'examine.
    #[test]
    fn security_un_contrat_oublie_est_signale_nominativement() {
        let ascenseur = contrat("Entretien ascenseur", NatureDuContrat::FournitureReguliere);
        let chauffage = contrat("Entretien chaudière", NatureDuContrat::FournitureReguliere);
        let rapports = vec![RapportDevaluation {
            exercice: 2026,
            ago_id: Uuid::new_v4(),
            contrats_evalues: vec![ascenseur.id],
        }];

        match verifier_exercice(2026, &[ascenseur, chauffage], &rapports) {
            Err(EvaluationManquante::ContratsNonEvalues { objets }) => {
                assert_eq!(objets.len(), 1);
                assert!(
                    objets[0].contains("Entretien chaudière") && objets[0].contains("Kone SA"),
                    "le rapport doit nommer le contrat ET son fournisseur : {objets:?}"
                );
            }
            autre => panic!("attendu un manquement : {autre:?}"),
        }
    }

    /// Les marchés ponctuels ne sont pas attendus au rapport.
    ///
    /// Une réfection de toiture n'est pas une fourniture régulière, et l'y
    /// inclure noierait le rapport sous des lignes qui n'appellent aucune
    /// décision de reconduction.
    #[test]
    fn happy_un_marche_ponctuel_nest_pas_attendu_au_rapport() {
        let toiture = contrat("Réfection toiture", NatureDuContrat::MarchePonctuel);
        let rapports = vec![RapportDevaluation {
            exercice: 2026,
            ago_id: Uuid::new_v4(),
            contrats_evalues: vec![],
        }];

        assert!(verifier_exercice(2026, &[toiture], &rapports).is_ok());
    }

    /// @security — pas de reconduction du rapport lui-même.
    ///
    /// « Soumettre à l'assemblée générale ordinaire » se lit chaque année. Un
    /// rapport présenté en 2025 ne couvre pas l'exercice 2026.
    #[test]
    fn security_un_rapport_de_lexercice_precedent_ne_couvre_pas_le_suivant() {
        let rapports = vec![RapportDevaluation {
            exercice: 2025,
            ago_id: Uuid::new_v4(),
            contrats_evalues: vec![],
        }];

        assert_eq!(
            verifier_exercice(2026, &[], &rapports),
            Err(EvaluationManquante::RapportAbsent { exercice: 2026 })
        );
    }

    #[test]
    fn happy_une_acp_sans_contrat_regulier_na_rien_a_evaluer() {
        let rapports = vec![RapportDevaluation {
            exercice: 2026,
            ago_id: Uuid::new_v4(),
            contrats_evalues: vec![],
        }];
        assert!(verifier_exercice(2026, &[], &rapports).is_ok());
    }
}
