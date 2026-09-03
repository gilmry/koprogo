//! Contexte borné **copropriété** — le noyau légal.
//!
//! Répond du Code civil, Livre 3, chapitre « De la copropriété forcée des
//! immeubles ou groupes d'immeubles bâtis » (Art. 3.84 à 3.100), texte
//! coordonné consulté sur Justel le 2026-09-02.
//!
//! Ce contexte ne connaît aucun autre : ni la plateforme qui l'héberge, ni
//! la norme comptable qui traduit ses chiffres, ni les fonctions
//! communautaires qui s'y greffent. Un identifiant venu d'ailleurs y reste
//! un `Uuid` nu — c'est ce qui empêche une notion de plateforme de
//! redevenir une clé d'accès à une notion légale (ADR-0045).
//!
//! Voir RFC-0002 pour le registre des invariants et leur couverture.
//!
//! **Arbitrages de placement.** `owner` y figure parce qu'un copropriétaire
//! est d'abord un membre de l'assemblée générale (Art. 3.87 § 1er), pas une
//! ligne de compte ; le mettre du côté comptable rejouerait l'erreur que
//! l'ADR-0045 corrige. `document` y figure pour le registre des décisions
//! (Art. 3.93 § 4), `technical_inspection` pour l'obligation de conservation
//! de l'immeuble qui est l'objet même de l'ACP (Art. 3.86 § 3), et
//! `contract_evaluation` pour l'évaluation annuelle des contrats
//! (Art. 3.89 § 5, 12°).


pub mod acp;
pub mod ag_session;
pub mod age_request;
pub mod board_decision;
pub mod board_member;
pub mod building;
pub mod conflit_dinterets;
pub mod consignation_pv;
pub mod commissaire_aux_comptes;
pub mod conseil_de_copropriete;
pub mod contrat_lie;
pub mod convocation;
pub mod convocation_recipient;
pub mod document;
pub mod mandate;
pub mod fenetre_ag_ordinaire;
pub mod fonds_de_reserve;
pub mod majorites;
pub mod meeting;
pub mod personnalite_juridique;
pub mod poll;
pub mod procurations;
pub mod poll_vote;
pub mod releve_notaire;
pub mod requete_ag;
pub mod resolution;
pub mod solidarite;
pub mod syndic_mandate;
pub mod syndic_response;
pub mod technical_inspection;
pub mod unit;
pub mod unit_owner;
pub mod vote;
pub mod owner;
pub mod contract_evaluation;

pub use acp::{
    Acp, AcpError, AcpLegalStatus, AcpMetrics, AcpNotConformantError, ReserveFundInsufficientError,
};
pub use ag_session::{AgSession, AgSessionStatus, VideoPlatform};
pub use age_request::{AgeRequest, AgeRequestCosignatory, AgeRequestStatus};
pub use board_decision::{BoardDecision, DecisionStatus};
pub use board_member::{BoardMember, BoardPosition};
pub use building::{Building, BuildingMetrics, BuildingNotConformantError};
pub use convocation::{Convocation, ConvocationStatus, ConvocationType};
pub use convocation_recipient::{AttendanceStatus, ConvocationRecipient};
pub use document::{Document, DocumentType};
pub use mandate::{Mandate, MandateKind, MandateScope, MAX_MANDATE_DURATION_DAYS};
pub use meeting::{
    Meeting, MeetingCompletionChecklist, MeetingNotCompletableError, MeetingStatus, MeetingType,
    MissingInvariant,
};
pub use poll::{Poll, PollOption, PollStatus, PollType};
pub use poll_vote::PollVote;
pub use resolution::{MajorityType, Resolution, ResolutionStatus, ResolutionType};
pub use syndic_mandate::{SyndicMandate, SyndicMandateError};
pub use syndic_response::{
    sla_window_for_severity, SyndicResponse, ALLOWED_ACTIONS, MAX_RESPONSE_BODY_LEN,
    MIN_RESPONSE_BODY_LEN,
};
pub use technical_inspection::{
    InspectionStatus, InspectionType, TechnicalInspection, TechnicalInspectionError,
};
pub use unit::{Unit, UnitType};
pub use unit_owner::{
    assert_single_voting_representative, assert_voting_right_active, voting_right_status,
    LotHolder, OwnershipType, UnitOwner, VotingRightError, VotingRightStatus,
    VotingRightSuspendedError,
};
pub use vote::{Vote, VoteChoice};
pub use owner::Owner;
pub use contract_evaluation::ContractEvaluation;
pub use procurations::{verifier_procurations, ProcurationRefusee};
pub use conflit_dinterets::{verifier_conflit_dinterets, ConflitDinterets};
pub use solidarite::{poursuivables_pour_le_tout, repartir_charge, Obligation, Titulaire};
pub use fenetre_ag_ordinaire::{FenetreAgOrdinaire, FenetreInvalide};
pub use fonds_de_reserve::{DotationInsuffisante, StatutFondsReserve};
pub use personnalite_juridique::{personnalite, PersonnaliteJuridique};
pub use consignation_pv::{echeance as echeance_consignation_pv, EtatConsignation};
pub use requete_ag::{deposer as deposer_requete_ag, RequeteAg, RequeteIrrecevable};
pub use contrat_lie::{autorisation_valable, ContratRefuse, LienAvecLeSyndic};
pub use majorites::{majorite_pour_modifier_les_quotes_parts, NatureDeDecision};
pub use releve_notaire::{DemandeDeReleve, EtatDemande};
pub use commissaire_aux_comptes::{commissariat_de_lexercice, Commissariat};
pub use conseil_de_copropriete::{MembreDuConseil, RegimeDuConseil};
