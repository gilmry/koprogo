//! La mention du numéro d'entreprise sur les documents de l'ACP.
//!
//! Art. 3.86 § 1er, alinéa 4 :
//!
//! > « **Tous les documents** émanant de l'association des copropriétaires
//! > mentionnent le **numéro d'entreprise** de ladite association. »
//!
//! « Tous » est sans exception : convocations, procès-verbaux, appels de
//! fonds, états datés, décomptes, courriers de relance. Le champ existait sur
//! l'ACP depuis longtemps ; ce qui manquait, c'est que les documents le
//! portent effectivement.
//!
//! **Le contrôle doit être exhaustif par construction, pas exporteur par
//! exporteur.** Vérifier chaque exporteur à la main garantit qu'on oubliera le
//! prochain — celui qu'un développeur ajoutera dans six mois sans avoir lu cet
//! article. D'où [`verifier_les_documents`], qui prend la liste des documents
//! produits et refuse celui qui omet la mention, et le test d'exhaustivité qui
//! l'accompagne.
//!
//! Un lien avec la personnalité juridique, qui n'est pas anodin : le numéro
//! d'entreprise est attribué à l'ACP quand elle acquiert la personnalité
//! (Art. 3.86 § 1er). Une ACP qui n'en a pas encore n'est pas fautive de ne
//! pas le mentionner — elle est incomplète, et le troisième état le dit,
//! comme ailleurs.
//!
//! Voir issue #748 et [`super::personnalite_juridique`].

/// Un document produit par l'ACP, du point de vue de cette obligation.
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentProduit {
    /// Ce qu'il est, pour que le manquement soit nommable.
    pub nature: &'static str,
    /// Le texte rendu, dans lequel la mention doit apparaître.
    pub contenu: String,
}

/// Le résultat de la vérification.
#[derive(Debug, Clone, PartialEq)]
pub enum MentionNumeroEntreprise {
    /// La mention est présente sur tous les documents.
    Presente,
    /// L'ACP n'a pas encore de numéro d'entreprise.
    ///
    /// Elle n'est pas fautive : le numéro est attribué avec la personnalité
    /// juridique. Le distinguer d'un manquement évite d'accuser une ACP en
    /// cours de constitution.
    NumeroNonAttribue,
    /// Des documents omettent la mention.
    Manquante { documents: Vec<&'static str> },
}

impl std::fmt::Display for MentionNumeroEntreprise {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Presente => write!(f, "Numéro d'entreprise mentionné sur tous les documents"),
            Self::NumeroNonAttribue => write!(
                f,
                "Numéro d'entreprise non attribué : l'ACP n'a pas encore acquis la \
                 personnalité juridique (Art. 3.86 § 1er)"
            ),
            Self::Manquante { documents } => write!(
                f,
                "Art. 3.86 § 1er : le numéro d'entreprise manque sur {} — « tous les \
                 documents émanant de l'association des copropriétaires mentionnent le \
                 numéro d'entreprise ».",
                documents.join(", ")
            ),
        }
    }
}

