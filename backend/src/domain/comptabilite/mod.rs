//! Contexte borné **comptabilité** — la traduction chiffrée de la vie de l'ACP.
//!
//! Répond de l'Arrêté royal du 12 juillet 2012 (plan comptable minimum
//! normalisé) et de l'Art. 3.89 § 5, 15° et 16° du Code civil : tenir les
//! comptes « de manière claire, précise et détaillée » suivant ce plan, et
//! préparer les deux budgets prévisionnels soumis chaque année au vote.
//!
//! Dépend de `copropriete` : une charge se répartit sur des quotités fixées
//! par l'acte de base (Art. 3.85 § 1er), elle ne peut pas les ignorer.
//! L'inverse est faux — la loi dit ce qu'est un lot sans le plan comptable.

pub mod account;
pub mod arrieres_mutation;
pub mod budget;
pub mod call_for_funds;
pub mod charge_distribution;
pub mod etat_date;
pub mod expense;
pub mod invoice_line_item;
pub mod journal_entry;
pub mod owner_contribution;
pub mod owner_credit_balance;
pub mod payment;
pub mod payment_method;
pub mod payment_reminder;
pub mod quote;
pub mod regime_comptable;

pub use account::{Account, AccountType};
pub use arrieres_mutation::{jours_ouvrables, ArrieresARetenir};
pub use budget::{Budget, BudgetStatus};
pub use call_for_funds::{CallForFunds, CallForFundsError, CallForFundsStatus};
pub use charge_distribution::{ChargeDistribution, ChargeDistributionError, DistributionCriteria};
pub use etat_date::{EtatDate, EtatDateError, EtatDateLanguage, EtatDateStatus};
pub use expense::{ApprovalStatus, Expense, ExpenseCategory, PaymentStatus};
pub use invoice_line_item::InvoiceLineItem;
pub use journal_entry::{JournalEntry, JournalEntryError, JournalEntryLine};
pub use owner_contribution::{
    ContributionPaymentMethod, ContributionPaymentStatus, ContributionType, OwnerContribution,
    OwnerContributionError,
};
pub use owner_credit_balance::{CreditStatus, OwnerCreditBalance, ParticipationLevel};
pub use payment::{Payment, PaymentMethodType, TransactionStatus};
pub use payment_method::PaymentMethod;
pub use payment_reminder::{DeliveryMethod, PaymentReminder, ReminderLevel, ReminderStatus};
pub use quote::{Quote, QuoteScore, QuoteStatus, QuoteSubmission};
pub use regime_comptable::{lots_comptes, regime_applicable, RegimeComptable};
