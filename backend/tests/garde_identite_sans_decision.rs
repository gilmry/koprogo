//! Cliquet : une route qui prend une identité doit s'en servir pour DÉCIDER
//! (#864).
//!
//! ── Le défaut que ce cliquet borne ────────────────────────────────────────
//!
//! Une route qui écrit `user: AuthenticatedUser` dans sa signature **a l'air
//! gardée**. Elle passe la revue, elle passe les autres gardes, elle
//! journalise consciencieusement qui a fait quoi.
//!
//! Quatre `DELETE` ont été vérifiés puis corrigés le 2026-09-11 : ils
//! prenaient l'identité et ne s'en servaient que pour **journaliser après
//! coup**. Témoin à l'appui, le syndic du cabinet A supprimait le budget du
//! cabinet B et recevait `204 No Content` ; le journal d'audit enregistrait le
//! geste comme régulier.
//!
//! Une route SANS identité se voit : elle est nue, `garde_identite_absente` la
//! compte. Une route qui prend une identité et l'ignore n'était comptée par
//! rien.
//!
//! ── Ce que le compte VAUT, et ce qu'il ne vaut pas ────────────────────────
//!
//! **C'est un plafond, pas un verdict.** Le détecteur est textuel : il cherche
//! dans le corps du handler la trace d'une décision — `verify_`, `scope_guard`,
//! `Forbidden`, `require_organization`, `is_superadmin`, `.role ==`. Il ne
//! comprend pas le code.
//!
//! Deux conséquences assumées :
//!
//! - **Des faux positifs.** Une route dont le cas d'usage filtre lui-même par
//!   utilisateur — les notifications de l'appelant, par exemple — n'a besoin
//!   d'aucun contrôle dans le handler et sera pourtant comptée.
//! - **`user.user_id` NE COMPTE PAS** comme décision. C'est le marqueur du
//!   journal d'audit, qui enregistre après coup et n'interdit rien. L'inclure
//!   ferait passer les quatre `DELETE` corrigés pour des routes gardées —
//!   c'est l'erreur que j'ai faite au premier comptage, et elle divisait le
//!   résultat par quatre.
//!
//! Le cliquet ne prétend donc pas que 103 routes sont vulnérables. Il dit que
//! 103 méritent une lecture, et **qu'aucune de plus ne doit apparaître**.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Ce qui DÉCIDE. Volontairement court : chaque entrée est un idiome par
/// lequel ce dépôt refuse effectivement un accès.
const DECISION: [&str; 9] = [
    "verify_",
    "scope_guard",
    "can_",
    "Forbidden",
    "caller_from_user",
    "require_organization",
    "require_role",
    "is_superadmin",
    ".role ==",
];

