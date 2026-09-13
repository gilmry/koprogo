use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AdminDashboardStats {
    pub total_organizations: i64,
    pub total_users: i64,
    pub total_buildings: i64,
    pub active_subscriptions: i64,
    pub total_owners: i64,
    pub total_units: i64,
    pub total_expenses: i64,
    pub total_meetings: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SeedDataStats {
    pub seed_organizations: i64,
    pub production_organizations: i64,
    pub seed_buildings: i64,
    pub seed_units: i64,
    pub seed_owners: i64,
    pub seed_unit_owners: i64,
    pub seed_expenses: i64,
    pub seed_meetings: i64,
    pub seed_users: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct NextMeetingInfo {
    pub id: String,
    pub date: DateTime<Utc>,
    pub building_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyndicDashboardStats {
    pub total_buildings: i64,
    /// Lots effectivement encodés (lignes de la table `units`).
    pub total_units: i64,
    /// Lots déclarés à l'acte de base, sommés sur les immeubles de
    /// l'organisation (`buildings.total_units`).
    ///
    /// Les deux nombres mesurent des choses différentes et étaient affichés
    /// sous le même libellé : le tableau de bord annonçait « 0 lots au
    /// total » pendant que la liste des immeubles affichait « 8 Lots ». Tous
    /// deux étaient exacts, mais l'un comptait l'encodage et l'autre la
    /// déclaration. Les exposer ensemble lève l'ambiguïté, comme le fait
    /// déjà la fiche immeuble avec son « 0/8 ».
    pub declared_units: i64,
    pub total_owners: i64,
    pub pending_expenses_count: i64,
    /// Total des dépenses en attente, en EUR.
    ///
    /// `Decimal` : la colonne `expenses.amount` est en `NUMERIC(12,2)` depuis
    /// la migration 20260502000000. La lire en `f64` était une dégradation
    /// gratuite d'une valeur déjà exacte (même défaut que celui trouvé dans
    /// `find_overdue_expenses_without_reminders` sous #661).
    ///
    /// `serde::float` conserve la représentation JSON numérique attendue par
    /// `OwnerDashboard.svelte` / `SyndicDashboard.svelte`, qui typent ce champ
    /// `number` — aucun drift de contrat.
    #[serde(with = "rust_decimal::serde::float")]
    pub pending_expenses_amount: Decimal,
    pub next_meeting: Option<NextMeetingInfo>,
}

/// Ce qu'un copropriétaire doit à UNE copropriété.
///
/// ── Pourquoi par ACP, et pas un montant global ──────────────────────────
///
/// Un copropriétaire peut détenir des lots dans plusieurs ACP — situation
/// ordinaire d'un investisseur, et `unit_owners` est une relation n:n sans
/// contrainte d'ACP unique.
///
/// Or **chaque ACP est une personne morale distincte, avec son propre compte
/// bancaire** : l'Art. 3.86 § 1er lui donne la personnalité juridique, le § 3
/// impose des comptes ouverts à son nom.
///
/// L'écran servait jusqu'ici un `SyndicDashboardStats` — un compte, un
/// montant, rien qui distingue les copropriétés. Un copropriétaire qui lit
/// « 1 262,50 € à payer » et fait un seul virement PAIE LA MAUVAISE PERSONNE
/// MORALE pour une partie de la somme : l'argent atterrit sur le compte de
/// l'ACP A pour des charges dues à l'ACP B. Le syndic de B devra réclamer,
/// celui de A rembourser.
///
/// Ce n'est donc pas un défaut d'affichage mais un paiement mal imputé.
/// Agréger pour informer, séparer pour agir (#867).
#[derive(Debug, Clone, Serialize)]
pub struct DuAupresDuneAcp {
    pub acp_id: String,
    pub acp_name: String,
    /// Le numéro d'entreprise, à recopier sur le virement.
    ///
    /// C'est lui qui identifie la personne morale créancière — l'Art. 3.86
    /// § 1er al. 4 impose d'ailleurs qu'il figure sur tous les documents qui
    /// émanent de l'association.
    pub bce_number: Option<String>,
    pub charges_en_attente: i64,
    /// Montant dû à CETTE association.
    ///
    /// `Decimal` sérialisé en flottant JSON, comme `pending_expenses_amount` :
    /// le contrat frontend type ce champ `number`, et le changer ici créerait
    /// une dérive silencieuse.
    #[serde(with = "rust_decimal::serde::float")]
    pub montant: Decimal,
}

#[derive(Debug, Clone, Serialize)]
pub struct UrgentTask {
    pub task_type: String,
    pub title: String,
    pub description: String,
    pub priority: String,
    pub building_name: Option<String>,
    pub entity_id: Option<String>,
    pub due_date: Option<DateTime<Utc>>,

    // ── Le décompte d'échéance légale ─────────────────────────────────────
    //
    // La remise de design en fait le différenciateur du produit : à côté de
    // chaque tâche, « 18 j sur 30 » et la référence de l'article. Aucun
    // concurrent ne dit sur quel fondement il réclame une action.
    //
    // Ces deux champs viennent du REGISTRE LÉGAL, jamais d'une chaîne écrite
    // dans un composant. La remise met en garde sur ce point précis, et sa
    // raison est bonne : un « 30 » recopié à l'écran est un nombre que rien
    // ne relie à la loi. Le jour où l'on corrige le domaine — parce qu'on
    // avait mal lu l'article — l'écran continue d'annoncer l'ancien délai, et
    // le syndic agit sur une échéance fausse en croyant lire le produit.
    /// L'article qui fonde l'échéance, tel qu'on le cite dans un courrier.
    /// `None` quand la tâche n'est pas d'origine légale.
    pub article: Option<String>,
    /// Le délai que cet article accorde, en jours. C'est le DÉNOMINATEUR du
    /// décompte : « 18 j sur 30 » n'a de sens que si l'on sait d'où vient le
    /// 30.
    pub delai_legal_jours: Option<i64>,
}
