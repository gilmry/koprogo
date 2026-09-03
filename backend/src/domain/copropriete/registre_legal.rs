//! Le registre des invariants légaux, exécutable.
//!
//! Le RFC-0002 recense les obligations computables du chapitre « copropriété »
//! du Code civil et dit, pour chacune, si le domaine la porte. Tant qu'il
//! restait en prose, il se périmait exactement comme
//! `docs/AUDIT_CONFORMITE_JURIDIQUE.md` s'était périmé : rien ne le reliait au
//! code.
//!
//! Ce module est ce lien. Il déclare chaque invariant avec **l'article qui le
//! fonde**, **le module qui le porte** et **le test qui le nomme**, et un test
//! d'intégrité vérifie que ces trois choses existent encore. Le jour où
//! quelqu'un supprime un test ou renomme un module, le registre ne se
//! contente pas de mentir : il échoue.
//!
//! ## Ce que ce registre ne fait pas
//!
//! Il ne prouve pas qu'un invariant est **correctement** implémenté — c'est le
//! travail des tests eux-mêmes, qui citent leur article et vérifient les
//! bords. Il prouve qu'il est **encore là**. La distinction compte : une
//! couverture qui se mesure elle-même n'est pas une garantie de justesse, et
//! le dire ici évite de la lire comme telle.
//!
//! Il ne couvre pas non plus tout le chapitre. Les articles hors périmètre
//! applicatif — dissolution (3.97), liquidation (3.98), transcription des
//! actes (3.99) — n'y figurent pas, et l'ADR-0010 diffère les associations
//! partielles en v0.2.0.
//!
//! ## Comment il sert
//!
//! Le rapport de conformité se génère depuis cette liste et s'adresse à un
//! juriste, pas à un développeur : il répond à « que dit la loi, et où le code
//! y répond ? », dans l'ordre des articles.
//!
//! Voir RFC-0002 et le lot J8 du WBS.

/// Un invariant légal porté par le domaine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvariantLegal {
    /// L'article qui le fonde, tel qu'on le cite dans un courrier.
    pub article: &'static str,
    /// Ce que la loi exige, en une phrase lisible par un juriste.
    pub obligation: &'static str,
    /// Le fichier du domaine qui le porte, relatif à `src/`.
    pub porte_par: &'static str,
    /// Un test qui le nomme, pour qu'on puisse aller le lire.
    pub atteste_par: &'static str,
}