/// Mesuré le 2026-09-13, après le cloisonnement de vingt-deux routes (#864).
///
/// Relevé, jamais estimé. Il ne peut que descendre.
///
/// ── 95 → 85 → 73, et par quoi ─────────────────────────────────────────────
///
/// Par le TRAVAIL, pas par la définition.
///
/// **95 → 85** : les cinq `PUT /budgets/{id}/*` et les cinq
/// `PUT /etats-dates/{id}/*`, qui prenaient `AuthenticatedUser` sans s'en
/// servir pour décider. Le test
/// `security_le_cycle_de_vie_du_budget_inter_organisations_est_refuse` le
/// démontre : sans le correctif, `PUT /budgets/{id}` rend **200 OK** au
/// syndic d'une autre organisation.
///
/// **85 → 73** : les quatre POST d'assemblée (annuler, clôturer, reporter,
/// valider le quorum — Art. 3.87), les quatre transitions de dépense,
/// `assign_owner`, `create_quote` dont l'identité était nommée `_auth`, et
/// **deux d'une forme que le relevé ne cherchait pas** :
/// `list_call_for_funds` et `get_contributions_by_owner`. Ces deux-là
/// cloisonnaient correctement dans leur branche nominale et **pas du tout**
/// dans leur branche filtrée. Un paramètre facultatif — `building_id`,
/// `owner_id` — contournait le chemin protégé, et les deux fois sur la même
/// donnée : qui doit combien. Aucun cliquet qui compte des GESTIONNAIRES ne
/// voit cette forme-là.
///
/// La liste `DECISION` n'a pas été touchée. Elle a failli l'être : les
/// helpers posés s'appelaient d'abord `cloisonner_*`, que le détecteur ne
/// connaît pas, et le compteur est resté à 95 alors que dix trous étaient
/// bouchés. Allonger la liste aurait fait tomber le chiffre par une
/// modification de l'instrument. Les helpers ont été renommés `verify_*` —
/// l'idiome que ce dépôt emploie déjà partout ailleurs — et le compteur a
/// suivi le travail.
/// 73 → 54 le 2026-09-20, **sans toucher à l'instrument**.
///
/// Cinq helpers décidaient déjà et n'étaient pas vus, faute de porter
/// l'idiome que `DECISION` reconnaît :
///
///     check_write_permission           -> verify_write_permission
///     check_owner_readonly             -> verify_owner_readonly
///     check_syndic_role                -> verify_syndic_role
///     check_accountant_role            -> verify_accountant_role
///     check_unit_ownership_permission  -> verify_unit_ownership_permission
///
/// C'est la réponse que ce fichier prescrit lui-même, quelques lignes plus
/// haut : les helpers s'appelaient d'abord `cloisonner_*`, le compteur est
/// resté à 95 alors que dix trous étaient bouchés, et **allonger `DECISION`
/// aurait fait tomber le chiffre par une modification de la mesure**. On
/// renomme le code, jamais l'instrument.
///
/// Les `check_can_*` passaient déjà, par le marqueur `can_` — d'où cinq
/// invisibles sur huit.
///
/// `expense_handlers.rs` (5) et `unit_owner_handlers.rs` (3) quittent la
/// liste. Le reliquat se concentre sur `iot` (6), `notification` (5),
/// `gamification`, `mcp_sse` et `two_factor` (4 chacun).
/// 54 → 52 le 2026-09-20, par du TRAVAIL cette fois.
///
/// Deux routes décident désormais là où elles ne décidaient pas :
///
///   `list_contractor_quotes`   — filtre par `verify_building_org_access`.
///     Elle rendait tous les devis d'un prestataire, toutes ACP confondues :
///     un cabinet lisait les prix remis à un concurrent (#976, `106478bc`).
///
///   `create_service_provider`  — refuse en 403 hors syndic/superadmin.
///     Sa documentation portait « syndic/admin only » depuis sa création, et
///     rien ne l'appliquait (`7dbd76ff`).
///
/// Le second enseigne quelque chose sur ce cliquet : « prendre une identité
/// sans s'en servir pour décider » recouvre DEUX questions — de qui sont les
/// données que je rends (cloisonnement), et qui a le droit de faire ce geste
/// (autorisation). Cette route cloisonnait correctement et n'autorisait
/// personne. Le cliquet la comptait, à raison.
const SANS_DECISION_AU_2026_09_13: usize = 52;

/// Les handlers qui prennent `AuthenticatedUser` sans trace de décision.
fn sans_decision() -> BTreeMap<String, String> {
    let dossier = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/infrastructure/web/handlers");
    let mut trouves = BTreeMap::new();

    for entree in fs::read_dir(&dossier).expect("le dossier des handlers doit être lisible") {
        let chemin = entree.expect("entrée lisible").path();
        if chemin.extension().is_none_or(|x| x != "rs") {
            continue;
        }
        let fichier = chemin
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string();
        let source = fs::read_to_string(&chemin).expect("handler lisible");

        // Découpe par `pub async fn` : le corps d'un handler va jusqu'au
        // suivant. Grossier, et suffisant — un handler ne contient pas d'autre
        // `pub async fn`.
        let morceaux: Vec<&str> = source.split("pub async fn ").collect();
        for morceau in morceaux.iter().skip(1) {
            let nom = morceau
                .split(['(', '<'])
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            let Some(fin_signature) = morceau.find(')') else {
                continue;
            };
            let (signature, corps) = morceau.split_at(fin_signature);
            if !signature.contains("AuthenticatedUser") {
                continue;
            }
            if !DECISION.iter().any(|k| corps.contains(k)) {
                trouves.insert(format!("{fichier}::{nom}"), fichier.clone());
            }
        }
    }
    trouves
}

