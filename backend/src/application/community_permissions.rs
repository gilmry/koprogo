//! Vérification de permission partagée entre les use-cases Community (SEL,
//! Poll, Notice, SharedObject) — Story 5.3 (#587), INV-4.
//!
//! Le droit de PARTICIPER (créer une offre, voter, emprunter) vient d'avoir
//! une fiche de copropriétaire, pas du rôle actif : un syndic qui a aussi un
//! lot participe ès qualités de copropriétaire (même raisonnement que
//! l'ADR-0052 pour le comptable). Ce module ne couvre PAS cette moitié-là —
//! elle se vérifie via `OwnerRepository` dans chaque use-case, comme avant.
//!
//! Le droit de MODÉRER (éditer/annuler/supprimer le contenu d'autrui, motif à
//! l'appui) vient en revanche du rôle actif. C'est ce que ce module expose,
//! en un seul endroit, pour que les quatre use-cases Community ne dupliquent
//! pas le parsing `&str` → `UserRole`.

use crate::domain::entities::UserRole;
use std::str::FromStr;

/// Le rôle actif (porté par le JWT, donc une `&str` côté handler) autorise-t-il
/// la modération communautaire (SEL/Poll/Notice/SharedObject) ?
///
/// Un rôle illisible (jamais censé arriver depuis un JWT signé par ce
/// backend, mais un appelant direct — test, BDD — peut passer n'importe
/// quoi) est traité comme "ne modère pas", jamais comme une erreur : côté
/// permissions, l'inconnu doit refuser, pas planter.
pub fn peut_moderer(actor_role: &str) -> bool {
    UserRole::from_str(actor_role)
        .map(|role| role.can_moderate_community())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_syndic_peut_moderer() {
        assert!(peut_moderer("syndic"));
    }

    #[test]
    fn happy_community_moderator_peut_moderer() {
        assert!(peut_moderer("community.moderator"));
    }

    #[test]
    fn edge_superadmin_peut_moderer() {
        assert!(peut_moderer("superadmin"));
    }

    #[test]
    fn security_owner_ne_peut_pas_moderer() {
        assert!(!peut_moderer("owner"));
    }

    #[test]
    fn negative_un_role_illisible_ne_declenche_pas_de_panique() {
        assert!(!peut_moderer("<script>alert(1)</script>"));
        assert!(!peut_moderer(""));
    }
}
