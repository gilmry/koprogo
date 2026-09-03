//! L'affectation des majorités par nature de décision.
//!
//! Art. 3.88. Le domaine portait déjà les seuils — absolue, deux tiers, quatre
//! cinquièmes, unanimité — et les calculait correctement, abstentions exclues
//! (Art. 3.87 § 8). Ce qui n'avait jamais été confronté à l'article, c'est
//! leur **affectation** : quel seuil pour quelle décision.
//!
//! L'écart est sérieux dans les deux sens. Un seuil trop bas rend la décision
//! annulable ; un seuil trop haut bloque une copropriété sans base légale.
//!
//! Le modèle portait `ResolutionType { Ordinary, Extraordinary }`, qui ne
//! correspond à rien dans le texte : la loi ne connaît pas de décisions
//! « ordinaires » et « extraordinaires », elle énumère des natures et leur
//! attache un seuil. D'où [`NatureDeDecision`], calquée sur l'énumération de
//! l'article, et une majorité **dérivée** plutôt que choisie — un appelant qui
//! choisit peut choisir mal.
//!
//! Trois subtilités que l'énumération seule ne rend pas :
//!
//! 1. **l'exception du 1°, b)** — les travaux affectant les parties communes
//!    exigent les deux tiers, *sauf* les travaux imposés par la loi, les
//!    travaux conservatoires et ceux d'administration provisoire, qui passent
//!    à la majorité absolue. Confondre les deux bloquerait une toiture qui
//!    fuit derrière un vote qualifié ;
//! 2. **le § 3, alinéa 2** — la modification de la répartition des quotes-parts
//!    exige l'unanimité, *mais* quand l'assemblée décide de travaux, d'une
//!    division de lots ou d'un acte de disposition à la majorité qualifiée,
//!    elle peut statuer **à cette même majorité** sur la modification de
//!    quotités qui en découle nécessairement. Sans cette porte, toute
//!    division de lot deviendrait impossible dès qu'un copropriétaire s'y
//!    oppose ;
//! 3. **le défaut est la majorité absolue** (Art. 3.87 § 8) : « sauf si la loi
//!    exige une majorité qualifiée ». Ce qui n'est pas énuméré relève du droit
//!    commun.
//!
//! Voir issue #751.

use super::resolution::MajorityType;

/// La nature d'une décision, telle que l'Art. 3.88 l'énumère.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NatureDeDecision {
    // ── § 1er, 1° — deux tiers ──────────────────────────────────────
    /// a) Modification des statuts portant sur la jouissance, l'usage ou
    /// l'administration des parties communes.
    ModificationStatutsUsageDesCommuns,
    /// b) Travaux affectant les parties communes.
    TravauxAuxPartiesCommunes,
    /// c) Montant des marchés à partir duquel la mise en concurrence est
    /// obligatoire.
    SeuilDeMiseEnConcurrence,
    /// d) Travaux à des parties privatives exécutés par l'ACP, sur motivation
    /// spéciale.
    TravauxPrivatifsExecutesParLacp,

    // ── § 1er, 1°, b), exception — majorité absolue ─────────────────
    /// Travaux imposés par la loi.
    TravauxImposesParLaLoi,
    /// Travaux conservatoires.
    TravauxConservatoires,
    /// Actes d'administration provisoire.
    AdministrationProvisoire,

    // ── § 1er, 2° — quatre cinquièmes ───────────────────────────────
    /// a) Toute autre modification des statuts, **y compris la répartition des
    /// charges**.
    ///
    /// À ne pas confondre avec la répartition des **quotes-parts**, qui relève
    /// de l'unanimité (§ 3). Les charges se répartissent, les quotités
    /// s'établissent.
    AutreModificationDesStatuts,
    /// b) Modification de la destination de l'immeuble ou d'une partie.
    ModificationDeLaDestination,
    /// c) Reconstruction ou remise en état après destruction partielle.
    ReconstructionApresDestructionPartielle,
    /// d) Acquisition de biens immobiliers destinés à devenir communs.
    AcquisitionDeBiensCommuns,
    /// e) Actes de disposition de biens immobiliers communs.
    ActeDeDispositionDeBiensCommuns,
    /// f) Modification des statuts pour créer des associations partielles.
    CreationDassociationsPartielles,
    /// g) Division d'un lot, ou réunion de plusieurs lots.
    DivisionOuReunionDeLots,
    /// h) Démolition et reconstruction totales pour raisons de salubrité, de
    /// sécurité, ou de coût excessif de mise en conformité.
    DemolitionReconstructionPourSalubriteOuCout,

    // ── § 3 — unanimité ────────────────────────────────────────────
    /// Modification de la répartition des quotes-parts de copropriété.
    ModificationDesQuotesParts,
    /// Démolition et reconstruction totales pour un autre motif que ceux du
    /// 2°, h).
    DemolitionReconstructionAutreMotif,

    /// Tout le reste : droit commun de l'Art. 3.87 § 8.
    DroitCommun,
}

