//! Le commissaire aux comptes.
//!
//! Art. 3.91, en une seule phrase :
//!
//! > « L'assemblée générale désigne **annuellement** un commissaire aux
//! > comptes ou un **collège** de commissaires aux comptes, **copropriétaires
//! > ou non**, qui **contrôlent les comptes** de l'association des
//! > copropriétaires, dont les compétences et obligations sont déterminées par
//! > le **règlement d'ordre intérieur**. »
//!
//! Le domaine n'en portait qu'une seule occurrence. C'est pourtant un organe
//! de l'ACP au même titre que l'assemblée, le syndic et le conseil de
//! copropriété — et sans lui, la comptabilité que ce logiciel tient n'a aucun
//! organe de vérification.
//!
//! Trois choses que la phrase dit et qu'il ne faut pas raboter :
//!
//! 1. **annuellement** — le mandat n'est pas reconduit tacitement. Une
//!    désignation qui date de deux exercices n'en est plus une ;
//! 2. **ou un collège** — la fonction peut être partagée, et le modèle doit
//!    donc accepter plusieurs titulaires pour un même exercice ;
//! 3. **copropriétaires ou non** — un expert-comptable externe est parfaitement
//!    admis. Exiger la qualité de copropriétaire serait ajouter une condition
//!    que la loi ne pose pas.
//!
//! Le seul empêchement vient d'ailleurs : Art. 3.89 § 9, « au sein d'une même
//! association de copropriétaires, un syndic ne peut être en même temps ni
//! membre du conseil de copropriété ni commissaire aux comptes ». On ne se
//! contrôle pas soi-même.
//!
//! Les compétences précises relèvent du ROI, donc du paramétrage de chaque
//! ACP, et non du code.
//!
//! Voir issue #754.

use uuid::Uuid;

/// Ce qui empêche une désignation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesignationRefusee {
    /// Art. 3.89 § 9 : on ne contrôle pas ses propres comptes.
    LeSyndicNePeutPasControlerSesComptes,
    /// Un collège vide n'est pas un collège.
    CollegeVide,
    /// Deux fois la même personne dans le même collège.
    DoublonDansLeCollege { titulaire: Uuid },
}

impl std::fmt::Display for DesignationRefusee {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LeSyndicNePeutPasControlerSesComptes => write!(
                f,
                "Art. 3.89 § 9 : le syndic ne peut pas être commissaire aux comptes de \
                 l'ACP qu'il gère."
            ),
            Self::CollegeVide => write!(
                f,
                "Art. 3.91 : l'assemblée doit désigner au moins un commissaire aux comptes."
            ),
            Self::DoublonDansLeCollege { titulaire } => write!(
                f,
                "Art. 3.91 : {titulaire} figure deux fois dans le collège."
            ),
        }
    }
}

/// La désignation d'un commissaire, ou d'un collège, pour un exercice.
#[derive(Debug, Clone, PartialEq)]
pub struct Commissariat {
    /// Un ou plusieurs titulaires — l'article prévoit expressément le collège.
    pub titulaires: Vec<Uuid>,
    /// L'exercice contrôlé. La désignation est **annuelle** : elle ne vaut que
    /// pour celui-là.
    pub exercice: i32,
}

impl Commissariat {
    /// Désigne le ou les commissaires pour un exercice.
    ///
    /// `syndic_owner_id` est l'identifiant du syndic **s'il est aussi
    /// copropriétaire** — le seul cas où il pourrait figurer dans la liste.
    pub fn designer(
        titulaires: Vec<Uuid>,
        exercice: i32,
        syndic_owner_id: Option<Uuid>,
    ) -> Result<Self, DesignationRefusee> {
        if titulaires.is_empty() {
            return Err(DesignationRefusee::CollegeVide);
        }
        if let Some(syndic) = syndic_owner_id {
            if titulaires.contains(&syndic) {
                return Err(DesignationRefusee::LeSyndicNePeutPasControlerSesComptes);
            }
        }
        for (i, titulaire) in titulaires.iter().enumerate() {
            if titulaires[i + 1..].contains(titulaire) {
                return Err(DesignationRefusee::DoublonDansLeCollege {
                    titulaire: *titulaire,
                });
            }
        }
        Ok(Self {
            titulaires,
            exercice,
        })
    }

    pub fn est_un_college(&self) -> bool {
        self.titulaires.len() > 1
    }