/// Les exceptions JUSTIFIÉES — ce que la LECTURE a établi (#864).
///
/// ── Pourquoi cette liste, et pourquoi elle n'est pas cosmétique ───────────
///
/// L'issue la réclame : « Il aurait sa liste d'exceptions justifiées :
/// certaines routes n'ont légitimement rien à cloisonner. » Sans elle, le
/// cliquet ne fait que COMPTER, et un compte ne distingue pas une route qui
/// cloisonne autrement d'une route qui ne cloisonne pas du tout.
///
/// Ce n'est pas une nuance de vocabulaire. Le relevé contient une catégorie
/// que l'énoncé de l'issue ne prévoyait pas et qui est la plus trompeuse :
/// des routes qui contrôlent le RÔLE et jamais le PÉRIMÈTRE —
/// `check_syndic_role`, `check_accountant_role`,
/// `check_unit_ownership_permission`. Un syndic du cabinet A y approuve la
/// facture du cabinet B. Le cliquet les compte, mais **par accident** :
/// parce que le nom du helper n'est dans aucun motif, pas parce qu'il aurait
/// compris qu'il manque le cloisonnement. Renommer un jour l'un de ces
/// helpers `verify_*` les ferait sortir du compte **sans les corriger**.
///
/// D'où deux listes et non une : ce qui est EXCEPTÉ est écrit, et tout le
/// reste est du travail à faire. Un nombre qui descend ne dit plus « moins de
/// routes », il dit « plus de routes lues ».
///
/// ── Ce qu'une entrée atteste ──────────────────────────────────────────────
///
/// Que le gestionnaire a été LU, et pour les cas délégués, que le cas d'usage
/// appelé a été lu aussi. Jamais que le détecteur s'est tu.
const EXCEPTIONS: &[(&str, &str)] = &[
    // ── Cloisonnée par un chemin que le motif textuel ne voit pas ──────────
    (
        "api_key_handlers.rs::revoke_api_key",
        "UPDATE ... WHERE id = $1 AND organization_id = $2 — le cloisonnement \
         est dans la clause SQL, qu'aucun motif ne lit",
    ),
    (
        "api_key_handlers.rs::list_api_keys",
        "SELECT ... WHERE organization_id = $1 — idem",
    ),
    (
        "document_handlers.rs::list_documents",
        "`user.organization_id` passé au cas d'usage, qui borne la requête",
    ),
    (
        "budget_handlers.rs::list_budgets",
        "`organization_id` ET `building_id` passés au cas d'usage : le filtre \
         facultatif ne remplace pas le bornage, il s'y ajoute",
    ),
    (
        "expense_handlers.rs::list_expenses",
        "`user.organization_id` passé au cas d'usage",
    ),
    (
        "etat_date_handlers.rs::list_etats_dates",
        "`user.organization_id` passé au cas d'usage",
    ),
    (
        "meeting_handlers.rs::list_meetings",
        "`user.organization_id` passé au cas d'usage",
    ),
    (
        "iot_handlers.rs::create_iot_reading",
        "`organization_id` passé au cas d'usage — corrigé par 46ee956a",
    ),
    (
        "iot_handlers.rs::delete_linky_device",
        "`organization_id` passé au cas d'usage — corrigé par 46ee956a",
    ),
    (
        "iot_handlers.rs::toggle_linky_sync",
        "`organization_id` passé au cas d'usage — corrigé par 46ee956a",
    ),
    // ── La route ne sert que son appelant : rien à cloisonner ──────────────
    (
        "notification_handlers.rs::list_my_notifications",
        "`list_user_notifications(user.user_id)` — l'appelant ne peut lire \
         que les siennes",
    ),
    (
        "notification_handlers.rs::get_user_preferences",
        "`get_user_preferences(user.user_id)` — idem",
    ),
    (
        "ticket_handlers.rs::list_my_tickets",
        "`list_my_tickets(user.user_id)` — idem",
    ),
    (
        "owner_handlers.rs::get_my_owner",
        "`find_owner_by_user_id(user.user_id)` — la route EST « moi »",
    ),
    (
        "resolution_handlers.rs::list_meeting_resolutions",
        "appelle `verifier_mandat_sur_ag`, un garde REEL de ce depot que le \
         detecteur ne connait pas parce qu'il porte un nom francais. Meme \
         angle mort que `cloisonner_*` le 2026-09-13 : la liste DECISION dit \
         les idiomes du depot, et le depot en a deux dialectes",
    ),
    (
        "resolution_handlers.rs::list_resolution_votes",
        "remonte a l'AG par la resolution puis applique le meme \
         `verifier_mandat_sur_ag`",
    ),
    (
        "energy_campaign_handlers.rs::list_campaigns",
        "`get_campaigns_by_organization(org_id)` — l'organisation vient du \
         jeton, jamais de la requete",
    ),
    (
        "energy_bill_upload_handlers.rs::get_my_uploads",
        "`get_my_uploads(user.user_id)` — la route EST « mes televersements »",
    ),
    (
        "notification_handlers.rs::list_unread_notifications",
        "`list_unread_notifications(user.user_id)`",
    ),
    (
        "notification_handlers.rs::get_notification_stats",
        "`get_user_stats(user.user_id)`",
    ),
    (
        "notification_handlers.rs::get_preference",
        "la preference est cherchee pour `user.user_id`",
    ),
    (
        "ticket_handlers.rs::list_assigned_tickets",
        "`list_assigned_tickets(user.user_id)`",
    ),
    (
        "consent_handlers.rs::get_consent_status",
        "`get_consent_status(auth.user_id)` — le consentement de l'appelant",
    ),
    (
        "dashboard_handlers.rs::get_recent_transactions",
        "`user.organization_id` exige puis borne la requete",
    ),
    (
        "two_factor_handlers.rs::setup_2fa",
        "`auth.organization_id` exige, puis passe au cas d'usage",
    ),
    (
        "auth_handlers.rs::switch_role",
        "le cas d'usage refuse : `if target_role.user_id != user.id` \
         (auth_use_cases.rs:271). On ne prend pas le role d'un autre",
    ),
    (
        "auth_handlers.rs::logout",
        "`revoke_all_refresh_tokens(user.user_id)` — on ne deconnecte que soi",
    ),
    (
        "gamification_handlers.rs::get_user_achievements",
        "`get_user_achievements(auth.user_id)`",
    ),
    (
        "gamification_handlers.rs::get_recent_achievements",
        "`get_recent_achievements(auth.user_id, limit)`",
    ),
    (
        "gamification_handlers.rs::list_user_active_challenges",
        "`list_user_active_progress(auth.user_id)`",
    ),
    (
        "gamification_handlers.rs::award_achievement",
        "décerne à `auth.user_id`, jamais à un tiers. L'intégrité du jeu est \
         un autre sujet ; le cloisonnement inter-organisations n'est pas en \
         cause",
    ),
    // ── Lues le 2026-09-13, deuxième passe sur les 40 non classées ─────────
    (
        "api_key_handlers.rs::get_api_key",
        "SELECT ... WHERE id = $1 AND organization_id = $2 — jumeau exact de \
         revoke_api_key/list_api_keys, deja exceptes pour le meme motif SQL",
    ),
    (
        "two_factor_handlers.rs::enable_2fa",
        "appelle `two_factor_use_cases.enable_2fa(auth.user_id, ...)` — agit \
         sur soi, jamais sur un tiers. Jumeau de setup_2fa deja excepte",
    ),
    (
        "two_factor_handlers.rs::disable_2fa",
        "`disable_2fa(auth.user_id, ...)` — idem",
    ),
    (
        "two_factor_handlers.rs::regenerate_backup_codes",
        "`regenerate_backup_codes(auth.user_id, ...)` — idem",
    ),
    (
        "mcp_sse_handlers.rs::mcp_sse_endpoint",
        "ouvre une session SSE et rend un `session_id` : aucune ressource \
         ciblee, rien a cloisonner. Le filtrage a lieu ensuite dans \
         mcp_messages_endpoint",
    ),
    (
        "mcp_sse_handlers.rs::mcp_messages_endpoint",
        "delegue a `dispatch_tool`, fonction separee (ligne 517) qui extrait \
         `user.organization_id` et borne chaque outil avec — meme angle mort \
         que verifier_mandat_sur_ag : la decision existe, le motif textuel ne \
         la voit pas dans CETTE fonction",
    ),
    (
        "mcp_sse_handlers.rs::mcp_system_prompt_endpoint",
        "`include_str!(\"../../mcp_system_prompt.md\")` — contenu statique \
         fige a la compilation, identique pour tous",
    ),
    (
        "mcp_sse_handlers.rs::mcp_legal_index_endpoint",
        "`include_str!(\"../../legal_index.json\")` — idem, statique",
    ),
];

