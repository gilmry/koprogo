//! Classification des erreurs applicatives encore portées par une `String`.
//!
//! ── Le défaut que ce module borne ──────────────────────────────────────────
//!
//! Dix-huit gestionnaires décident du code HTTP en cherchant des sous-chaînes
//! dans un message d'erreur. Cent vingt-six occurrences au 2026-09-07.
//!
//! Le relevé des motifs cherchés dit tout :
//!
//! ```text
//! 52 × .contains("not found")
//!  6 × .contains("introuvable")
//! ```
//!
//! **Le même concept, en deux langues.** Les messages du domaine juridique
//! sont en français ; ceux des couches techniques en anglais. Un gestionnaire
//! qui ne cherche que « not found » renvoie donc **500** sur un « introuvable »
//! — une panne serveur là où l'utilisateur a simplement demandé une ressource
//! qui n'existe pas.
//!
//! C'est exactement ce qui s'est produit le 2026-09-04 : « Impossible de
//! déterminer l'ACP : une écriture manuelle doit désigner un immeuble » ne
//! correspondait à aucun motif, et une saisie incomplète ressortait en 500.
//! La rustine a consisté à ajouter les deux motifs manquants — au site
//! concerné seulement.
//!
//! ── Ce que ce module fait, et ce qu'il ne fait pas ────────────────────────
//!
//! Il **ne remplace pas** la vraie correction, qui est de typer les erreurs
//! (#762, et la migration `Result<_, String>` → `Result<_, AppError>` de
//! #555). Un jour où le domaine rendra `AppError::NotFound`, ce module n'aura
//! plus lieu d'être.
//!
//! En attendant, il rassemble en **un seul endroit** le lexique bilingue, pour
//! que la prochaine langue ou le prochain synonyme s'ajoute une fois et
//! profite à tous les gestionnaires — au lieu d'être découvert site par site,
//! à chaque 500 injustifié.
//!
//! Un lexique dispersé sur cent vingt-six sites ne se corrige jamais
//! entièrement : on corrige celui qui a fait mal.

/// L'erreur dit-elle qu'une ressource n'existe pas ?
///
/// Comparaison insensible à la casse, sur les deux langues du produit. Le
/// domaine juridique écrit en français, les couches techniques en anglais, et
/// les deux remontent par le même canal.
pub fn est_introuvable(message: &str) -> bool {
    let m = message.to_lowercase();
    m.contains("not found") || m.contains("introuvable") || m.contains("inexistant")
}

/// L'erreur dit-elle que l'appelant n'a pas le droit ?
///
/// À distinguer d'une authentification manquante : ici l'identité est connue,
/// c'est la permission qui manque.
pub fn est_interdit(message: &str) -> bool {
    let m = message.to_lowercase();
    m.contains("unauthorized")
        || m.contains("forbidden")
        || m.contains("not allowed")
        || m.contains("refusé")
        || m.contains("réservée aux")
        || m.contains("only the")
}

/// L'erreur est-elle PRÉCISÉMENT le refus opposé à qui n'a pas de fiche de
/// copropriétaire (skill/shared_object/resource_booking `resolve_owner()`) ?
///
/// Issue #781 — ce refus est déjà reconnu par `est_interdit` (403), mais par
/// mot-clé générique. Un `kind` stable, distinct des autres 403, permet au
/// frontend de router vers un message traduit dans les quatre locales sans
/// dépendre du libellé français — cf. `REFUS_RESERVE_AUX_COPROPRIETAIRES`,
/// dont le commentaire documente pourquoi une comparaison de libellé est
/// fragile pour les TESTS ; ici c'est le même risque, côté frontend, qu'un
/// `kind` stable évite.
pub fn est_refus_owner_requis(message: &str) -> bool {
    message == crate::application::error::REFUS_RESERVE_AUX_COPROPRIETAIRES
}

