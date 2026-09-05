//! La solidarité des titulaires de droits réels pour les charges.
//!
//! Art. 3.86 § 3, alinéa 8 :
//!
//! > « Lorsque la propriété d'un lot est grevée d'un droit d'usufruit, les
//! > titulaires des droits réels sont **solidairement tenus** du paiement de
//! > ces charges. »
//!
//! Ce que ça change concrètement : quand un lot est démembré, l'ACP n'a pas à
//! répartir sa créance entre l'usufruitier et le nu-propriétaire, ni à
//! poursuivre les deux. Elle réclame **le tout à l'un d'eux**, à son choix, et
//! c'est ensuite entre eux que le partage se règle — hors du champ de la
//! copropriété.
//!
//! L'intérêt pratique est direct : sans la solidarité, un syndic qui ne
//! parvient pas à joindre l'usufruitier reste bloqué alors que le
//! nu-propriétaire est solvable et joignable. Avec elle, il poursuit celui
//! qu'il peut atteindre.
//!
//! La règle ne joue **que** pour l'usufruit. L'indivision ordinaire, elle,
//! répartit au prorata des quotes-parts : deux indivisaires à moitié doivent
//! chacun la moitié, et l'ACP ne peut pas réclamer le tout à l'un d'eux.
//! Confondre les deux ferait réclamer à un indivisaire une somme qu'il ne
//! doit pas.
//!
//! Voir issue #739.

use super::unit_owner::OwnershipType;
use rust_decimal::Decimal;
use uuid::Uuid;

/// Un titulaire de droit réel sur un lot, tel que le recouvrement le voit.
#[derive(Debug, Clone, PartialEq)]
pub struct Titulaire {
    pub owner_id: Uuid,
    pub nature: OwnershipType,
    /// Quote-part de détention (0 à 1), qui sert à répartir hors solidarité.
    pub quote_part: Decimal,
}

/// Ce qu'un titulaire doit à l'ACP pour une charge donnée.
#[derive(Debug, Clone, PartialEq)]
pub struct Obligation {
    pub owner_id: Uuid,
    /// Somme réclamable à ce titulaire.
    pub montant: Decimal,
    /// Peut-on lui réclamer le tout, ou seulement sa part ?
    ///
    /// `true` uniquement en cas de démembrement par usufruit
    /// (Art. 3.86 § 3 al. 8). Les obligations solidaires d'un même lot
    /// portent chacune le montant **entier** : ce n'est pas une somme à
    /// additionner, mais un choix offert à l'ACP.
    pub solidaire: bool,
}

/// Le lot est-il grevé d'un usufruit ?
///
/// Il l'est dès qu'un usufruitier y figure. Le nu-propriétaire seul ne suffit
/// pas — sans usufruitier en face, il est plein propriétaire de fait pour ce
/// qui nous occupe.
fn greve_dusufruit(titulaires: &[Titulaire]) -> bool {
    titulaires
        .iter()
        .any(|t| t.nature == OwnershipType::Usufruct)
}

/// Répartit une charge entre les titulaires d'un lot.
///
/// En cas d'usufruit, chaque titulaire de droit réel doit **le tout** : ce
/// sont des obligations solidaires, pas des parts. En dehors de ce cas, chacun
/// doit sa quote-part.
pub fn repartir_charge(titulaires: &[Titulaire], montant: Decimal) -> Vec<Obligation> {
    if titulaires.is_empty() {
        return Vec::new();
    }

    if greve_dusufruit(titulaires) {
        return titulaires
            .iter()
            .map(|t| Obligation {
                owner_id: t.owner_id,
                montant,
                solidaire: true,
            })
            .collect();
    }

    titulaires
        .iter()
        .map(|t| Obligation {
            owner_id: t.owner_id,
            montant: (montant * t.quote_part).round_dp(2),
            solidaire: false,
        })
        .collect()
}

