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
}
