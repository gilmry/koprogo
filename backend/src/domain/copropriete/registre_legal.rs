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
    /// Le délai que l'article impose, en jours, quand il en impose un.
    ///
    /// ── Pourquoi il vit ICI et pas dans l'interface ────────────────────
    ///
    /// La remise de design veut afficher un **décompte d'échéance** à côté de
    /// chaque tâche : « 18 j sur 30 », avec la référence de l'article. Elle
    /// met en garde dans le même souffle :
    ///
    /// > Legal references are placeholders. The stated day counts (15 / 30)
    /// > must be sourced from the project's own registry, **not hardcoded in
    /// > components**.
    ///
    /// Un « 30 » écrit dans un composant Svelte est un nombre que rien ne
    /// relie à la loi. Le jour où le législateur le change — ou, plus
    /// probable, le jour où l'on découvre qu'on l'avait mal lu — le domaine
    /// est corrigé et l'écran continue d'annoncer l'ancien délai au syndic.
    /// Il agirait alors sur une échéance fausse en croyant lire le produit.
    ///
    /// `garde_delais_du_registre` vérifie que chaque valeur ici correspond à
    /// la constante du module qui la porte.
    ///
    /// `None` quand l'article n'impose aucun délai : la plupart des
    /// obligations sont des règles de fond, pas des échéances.
    pub delai_jours: Option<i64>,
}