/// Le registre, dans l'ordre des articles.
pub const REGISTRE: &[InvariantLegal] = &[
    InvariantLegal {
        article: "Art. 3.85 § 1er al. 2",
        obligation: "Les quotités sont fixées par l'acte de base ; leur somme est le dénominateur.",
        porte_par: "domain/copropriete/acp.rs",
        atteste_par: "acp::tests::is_conformant",
    },
    InvariantLegal {
        article: "Art. 3.85 § 3, 3°",
        obligation: "Le ROI fixe la période annuelle de quinze jours de l'AG ordinaire.",
        porte_par: "domain/copropriete/fenetre_ag_ordinaire.rs",
        atteste_par: "fenetre_ag_ordinaire::tests::happy_la_periode_dure_quinze_jours_bornes_comprises",
    },
    InvariantLegal {
        article: "Art. 3.86 § 1er",
        obligation: "La personnalité juridique tient à deux conditions cumulatives ; sans transcription, l'ACP ne peut s'en prévaloir mais un tiers le peut.",
        porte_par: "domain/copropriete/personnalite_juridique.rs",
        atteste_par: "personnalite_juridique::tests::security_sans_transcription_la_protection_ne_joue_que_dans_un_sens",
    },
    InvariantLegal {
        article: "Art. 3.86 § 1er al. 4",
        obligation: "Tous les documents émanant de l'ACP mentionnent son numéro d'entreprise.",
        porte_par: "domain/copropriete/mention_numero_entreprise.rs",
        atteste_par: "mention_numero_entreprise::tests::security_un_seul_oubli_parmi_cinq_est_releve",
    },
    InvariantLegal {
        article: "Art. 3.86 § 3",
        obligation: "Comptes distincts pour le fonds de roulement et le fonds de réserve, ouverts au nom de l'ACP.",
        porte_par: "domain/copropriete/comptes_de_lacp.rs",
        atteste_par: "comptes_de_lacp::tests::security_un_compte_unique_pour_les_deux_fonds_est_signale",
    },
    InvariantLegal {
        article: "Art. 3.86 § 3 al. 4",
        obligation: "Fonds de réserve exigible cinq ans après la réception provisoire ; contribution annuelle d'au moins 5 % des charges ordinaires de l'exercice précédent.",
        porte_par: "domain/copropriete/fonds_de_reserve.rs",
        atteste_par: "fonds_de_reserve::tests::happy_passe_cinq_ans_le_plancher_sapplique",
    },
    InvariantLegal {
        article: "Art. 3.86 § 3 al. 7",
        obligation: "Le syndic communique, lors de l'appel de fonds, la part affectée au fonds de réserve.",
        porte_par: "domain/comptabilite/call_for_funds.rs",
        atteste_par: "call_for_funds::tests_art_3_86_fonds_de_reserve::happy_lappel_porte_la_part_affectee_au_fonds_de_reserve",
    },
    InvariantLegal {
        article: "Art. 3.86 § 3 al. 8",
        obligation: "En cas d'usufruit, les titulaires de droits réels sont solidairement tenus des charges.",
        porte_par: "domain/copropriete/solidarite.rs",
        atteste_par: "solidarite::tests::happy_lusufruit_rend_les_deux_titulaires_tenus_du_tout",
    },
    InvariantLegal {
        article: "Art. 3.87 § 2",
        obligation: "AG sur requête d'un cinquième des parts ; convocation sous trente jours, à défaut de quoi un cosignataire convoque lui-même.",
        porte_par: "domain/copropriete/requete_ag.rs",
        atteste_par: "requete_ag::tests::happy_le_cosignataire_recupere_le_pouvoir_de_convoquer",
    },
    InvariantLegal {
        article: "Art. 3.87 § 3",
        obligation: "Convocation par recommandé, sauf accord individuel, explicite et écrit du destinataire.",
        porte_par: "domain/copropriete/envoi_convocation.rs",
        atteste_par: "envoi_convocation::tests::security_un_courriel_sans_accord_rend_la_convocation_irreguliere",
    },
    InvariantLegal {
        article: "Art. 3.87 § 5",
        obligation: "Double quorum : plus de la moitié des copropriétaires détenant la moitié des quotités, ou trois quarts des quotités.",
        porte_par: "domain/copropriete/ag_session.rs",
        atteste_par: "ag_session::tests::quorum",
    },
    InvariantLegal {
        article: "Art. 3.87 § 6",
        obligation: "Chaque copropriétaire dispose d'un nombre de voix correspondant à sa quote-part.",
        porte_par: "domain/copropriete/vote.rs",
        atteste_par: "vote::tests",
    },
    InvariantLegal {
        article: "Art. 3.87 § 7",
        obligation: "Trois procurations au plus, sauf sous 10 % des voix ; nul ne pèse plus que les autres réunis ; le syndic n'est pas mandataire.",
        porte_par: "domain/copropriete/procurations.rs",
        atteste_par: "procurations::tests::negative_quatre_procurations_au_dessus_de_dix_pourcents_sont_refusees",
    },
    InvariantLegal {
        article: "Art. 3.87 § 8",
        obligation: "Majorité absolue des présents ; abstentions, blancs et nuls exclus du calcul.",
        porte_par: "domain/copropriete/resolution.rs",
        atteste_par: "resolution::tests::test_absolute_majority_abstentions_excluded",
    },
    InvariantLegal {
        article: "Art. 3.87 § 9",
        obligation: "Le prestataire de l'ACP ne participe ni aux délibérations ni au vote sur sa propre mission.",
        porte_par: "domain/copropriete/conflit_dinterets.rs",
        atteste_par: "conflit_dinterets::tests::security_donner_procuration_ne_contourne_pas_la_regle",
    },
    InvariantLegal {
        article: "Art. 3.87 § 10",
        obligation: "Le PV est signé par le président copropriétaire, le secrétaire désigné à l'ouverture, et les copropriétaires encore présents.",
        porte_par: "domain/copropriete/signatures_pv.rs",
        atteste_par: "signatures_pv::tests::security_un_president_non_coproprietaire_vicie_le_pv",
    },
    InvariantLegal {
        article: "Art. 3.87 § 12",
        obligation: "Le PV est consigné au registre et transmis à chaque destinataire dans les trente jours.",
        porte_par: "domain/copropriete/consignation_pv.rs",
        atteste_par: "consignation_pv::tests::security_un_seul_destinataire_oublie_suffit_a_faire_defaut",
    },
    InvariantLegal {
        article: "Art. 3.88",
        obligation: "Chaque nature de décision porte la majorité que la loi lui attache ; les quotités exigent l'unanimité, sauf porte du § 3 al. 2.",
        porte_par: "domain/copropriete/majorites.rs",
        atteste_par: "majorites::tests::security_les_charges_et_les_quotes_parts_nont_pas_la_meme_majorite",
    },
    InvariantLegal {
        article: "Art. 3.89 § 1er",
        obligation: "Le mandat de syndic n'excède pas trois ans.",
        porte_par: "domain/copropriete/syndic_mandate.rs",
        atteste_par: "syndic_mandate::tests::happy_un_mandat_neuf_est_en_cours",
    },
    InvariantLegal {
        article: "Art. 3.89 § 5, 5°",
        obligation: "Le relevé des dettes est fourni au notaire dans les trente jours de sa demande.",
        porte_par: "domain/copropriete/releve_notaire.rs",
        atteste_par: "releve_notaire::tests::negative_passe_trente_jours_sans_releve_le_syndic_est_en_defaut",
    },
    InvariantLegal {
        article: "Art. 3.89 § 5, 7°",
        obligation: "L'ensemble du dossier de gestion est transmis au successeur dans les trente jours.",
        porte_par: "domain/services/dossier_de_gestion.rs",
        atteste_par: "dossier_de_gestion::tests::le_dossier_de_gestion_suit_lacp_lors_dune_passation",
    },
    InvariantLegal {
        article: "Art. 3.89 § 5, 12°",
        obligation: "Un rapport d'évaluation des contrats de fournitures régulières est soumis à chaque assemblée générale ordinaire.",
        porte_par: "domain/copropriete/evaluation_des_contrats.rs",
        atteste_par: "evaluation_des_contrats::tests::security_un_contrat_oublie_est_signale_nominativement",
    },
    InvariantLegal {
        article: "Art. 3.89 § 5, 13°",
        obligation: "Tout contrat entre l'ACP et le syndic ou ses proches exige l'autorisation préalable de l'AG.",
        porte_par: "domain/copropriete/contrat_lie.rs",
        atteste_par: "contrat_lie::tests::security_une_autorisation_posterieure_ne_regularise_rien",
    },
    InvariantLegal {
        article: "Art. 3.89 § 5, 15°",
        obligation: "Comptabilité simplifiée autorisée sous vingt lots, caves, garages et parkings exclus du décompte.",
        porte_par: "domain/comptabilite/regime_comptable.rs",
        atteste_par: "regime_comptable::tests::happy_caves_et_parkings_sortent_du_decompte",
    },
    InvariantLegal {
        article: "Art. 3.89 § 9",
        obligation: "Le syndic n'est ni membre du conseil de copropriété ni commissaire aux comptes de la même ACP.",
        porte_par: "domain/copropriete/commissaire_aux_comptes.rs",
        atteste_par: "commissaire_aux_comptes::tests::security_le_syndic_ne_peut_pas_controler_ses_propres_comptes",
    },
    InvariantLegal {
        article: "Art. 3.90",
        obligation: "Conseil de copropriété obligatoire dès vingt lots ; membres titulaires d'un droit réel votant ; mandat jusqu'à la prochaine AGO.",
        porte_par: "domain/copropriete/conseil_de_copropriete.rs",
        atteste_par: "conseil_de_copropriete::tests::edge_dix_neuf_lots_rendent_le_conseil_facultatif",
    },
    InvariantLegal {
        article: "Art. 3.91",
        obligation: "L'AG désigne annuellement un commissaire aux comptes ou un collège, copropriétaires ou non.",
        porte_par: "domain/copropriete/commissaire_aux_comptes.rs",
        atteste_par: "commissaire_aux_comptes::tests::security_une_designation_ne_se_reconduit_pas_tacitement",
    },
    InvariantLegal {
        article: "Art. 3.94 § 1er et § 2",
        obligation: "État daté sous quinze jours calendaires (demande simple) ou trente (notaire, recommandé).",
        porte_par: "domain/comptabilite/etat_date.rs",
        atteste_par: "etat_date::tests::test_delai_art_3_94_se_compte_en_jours_calendaires",
    },
    InvariantLegal {
        article: "Art. 3.95",
        obligation: "Le notaire retient les arriérés de charges, les frais de récupération et les frais de transmission ; contestation en jours ouvrables.",
        porte_par: "domain/comptabilite/arrieres_mutation.rs",
        atteste_par: "arrieres_mutation::tests::security_le_calcul_calendaire_libererait_les_fonds_trop_tot",
    },
];