    /// Cette désignation couvre-t-elle l'exercice demandé ?
    ///
    /// Elle ne couvre que le sien : « désigne annuellement ». Une désignation
    /// qui date de l'exercice précédent ne vaut pas pour celui-ci, faute de
    /// reconduction tacite.
    pub fn couvre(&self, exercice: i32) -> bool {
        self.exercice == exercice
    }
}

/// L'ACP a-t-elle un commissaire pour cet exercice ?
///
/// Le manque n'est pas une faute du syndic mais un défaut de l'assemblée : les
/// comptes de l'exercice n'ont alors personne pour les contrôler.
pub fn commissariat_de_lexercice(
    designations: &[Commissariat],
    exercice: i32,
) -> Option<&Commissariat> {
    designations.iter().find(|d| d.couvre(exercice))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_un_commissaire_unique_est_valable() {
        let c = Commissariat::designer(vec![Uuid::new_v4()], 2026, None).unwrap();
        assert!(!c.est_un_college());
    }

    /// L'article prévoit expressément le collège.
    #[test]
    fn happy_un_college_de_trois_est_valable() {
        let c = Commissariat::designer(
            vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()],
            2026,
            None,
        )
        .unwrap();
        assert!(c.est_un_college());
        assert_eq!(c.titulaires.len(), 3);
    }

    /// « Copropriétaires ou non » : un expert externe est admis, et rien dans
    /// la désignation n'exige la qualité de copropriétaire.
    #[test]
    fn happy_un_expert_externe_peut_etre_designe() {
        let externe = Uuid::new_v4();
        let c = Commissariat::designer(vec![externe], 2026, Some(Uuid::new_v4())).unwrap();
        assert_eq!(c.titulaires, vec![externe]);
    }

    /// @security — Art. 3.89 § 9 : on ne contrôle pas ses propres comptes.
    #[test]
    fn security_le_syndic_ne_peut_pas_controler_ses_propres_comptes() {
        let syndic = Uuid::new_v4();
        assert_eq!(
            Commissariat::designer(vec![Uuid::new_v4(), syndic], 2026, Some(syndic)),
            Err(DesignationRefusee::LeSyndicNePeutPasControlerSesComptes)
        );
    }

    #[test]
    fn negative_un_college_vide_est_refuse() {
        assert_eq!(
            Commissariat::designer(vec![], 2026, None),
            Err(DesignationRefusee::CollegeVide)
        );
    }

    #[test]
    fn negative_un_doublon_dans_le_college_est_refuse() {
        let deux_fois = Uuid::new_v4();
        assert_eq!(
            Commissariat::designer(vec![deux_fois, Uuid::new_v4(), deux_fois], 2026, None),
            Err(DesignationRefusee::DoublonDansLeCollege {
                titulaire: deux_fois
            })
        );
    }

    // ── Le caractère annuel ────────────────────────────────────────

    #[test]
    fn happy_une_designation_couvre_son_exercice() {
        let designations =
            vec![Commissariat::designer(vec![Uuid::new_v4()], 2026, None).unwrap()];
        assert!(commissariat_de_lexercice(&designations, 2026).is_some());
    }

    /// @security — pas de reconduction tacite.
    ///
    /// « Désigne annuellement » : une désignation de l'exercice précédent ne
    /// vaut pas pour celui-ci, et les comptes de 2027 n'ont alors personne
    /// pour les contrôler.
    #[test]
    fn security_une_designation_ne_se_reconduit_pas_tacitement() {
        let designations =
            vec![Commissariat::designer(vec![Uuid::new_v4()], 2026, None).unwrap()];
        assert!(
            commissariat_de_lexercice(&designations, 2027).is_none(),
            "la désignation de 2026 ne couvre pas 2027"
        );
    }

    #[test]
    fn happy_plusieurs_exercices_se_suivent_sans_se_confondre() {
        let designations = vec![
            Commissariat::designer(vec![Uuid::new_v4()], 2025, None).unwrap(),
            Commissariat::designer(vec![Uuid::new_v4()], 2026, None).unwrap(),
        ];
        assert_ne!(
            commissariat_de_lexercice(&designations, 2025)
                .unwrap()
                .titulaires,
            commissariat_de_lexercice(&designations, 2026)
                .unwrap()
                .titulaires
        );
    }
}