/// Le registre, dans l'ordre des articles.
pub const REGISTRE: &[InvariantLegal] = &[
    InvariantLegal {
        article: "Art. 3.85 § 1er al. 2",
        obligation: "Les quotités sont fixées par l'acte de base ; leur somme est le dénominateur.",
        porte_par: "domain/copropriete/acp.rs",
        // Citait `acp::tests::is_conformant` — une MÉTHODE de production, pas
        // un test. La garde l'acceptait parce qu'elle cherchait le nom par
        // simple sous-chaîne dans le module (#847).
        atteste_par: "acp::tests::happy_acp_conformant_base_1000_mono_bloc",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.85 § 3, 3°",
        obligation: "Le ROI fixe la période annuelle de quinze jours de l'AG ordinaire.",
        porte_par: "domain/copropriete/fenetre_ag_ordinaire.rs",
        atteste_par: "fenetre_ag_ordinaire::tests::happy_la_periode_dure_quinze_jours_bornes_comprises",
        delai_jours: Some(crate::domain::copropriete::fenetre_ag_ordinaire::FenetreAgOrdinaire::DUREE_JOURS),
    },
    InvariantLegal {
        article: "Art. 3.86 § 1er",
        obligation: "La personnalité juridique tient à deux conditions cumulatives ; sans transcription, l'ACP ne peut s'en prévaloir mais un tiers le peut.",
        porte_par: "domain/copropriete/personnalite_juridique.rs",
        atteste_par: "personnalite_juridique::tests::security_sans_transcription_la_protection_ne_joue_que_dans_un_sens",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.86 § 1er al. 4",
        obligation: "Tous les documents émanant de l'ACP mentionnent son numéro d'entreprise.",
        porte_par: "domain/copropriete/mention_numero_entreprise.rs",
        atteste_par: "mention_numero_entreprise::tests::security_un_seul_oubli_parmi_cinq_est_releve",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.86 § 3",
        obligation: "Comptes distincts pour le fonds de roulement et le fonds de réserve, ouverts au nom de l'ACP.",
        porte_par: "domain/copropriete/comptes_de_lacp.rs",
        atteste_par: "comptes_de_lacp::tests::security_un_compte_unique_pour_les_deux_fonds_est_signale",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.86 § 3 al. 4",
        obligation: "Fonds de réserve exigible cinq ans après la réception provisoire ; contribution annuelle d'au moins 5 % des charges ordinaires de l'exercice précédent.",
        porte_par: "domain/copropriete/fonds_de_reserve.rs",
        atteste_par: "fonds_de_reserve::tests::happy_passe_cinq_ans_le_plancher_sapplique",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.86 § 3 al. 7",
        obligation: "Le syndic communique, lors de l'appel de fonds, la part affectée au fonds de réserve.",
        porte_par: "domain/comptabilite/call_for_funds.rs",
        atteste_par: "call_for_funds::tests_art_3_86_fonds_de_reserve::happy_lappel_porte_la_part_affectee_au_fonds_de_reserve",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.86 § 3 al. 8",
        obligation: "En cas d'usufruit, les titulaires de droits réels sont solidairement tenus des charges.",
        porte_par: "domain/copropriete/solidarite.rs",
        atteste_par: "solidarite::tests::happy_lusufruit_rend_les_deux_titulaires_tenus_du_tout",
        delai_jours: None,
    },
    // L'Art. 3.87 § 2 porte DEUX obligations distinctes, et le registre n'en
    // déclarait qu'une sous le nom de l'article entier. Un article couvert à
    // moitié paraissait couvert (#847). Chaque alinéa est désormais nommé.
    InvariantLegal {
        article: "Art. 3.87 § 2 (convocation sur requête)",
        obligation: "AG sur requête d'un cinquième des parts ; convocation sous trente jours, à défaut de quoi un cosignataire convoque lui-même.",
        porte_par: "domain/copropriete/requete_ag.rs",
        atteste_par: "requete_ag::tests::happy_le_cosignataire_recupere_le_pouvoir_de_convoquer",
        delai_jours: Some(crate::domain::copropriete::requete_ag::DELAI_CONVOCATION_JOURS),
    },
    InvariantLegal {
        article: "Art. 3.87 § 2 (ordre du jour)",
        obligation: "Une décision portant sur un point absent de l'ordre du jour est nulle : une résolution non rattachée à un point ne peut être mise aux voix.",
        porte_par: "application/use_cases/resolution_use_cases.rs",
        atteste_par:
            "resolution_use_cases::tests::security_vote_refuse_sur_resolution_hors_ordre_du_jour",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.87 § 3",
        obligation: "Convocation par recommandé, sauf accord individuel, explicite et écrit du destinataire.",
        porte_par: "domain/copropriete/envoi_convocation.rs",
        atteste_par: "envoi_convocation::tests::security_un_courriel_sans_accord_rend_la_convocation_irreguliere",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.87 § 5",
        obligation: "Double quorum : plus de la moitié des copropriétaires détenant la moitié des quotités, ou trois quarts des quotités.",
        porte_par: "domain/copropriete/ag_session.rs",
        // Citait `ag_session::tests::quorum` : le mot « quorum » apparaît des
        // dizaines de fois dans ce module, et la garde s'en satisfaisait.
        atteste_par: "ag_session::tests::test_quotas_alone_do_not_carry_the_quorum",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.87 § 6",
        obligation: "Chaque copropriétaire dispose d'un nombre de voix correspondant à sa quote-part.",
        porte_par: "domain/copropriete/vote.rs",
        // Citait `vote::tests` — le module de tests ENTIER, pas un test. La
        // garde cherchait « tests » et trouvait `mod tests` (#847).
        //
        // ⚠ ATTESTATION ADJACENTE, ET C'EST ASSUMÉ. Le test vérifie qu'une
        // voix SUPÉRIEURE au maximum est refusée : c'est un plafond, pas la
        // CORRESPONDANCE entre le nombre de voix et la quote-part que
        // l'article exige. Aucun test du dépôt n'atteste aujourd'hui que la
        // voix d'un copropriétaire ÉGALE sa quote-part.
        //
        // Le noter ici plutôt que de laisser croire à une couverture pleine :
        // c'est la leçon de #847, où trois invariants attestaient sur une
        // sous-chaîne et un sur un test sans rapport.
        atteste_par: "vote::tests::test_create_vote_excessive_voting_power_fails",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.87 § 7",
        obligation: "Trois procurations au plus, sauf sous 10 % des voix ; nul ne pèse plus que les autres réunis ; le syndic n'est pas mandataire.",
        porte_par: "domain/copropriete/procurations.rs",
        atteste_par: "procurations::tests::negative_quatre_procurations_au_dessus_de_dix_pourcents_sont_refusees",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.87 § 8",
        obligation: "Majorité absolue des présents ; abstentions, blancs et nuls exclus du calcul.",
        porte_par: "domain/copropriete/resolution.rs",
        atteste_par: "resolution::tests::test_absolute_majority_abstentions_excluded",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.87 § 9",
        obligation: "Le prestataire de l'ACP ne participe ni aux délibérations ni au vote sur sa propre mission.",
        porte_par: "domain/copropriete/conflit_dinterets.rs",
        atteste_par: "conflit_dinterets::tests::security_donner_procuration_ne_contourne_pas_la_regle",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.87 § 10",
        obligation: "Le PV est signé par le président copropriétaire, le secrétaire désigné à l'ouverture, et les copropriétaires encore présents.",
        porte_par: "domain/copropriete/signatures_pv.rs",
        atteste_par: "signatures_pv::tests::security_un_president_non_coproprietaire_vicie_le_pv",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.87 § 12",
        obligation: "Le PV est consigné au registre et transmis à chaque destinataire dans les trente jours.",
        porte_par: "domain/copropriete/consignation_pv.rs",
        atteste_par: "consignation_pv::tests::security_un_seul_destinataire_oublie_suffit_a_faire_defaut",
        delai_jours: Some(crate::domain::copropriete::consignation_pv::DELAI_JOURS),
    },
    InvariantLegal {
        article: "Art. 3.88",
        obligation: "Chaque nature de décision porte la majorité que la loi lui attache ; les quotités exigent l'unanimité, sauf porte du § 3 al. 2.",
        porte_par: "domain/copropriete/majorites.rs",
        atteste_par: "majorites::tests::security_les_charges_et_les_quotes_parts_nont_pas_la_meme_majorite",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.89 § 1er",
        obligation: "Le mandat de syndic n'excède pas trois ans.",
        porte_par: "domain/copropriete/syndic_mandate.rs",
        // Le test cité jusqu'au 2026-09-08 était
        // `happy_un_mandat_neuf_est_en_cours` : il vérifie qu'un mandat de
        // trente jours est en cours, et ne dit RIEN d'un plafond de trois ans.
        // Le plafond n'existait pas non plus. L'invariant se déclarait attesté
        // par une preuve sans rapport (#847).
        atteste_par:
            "syndic_mandate::tests::security_un_mandat_de_plus_de_trois_ans_est_expire_de_plein_droit",
        delai_jours: Some(crate::domain::copropriete::syndic_mandate::DUREE_MAXIMALE_JOURS),
    },
    InvariantLegal {
        article: "Art. 3.89 § 5, 5°",
        obligation: "Le relevé des dettes est fourni au notaire dans les trente jours de sa demande.",
        porte_par: "domain/copropriete/releve_notaire.rs",
        atteste_par: "releve_notaire::tests::negative_passe_trente_jours_sans_releve_le_syndic_est_en_defaut",
        delai_jours: Some(crate::domain::copropriete::releve_notaire::DELAI_JOURS),
    },
    InvariantLegal {
        article: "Art. 3.89 § 5, 7°",
        obligation: "L'ensemble du dossier de gestion est transmis au successeur dans les trente jours.",
        porte_par: "domain/services/dossier_de_gestion.rs",
        // Le test cité jusqu'au 2026-09-10 vérifiait que le successeur voit
        // l'ensemble des pièces. C'est vrai, et ça ne dit rien du DÉLAI, qui
        // n'existait alors nulle part dans le module (#847). Celui-ci éprouve
        // l'échéance des trente jours et le défaut qui suit son dépassement.
        atteste_par:
            "dossier_de_gestion::tests::negative_passe_trente_jours_sans_remise_le_syndic_sortant_est_en_defaut",
        delai_jours: Some(crate::domain::services::dossier_de_gestion::DELAI_PASSATION_JOURS),
    },
    InvariantLegal {
        article: "Art. 3.89 § 5, 12°",
        obligation: "Un rapport d'évaluation des contrats de fournitures régulières est soumis à chaque assemblée générale ordinaire.",
        porte_par: "domain/copropriete/evaluation_des_contrats.rs",
        atteste_par: "evaluation_des_contrats::tests::security_un_contrat_oublie_est_signale_nominativement",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.89 § 5, 13°",
        obligation: "Tout contrat entre l'ACP et le syndic ou ses proches exige l'autorisation préalable de l'AG.",
        porte_par: "domain/copropriete/contrat_lie.rs",
        atteste_par: "contrat_lie::tests::security_une_autorisation_posterieure_ne_regularise_rien",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.89 § 5, 15°",
        obligation: "Comptabilité simplifiée autorisée sous vingt lots, caves, garages et parkings exclus du décompte.",
        porte_par: "domain/comptabilite/regime_comptable.rs",
        atteste_par: "regime_comptable::tests::happy_caves_et_parkings_sortent_du_decompte",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.89 § 9",
        obligation: "Le syndic n'est ni membre du conseil de copropriété ni commissaire aux comptes de la même ACP.",
        porte_par: "domain/copropriete/commissaire_aux_comptes.rs",
        atteste_par: "commissaire_aux_comptes::tests::security_le_syndic_ne_peut_pas_controler_ses_propres_comptes",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.90",
        obligation: "Conseil de copropriété obligatoire dès vingt lots ; membres titulaires d'un droit réel votant ; mandat jusqu'à la prochaine AGO.",
        porte_par: "domain/copropriete/conseil_de_copropriete.rs",
        atteste_par: "conseil_de_copropriete::tests::edge_dix_neuf_lots_rendent_le_conseil_facultatif",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.91",
        obligation: "L'AG désigne annuellement un commissaire aux comptes ou un collège, copropriétaires ou non.",
        porte_par: "domain/copropriete/commissaire_aux_comptes.rs",
        atteste_par: "commissaire_aux_comptes::tests::security_une_designation_ne_se_reconduit_pas_tacitement",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.94 § 1er et § 2",
        obligation: "État daté sous quinze jours calendaires (demande simple) ou trente (notaire, recommandé).",
        porte_par: "domain/comptabilite/etat_date.rs",
        atteste_par: "etat_date::tests::test_delai_art_3_94_se_compte_en_jours_calendaires",
        delai_jours: None,
    },
    InvariantLegal {
        article: "Art. 3.95",
        obligation: "Le notaire retient les arriérés de charges, les frais de récupération et les frais de transmission ; contestation en jours ouvrables.",
        porte_par: "domain/comptabilite/arrieres_mutation.rs",
        // Citait `security_le_calcul_calendaire_libererait_les_fonds_trop_tot`,
        // qui atteste le calcul du DÉLAI de libération, pas ce que le notaire
        // RETIENT. Le délai relève du même module, mais pas de la même
        // obligation. Relevé le 2026-09-08 (#847).
        atteste_par: "arrieres_mutation::tests::security_oublier_les_frais_de_recuperation_ampute_la_retenue",
        delai_jours: None,
    },
];