/// Les titulaires que l'ACP peut poursuivre pour la totalité d'une charge.
///
/// Vide quand il n'y a pas de solidarité : c'est alors le montant de chacun
/// qui fait foi, pas la faculté de choisir.
pub fn poursuivables_pour_le_tout(titulaires: &[Titulaire]) -> Vec<Uuid> {
    if !greve_dusufruit(titulaires) {
        return Vec::new();
    }
    titulaires.iter().map(|t| t.owner_id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn titulaire(nature: OwnershipType, quote_part: Decimal) -> Titulaire {
        Titulaire {
            owner_id: Uuid::new_v4(),
            nature,
            quote_part,
        }
    }

    #[test]
    fn happy_un_plein_proprietaire_doit_tout_sans_solidarite() {
        let titulaires = vec![titulaire(OwnershipType::FullOwner, dec!(1))];
        let obligations = repartir_charge(&titulaires, dec!(1200));

        assert_eq!(obligations.len(), 1);
        assert_eq!(obligations[0].montant, dec!(1200));
        assert!(
            !obligations[0].solidaire,
            "seul, il n'y a personne avec qui être solidaire"
        );
    }

    #[test]
    fn happy_lusufruit_rend_les_deux_titulaires_tenus_du_tout() {
        let titulaires = vec![
            titulaire(OwnershipType::Usufruct, dec!(0.4)),
            titulaire(OwnershipType::BareOwner, dec!(0.6)),
        ];
        let obligations = repartir_charge(&titulaires, dec!(1200));

        assert_eq!(obligations.len(), 2);
        for obligation in &obligations {
            assert_eq!(
                obligation.montant,
                dec!(1200),
                "chacun doit LE TOUT : les obligations solidaires ne s'additionnent pas"
            );
            assert!(obligation.solidaire);
        }
    }

    #[test]
    fn happy_lindivision_ordinaire_repartit_au_prorata() {
        let titulaires = vec![
            titulaire(OwnershipType::Indivisaire, dec!(0.5)),
            titulaire(OwnershipType::Indivisaire, dec!(0.5)),
        ];
        let obligations = repartir_charge(&titulaires, dec!(1200));

        for obligation in &obligations {
            assert_eq!(obligation.montant, dec!(600));
            assert!(
                !obligation.solidaire,
                "l'Art. 3.86 § 3 al. 8 ne vise que l'usufruit : réclamer le tout \
                 à un indivisaire lui ferait payer ce qu'il ne doit pas"
            );
        }
    }

    #[test]
    fn edge_un_nu_proprietaire_seul_nest_pas_solidaire() {
        // Sans usufruitier en face, il est plein propriétaire de fait pour ce
        // qui nous occupe : il n'y a pas de démembrement effectif.
        let titulaires = vec![titulaire(OwnershipType::BareOwner, dec!(1))];
        let obligations = repartir_charge(&titulaires, dec!(1200));

        assert!(!obligations[0].solidaire);
        assert!(poursuivables_pour_le_tout(&titulaires).is_empty());
    }

    #[test]
    fn happy_le_syndic_choisit_qui_poursuivre_en_cas_dusufruit() {
        // L'intérêt pratique de la règle : si l'usufruitier est injoignable,
        // le syndic poursuit le nu-propriétaire pour la totalité.
        let usufruitier = titulaire(OwnershipType::Usufruct, dec!(0.4));
        let nu_proprietaire = titulaire(OwnershipType::BareOwner, dec!(0.6));
        let attendus = vec![usufruitier.owner_id, nu_proprietaire.owner_id];

        let poursuivables = poursuivables_pour_le_tout(&[usufruitier, nu_proprietaire]);

        assert_eq!(poursuivables, attendus);
    }

    #[test]
    fn negative_aucun_titulaire_ne_produit_aucune_obligation() {
        assert!(repartir_charge(&[], dec!(1200)).is_empty());
    }

    #[test]
    fn edge_larrondi_de_la_repartition_reste_au_centime() {
        // 1000 / 3 ne tombe pas juste : on veut deux décimales exactes, pas
        // une dérive flottante (ADR-0007).
        let tiers = dec!(0.333333);
        let titulaires = vec![
            titulaire(OwnershipType::Indivisaire, tiers),
            titulaire(OwnershipType::Indivisaire, tiers),
            titulaire(OwnershipType::Indivisaire, tiers),
        ];
        let obligations = repartir_charge(&titulaires, dec!(1000));

        for obligation in &obligations {
            assert_eq!(obligation.montant, dec!(333.33));
        }
    }
}
