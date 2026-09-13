//! Les signatures du procès-verbal d'assemblée.
//!
//! Art. 3.87 § 10 :
//!
//! > « Le syndic rédige le procès-verbal des décisions prises par l'assemblée
//! > générale **avec indication des majorités obtenues et du nom des
//! > copropriétaires qui ont voté contre ou qui se sont abstenus**.
//! >
//! > **A la fin de la séance et après lecture**, ce procès-verbal est signé
//! > par **le président de l'assemblée générale**, par **le secrétaire désigné
//! > lors de l'ouverture de la séance** et par **tous les copropriétaires
//! > encore présents à ce moment** ou leurs mandataires. »
//!
//! Le procès-verbal était produit, avec ses majorités. Les signatures, elles,
//! n'étaient pas un état du domaine : un PV non signé était indiscernable d'un
//! PV signé, alors que la signature conditionne sa valeur probante.
//!
//! Quatre exigences, et chacune est piégeuse à sa façon :
//!
//! 1. **le président est un copropriétaire** (Art. 3.87 § 5, alinéa 1er). Un
//!    syndic qui préside — pratique courante et commode — vicie le PV ;
//! 2. **le secrétaire est désigné à l'ouverture**, donc avant les votes. Le
//!    désigner après coup, quand il s'agit de trouver un signataire, ne
//!    respecte pas l'article ;
//! 3. **tous les copropriétaires encore présents** signent. Pas tous les
//!    convoqués, pas tous les votants : ceux qui sont encore là à la fin de la
//!    séance. Les partis en cours de route ne manquent à rien ;
//! 4. **à la fin de la séance et après lecture** : la signature est
//!    contemporaine, pas différée à la semaine suivante.
//!
//! La distinction du 3° compte : exiger la signature d'un copropriétaire parti
//! bloquerait un PV parfaitement régulier, et se contenter du président
//! laisserait passer un PV qui ne l'est pas.
//!
//! Voir issue #750 et [`super::consignation_pv`], qui traite la suite —
//! consignation au registre et transmission sous trente jours.

use uuid::Uuid;

/// Qui a présidé, tenu la plume, et qui était encore là.
#[derive(Debug, Clone, PartialEq)]
pub struct SeanceCloturee {
    /// Le président, qui doit être copropriétaire (Art. 3.87 § 5).
    pub president: Uuid,
    pub president_est_coproprietaire: bool,
    /// Le secrétaire, désigné à l'ouverture de la séance.
    pub secretaire: Option<Uuid>,
    /// Les copropriétaires — ou leurs mandataires — encore présents à la
    /// clôture.
    pub presents_a_la_cloture: Vec<Uuid>,
}

/// Ce qui manque à un procès-verbal pour être valablement signé.
#[derive(Debug, Clone, PartialEq)]
pub enum SignatureManquante {
    /// Le président n'a pas signé.
    President { president: Uuid },
    /// Aucun secrétaire n'a été désigné à l'ouverture.
    SecretaireNonDesigne,
    /// Le secrétaire n'a pas signé.
    Secretaire { secretaire: Uuid },
    /// Des copropriétaires présents à la clôture n'ont pas signé.
    CoproprietairesPresents { manquants: Vec<Uuid> },
    /// Le président n'est pas copropriétaire — souvent, c'est le syndic.
    PresidentNonCoproprietaire { president: Uuid },
}

impl std::fmt::Display for SignatureManquante {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::President { president } => {
                write!(
                    f,
                    "Art. 3.87 § 10 : le président {president} n'a pas signé."
                )
            }
            Self::SecretaireNonDesigne => write!(
                f,
                "Art. 3.87 § 10 : aucun secrétaire n'a été désigné à l'ouverture de la séance."
            ),
            Self::Secretaire { secretaire } => {
                write!(
                    f,
                    "Art. 3.87 § 10 : le secrétaire {secretaire} n'a pas signé."
                )
            }
            Self::CoproprietairesPresents { manquants } => write!(
                f,
                "Art. 3.87 § 10 : {} copropriétaire(s) encore présent(s) à la clôture n'ont \
                 pas signé.",
                manquants.len()
            ),
            Self::PresidentNonCoproprietaire { president } => write!(
                f,
                "Art. 3.87 § 5 : {president} a présidé sans être copropriétaire. \
                 « L'assemblée générale est présidée par un copropriétaire » — un syndic \
                 qui préside vicie le procès-verbal."
            ),
        }
    }
}

/// Vérifie les signatures d'un procès-verbal.
///
/// `signataires` sont ceux qui ont effectivement signé, à la fin de la séance.
///
/// Renvoie **tous** les manquements : un PV qu'on doit faire resigner appelle
/// la liste complète, pas un nom à la fois.
pub fn verifier_signatures(
    seance: &SeanceCloturee,
    signataires: &[Uuid],
) -> Vec<SignatureManquante> {
    let mut manquements = Vec::new();

    if !seance.president_est_coproprietaire {
        manquements.push(SignatureManquante::PresidentNonCoproprietaire {
            president: seance.president,
        });
    }
    if !signataires.contains(&seance.president) {
        manquements.push(SignatureManquante::President {
            president: seance.president,
        });
    }

    match seance.secretaire {
        None => manquements.push(SignatureManquante::SecretaireNonDesigne),
        Some(secretaire) if !signataires.contains(&secretaire) => {
            manquements.push(SignatureManquante::Secretaire { secretaire })
        }
        Some(_) => {}
    }

    let manquants: Vec<Uuid> = seance
        .presents_a_la_cloture
        .iter()
        .filter(|p| !signataires.contains(p))
        .copied()
        .collect();
    if !manquants.is_empty() {
        manquements.push(SignatureManquante::CoproprietairesPresents { manquants });
    }

    manquements
}

