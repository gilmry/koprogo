//! Les comptes bancaires de l'ACP.
//!
//! Art. 3.86 § 3, alinéa 6 :
//!
//! > « Ces fonds doivent être placés sur divers comptes, dont **obligatoirement
//! > un compte distinct pour le fonds de roulement et un compte distinct pour
//! > le fonds de réserve** ; tous ces comptes doivent être **ouverts au nom de
//! > l'association des copropriétaires**. »
//!
//! Les deux fonds étaient modélisés comme des soldes (ADR-0012). Ce qui
//! manquait, ce sont les **comptes** eux-mêmes : leur caractère distinct et
//! leur titulaire.
//!
//! C'est exactement la protection que l'article vise, et elle est double :
//!
//! 1. **des comptes distincts** empêchent de puiser dans le fonds de réserve
//!    pour payer les charges courantes. Un compte unique rend l'arbitrage
//!    invisible — il n'y a rien à franchir ;
//! 2. **au nom de l'ACP** empêche le syndic de mélanger les fonds d'une
//!    copropriété avec les siens ou avec ceux d'une autre. C'est le pendant
//!    bancaire du principe que l'ADR-0045 pose côté données : le patrimoine
//!    appartient à l'ACP, le syndic ne fait que l'administrer
//!    (Art. 3.89 § 5, 3°).
//!
//! L'article dit « divers comptes, **dont** obligatoirement » : il en impose
//! deux au minimum, il n'en interdit pas d'autres. Une ACP peut avoir un
//! compte de travaux en plus, et ce n'est pas une irrégularité.
//!
//! Voir issue #756 et ADR-0012.

use uuid::Uuid;

/// L'affectation d'un compte bancaire de l'ACP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AffectationDuCompte {
    /// Fonds de roulement : avances pour les dépenses périodiques.
    FondsDeRoulement,
    /// Fonds de réserve : apports pour les dépenses non périodiques.
    FondsDeReserve,
    /// Tout autre compte — l'article les autorise (« divers comptes »).
    Autre,
}

/// Un compte bancaire, tel que la conformité le regarde.
#[derive(Debug, Clone, PartialEq)]
pub struct CompteBancaire {
    pub id: Uuid,
    pub iban: String,
    pub affectation: AffectationDuCompte,
    /// Le titulaire déclaré du compte.
    ///
    /// C'est ce champ qui porte l'exigence « ouverts au nom de l'association
    /// des copropriétaires ».
    pub titulaire_acp_id: Option<Uuid>,
}

impl CompteBancaire {
    /// Le compte est-il bien ouvert au nom de cette ACP ?
    pub fn au_nom_de(&self, acp_id: Uuid) -> bool {
        self.titulaire_acp_id == Some(acp_id)
    }
}

/// Ce qui manque à une ACP pour être en règle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManquementBancaire {
    /// Aucun compte affecté au fonds de roulement.
    FondsDeRoulementSansCompte,
    /// Aucun compte affecté au fonds de réserve.
    FondsDeReserveSansCompte,
    /// Un seul compte sert aux deux fonds.
    ///
    /// Le manquement le plus fréquent, et le plus grave des trois : il n'y a
    /// alors rien à franchir pour puiser dans la réserve.
    FondsConfondusSurUnSeulCompte { iban: String },
    /// Un compte n'est pas ouvert au nom de l'ACP.
    CompteHorsDuNomDeLacp {
        iban: String,
        affectation: AffectationDuCompte,
    },
}

impl std::fmt::Display for ManquementBancaire {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FondsDeRoulementSansCompte => write!(
                f,
                "Art. 3.86 § 3 : aucun compte distinct pour le fonds de roulement."
            ),
            Self::FondsDeReserveSansCompte => write!(
                f,
                "Art. 3.86 § 3 : aucun compte distinct pour le fonds de réserve."
            ),
            Self::FondsConfondusSurUnSeulCompte { iban } => write!(
                f,
                "Art. 3.86 § 3 : le fonds de roulement et le fonds de réserve partagent le \
                 compte {iban}. Les comptes doivent être distincts — sinon rien n'empêche \
                 de puiser dans la réserve pour payer les charges courantes."
            ),
            Self::CompteHorsDuNomDeLacp { iban, affectation } => write!(
                f,
                "Art. 3.86 § 3 : le compte {iban} ({affectation:?}) n'est pas ouvert au nom \
                 de l'association des copropriétaires."
            ),
        }
    }
}

