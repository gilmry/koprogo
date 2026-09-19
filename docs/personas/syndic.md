---
persona: "Syndic"
role_code: "syndic"
statut: brouillon
population_recette: 5
eprouve: true
date: "2026-09-13"
version: "0.1"
superviseur: null
signature_humaine: null
issue_cadre: 805
issue_parcours: null
videos: []
---

# Syndic

Mandataire de l'ACP (Art. 3.89). Retranscrit l'acte de base, convoque,
préside, exécute. C'est le rôle le plus couvert du produit : ses
5 comptes de recette en font le rôle le mieux éprouvé des six.

## Parcours nominal

1. Se connecter → `syndic-dashboard`.
2. Retranscrire l'acte de base de l'ACP dont il est mandataire (quotités,
   coordonnées) → `acp-edit-form`, `acp-edit-name`. Référence légale : voir
   ci-dessous.
3. Convoquer l'assemblée générale → `convocation-btn-create`,
   `convocation-field-legal-deadline`, `convocation-btn-send`. Référence
   légale : voir ci-dessous.
4. Préparer les résolutions à l'ordre du jour → `resolution-create-btn`,
   `resolution-majority-select`. Référence légale : voir ci-dessous.
5. Tenir l'assemblée et faire adopter les résolutions →
   `resolution-adopted-count`, `resolution-vote-submit-button` (le vote
   lui-même est un acte du copropriétaire, cf. `owner.md`).

La clôture de l'assemblée et la publication du procès-verbal ne sont pas
incluses ci-dessus : aucune ancre `data-testid` dédiée n'a été trouvée au
2026-09-13 dans le contrat gelé pour cette étape précise. Ce n'est pas une
étape rouge (aucune issue connue la nomme) — c'est une lacune du présent
inventaire, à compléter par la story de parcours qui détaillera ce document
(#806-#809, #815 ou #816, association à confirmer par le PO).

## Ce que ce rôle ne peut pas faire, et pourquoi

- Ne peut pas **créer une nouvelle ACP** — c'est un geste de provisionnement
  plateforme réservé au superadmin (cf. `superadmin.md`), distinct de la
  retranscription de l'acte de base qui, elle, relève du mandat de syndic.
- Ne peut pas être **mandataire en assemblée générale, ni membre du conseil
  de copropriété** pendant son mandat — conflit d'intérêt écarté par la loi
  (Art. 3.89 § 9).

  **Conséquence, tranchée par le PO le 2026-09-18 et désormais testée** : ne
  pouvant recevoir aucun mandat, le syndic ne dépose **aucune voix**. Ni la
  sienne — il n'a pas de lot —, ni celle d'un autre, ce serait un mandat.
  `POST /resolutions/{id}/vote` rend `403 owner_not_linked` à un compte de
  syndic, et le harnais
  `e2e_resolutions::security_le_syndic_ne_peut_deposer_aucune_voix` le
  vérifie sur les deux chemins : le vote direct et la déclaration de
  procuration.

  Si l'usage exige un jour que le syndic **saisisse** les voix en séance, ce
  n'est pas ce refus qu'il faut lever : c'est une route dédiée, journalisée
  comme saisie pour compte de tiers, qui reste à concevoir.
- Ne peut pas exercer son mandat **au-delà de trois ans** sans
  renouvellement explicite par l'assemblée (Art. 3.89 § 1er).

## Références légales

Articles cités par code, lus depuis le registre exécutable — jamais
recopiés ici (délais, quotités : voir le registre, qui reste la seule
source qui ne se périme pas en silence).

- Art. 3.85 § 1er al. 2 — quotités de l'acte de base.
- Art. 3.87 § 3 — délai et forme de la convocation.
- Art. 3.88 — calcul des majorités en assemblée.
- Art. 3.89 § 1er — durée et renouvellement du mandat.
- Art. 3.89 § 5 — missions légales du syndic.
- Art. 3.89 § 9 — incompatibilités du mandat.

Voir `backend/src/domain/copropriete/registre_legal.rs` et
`docs/legal/syndic/` (`missions_legales.rst`, `mandat.rst`,
`deontologie_generale.rst`, `deontologie_specifique.rst`, `travaux.rst`).

## Ce qui ne marche pas encore

Aucune étape rouge tracée au 2026-09-13.

## Comptes de recette

- François Leroy, syndic professionnel, cf.
  `docs/specs/00-personas-et-seed.rst` (« Personas : Professionnels »).
