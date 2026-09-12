//! Le dossier de gestion d'une ACP, au sens de l'Art. 3.89 § 5, 7° du Code civil.
//!
//! Le syndic sortant doit remettre à son successeur, dans les trente jours,
//! *« l'ensemble du dossier de la gestion de l'immeuble, y compris la
//! comptabilité et les archives »*. La loi désigne donc un ensemble hétérogène
//! de pièces (budgets, écritures, appels de fonds, convocations…) qui ont un
//! point commun : elles appartiennent à l'**ACP**, personne morale (Art. 3.86),
//! et non au mandataire qui les a produites.
//!
//! Ce module rend ce fait explicite dans le code plutôt que de le laisser
//! deviner : une pièce sait de quelle ACP elle relève, et le périmètre d'un
//! syndic se **dérive** de son mandat au lieu d'être gravé dans la pièce.
//!
//! Voir ADR-0045.

use crate::domain::entities::SyndicMandate;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Une pièce du dossier de gestion.
///
/// Implémenter ce trait, c'est déclarer qu'une entité est un acte de gestion
/// posé pour le compte d'une ACP, et donc qu'elle se transmet avec elle.
pub trait PieceDeGestion {
    /// L'ACP dont relève la pièce. C'est elle qui en est propriétaire.
    fn acp_id(&self) -> Uuid;
}

/// Les pièces qu'un syndic peut légitimement consulter à un moment donné.
///
/// Le filtre n'interroge jamais l'auteur de la pièce : il interroge le mandat.
/// Un syndic voit ce que son mandat lui confie, ni plus, ni après.
pub fn perimetre_du_mandataire<'a>(
    pieces: &'a [&'a dyn PieceDeGestion],
    mandats: &[SyndicMandate],
    syndic: Uuid,
    moment: DateTime<Utc>,
) -> Vec<&'a dyn PieceDeGestion> {
    pieces
        .iter()
        .filter(|piece| {
            let acp = piece.acp_id();
            mandats
                .iter()
                .filter(|m| m.acp_id == acp)
                .any(|m| m.covers(moment) && m.organization_id == syndic)
        })
        .copied()
        .collect()
}

/// Le délai de l'Art. 3.89 § 5, 7°, en jours calendaires.
///
/// Calendaires, comme les autres délais de ce chapitre : l'Art. 3.31 § 2 dit
/// « jour ouvrable » quand il le veut, et son silence ailleurs est délibéré.
/// Même choix que `releve_notaire::DELAI_JOURS`, pour la même raison.
pub const DELAI_PASSATION_JOURS: i64 = 30;

/// L'état d'une passation, du point de vue du syndic sortant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtatPassation {
    /// Dossier remis dans les trente jours.
    RemisATemps,
    /// Remis, mais après l'échéance.
    RemisEnRetard,
    /// Pas encore remis, délai non écoulé.
    EnCours,
    /// Pas remis, délai écoulé. Le successeur gère sans les archives.
    EnDefaut,
}

/// La remise du dossier au syndic successeur, et son échéance.
///
/// ── Pourquoi cet objet existe ────────────────────────────────────────────
///
/// Le module documentait le délai de trente jours depuis toujours, en tête de
/// fichier, et ne le tenait **nulle part** : aucune échéance, aucun état, rien
/// qui puisse être dépassé. Le registre légal déclarait pourtant l'obligation
/// attestée par un test qui vérifie que le successeur voit l'ensemble des
/// pièces — ce qui est vrai, et ne dit rien du délai (#847).
///
/// Un dossier remis en retard n'est pas une formalité manquée : le successeur
/// gère une copropriété dont il ignore les dettes, les procédures en cours et
/// les décisions d'assemblée. Il engage sa responsabilité sur des faits qu'il
/// n'a pas pu connaître.
#[derive(Debug, Clone, PartialEq)]
pub struct PassationDeDossier {
    pub acp_id: Uuid,
    pub syndic_sortant: Uuid,
    pub syndic_entrant: Uuid,
    /// Fin du mandat sortant — c'est elle qui fait courir le délai.
    pub fin_de_mandat: DateTime<Utc>,
    pub echeance: DateTime<Utc>,
    /// Date de remise effective, si elle a eu lieu.
    pub remis_le: Option<DateTime<Utc>>,
}