impl NatureDeDecision {
    /// La majorité que la loi exige pour cette nature de décision.
    ///
    /// Dérivée, jamais choisie : c'est le point de cette construction.
    pub fn majorite_requise(&self) -> MajorityType {
        match self {
            Self::ModificationStatutsUsageDesCommuns
            | Self::TravauxAuxPartiesCommunes
            | Self::SeuilDeMiseEnConcurrence
            | Self::TravauxPrivatifsExecutesParLacp => MajorityType::TwoThirds,

            Self::TravauxImposesParLaLoi
            | Self::TravauxConservatoires
            | Self::AdministrationProvisoire
            | Self::DroitCommun => MajorityType::Absolute,

            Self::AutreModificationDesStatuts
            | Self::ModificationDeLaDestination
            | Self::ReconstructionApresDestructionPartielle
            | Self::AcquisitionDeBiensCommuns
            | Self::ActeDeDispositionDeBiensCommuns
            | Self::CreationDassociationsPartielles
            | Self::DivisionOuReunionDeLots
            | Self::DemolitionReconstructionPourSalubriteOuCout => MajorityType::FourFifths,

            Self::ModificationDesQuotesParts | Self::DemolitionReconstructionAutreMotif => {
                MajorityType::Unanimity
            }
        }
    }

    /// Cette nature ouvre-t-elle la porte du § 3, alinéa 2 ?
    ///
    /// « Lorsque l'assemblée générale, à la majorité qualifiée requise par la
    /// loi, décide de **travaux**, de la **division ou la réunion de lots** ou
    /// d'**actes de disposition**, elle peut statuer, à la même majorité
    /// qualifiée, sur la modification de la répartition des quotes-parts dans
    /// les cas où cette modification est nécessaire. »
    pub fn autorise_quotes_parts_a_la_meme_majorite(&self) -> bool {
        matches!(
            self,
            Self::TravauxAuxPartiesCommunes
                | Self::DivisionOuReunionDeLots
                | Self::ActeDeDispositionDeBiensCommuns
                | Self::CreationDassociationsPartielles
        )
    }
}

