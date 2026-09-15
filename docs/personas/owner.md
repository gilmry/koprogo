---
persona: "Copropriétaire (owner)"
role_code: "owner"
statut: brouillon
population_recette: 5
eprouve: true
date: 2026-09-13
version: "0.1"
superviseur: null
signature_humaine: null
issue_cadre: 805
issue_parcours: null
videos: []
---

# Copropriétaire (owner)

Vote, paie, consulte ses quotes-parts, participe à la communauté. C'est le
rôle le plus peuplé en base (5 comptes) — et pourtant celui qu'aucune des
cinq recettes conduites jusqu'ici n'a systématiquement éprouvé de bout en
bout (#805).

## Parcours nominal

1. Se connecter → `owner-dashboard`.
2. Consulter ses lots et ses quotes-parts → `owner-buildings-see-all`.
3. Configurer son moyen de paiement des contributions →
   `owner-contribution-payment-method-select`,
   `owner-contribution-payment-submit-button`.
4. Voter une résolution en assemblée générale →
   `resolution-vote-submit-button`. Référence légale : voir ci-dessous.
5. Signaler un ticket de maintenance → `owner-tickets-create-button`.
6. Offrir ou demander un échange dans le SEL → `create-exchange-form`.

## Ce que ce rôle ne peut pas faire, et pourquoi

- Ne peut pas **saisir une écriture comptable** — la tenue des livres est
  réservée au comptable (cf. `accountant.md`), séparation des tâches
  élémentaire entre qui décide de la dépense et qui la comptabilise.
- Ne peut pas **convoquer ni présider une assemblée** — c'est une mission du
  syndic (Art. 3.89 § 5), pas du copropriétaire.
- Ne peut pas **voter au-delà du plafond de procuration** (3 mandats,
  exception à 10 % des quotités) — limite anti-concentration du pouvoir de
  vote (Art. 3.87 § 6).

## Références légales

- Art. 3.87 § 6 — plafonnement des procurations.
- Art. 3.87 § 5 — droit à l'information avant l'assemblée.
- Art. 3.88 — majorités applicables au vote.
- Art. 3.90 — conseil de copropriété (quand le copropriétaire y siège).

Voir `backend/src/domain/copropriete/registre_legal.rs` et
`docs/legal/coproprietaire/`.

## Ce qui ne marche pas encore

Aucune étape rouge tracée au 2026-09-13. C'est précisément l'absence
d'éprouvage systématique de ce rôle en recette (cf. l'introduction
ci-dessus) qui rend cette liste la moins fiable des quatre rôles peuplés :
une case vide ici ne prouve pas l'absence de défaut, seulement l'absence
d'observation.

## Comptes de recette

Les dix personas copropriétaires de
`docs/specs/00-personas-et-seed.rst` (« Résidence du Parc Royal ») :
Alice Dubois, Bob Janssen, Charlie Martin, Diane Peeters, Emmanuel Claes,
Nadia Benali, Marguerite Lemaire, Jeanne Devos, Philippe Vandermeulen,
Marcel Dupont — chacun désigné par son nom, jamais par une adresse
recopiée ici.