/// L'erreur est-elle PRÉCISÉMENT le refus « motif obligatoire » d'une
/// réservation `on_behalf_of_acp` (story #588, INV-5/FR27) ?
///
/// Doit router vers 422 (règle métier sur une requête par ailleurs valide),
/// pas 400/403 — d'où un `kind` stable distinct, même raisonnement que
/// `est_refus_owner_requis`.
pub fn est_motif_acp_manquant(message: &str) -> bool {
    message == crate::domain::entities::ReservationOnBehalfError::MotifRequired.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le cas qui a produit un 500 le 2026-09-04.
    #[test]
    fn un_message_francais_est_reconnu_comme_introuvable() {
        assert!(est_introuvable("Mandataire introuvable : 7bcd5e2f"));
        assert!(est_introuvable("Immeuble introuvable"));
    }

    #[test]
    fn un_message_anglais_lest_aussi() {
        assert!(est_introuvable("Poll not found"));
        assert!(est_introuvable("Building Not Found"));
    }

    /// Sans ce cas, `est_introuvable` pourrait rendre `true` partout et les
    /// deux tests ci-dessus passeraient sans rien prouver.
    #[test]
    fn une_erreur_technique_nest_pas_une_absence() {
        assert!(!est_introuvable("Database error: connection refused"));
        assert!(!est_introuvable(
            "Impossible de déterminer l'ACP créancière : la quote-part doit porter un lot"
        ));
    }

    #[test]
    fn le_refus_de_droit_est_reconnu_dans_les_deux_langues() {
        assert!(est_interdit("Unauthorized"));
        assert!(est_interdit("Only the poll creator can update it"));
        assert!(est_interdit(
            "Cette action est réservée aux copropriétaires : elle engage une personne"
        ));
        assert!(!est_interdit("Poll not found"));
    }

    // ------------------------------------------------------------------------
    // Issue #781 — est_refus_owner_requis (kind stable pour le frontend)
    // ------------------------------------------------------------------------

    #[test]
    fn happy_le_refus_owner_requis_est_reconnu() {
        assert!(est_refus_owner_requis(
            crate::application::error::REFUS_RESERVE_AUX_COPROPRIETAIRES
        ));
    }

    #[test]
    fn edge_un_prefixe_ou_suffixe_ne_suffit_pas() {
        // La comparaison est stricte : un message qui ne fait que CONTENIR le
        // refus (ex. concaténé à un contexte) n'est pas CE refus précis — le
        // kind ne doit s'attacher qu'à une correspondance exacte.
        assert!(!est_refus_owner_requis(&format!(
            "{} (contexte additionnel)",
            crate::application::error::REFUS_RESERVE_AUX_COPROPRIETAIRES
        )));
    }

    #[test]
    fn negative_un_autre_refus_de_droit_ne_declenche_pas_ce_kind() {
        // `est_interdit` reconnaît aussi ce message (403 générique) — mais il
        // ne s'agit PAS du refus "owner requis" : les deux fonctions doivent
        // pouvoir diverger.
        let autre = "Unauthorized: only owner can update skill";
        assert!(est_interdit(autre));
        assert!(!est_refus_owner_requis(autre));
    }

    #[test]
    fn security_un_message_vide_nest_jamais_pris_pour_ce_refus() {
        assert!(!est_refus_owner_requis(""));
    }

    // ------------------------------------------------------------------------
    // Story 5.4 — est_motif_acp_manquant (#588, INV-5/FR27)
    // ------------------------------------------------------------------------

    #[test]
    fn happy_le_motif_manquant_est_reconnu() {
        assert!(est_motif_acp_manquant(
            &crate::domain::entities::ReservationOnBehalfError::MotifRequired.to_string()
        ));
    }

    #[test]
    fn negative_un_autre_refus_ne_declenche_pas_ce_kind() {
        assert!(!est_motif_acp_manquant(
            crate::application::error::REFUS_RESERVE_AUX_COPROPRIETAIRES
        ));
        assert!(!est_motif_acp_manquant("Booking not found"));
    }
}
