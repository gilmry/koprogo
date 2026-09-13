---
persona: "Comptable (accountant)"
role_code: "accountant"
statut: brouillon
population_recette: 2
eprouve: true
date: 2026-09-13
version: "0.1"
superviseur: null
signature_humaine: null
issue_cadre: 805
issue_parcours: null
videos: []
---

# Comptable (accountant)

Tient les livres. Écritures, PCMN, rapports. Ne préside pas.

Le rôle se raffine en deux sous-rôles côté code
(`accountant.encodeur` pour la facture et le devis, `accountant.emetteur`
pour les charges et les appels de fonds — cf.
`frontend/src/lib/auth/permissions.ts`), pas encore documentés
séparément : ce document couvre le rôle `accountant` de base.

## Parcours nominal

1. Se connecter → `accountant-dashboard`.
2. Consulter les tuiles de son périmètre (immeubles, dépenses, factures,
   rapports) → `accountant-buildings-tile`, `accountant-expenses-tile`,
   `accountant-invoices-tile`, `accountant-reports-tile`.
3. Générer un rapport financier (bilan, compte de résultat) →
   `building-report-generate-button`,
   `building-report-balance-tab-button`,
   `building-report-income-tab-button`.
4. Exporter le rapport pour l'assemblée → `building-report-balance-export-pdf-button`.

## Ce que ce rôle ne peut pas faire, et pourquoi

- Ne peut pas **présider une assemblée générale** — la présidence est un
  acte du syndic ou d'un copropriétaire mandaté (cf. `syndic.md`), pas du
  comptable, dont la mission se limite à la tenue des comptes.
- Ne peut pas **convoquer** ni **décider** des travaux — il rapporte des
  chiffres, il ne prend pas les décisions que ces chiffres éclairent.

## Références légales

Le mandat du comptable externe n'est pas un mandat statutaire du Code
civil (à la différence du syndic ou du commissaire aux comptes, qui est
lui un copropriétaire désigné, Art. 3.91) : sa base est le plan comptable
normalisé, pas le chapitre copropriété.

- `docs/legal/pcmn_ar_12_07_2012.rst` — plan comptable minimum normalisé
  applicable aux ACP.
- Art. 3.91 — accès du commissaire aux comptes aux pièces comptables (rôle
  distinct, tenu par un copropriétaire — cf. `owner.md` — pas par ce
  comptable externe).

## Ce qui ne marche pas encore

Aucune étape rouge tracée au 2026-09-13.

## Comptes de recette

- Gisele Vandenberghe, comptable externe, cf.
  `docs/specs/00-personas-et-seed.rst` (« Personas : Professionnels »).
