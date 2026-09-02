//! Façade de transition vers les contextes bornés du domaine.
//!
//! Cette couche plate a longtemps été le domaine tout entier : soixante-dix
//! entités côte à côte, sans frontière entre le Code civil, la norme
//! comptable, les fonctions communautaires et la plateforme SaaS. Les
//! règles y fuyaient d'un univers à l'autre (ADR-0045, RFC-0002).
//!
//! Elle ne re-exporte plus que ce qui vit désormais dans
//! `domain::copropriete`, `domain::comptabilite`,
//! `domain::economie_circulaire` et `domain::plateforme`. Elle existe pour
//! que les sites d'appel convergent au fil des reprises plutôt qu'en une
//! seule secousse.
//!
//! **Rien ne s'ajoute ici.** Une entité neuve appartient à un contexte.
//! `tests/architecture.rs` fige le solde.


// --- copropriete ---
pub use crate::domain::copropriete::acp;
pub use crate::domain::copropriete::ag_session;
pub use crate::domain::copropriete::age_request;
pub use crate::domain::copropriete::board_decision;
pub use crate::domain::copropriete::board_member;
pub use crate::domain::copropriete::building;
pub use crate::domain::copropriete::convocation;
pub use crate::domain::copropriete::convocation_recipient;
pub use crate::domain::copropriete::document;
pub use crate::domain::copropriete::mandate;
pub use crate::domain::copropriete::meeting;
pub use crate::domain::copropriete::poll;
pub use crate::domain::copropriete::poll_vote;
pub use crate::domain::copropriete::resolution;
pub use crate::domain::copropriete::syndic_mandate;
pub use crate::domain::copropriete::syndic_response;
pub use crate::domain::copropriete::technical_inspection;
pub use crate::domain::copropriete::unit;
pub use crate::domain::copropriete::unit_owner;
pub use crate::domain::copropriete::vote;
pub use crate::domain::copropriete::acp::{
    Acp, AcpError, AcpLegalStatus, AcpMetrics, AcpNotConformantError, ReserveFundInsufficientError,
};
pub use crate::domain::copropriete::ag_session::{AgSession, AgSessionStatus, VideoPlatform};
pub use crate::domain::copropriete::age_request::{AgeRequest, AgeRequestCosignatory, AgeRequestStatus};
pub use crate::domain::copropriete::board_decision::{BoardDecision, DecisionStatus};
pub use crate::domain::copropriete::board_member::{BoardMember, BoardPosition};
pub use crate::domain::copropriete::building::{Building, BuildingMetrics, BuildingNotConformantError};
pub use crate::domain::copropriete::convocation::{Convocation, ConvocationStatus, ConvocationType};
pub use crate::domain::copropriete::convocation_recipient::{AttendanceStatus, ConvocationRecipient};
pub use crate::domain::copropriete::document::{Document, DocumentType};
pub use crate::domain::copropriete::mandate::{Mandate, MandateKind, MandateScope, MAX_MANDATE_DURATION_DAYS};
pub use crate::domain::copropriete::meeting::{
    Meeting, MeetingCompletionChecklist, MeetingNotCompletableError, MeetingStatus, MeetingType,
    MissingInvariant,
};
pub use crate::domain::copropriete::poll::{Poll, PollOption, PollStatus, PollType};
pub use crate::domain::copropriete::poll_vote::PollVote;
pub use crate::domain::copropriete::resolution::{MajorityType, Resolution, ResolutionStatus, ResolutionType};
pub use crate::domain::copropriete::syndic_mandate::{SyndicMandate, SyndicMandateError};
pub use crate::domain::copropriete::syndic_response::{
    sla_window_for_severity, SyndicResponse, ALLOWED_ACTIONS, MAX_RESPONSE_BODY_LEN,
    MIN_RESPONSE_BODY_LEN,
};
pub use crate::domain::copropriete::technical_inspection::{
    InspectionStatus, InspectionType, TechnicalInspection, TechnicalInspectionError,
};
pub use crate::domain::copropriete::unit::{Unit, UnitType};
pub use crate::domain::copropriete::unit_owner::{
    assert_single_voting_representative, assert_voting_right_active, voting_right_status,
    LotHolder, OwnershipType, UnitOwner, VotingRightError, VotingRightStatus,
    VotingRightSuspendedError,
};
pub use crate::domain::copropriete::vote::{Vote, VoteChoice};

