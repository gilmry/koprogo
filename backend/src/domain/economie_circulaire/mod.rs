//! Contexte borné **économie circulaire** — ce que le logiciel ajoute à la loi.
//!
//! Mutualisation d'objets et de compétences, échanges locaux, réservation de
//! ressources communes, achat groupé d'énergie, suivi de consommation. C'est
//! l'augmentation du jumeau : le Code civil dit comment une copropriété se
//! gouverne, pas comment elle mutualise.
//!
//! **Il a néanmoins ses propres autorités**, et c'est pourquoi il est un
//! contexte à part entière plutôt qu'un fourre-tout produit :
//!
//! - l'achat groupé d'énergie et l'adhésion de non-copropriétaires relèvent
//!   de l'article 22 de la directive RED II sur les communautés d'énergie
//!   renouvelable (cf. `individual_member`) ;
//! - la collecte de factures d'énergie relève du RGPD, articles 7.3 et 17
//!   (cf. `energy_bill_upload`).
//!
//! Ce qui portait une obligation du Code civil en a été retiré :
//! `contract_evaluation` a rejoint `copropriete`, l'évaluation annuelle des
//! contrats étant une obligation du syndic (Art. 3.89 § 5, 12°).
//!
//! Dépend de `copropriete` : un échange, une réservation, une campagne ont
//! lieu au sein d'une ACP et n'existent pas sans elle.


pub mod achievement;
pub mod challenge;
pub mod contractor_evaluation;
pub mod contractor_report;
pub mod energy_bill_upload;
pub mod energy_campaign;
pub mod individual_member;
pub mod iot_reading;
pub mod linky_device;
pub mod local_exchange;
pub mod notice;
pub mod resource_booking;
pub mod service_provider;
pub mod shared_object;
pub mod skill;
pub mod technical_spec;
pub mod ticket;
pub mod work_report;

pub use achievement::{Achievement, AchievementCategory, AchievementTier, UserAchievement};
pub use challenge::{Challenge, ChallengeProgress, ChallengeStatus, ChallengeType};
pub use contractor_evaluation::{
    ContractorEvaluation, EvaluationScores, MAX_COMMENT_LEN as CONTRACTOR_EVAL_MAX_COMMENT_LEN,
    MAX_LINKED_TICKETS as CONTRACTOR_EVAL_MAX_LINKED_TICKETS,
    MAX_SCORE as CONTRACTOR_EVAL_MAX_SCORE, MIN_COMMENT_LEN as CONTRACTOR_EVAL_MIN_COMMENT_LEN,
    MIN_SCORE as CONTRACTOR_EVAL_MIN_SCORE,
};
pub use contractor_report::{ContractorReport, ContractorReportStatus, ReplacedPart};
pub use energy_bill_upload::EnergyBillUpload;
pub use energy_campaign::{
    CampaignStatus, CampaignType, ContractType, EnergyCampaign, EnergyType, ProviderOffer,
};
pub use individual_member::IndividualMember;
pub use iot_reading::{DeviceType, IoTReading, MetricType};
pub use linky_device::{LinkyDevice, LinkyProvider};
pub use local_exchange::{ExchangeStatus, ExchangeType, LocalExchange};
pub use notice::{Notice, NoticeCategory, NoticeStatus, NoticeType};
pub use resource_booking::{BookingStatus, RecurringPattern, ResourceBooking, ResourceType};
pub use service_provider::{ServiceProvider, TradeCategory};
pub use shared_object::{ObjectCondition, SharedObject, SharedObjectCategory};
pub use skill::{ExpertiseLevel, Skill, SkillCategory};
pub use technical_spec::{
    SemVer, SignatoryRole, TechnicalSpec, TechnicalSpecSignature, TechnicalSpecStatus,
    MAX_ATTACHMENTS as TECH_SPEC_MAX_ATTACHMENTS, MAX_DELIVERABLES as TECH_SPEC_MAX_DELIVERABLES,
    MAX_DESCRIPTION_LEN as TECH_SPEC_MAX_DESCRIPTION_LEN,
    MAX_REQUIRED_SIGNATURES as TECH_SPEC_MAX_REQUIRED_SIGNATURES,
    MAX_TITLE_LEN as TECH_SPEC_MAX_TITLE_LEN, MIN_DESCRIPTION_LEN as TECH_SPEC_MIN_DESCRIPTION_LEN,
    MIN_TITLE_LEN as TECH_SPEC_MIN_TITLE_LEN,
};
pub use ticket::{
    Ticket, TicketCategory, TicketKind, TicketPriority, TicketSeverity, TicketStatus,
    MAX_EVIDENCE_ATTACHMENTS, MAX_WITNESSES, TICKET_EDIT_WINDOW_MINUTES,
};
pub use work_report::{WarrantyType, WorkReport, WorkReportError, WorkType};