/// Le nombre d'obligations computables recensées par le RFC-0002.
pub const OBLIGATIONS_RECENSEES: usize = 29;

/// Rend le registre lisible par un juriste, dans l'ordre des articles.
pub fn rapport_de_conformite() -> String {
    let mut lignes = vec![
        "# Conformité au Code civil, Livre 3, chapitre « copropriété »".to_string(),
        String::new(),
        format!(
            "{} invariants portés par le domaine, chacun attesté par un test qui cite son article.",
            REGISTRE.len()
        ),
        String::new(),
        "| Article | Obligation | Porté par | Attesté par |".to_string(),
        "|---|---|---|---|".to_string(),
    ];
    for invariant in REGISTRE {
        lignes.push(format!(
            "| {} | {} | `{}` | `{}` |",
            invariant.article, invariant.obligation, invariant.porte_par, invariant.atteste_par
        ));
    }
    lignes.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn racine_src() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
    }

    /// **Le test qui empêche le registre de mentir.**
    ///
    /// Chaque invariant désigne un fichier du domaine. Le jour où quelqu'un le
    /// renomme ou le supprime, le registre ne se contente pas de pointer dans
    /// le vide : il échoue ici.
    #[test]
    fn chaque_invariant_designe_un_module_qui_existe() {
        let manquants: Vec<&str> = REGISTRE
            .iter()
            .filter(|i| !racine_src().join("src").join(i.porte_par).is_file())
            .map(|i| i.porte_par)
            .collect();

        assert!(
            manquants.is_empty(),
            "le registre désigne des modules qui n'existent plus : {manquants:?}\n\
             Un invariant sans code qui le porte n'est pas un invariant."
        );
    }

    /// Le nom du test attesté doit se retrouver dans le module désigné.
    ///
    /// Sans cette vérification, un test supprimé laisserait le registre
    /// affirmer une couverture qui n'existe plus — exactement ce qu'un
    /// document en prose fait, et qu'on veut éviter ici.
    #[test]
    fn chaque_invariant_designe_un_test_qui_existe() {
        let mut introuvables = Vec::new();

        for invariant in REGISTRE {
            let chemin = racine_src().join("src").join(invariant.porte_par);
            let Ok(source) = std::fs::read_to_string(&chemin) else {
                continue; // couvert par le test précédent
            };
            // On cherche le dernier segment : `module::tests::nom_du_test`.
            let nom = invariant
                .atteste_par
                .rsplit("::")
                .next()
                .unwrap_or(invariant.atteste_par);
            if !source.contains(nom) {
                introuvables.push(format!("{} → {}", invariant.article, invariant.atteste_par));
            }
        }

        assert!(
            introuvables.is_empty(),
            "le registre atteste des tests introuvables dans leur module :\n  {}\n\n\
             Un invariant sans test qui le nomme n'est pas un invariant, c'est une intention.",
            introuvables.join("\n  ")
        );
    }

    #[test]
    fn aucun_article_nest_declare_deux_fois_pour_la_meme_obligation() {
        let mut vus: Vec<(&str, &str)> = Vec::new();
        for invariant in REGISTRE {
            let cle = (invariant.article, invariant.obligation);
            assert!(
                !vus.contains(&cle),
                "{} déclaré deux fois pour la même obligation",
                invariant.article
            );
            vus.push(cle);
        }
    }

    /// La couverture ne recule pas.
    ///
    /// Le RFC-0002 recensait vingt-neuf obligations computables. Le registre
    /// en porte autant : c'est le solde à la clôture du lot J7, et il sert de
    /// plancher.
    #[test]
    fn la_couverture_ne_recule_pas() {
        assert!(
            REGISTRE.len() >= OBLIGATIONS_RECENSEES,
            "le registre est passé de {OBLIGATIONS_RECENSEES} à {} invariants : \
             une couverture ne se retire pas sans qu'on le dise.",
            REGISTRE.len()
        );
    }

    /// Le rapport doit rester lisible : c'est un juriste qui le lit.
    #[test]
    fn le_rapport_cite_chaque_article() {
        let rapport = rapport_de_conformite();
        for invariant in REGISTRE {
            assert!(
                rapport.contains(invariant.article),
                "{} absent du rapport",
                invariant.article
            );
        }
    }
}