// --- comptabilite ---
pub use crate::domain::comptabilite::account;
pub use crate::domain::comptabilite::budget;
pub use crate::domain::comptabilite::call_for_funds;
pub use crate::domain::comptabilite::charge_distribution;
pub use crate::domain::comptabilite::etat_date;
pub use crate::domain::comptabilite::expense;
pub use crate::domain::comptabilite::invoice_line_item;
pub use crate::domain::comptabilite::journal_entry;
pub use crate::domain::copropriete::owner;
pub use crate::domain::comptabilite::owner_contribution;
pub use crate::domain::comptabilite::owner_credit_balance;
pub use crate::domain::comptabilite::payment;
pub use crate::domain::comptabilite::payment_method;
pub use crate::domain::comptabilite::payment_reminder;
pub use crate::domain::comptabilite::quote;
pub use crate::domain::comptabilite::account::{Account, AccountType};
pub use crate::domain::comptabilite::budget::{Budget, BudgetStatus};
pub use crate::domain::comptabilite::call_for_funds::{CallForFunds, CallForFundsError, CallForFundsStatus};
pub use crate::domain::comptabilite::charge_distribution::{ChargeDistribution, ChargeDistributionError, DistributionCriteria};
pub use crate::domain::comptabilite::etat_date::{EtatDate, EtatDateError, EtatDateLanguage, EtatDateStatus};
pub use crate::domain::comptabilite::expense::{ApprovalStatus, Expense, ExpenseCategory, PaymentStatus};
pub use crate::domain::comptabilite::invoice_line_item::InvoiceLineItem;
pub use crate::domain::comptabilite::journal_entry::{JournalEntry, JournalEntryError, JournalEntryLine};
pub use crate::domain::copropriete::owner::Owner;
pub use crate::domain::comptabilite::owner_contribution::{
    ContributionPaymentMethod, ContributionPaymentStatus, ContributionType, OwnerContribution,
    OwnerContributionError,
};
pub use crate::domain::comptabilite::owner_credit_balance::{CreditStatus, OwnerCreditBalance, ParticipationLevel};
pub use crate::domain::comptabilite::payment::{Payment, PaymentMethodType, TransactionStatus};
pub use crate::domain::comptabilite::payment_method::PaymentMethod;
pub use crate::domain::comptabilite::payment_reminder::{DeliveryMethod, PaymentReminder, ReminderLevel, ReminderStatus};
pub use crate::domain::comptabilite::quote::{Quote, QuoteScore, QuoteStatus, QuoteSubmission};

// --- economie_circulaire ---
pub use crate::domain::economie_circulaire::achievement;
pub use crate::domain::economie_circulaire::challenge;
pub use crate::domain::copropriete::contract_evaluation;
pub use crate::domain::economie_circulaire::contractor_evaluation;
pub use crate::domain::economie_circulaire::contractor_report;
pub use crate::domain::economie_circulaire::energy_bill_upload;
pub use crate::domain::economie_circulaire::energy_campaign;
pub use crate::domain::economie_circulaire::individual_member;
pub use crate::domain::economie_circulaire::iot_reading;
pub use crate::domain::economie_circulaire::linky_device;
pub use crate::domain::economie_circulaire::local_exchange;
pub use crate::domain::economie_circulaire::notice;
pub use crate::domain::economie_circulaire::resource_booking;
pub use crate::domain::economie_circulaire::service_provider;
pub use crate::domain::economie_circulaire::shared_object;
pub use crate::domain::economie_circulaire::skill;
pub use crate::domain::economie_circulaire::technical_spec;
pub use crate::domain::economie_circulaire::ticket;
pub use crate::domain::economie_circulaire::work_report;
pub use crate::domain::economie_circulaire::achievement::{Achievement, AchievementCategory, AchievementTier, UserAchievement};
pub use crate::domain::economie_circulaire::challenge::{Challenge, ChallengeProgress, ChallengeStatus, ChallengeType};
pub use crate::domain::copropriete::contract_evaluation::ContractEvaluation;
pub use crate::domain::economie_circulaire::contractor_evaluation::{
    ContractorEvaluation, EvaluationScores, MAX_COMMENT_LEN as CONTRACTOR_EVAL_MAX_COMMENT_LEN,
    MAX_LINKED_TICKETS as CONTRACTOR_EVAL_MAX_LINKED_TICKETS,
    MAX_SCORE as CONTRACTOR_EVAL_MAX_SCORE, MIN_COMMENT_LEN as CONTRACTOR_EVAL_MIN_COMMENT_LEN,
    MIN_SCORE as CONTRACTOR_EVAL_MIN_SCORE,
};
pub use crate::domain::economie_circulaire::contractor_report::{ContractorReport, ContractorReportStatus, ReplacedPart};
pub use crate::domain::economie_circulaire::energy_bill_upload::EnergyBillUpload;
pub use crate::domain::economie_circulaire::energy_campaign::{
    CampaignStatus, CampaignType, ContractType, EnergyCampaign, EnergyType, ProviderOffer,
};
pub use crate::domain::economie_circulaire::individual_member::IndividualMember;
pub use crate::domain::economie_circulaire::iot_reading::{DeviceType, IoTReading, MetricType};
pub use crate::domain::economie_circulaire::linky_device::{LinkyDevice, LinkyProvider};
pub use crate::domain::economie_circulaire::local_exchange::{ExchangeStatus, ExchangeType, LocalExchange};
pub use crate::domain::economie_circulaire::notice::{Notice, NoticeCategory, NoticeStatus, NoticeType};
pub use crate::domain::economie_circulaire::resource_booking::{BookingStatus, RecurringPattern, ResourceBooking, ResourceType};
pub use crate::domain::economie_circulaire::service_provider::{ServiceProvider, TradeCategory};
pub use crate::domain::economie_circulaire::shared_object::{ObjectCondition, SharedObject, SharedObjectCategory};
pub use crate::domain::economie_circulaire::skill::{ExpertiseLevel, Skill, SkillCategory};
pub use crate::domain::economie_circulaire::technical_spec::{
    SemVer, SignatoryRole, TechnicalSpec, TechnicalSpecSignature, TechnicalSpecStatus,
    MAX_ATTACHMENTS as TECH_SPEC_MAX_ATTACHMENTS, MAX_DELIVERABLES as TECH_SPEC_MAX_DELIVERABLES,
    MAX_DESCRIPTION_LEN as TECH_SPEC_MAX_DESCRIPTION_LEN,
    MAX_REQUIRED_SIGNATURES as TECH_SPEC_MAX_REQUIRED_SIGNATURES,
    MAX_TITLE_LEN as TECH_SPEC_MAX_TITLE_LEN, MIN_DESCRIPTION_LEN as TECH_SPEC_MIN_DESCRIPTION_LEN,
    MIN_TITLE_LEN as TECH_SPEC_MIN_TITLE_LEN,
};
pub use crate::domain::economie_circulaire::ticket::{
    Ticket, TicketCategory, TicketKind, TicketPriority, TicketSeverity, TicketStatus,
    MAX_EVIDENCE_ATTACHMENTS, MAX_WITNESSES, TICKET_EDIT_WINDOW_MINUTES,
};
pub use crate::domain::economie_circulaire::work_report::{WarrantyType, WorkReport, WorkReportError, WorkType};