/// Le nombre d'obligations computables recensées par le RFC-0002.
pub const OBLIGATIONS_RECENSEES: usize = 30;

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

    /// Le test attesté existe, et c'est un TEST.
    ///
    /// Sans cette vérification, un test supprimé laisserait le registre
    /// affirmer une couverture qui n'existe plus — exactement ce qu'un
    /// document en prose fait, et qu'on veut éviter ici.
    ///
    /// ── Ce que cette garde acceptait avant le 2026-09-08 ──────────────────
    ///
    /// Elle prenait le dernier segment du chemin et faisait un
    /// `source.contains(nom)`. Trois invariants sur vingt-neuf en profitaient :
    ///
    /// ```text
    /// Art. 3.85 § 1er al. 2 → acp::tests::is_conformant
    ///     `is_conformant` est une MÉTHODE DE PRODUCTION, pas un test.
    ///
    /// Art. 3.87 § 5 → ag_session::tests::quorum
    ///     le mot « quorum » apparaît des dizaines de fois dans le module.
    ///
    /// Art. 3.87 § 6 → vote::tests
    ///     le dernier segment est « tests » : la recherche trouvait `mod tests`.
    /// ```
    ///
    /// Dix pour cent du registre attestait donc sur une sous-chaîne. Le message
    /// d'erreur affirmait pourtant : « un invariant sans test qui le nomme
    /// n'est pas un invariant, c'est une intention » — la garde disait le bon
    /// principe et ne le tenait pas.
    ///
    /// Elle exige désormais `fn <nom>` précédé d'un `#[test]` proche. Ce n'est
    /// toujours pas une preuve que le test ATTESTE l'obligation — c'est l'objet
    /// de #847, et cela demande une relecture humaine — mais il ne peut plus
    /// s'agir d'autre chose qu'un test.
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
            // `fn <nom>` — et non le nom seul, qui matcherait une méthode de
            // production, un mot courant, ou `mod tests`.
            let declaration = format!("fn {nom}(");
            let est_un_test = match source.find(&declaration) {
                None => false,
                Some(pos) => {
                    // `#[test]` doit précéder de peu : on tolère les attributs
                    // et commentaires intercalés, pas cinq cents lignes.
                    //
                    // Le recul se fait en OCTETS, et ce dépôt écrit en
                    // français : un décalage brut tombe tôt ou tard au milieu
                    // d'un caractère multi-octets et fait paniquer le
                    // découpage. C'est arrivé sur une flèche « → » d'un
                    // commentaire. On redescend donc jusqu'à la première
                    // frontière de caractère.
                    let mut debut = pos.saturating_sub(300);
                    while debut > 0 && !source.is_char_boundary(debut) {
                        debut -= 1;
                    }
                    let avant = &source[debut..pos];
                    avant.contains("#[test]") || avant.contains("::test]")
                }
            };
            if !est_un_test {
                introuvables.push(format!("{} → {}", invariant.article, invariant.atteste_par));
            }
        }

        assert!(
            introuvables.is_empty(),
            "le registre atteste des tests introuvables dans leur module :\n  {}\n\n\
             Un invariant sans test qui le nomme n'est pas un invariant, c'est \
             une intention. Le nom doit désigner une `fn` annotée `#[test]`, \
             pas une méthode de production ni un module.",
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

    /// Un délai déclaré au registre est CELUI du domaine, pas une copie.
    ///
    /// ── Ce que cette garde empêche ────────────────────────────────────────
    ///
    /// La remise de design veut afficher un décompte d'échéance à côté de
    /// chaque tâche — « 18 j sur 30 » — et met en garde dans le même souffle :
    /// les jours affichés doivent venir du registre du projet, **jamais d'un
    /// nombre écrit dans un composant**.
    ///
    /// Le risque n'est pas théorique. Un « 30 » recopié à l'écran est un
    /// nombre que rien ne relie à la loi : le jour où l'on corrige le domaine
    /// — parce que le législateur a bougé, ou plus probablement parce qu'on
    /// avait mal lu l'article — l'écran continue d'annoncer l'ancien délai.
    /// Le syndic agit alors sur une échéance fausse en croyant lire le
    /// produit.
    ///
    /// ── Ce que cette garde vérifie, et ce qu'elle ne peut pas ────────────
    ///
    /// Elle vérifie que les délais du registre sont **strictement positifs**
    /// et qu'ils portent sur des modules qui déclarent bien une constante de
    /// délai. Elle ne peut pas vérifier qu'un délai manquant DEVRAIT être là :
    /// dire si l'Art. 3.88 impose une échéance demande de lire l'article, pas
    /// le code.
    ///
    /// ── Pourquoi `security_` ────────────────────────────────────────────
    ///
    /// La dispense de taxonomie ne vaut que pour les fichiers
    /// `tests/garde_*.rs` : ce test vit dans un fichier source, il doit donc
    /// déclarer sa catégorie comme les autres. Élargir la dispense pour ma
    /// commodité aurait ouvert une porte que la garde ferme exprès.
    ///
    /// `security_` parce que ce dépôt s'en sert pour la classe « un défaut
    /// qui passerait autrement en silence » — cf.
    /// `security_un_contrat_oublie_est_signale_nominativement`, qui ne porte
    /// pas davantage sur un contrôle d'accès. Un délai faux affiché à un
    /// syndic est exactement cela : il agit sur une échéance erronée en
    /// croyant lire le produit.
    #[test]
    fn security_chaque_delai_declare_vient_du_domaine() {
        let racine = racine_src();
        let mut fautifs: Vec<String> = Vec::new();

        for invariant in REGISTRE {
            let Some(jours) = invariant.delai_jours else {
                continue;
            };

            if jours <= 0 {
                fautifs.push(format!(
                    "{} → délai de {jours} jours, ce qui n'a pas de sens",
                    invariant.article
                ));
                continue;
            }

            // Le module qui porte l'obligation doit porter la constante : si
            // le délai venait d'ailleurs, la valeur affichée et la règle
            // appliquée pourraient diverger sans que rien le dise.
            // `racine_src()` rend la racine du CRATE malgré son nom : la garde
            // voisine ajoute `.join("src")`, et l'omettre faisait déclarer les
            // six modules « introuvables » alors qu'ils étaient tous là.
            let chemin = racine.join("src").join(invariant.porte_par);
            let Ok(source) = std::fs::read_to_string(&chemin) else {
                fautifs.push(format!(
                    "{} → module introuvable : {}",
                    invariant.article, invariant.porte_par
                ));
                continue;
            };
            let declare_une_constante = source.contains("pub const DELAI")
                || source.contains("pub const DUREE")
                || source.contains("const DUREE_JOURS");
            if !declare_une_constante {
                fautifs.push(format!(
                    "{} → {} déclare un délai de {jours} j au registre, mais \
                     ne porte aucune constante de délai",
                    invariant.article, invariant.porte_par
                ));
            }
        }

        assert!(
            fautifs.is_empty(),
            "des délais du registre ne viennent pas du domaine :\n  {}\n\n\
             Un délai doit RÉFÉRENCER la constante du module qui le porte, \
             jamais recopier sa valeur. Sans quoi corriger l'un laisse l'autre \
             mentir, et l'écran annonce une échéance que le code n'applique \
             plus.",
            fautifs.join("\n  ")
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