/// Vérifie les deux exigences de l'Art. 3.86 § 3, alinéa 6.
///
/// Renvoie **tous** les manquements, pas seulement le premier : un syndic qui
/// doit régulariser a besoin de la liste complète, pas d'un défaut à la fois.
pub fn verifier(comptes: &[CompteBancaire], acp_id: Uuid) -> Vec<ManquementBancaire> {
    let mut manquements = Vec::new();

    let roulement: Vec<&CompteBancaire> = comptes
        .iter()
        .filter(|c| c.affectation == AffectationDuCompte::FondsDeRoulement)
        .collect();
    let reserve: Vec<&CompteBancaire> = comptes
        .iter()
        .filter(|c| c.affectation == AffectationDuCompte::FondsDeReserve)
        .collect();

    if roulement.is_empty() {
        manquements.push(ManquementBancaire::FondsDeRoulementSansCompte);
    }
    if reserve.is_empty() {
        manquements.push(ManquementBancaire::FondsDeReserveSansCompte);
    }

    // Deux affectations sur un même IBAN : les fonds sont confondus.
    for compte_roulement in &roulement {
        if reserve.iter().any(|r| r.iban == compte_roulement.iban) {
            manquements.push(ManquementBancaire::FondsConfondusSurUnSeulCompte {
                iban: compte_roulement.iban.clone(),
            });
        }
    }

    for compte in comptes {
        if !compte.au_nom_de(acp_id) {
            manquements.push(ManquementBancaire::CompteHorsDuNomDeLacp {
                iban: compte.iban.clone(),
                affectation: compte.affectation,
            });
        }
    }

    manquements
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compte(iban: &str, affectation: AffectationDuCompte, acp: Option<Uuid>) -> CompteBancaire {
        CompteBancaire {
            id: Uuid::new_v4(),
            iban: iban.to_string(),
            affectation,
            titulaire_acp_id: acp,
        }
    }

    #[test]
    fn happy_deux_comptes_distincts_au_nom_de_lacp_sont_en_regle() {
        let acp = Uuid::new_v4();
        let comptes = vec![
            compte("BE68 5390 0754 7034", AffectationDuCompte::FondsDeRoulement, Some(acp)),
            compte("BE71 0961 2345 6769", AffectationDuCompte::FondsDeReserve, Some(acp)),
        ];
        assert!(verifier(&comptes, acp).is_empty());
    }

    /// L'article dit « divers comptes, DONT obligatoirement » : deux au
    /// minimum, pas au maximum.
    #[test]
    fn happy_un_troisieme_compte_nest_pas_une_irregularite() {
        let acp = Uuid::new_v4();
        let comptes = vec![
            compte("BE68 5390 0754 7034", AffectationDuCompte::FondsDeRoulement, Some(acp)),
            compte("BE71 0961 2345 6769", AffectationDuCompte::FondsDeReserve, Some(acp)),
            compte("BE62 5100 0754 7061", AffectationDuCompte::Autre, Some(acp)),
        ];
        assert!(verifier(&comptes, acp).is_empty());
    }

    /// @security — le manquement le plus fréquent et le plus grave.
    ///
    /// Un compte unique rend l'arbitrage invisible : il n'y a rien à franchir
    /// pour puiser dans la réserve.
    #[test]
    fn security_un_compte_unique_pour_les_deux_fonds_est_signale() {
        let acp = Uuid::new_v4();
        let comptes = vec![
            compte("BE68 5390 0754 7034", AffectationDuCompte::FondsDeRoulement, Some(acp)),
            compte("BE68 5390 0754 7034", AffectationDuCompte::FondsDeReserve, Some(acp)),
        ];
        let manquements = verifier(&comptes, acp);
        assert_eq!(
            manquements,
            vec![ManquementBancaire::FondsConfondusSurUnSeulCompte {
                iban: "BE68 5390 0754 7034".to_string()
            }]
        );
    }

    /// @security — le pendant bancaire de l'ADR-0045.
    ///
    /// Un compte au nom du cabinet mélange les fonds d'une copropriété avec
    /// ceux d'une autre, ou avec les siens.
    #[test]
    fn security_un_compte_au_nom_du_syndic_est_signale() {
        let acp = Uuid::new_v4();
        let comptes = vec![
            compte("BE68 5390 0754 7034", AffectationDuCompte::FondsDeRoulement, None),
            compte("BE71 0961 2345 6769", AffectationDuCompte::FondsDeReserve, Some(acp)),
        ];
        let manquements = verifier(&comptes, acp);
        assert_eq!(manquements.len(), 1);
        assert!(matches!(
            manquements[0],
            ManquementBancaire::CompteHorsDuNomDeLacp { .. }
        ));
    }

    /// @security — un compte au nom d'une AUTRE ACP est tout aussi refusé.
    #[test]
    fn security_un_compte_au_nom_dune_autre_acp_est_signale() {
        let acp = Uuid::new_v4();
        let voisine = Uuid::new_v4();
        let comptes = vec![
            compte("BE68 5390 0754 7034", AffectationDuCompte::FondsDeRoulement, Some(voisine)),
            compte("BE71 0961 2345 6769", AffectationDuCompte::FondsDeReserve, Some(acp)),
        ];
        assert_eq!(verifier(&comptes, acp).len(), 1);
    }

    #[test]
    fn negative_une_acp_sans_compte_cumule_les_deux_manquements() {
        let manquements = verifier(&[], Uuid::new_v4());
        assert_eq!(
            manquements,
            vec![
                ManquementBancaire::FondsDeRoulementSansCompte,
                ManquementBancaire::FondsDeReserveSansCompte
            ]
        );
    }

    /// La liste est complète, pas un défaut à la fois.
    ///
    /// Un syndic qui régularise a besoin de tout voir : corriger un point pour
    /// découvrir le suivant coûte un aller-retour bancaire à chaque fois.
    #[test]
    fn happy_tous_les_manquements_remontent_ensemble() {
        let acp = Uuid::new_v4();
        let comptes = vec![compte(
            "BE68 5390 0754 7034",
            AffectationDuCompte::FondsDeRoulement,
            None,
        )];
        let manquements = verifier(&comptes, acp);

        assert_eq!(manquements.len(), 2, "réserve absente ET compte hors du nom");
    }
}