/// Vérifie que tous les documents portent le numéro d'entreprise.
///
/// La comparaison ignore les espaces : un numéro s'écrit « BE 0123.456.789 »
/// ou « BE0123456789 » selon les habitudes, et refuser la seconde forme serait
/// signaler un manquement là où il n'y en a pas.
pub fn verifier_les_documents(
    numero_entreprise: Option<&str>,
    documents: &[DocumentProduit],
) -> MentionNumeroEntreprise {
    let Some(numero) = numero_entreprise else {
        return MentionNumeroEntreprise::NumeroNonAttribue;
    };

    let normalise = |s: &str| {
        s.chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
    };
    let attendu = normalise(numero);

    let manquants: Vec<&'static str> = documents
        .iter()
        .filter(|d| !normalise(&d.contenu).contains(&attendu))
        .map(|d| d.nature)
        .collect();

    if manquants.is_empty() {
        MentionNumeroEntreprise::Presente
    } else {
        MentionNumeroEntreprise::Manquante {
            documents: manquants,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document(nature: &'static str, contenu: &str) -> DocumentProduit {
        DocumentProduit {
            nature,
            contenu: contenu.to_string(),
        }
    }

    const NUMERO: &str = "BE 0123.456.789";

    #[test]
    fn happy_un_document_portant_la_mention_passe() {
        let docs = vec![document(
            "convocation",
            "ACP Résidence du Parc — BE 0123.456.789\nConvocation à l'AGO...",
        )];
        assert_eq!(
            verifier_les_documents(Some(NUMERO), &docs),
            MentionNumeroEntreprise::Presente
        );
    }

    /// Un numéro s'écrit avec ou sans séparateurs selon les habitudes.
    ///
    /// Refuser la forme compacte signalerait un manquement là où il n'y en a
    /// pas.
    #[test]
    fn happy_la_forme_compacte_est_acceptee() {
        let docs = vec![document("état daté", "ACP — BE0123456789\nÉtat daté...")];
        assert_eq!(
            verifier_les_documents(Some(NUMERO), &docs),
            MentionNumeroEntreprise::Presente
        );
    }

    #[test]
    fn negative_un_document_sans_mention_est_signale() {
        let docs = vec![document(
            "appel de fonds",
            "Appel de fonds T1 2026\nMontant : 1200 €",
        )];
        assert_eq!(
            verifier_les_documents(Some(NUMERO), &docs),
            MentionNumeroEntreprise::Manquante {
                documents: vec!["appel de fonds"]
            }
        );
    }

    /// « Tous les documents » : un seul oubli parmi cinq suffit.
    #[test]
    fn security_un_seul_oubli_parmi_cinq_est_releve() {
        let avec = format!("En-tête — {NUMERO}\nCorps du document");
        let docs = vec![
            document("convocation", &avec),
            document("procès-verbal", &avec),
            document("appel de fonds", "Appel de fonds T1 2026"),
            document("état daté", &avec),
            document("décompte", &avec),
        ];
        assert_eq!(
            verifier_les_documents(Some(NUMERO), &docs),
            MentionNumeroEntreprise::Manquante {
                documents: vec!["appel de fonds"]
            }
        );
    }

    #[test]
    fn negative_plusieurs_oublis_remontent_ensemble() {
        let docs = vec![
            document("convocation", "Convocation"),
            document("relance", "Relance de paiement"),
        ];
        match verifier_les_documents(Some(NUMERO), &docs) {
            MentionNumeroEntreprise::Manquante { documents } => {
                assert_eq!(documents.len(), 2);
            }
            autre => panic!("attendu un manquement : {autre}"),
        }
    }

    /// @edge — une ACP sans numéro n'est pas fautive.
    ///
    /// Le numéro est attribué avec la personnalité juridique
    /// (Art. 3.86 § 1er). Signaler un manquement accuserait une ACP en cours
    /// de constitution de ne pas mentionner ce qu'elle n'a pas.
    #[test]
    fn edge_une_acp_sans_numero_nest_pas_en_manquement() {
        let docs = vec![document("convocation", "Convocation à l'AGO")];
        assert_eq!(
            verifier_les_documents(None, &docs),
            MentionNumeroEntreprise::NumeroNonAttribue
        );
    }

    #[test]
    fn happy_aucun_document_produit_ne_pose_pas_de_probleme() {
        assert_eq!(
            verifier_les_documents(Some(NUMERO), &[]),
            MentionNumeroEntreprise::Presente
        );
    }

    /// @security — un numéro voisin ne passe pas pour le bon.
    ///
    /// Le cas se produit après une reprise de dossier : l'en-tête garde le
    /// numéro de l'ACP précédente.
    #[test]
    fn security_le_numero_dune_autre_acp_ne_satisfait_pas_lobligation() {
        let docs = vec![document(
            "convocation",
            "ACP voisine — BE 0987.654.321\nConvocation",
        )];
        assert!(matches!(
            verifier_les_documents(Some(NUMERO), &docs),
            MentionNumeroEntreprise::Manquante { .. }
        ));
    }
}