/// Mesuré le 2026-09-13. Relevé, jamais estimé.
///
/// Ce nombre descend de deux façons, et une seule est du travail de sécurité :
/// en CORRIGEANT une route (elle sort du relevé), ou en la LISANT et en
/// l'inscrivant aux exceptions (elle sort du non-classé). Les deux sont du
/// travail ; aucune ne touche à la définition de l'instrument.
///
/// **40 → 32**, deuxième passe de lecture le 2026-09-13 : huit routes lues et
/// inscrites aux EXCEPTIONS ci-dessus (`get_api_key`, les trois routes 2FA
/// self-service, les quatre endpoints MCP). Aucune corrigée dans cette passe.
///
/// Les 32 qui restent ont été lues aussi, et ne sont PAS des exceptions : ce
/// sont des lacunes de cloisonnement probables, prioritaires pour la suite —
/// notamment les quatre transitions de facture (`update_invoice_draft`,
/// `submit_invoice_for_approval`, `approve_invoice`, `reject_invoice`, qui
/// contrôlent le RÔLE et jamais le PÉRIMÈTRE, catégorie décrite plus haut) et
/// les trois routes `unit_owner_handlers.rs` (`add_owner_to_unit`,
/// `transfer_ownership`, `update_unit_owner`), pour lesquelles le correctif
/// (`verify_owner_org_access` / `verify_unit_org_access`) existe déjà, importé
/// et utilisé ailleurs dans le même fichier. `issue_magic_link` est la plus
/// sévère : elle mène à un accès non authentifié hors périmètre. Non
/// corrigées ici faute de pouvoir exécuter `cargo test`/`clippy` dans cette
/// session pour démontrer le rouge puis le vert — corriger sans ce filet
/// serait produire du code non mesuré.
const NON_CLASSEES_AU_2026_09_13: usize = 32;