/// Le procès-verbal est-il valablement signé ?
pub fn pv_valablement_signe(seance: &SeanceCloturee, signataires: &[Uuid]) -> bool {
    verifier_signatures(seance, signataires).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seance(president: Uuid, secretaire: Uuid, presents: Vec<Uuid>) -> SeanceCloturee {
        SeanceCloturee {
            president,
            president_est_coproprietaire: true,
            secretaire: Some(secretaire),
            presents_a_la_cloture: presents,
        }
    }

    #[test]
    fn happy_un_pv_signe_par_tous_est_valable() {
        let president = Uuid::new_v4();
        let secretaire = Uuid::new_v4();
        let present = Uuid::new_v4();
        let s = seance(president, secretaire, vec![present]);

        assert!(pv_valablement_signe(&s, &[president, secretaire, present]));
    }

    #[test]
    fn negative_un_pv_que_le_president_na_pas_signe_est_incomplet() {
        let president = Uuid::new_v4();
        let secretaire = Uuid::new_v4();
        let s = seance(president, secretaire, vec![]);

        assert_eq!(
            verifier_signatures(&s, &[secretaire]),
            vec![SignatureManquante::President { president }]
        );
    }

    /// @security — le piège le plus courant : le syndic préside.
    ///
    /// Art. 3.87 § 5 : « L'assemblée générale est présidée par un
    /// copropriétaire. » C'est commode et c'est vicié.
    #[test]
    fn security_un_president_non_coproprietaire_vicie_le_pv() {
        let syndic = Uuid::new_v4();
        let secretaire = Uuid::new_v4();
        let s = SeanceCloturee {
            president: syndic,
            president_est_coproprietaire: false,
            secretaire: Some(secretaire),
            presents_a_la_cloture: vec![],
        };

        let manquements = verifier_signatures(&s, &[syndic, secretaire]);
        assert_eq!(
            manquements,
            vec![SignatureManquante::PresidentNonCoproprietaire { president: syndic }],
            "il a bien signé, mais il n'aurait pas dû présider"
        );
    }

    /// Le secrétaire est désigné **à l'ouverture**. Ne pas l'avoir désigné est
    /// un manquement distinct de ne pas l'avoir fait signer.
    #[test]
    fn negative_sans_secretaire_designe_le_pv_est_incomplet() {
        let president = Uuid::new_v4();
        let s = SeanceCloturee {
            president,
            president_est_coproprietaire: true,
            secretaire: None,
            presents_a_la_cloture: vec![],
        };

        assert_eq!(
            verifier_signatures(&s, &[president]),
            vec![SignatureManquante::SecretaireNonDesigne]
        );
    }

    /// « Tous les copropriétaires **encore présents à ce moment** » : ceux qui
    /// sont partis en cours de séance ne manquent à rien.
    ///
    /// Exiger leur signature bloquerait un PV parfaitement régulier.
    #[test]
    fn happy_un_coproprietaire_parti_en_cours_de_seance_na_pas_a_signer() {
        let president = Uuid::new_v4();
        let secretaire = Uuid::new_v4();
        let reste = Uuid::new_v4();
        let _parti_plus_tot = Uuid::new_v4();

        // `presents_a_la_cloture` ne contient que ceux encore là.
        let s = seance(president, secretaire, vec![reste]);

        assert!(pv_valablement_signe(&s, &[president, secretaire, reste]));
    }

    /// @security — mais ceux qui sont restés doivent bien signer.
    #[test]
    fn security_un_present_a_la_cloture_qui_ne_signe_pas_est_releve() {
        let president = Uuid::new_v4();
        let secretaire = Uuid::new_v4();
        let recalcitrant = Uuid::new_v4();
        let s = seance(president, secretaire, vec![recalcitrant]);

        assert_eq!(
            verifier_signatures(&s, &[president, secretaire]),
            vec![SignatureManquante::CoproprietairesPresents {
                manquants: vec![recalcitrant]
            }]
        );
    }

    /// La liste est complète, pas un manquement à la fois.
    ///
    /// Un PV qu'on doit faire resigner appelle tous les noms d'un coup : les
    /// signataires ne se réunissent pas deux fois.
    #[test]
    fn negative_tous_les_manquements_remontent_ensemble() {
        let syndic = Uuid::new_v4();
        let present = Uuid::new_v4();
        let s = SeanceCloturee {
            president: syndic,
            president_est_coproprietaire: false,
            secretaire: None,
            presents_a_la_cloture: vec![present],
        };

        let manquements = verifier_signatures(&s, &[]);
        assert_eq!(
            manquements.len(),
            4,
            "président non copropriétaire, président non signataire, \
             secrétaire non désigné, présent non signataire"
        );
    }

    #[test]
    fn happy_une_seance_sans_personne_a_la_cloture_ne_demande_que_deux_signatures() {
        // Cas réel : tout le monde est parti sauf le bureau.
        let president = Uuid::new_v4();
        let secretaire = Uuid::new_v4();
        let s = seance(president, secretaire, vec![]);

        assert!(pv_valablement_signe(&s, &[president, secretaire]));
    }
}