// --- plateforme ---
pub use crate::domain::plateforme::consent;
pub use crate::domain::plateforme::gdpr_art30;
pub use crate::domain::plateforme::gdpr_export;
pub use crate::domain::plateforme::gdpr_objection;
pub use crate::domain::plateforme::gdpr_rectification;
pub use crate::domain::plateforme::gdpr_restriction;
pub use crate::domain::plateforme::magic_link;
pub use crate::domain::plateforme::notification;
pub use crate::domain::plateforme::organization;
pub use crate::domain::plateforme::portfolio;
pub use crate::domain::plateforme::refresh_token;
pub use crate::domain::plateforme::security_incident;
pub use crate::domain::plateforme::two_factor_secret;
pub use crate::domain::plateforme::user;
pub use crate::domain::plateforme::user_role_assignment;
pub use crate::domain::plateforme::consent::{ConsentRecord, ConsentStatus};
pub use crate::domain::plateforme::gdpr_art30::{ProcessingActivity, ProcessorAgreement};
pub use crate::domain::plateforme::gdpr_export::{
    DocumentData, ExpenseData, GdprExport, MeetingData, OwnerData, RelatedData, UnitOwnershipData,
    UserData,
};
pub use crate::domain::plateforme::gdpr_objection::{GdprObjectionRequest, ObjectionStatus, ObjectionType, ProcessingPurpose};
pub use crate::domain::plateforme::gdpr_rectification::{FieldChange, GdprRectificationRequest, RectificationStatus};
pub use crate::domain::plateforme::gdpr_restriction::{GdprRestrictionRequest, RestrictionReason, RestrictionStatus};
pub use crate::domain::plateforme::magic_link::{MagicLink, MagicLinkScopeKind};
pub use crate::domain::plateforme::notification::{
    Notification, NotificationChannel, NotificationPreference, NotificationPriority,
    NotificationStatus, NotificationType,
};
pub use crate::domain::plateforme::organization::{Organization, SubscriptionPlan};
pub use crate::domain::plateforme::portfolio::{Portfolio, PortfolioBuilding, PortfolioError, PortfolioShare};
pub use crate::domain::plateforme::refresh_token::RefreshToken;
pub use crate::domain::plateforme::security_incident::{IncidentSeverity, IncidentStatus, SecurityIncident};
pub use crate::domain::plateforme::two_factor_secret::TwoFactorSecret;
pub use crate::domain::plateforme::user::{User, UserRole};
pub use crate::domain::plateforme::user_role_assignment::UserRoleAssignment;