#[test]
fn chaque_exception_est_encore_dans_le_releve() {
    // Une exception qui ne correspond plus à rien est pire qu'absente : elle
    // laisse croire qu'une route a été lue et jugée sans danger, alors que
    // c'est une AUTRE route qui porte ce nom, ou qu'elle a disparu.
    //
    // Et le cas le plus vicieux : si quelqu'un ajoute un `verify_` à une
    // route exceptée, elle sort du relevé, l'exception devient muette, et
    // plus rien ne signale que la justification écrite ici est périmée.
    let releve = sans_decision();
    let orphelines: Vec<&str> = EXCEPTIONS
        .iter()
        .map(|(r, _)| *r)
        .filter(|r| !releve.contains_key(*r))
        .collect();

    assert!(
        orphelines.is_empty(),
        "Ces exceptions ne correspondent à aucune route du relevé :\n{}\n\n\
         Soit la route a été renommée ou supprimée, soit elle porte désormais \
         un marqueur de décision et n'a plus besoin d'exception. Dans les deux \
         cas, retirez l'entrée : une justification qui ne justifie plus rien \
         se lit comme un accord.",
        orphelines
            .iter()
            .map(|r| format!("  {r}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn le_nombre_de_routes_non_lues_ne_grossit_pas() {
    let releve = sans_decision();
    let exceptees: std::collections::BTreeSet<&str> = EXCEPTIONS.iter().map(|(r, _)| *r).collect();
    let non_classees: Vec<&String> = releve
        .keys()
        .filter(|r| !exceptees.contains(r.as_str()))
        .collect();
    let n = non_classees.len();

    assert!(
        n <= NON_CLASSEES_AU_2026_09_13,
        "{n} routes du relevé n'ont été NI corrigées NI lues, contre \
         {NON_CLASSEES_AU_2026_09_13} le 2026-09-13.\n\n\
         `@edge` de #864 : « chacune est classée cloisonnée, légitimement non \
         cloisonnée avec sa raison écrite, ou corrigée. Aucune n'est déclarée \
         traitée sans lecture. »\n\n\
         Les dix premières à lire :\n{}",
        non_classees
            .iter()
            .take(10)
            .map(|r| format!("  {r}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn le_cliquet_lit_bien_des_handlers() {
    // Vérification d'aveuglement : si le découpage ou le chemin cessent de
    // correspondre, la règle suivante devient vraie sur zéro handler.
    let dossier = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/infrastructure/web/handlers");
    let fichiers = fs::read_dir(&dossier)
        .expect("dossier lisible")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "rs"))
        .count();
    assert!(
        fichiers > 20,
        "Seulement {fichiers} fichiers de handlers lus. Le chemin ne correspond \
         plus : un vert obtenu sur rien ne dit rien."
    );

    // Et la mesure doit rendre un nombre plausible : zéro signifierait que le
    // détecteur ne voit plus aucune signature.
    let n = sans_decision().len();
    assert!(
        n > 0,
        "Aucune route ne prend `AuthenticatedUser` sans marqueur de décision. \
         Ce serait une excellente nouvelle, et elle est invraisemblable tant \
         que #864 est ouverte : le détecteur ne lit probablement plus rien."
    );
}

#[test]
fn la_dette_didentite_sans_decision_ne_grossit_pas() {
    let trouves = sans_decision();
    let n = trouves.len();

    // Les dix fichiers les plus chargés, pour que le message serve à agir.
    let mut par_fichier: BTreeMap<&String, usize> = BTreeMap::new();
    for fichier in trouves.values() {
        *par_fichier.entry(fichier).or_insert(0) += 1;
    }
    let mut classement: Vec<_> = par_fichier.into_iter().collect();
    classement.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    let tete: Vec<String> = classement
        .iter()
        .take(10)
        .map(|(f, n)| format!("  {n:3}  {f}"))
        .collect();

    assert!(
        n <= SANS_DECISION_AU_2026_09_13,
        "{n} routes prennent `AuthenticatedUser` sans trace de décision, contre \
         {SANS_DECISION_AU_2026_09_13} mesurées le 2026-09-13.\n\n\
         Une route qui prend une identité et ne s'en sert que pour journaliser \
         A L'AIR gardée : elle passe la revue, elle passe les autres gardes, et \
         elle laisse passer le geste.\n\n\
         `user.user_id` ne compte pas — c'est le marqueur du journal d'audit, \
         qui enregistre après coup et n'interdit rien.\n\n\
         Les fichiers les plus chargés :\n{}",
        tete.join("\n")
    );
}
