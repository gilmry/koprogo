//! Contexte borné **plateforme** — le service, pas le métier.
//!
//! Répond du RGPD, des exigences de sécurité et du contrat SaaS :
//! authentification, rôles, consentements, droits des personnes, incidents.
//!
//! Ne dépend d'aucun autre contexte, et surtout n'est dépendu par aucun. Le
//! jour où une règle légale s'est mise à passer par `organization`, le
//! dossier de gestion a cessé d'appartenir à l'ACP (ADR-0045). Cette
//! frontière existe pour que ça ne se reproduise pas.


pub mod consent;
pub mod gdpr_art30;
pub mod gdpr_export;
pub mod gdpr_objection;
pub mod gdpr_rectification;
pub mod gdpr_restriction;
pub mod magic_link;
pub mod notification;
pub mod organization;
pub mod portfolio;
pub mod refresh_token;
pub mod security_incident;
pub mod two_factor_secret;
pub mod user;
pub mod user_role_assignment;

pub use consent::{ConsentRecord, ConsentStatus};
pub use gdpr_art30::{ProcessingActivity, ProcessorAgreement};
pub use gdpr_export::{
    DocumentData, ExpenseData, GdprExport, MeetingData, OwnerData, RelatedData, UnitOwnershipData,
    UserData,
};
pub use gdpr_objection::{GdprObjectionRequest, ObjectionStatus, ObjectionType, ProcessingPurpose};
pub use gdpr_rectification::{FieldChange, GdprRectificationRequest, RectificationStatus};
pub use gdpr_restriction::{GdprRestrictionRequest, RestrictionReason, RestrictionStatus};
pub use magic_link::{MagicLink, MagicLinkScopeKind};
pub use notification::{
    Notification, NotificationChannel, NotificationPreference, NotificationPriority,
    NotificationStatus, NotificationType,
};
pub use organization::{Organization, SubscriptionPlan};
pub use portfolio::{Portfolio, PortfolioBuilding, PortfolioError, PortfolioShare};
pub use refresh_token::RefreshToken;
pub use security_incident::{IncidentSeverity, IncidentStatus, SecurityIncident};
pub use two_factor_secret::TwoFactorSecret;
pub use user::{User, UserRole};
pub use user_role_assignment::UserRoleAssignment;