/// La majorité applicable à une modification de quotes-parts, selon qu'elle
/// découle ou non d'une décision qui l'autorise (Art. 3.88 § 3, alinéa 2).
///
/// `decision_dorigine` est la décision qui rend la modification nécessaire.
/// `None` — une modification de quotités décidée pour elle-même — exige
/// l'unanimité.
///
/// Sans cette porte, toute division de lot deviendrait impossible dès qu'un
/// seul copropriétaire s'y oppose, alors que la loi organise précisément le
/// contraire.
pub fn majorite_pour_modifier_les_quotes_parts(
    decision_dorigine: Option<NatureDeDecision>,
) -> MajorityType {
    match decision_dorigine {
        Some(nature) if nature.autorise_quotes_parts_a_la_meme_majorite() => {
            nature.majorite_requise()
        }
        _ => MajorityType::Unanimity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── § 1er, 1° — les deux tiers ─────────────────────────────────

    #[test]
    fn happy_les_quatre_natures_du_premier_point_exigent_les_deux_tiers() {
        for nature in [
            NatureDeDecision::ModificationStatutsUsageDesCommuns,
            NatureDeDecision::TravauxAuxPartiesCommunes,
            NatureDeDecision::SeuilDeMiseEnConcurrence,
            NatureDeDecision::TravauxPrivatifsExecutesParLacp,
        ] {
            assert_eq!(
                nature.majorite_requise(),
                MajorityType::TwoThirds,
                "{nature:?}"
            );
        }
    }

    /// L'exception du 1°, b), et pourquoi elle compte.
    ///
    /// Confondre travaux ordinaires et travaux conservatoires bloquerait une
    /// toiture qui fuit derrière un vote qualifié, alors que la loi veut
    /// précisément qu'on puisse agir vite.
    #[test]
    fn happy_les_travaux_conservatoires_passent_a_la_majorite_absolue() {
        for nature in [
            NatureDeDecision::TravauxImposesParLaLoi,
            NatureDeDecision::TravauxConservatoires,
            NatureDeDecision::AdministrationProvisoire,
        ] {
            assert_eq!(
                nature.majorite_requise(),
                MajorityType::Absolute,
                "{nature:?} relève de l'exception du 1°, b)"
            );
        }
        assert_eq!(
            NatureDeDecision::TravauxAuxPartiesCommunes.majorite_requise(),
            MajorityType::TwoThirds,
            "les autres travaux restent aux deux tiers"
        );
    }

    // ── § 1er, 2° — les quatre cinquièmes ──────────────────────────

    #[test]
    fn happy_les_huit_natures_du_second_point_exigent_les_quatre_cinquiemes() {
        for nature in [
            NatureDeDecision::AutreModificationDesStatuts,
            NatureDeDecision::ModificationDeLaDestination,
            NatureDeDecision::ReconstructionApresDestructionPartielle,
            NatureDeDecision::AcquisitionDeBiensCommuns,
            NatureDeDecision::ActeDeDispositionDeBiensCommuns,
            NatureDeDecision::CreationDassociationsPartielles,
            NatureDeDecision::DivisionOuReunionDeLots,
            NatureDeDecision::DemolitionReconstructionPourSalubriteOuCout,
        ] {
            assert_eq!(
                nature.majorite_requise(),
                MajorityType::FourFifths,
                "{nature:?}"
            );
        }
    }

    /// La confusion la plus coûteuse de l'article : charges contre quotités.
    ///
    /// La **répartition des charges** se modifie aux quatre cinquièmes (2°, a) ;
    /// la **répartition des quotes-parts** exige l'unanimité (§ 3). Les charges
    /// se répartissent, les quotités s'établissent — et une AG qui croirait
    /// pouvoir toucher aux secondes aux quatre cinquièmes prendrait une
    /// décision annulable.
    #[test]
    fn security_les_charges_et_les_quotes_parts_nont_pas_la_meme_majorite() {
        assert_eq!(
            NatureDeDecision::AutreModificationDesStatuts.majorite_requise(),
            MajorityType::FourFifths,
            "la répartition des CHARGES est aux 4/5"
        );
        assert_eq!(
            NatureDeDecision::ModificationDesQuotesParts.majorite_requise(),
            MajorityType::Unanimity,
            "la répartition des QUOTES-PARTS exige l'unanimité"
        );
    }

    // ── § 3 — l'unanimité et sa porte ──────────────────────────────

    #[test]
    fn happy_une_modification_de_quotites_pour_elle_meme_exige_lunanimite() {
        assert_eq!(
            majorite_pour_modifier_les_quotes_parts(None),
            MajorityType::Unanimity
        );
    }

    /// § 3, alinéa 2 : la porte que la loi ouvre.
    ///
    /// Sans elle, toute division de lot deviendrait impossible dès qu'un seul
    /// copropriétaire s'y oppose.
    #[test]
    fn happy_une_division_de_lot_emporte_les_quotites_a_sa_propre_majorite() {
        assert_eq!(
            majorite_pour_modifier_les_quotes_parts(Some(
                NatureDeDecision::DivisionOuReunionDeLots
            )),
            MajorityType::FourFifths
        );
    }

    #[test]
    fn happy_des_travaux_emportent_les_quotites_aux_deux_tiers() {
        assert_eq!(
            majorite_pour_modifier_les_quotes_parts(Some(
                NatureDeDecision::TravauxAuxPartiesCommunes
            )),
            MajorityType::TwoThirds
        );
    }

    /// @security — la porte ne s'ouvre pas pour n'importe quelle décision.
    ///
    /// Une modification de destination ne rend pas nécessaire un remaniement
    /// des quotités : l'article énumère travaux, division/réunion et actes de
    /// disposition, et l'étendre reviendrait à contourner l'unanimité.
    #[test]
    fn security_toute_decision_qualifiee_nouvre_pas_la_porte() {
        assert_eq!(
            majorite_pour_modifier_les_quotes_parts(Some(
                NatureDeDecision::ModificationDeLaDestination
            )),
            MajorityType::Unanimity,
            "seuls travaux, division/réunion et actes de disposition l'ouvrent"
        );
    }

    // ── Le droit commun ────────────────────────────────────────────

    /// Art. 3.87 § 8 : « sauf si la loi exige une majorité qualifiée ».
    #[test]
    fn happy_ce_qui_nest_pas_enumere_releve_de_la_majorite_absolue() {
        assert_eq!(
            NatureDeDecision::DroitCommun.majorite_requise(),
            MajorityType::Absolute
        );
    }

    /// La démolition totale relève de deux régimes selon son motif.
    #[test]
    fn edge_la_demolition_totale_change_de_majorite_selon_son_motif() {
        assert_eq!(
            NatureDeDecision::DemolitionReconstructionPourSalubriteOuCout.majorite_requise(),
            MajorityType::FourFifths,
            "salubrité, sécurité ou coût excessif : 4/5 (§ 1er, 2°, h)"
        );
        assert_eq!(
            NatureDeDecision::DemolitionReconstructionAutreMotif.majorite_requise(),
            MajorityType::Unanimity,
            "tout autre motif : unanimité (§ 3)"
        );
    }
}