impl PassationDeDossier {
    pub fn nouvelle(
        acp_id: Uuid,
        syndic_sortant: Uuid,
        syndic_entrant: Uuid,
        fin_de_mandat: DateTime<Utc>,
    ) -> Self {
        Self {
            acp_id,
            syndic_sortant,
            syndic_entrant,
            fin_de_mandat,
            echeance: fin_de_mandat + chrono::Duration::days(DELAI_PASSATION_JOURS),
            remis_le: None,
        }
    }

    pub fn remettre(&mut self, le: DateTime<Utc>) {
        self.remis_le = Some(le);
    }

    pub fn etat(&self, moment: DateTime<Utc>) -> EtatPassation {
        match self.remis_le {
            Some(remis) if remis <= self.echeance => EtatPassation::RemisATemps,
            Some(_) => EtatPassation::RemisEnRetard,
            None if moment <= self.echeance => EtatPassation::EnCours,
            None => EtatPassation::EnDefaut,
        }
    }

    /// Combien de jours restent avant l'échéance ? Négatif une fois dépassée.
    ///
    /// Sert à prévenir **avant** plutôt qu'à constater après : c'est la seule
    /// forme utile d'un délai légal dans un logiciel de gestion.
    pub fn jours_restants(&self, moment: DateTime<Utc>) -> i64 {
        (self.echeance - moment).num_days()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{
        Budget, CallForFunds, ContributionType, Convocation, ConvocationType, EtatDate,
        EtatDateLanguage, Expense, ExpenseCategory, JournalEntry, JournalEntryLine, Meeting,
        MeetingType, OwnerContribution, PaymentReminder, ReminderLevel,
    };
    use chrono::Duration;
    use rust_decimal_macros::dec;

    /// Le dossier de gestion d'une ACP conforme, tel qu'il se présente le jour
    /// d'une passation. Chaque champ est une famille de pièces que
    /// l'Art. 3.89 § 5, 7° oblige à transmettre.
    struct DossierComplet {
        charge: Expense,
        budget: Budget,
        appel_de_fonds: CallForFunds,
        quote_part: OwnerContribution,
        ecriture: JournalEntry,
        assemblee: Meeting,
        convocation: Convocation,
        etat_date: EtatDate,
        relance: PaymentReminder,
    }

    impl DossierComplet {
        fn pour(acp: Uuid, syndic: Uuid, immeuble: Uuid) -> Self {
            Self {
                charge: Expense::new(
                    acp,
                    syndic,
                    immeuble,
                    ExpenseCategory::Maintenance,
                    "Entretien de la chaudière".to_string(),
                    dec!(1200.00),
                    Utc::now(),
                    None,
                    None,
                    None,
                )
                .expect("charge valide"),
                budget: Budget::new(acp, syndic, immeuble, 2026, dec!(48000.00), dec!(12000.00))
                    .expect("budget valide"),
                appel_de_fonds: CallForFunds::new(
                    acp,
                    syndic,
                    immeuble,
                    "Provision T1 2026".to_string(),
                    "Charges ordinaires du premier trimestre".to_string(),
                    dec!(12000.00),
                    ContributionType::Regular,
                    Utc::now(),
                    Utc::now() + Duration::days(30),
                    None,
                    rust_decimal::Decimal::ZERO, // part fonds de réserve
                )
                .expect("appel de fonds valide"),
                quote_part: OwnerContribution::new(
                    acp,
                    syndic,
                    Uuid::new_v4(),
                    Some(Uuid::new_v4()),
                    "Quote-part provision T1 2026".to_string(),
                    dec!(1200.00),
                    ContributionType::Regular,
                    Utc::now(),
                    Some("7000".to_string()),
                )
                .expect("quote-part valide"),
                ecriture: {
                    let id = Uuid::new_v4();
                    JournalEntry::new(
                        acp,
                        syndic,
                        Some(immeuble),
                        Utc::now(),
                        Some("Entretien de la chaudière".to_string()),
                        None,
                        Some("ACH".to_string()),
                        None,
                        None,
                        vec![
                            JournalEntryLine::new_debit(
                                id,
                                syndic,
                                "610".to_string(),
                                dec!(1200.00),
                                None,
                            )
                            .expect("débit valide"),
                            JournalEntryLine::new_credit(
                                id,
                                syndic,
                                "440".to_string(),
                                dec!(1200.00),
                                None,
                            )
                            .expect("crédit valide"),
                        ],
                        None,
                    )
                    .expect("écriture valide")
                },
                assemblee: Meeting::new(
                    acp,
                    syndic,
                    immeuble,
                    MeetingType::Ordinary,
                    "AGO 2026".to_string(),
                    None,
                    Utc::now() + Duration::days(30),
                    "Salle communale".to_string(),
                )
                .expect("assemblée valide"),
                convocation: Convocation::new(
                    acp,
                    syndic,
                    immeuble,
                    Uuid::new_v4(),
                    ConvocationType::Ordinary,
                    Utc::now() + Duration::days(30),
                    "FR".to_string(),
                    Uuid::new_v4(),
                )
                .expect("convocation valide"),
                etat_date: EtatDate::new(
                    acp,
                    syndic,
                    immeuble,
                    Uuid::new_v4(),
                    Utc::now(),
                    EtatDateLanguage::Fr,
                    "Me Dupont".to_string(),
                    "dupont@notaire.be".to_string(),
                    None,
                    "Résidence du Parc".to_string(),
                    "12 Rue de la Loi".to_string(),
                    "A101".to_string(),
                    Some("1".to_string()),
                    Some(85.0),
                    dec!(100),
                    dec!(100),
                )
                .expect("état daté valide"),
                relance: PaymentReminder::new(
                    acp,
                    syndic,
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    ReminderLevel::FirstReminder,
                    dec!(450.00),
                    Utc::now() - Duration::days(45),
                    45,
                )
                .expect("relance valide"),
            }
        }

        fn pieces(&self) -> Vec<&dyn PieceDeGestion> {
            vec![
                &self.charge,
                &self.budget,
                &self.appel_de_fonds,
                &self.quote_part,
                &self.ecriture,
                &self.assemblee,
                &self.convocation,
                &self.etat_date,
                &self.relance,
            ]
        }
    }

    /// Art. 3.89 § 5, 7° : à la passation, le dossier passe en entier au
    /// successeur, et cesse d'être accessible au sortant.
    /// Art. 3.89 § 5, 7° : le dossier est transmis **dans les trente jours**.
    ///
    /// Le module portait ce délai en commentaire de tête depuis toujours et ne
    /// le tenait nulle part. Le registre légal le déclarait pourtant attesté,
    /// par un test qui vérifie que le successeur voit l'ensemble des pièces —
    /// vrai, et muet sur le délai (#847). C'est la variante la plus sournoise
    /// du motif : une preuve qui existe, qui passe, et qui prouve autre chose.
    #[test]
    fn negative_passe_trente_jours_sans_remise_le_syndic_sortant_est_en_defaut() {
        let acp = Uuid::new_v4();
        let sortant = Uuid::new_v4();
        let entrant = Uuid::new_v4();
        let fin = Utc::now() - Duration::days(40);

        let passation = PassationDeDossier::nouvelle(acp, sortant, entrant, fin);

        // L'échéance tombe trente jours après la fin du mandat, pas un de plus.
        assert_eq!(
            passation.echeance,
            fin + Duration::days(30),
            "le délai court depuis la fin du mandat"
        );

        // Le vingt-neuvième jour, le sortant est encore dans les temps.
        assert_eq!(
            passation.etat(fin + Duration::days(29)),
            EtatPassation::EnCours
        );
        assert_eq!(passation.jours_restants(fin + Duration::days(29)), 1);

        // Le trente et unième, il est en défaut : le successeur gère une
        // copropriété dont il ignore les dettes et les procédures en cours.
        assert_eq!(
            passation.etat(fin + Duration::days(31)),
            EtatPassation::EnDefaut
        );

        // Une remise tardive ne rétroagit pas : elle est constatée en retard.
        let mut tardive = passation.clone();
        tardive.remettre(fin + Duration::days(45));
        assert_eq!(
            tardive.etat(Utc::now()),
            EtatPassation::RemisEnRetard,
            "remettre après l'échéance ne régularise rien"
        );

        // Remise dans les temps : honorée, quel que soit le moment où on juge.
        let mut a_temps = passation.clone();
        a_temps.remettre(fin + Duration::days(10));
        assert_eq!(a_temps.etat(Utc::now()), EtatPassation::RemisATemps);
    }

    #[test]
    fn le_dossier_de_gestion_suit_lacp_lors_dune_passation() {
        let acp = Uuid::new_v4();
        let immeuble = Uuid::new_v4();
        let cabinet_sortant = Uuid::new_v4();
        let cabinet_entrant = Uuid::new_v4();

        let passation = Utc::now();
        let avant = passation - Duration::days(30);
        let apres = passation + Duration::days(1);

        // Le dossier a été constitué par le cabinet sortant, pour l'ACP.
        let dossier = DossierComplet::pour(acp, cabinet_sortant, immeuble);
        let pieces = dossier.pieces();

        let mut mandat_sortant = SyndicMandate::new(acp, cabinet_sortant, avant, None);
        mandat_sortant
            .revoke(
                passation,
                None,
                Some("Fin de mandat votée en AG".to_string()),
            )
            .expect("révocation valide");
        let mandat_entrant = SyndicMandate::new(acp, cabinet_entrant, passation, None);
        let mandats = vec![mandat_sortant, mandat_entrant];

        // Avant la passation : le sortant tient le dossier, l'entrant n'existe pas encore.
        assert_eq!(
            perimetre_du_mandataire(&pieces, &mandats, cabinet_sortant, avant).len(),
            pieces.len(),
            "le mandataire en fonction doit voir tout le dossier"
        );
        assert!(
            perimetre_du_mandataire(&pieces, &mandats, cabinet_entrant, avant).is_empty(),
            "un cabinet sans mandat ne voit rien, même une pièce future"
        );

        // Après la passation : le dossier a suivi l'ACP, sans qu'une seule pièce bouge.
        assert_eq!(
            perimetre_du_mandataire(&pieces, &mandats, cabinet_entrant, apres).len(),
            pieces.len(),
            "Art. 3.89 § 5, 7° : le successeur reçoit l'ensemble du dossier"
        );
        assert!(
            perimetre_du_mandataire(&pieces, &mandats, cabinet_sortant, apres).is_empty(),
            "le mandat éteint, le sortant n'a plus de base pour consulter le dossier"
        );
    }

    /// Le dossier reste rattaché même quand personne ne le gère : une ACP
    /// entre deux mandats n'est pas une ACP sans comptabilité.
    #[test]
    fn une_acp_sans_mandataire_conserve_son_dossier() {
        let acp = Uuid::new_v4();
        let ancien_syndic = Uuid::new_v4();
        let passation = Utc::now();

        let dossier = DossierComplet::pour(acp, ancien_syndic, Uuid::new_v4());
        let pieces = dossier.pieces();

        let mut mandat =
            SyndicMandate::new(acp, ancien_syndic, passation - Duration::days(90), None);
        mandat
            .revoke(passation, None, None)
            .expect("révocation valide");

        assert!(
            perimetre_du_mandataire(&pieces, &[mandat], ancien_syndic, passation).is_empty(),
            "plus personne ne voit le dossier"
        );
        for piece in &pieces {
            assert_eq!(
                piece.acp_id(),
                acp,
                "mais chaque pièce sait encore à qui elle est"
            );
        }
    }

    /// Deux ACP confiées au même cabinet ne se mélangent pas.
    #[test]
    fn un_syndic_ne_voit_pas_le_dossier_dune_acp_quil_ne_gere_pas() {
        let acp_geree = Uuid::new_v4();
        let acp_voisine = Uuid::new_v4();
        let cabinet = Uuid::new_v4();
        let maintenant = Utc::now();

        let dossier_voisin = DossierComplet::pour(acp_voisine, cabinet, Uuid::new_v4());
        let pieces = dossier_voisin.pieces();

        let mandat = SyndicMandate::new(acp_geree, cabinet, maintenant - Duration::days(10), None);

        assert!(
            perimetre_du_mandataire(&pieces, &[mandat], cabinet, maintenant).is_empty(),
            "un mandat sur une ACP n'ouvre rien sur une autre, même chez le même cabinet"
        );
    }
}
