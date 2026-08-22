# ADR 0012: Fonds de réserve (≥ 5%) & fonds de roulement obligatoires

- **Status**: Accepted (validé @gilmry 2026-07-25)
- **Date**: 2026-06-15
- **Track**: Software / Legal-compliance / Finance
- **Authors**: Claude Opus 4.8 (drafting) + @gilmry sign-off
- **Related**: [ADR 0010](0010-acte-de-base-conformite-copropriete.md), [ADR 0007](0007-decimal-vs-f64-for-money.md) ; issue [#618](https://github.com/gilmry/koprogo/issues/618) ; story CL4 (H13)

## Context

La revue domain 2026-06-15 a constaté l'**absence** de modélisation des fonds obligatoires d'une copropriété belge.

- **Loi du 18/06/2018** (en vigueur **01/01/2019**), Art. 3.86 §3 CC (ex-577-5 §3) — toute association des copropriétaires doit constituer **un fonds de roulement** ET **un fonds de réserve**.
  - **Fonds de réserve** : contribution annuelle **≥ 5% du total des charges communes ordinaires de l'exercice précédent** (pas de maximum légal). Obligatoire pour les immeubles dont les parties communes ont été réceptionnées ≥ 5 ans avant l'entrée en vigueur.
  - **Comptes distincts** : réserve et roulement sur des **comptes bancaires séparés ouverts au nom de l'ACP** ; le fonds de réserve ne sert pas de fonds de roulement.
  - **Renonciation** : l'AG peut décider de **ne pas** constituer de fonds de réserve par vote à la **majorité des 4/5** (le fonds de roulement, lui, ne peut être écarté).

Source : [VJN — fonds de réserve et roulement obligatoires](https://vjn-legal.be/copropriete-fonds-de-reserve-et-fonds-de-roulement-desormais-obligatoires/).

**Problème** : `budget.rs` ne modélise ni fonds de réserve ni fonds de roulement, ni le seuil 5%, ni la séparation des comptes. De plus `budget.rs` utilise `f64` (violation [ADR 0007](0007-decimal-vs-f64-for-money.md)).

## Decision

1. **Fonds sur l'ACP** : colonnes `acps.reserve_fund_balance DECIMAL(14,2)`, `acps.working_capital_balance DECIMAL(14,2)`, `acps.reserve_fund_waived BOOLEAN` (renonciation 4/5).
2. **Invariant réserve** : `Acp::assert_reserve_fund_compliant(ordinary_charges_n1: Decimal)` → `Ok` si `reserve_fund_waived` OU `reserve_fund_balance ≥ 0.05 × ordinary_charges_n1` ; sinon `ReserveFundInsufficient { acp_id, required, current }` (typé).
3. **Appels de fonds typés** : `call_for_funds.fund_type ∈ {ordinary, working_capital, reserve}` (comptes/affectations distincts, Art. 3.86 §3).
4. **Budget en Decimal** : `budget.rs` migre `f64 → rust_decimal::Decimal` sur `ordinary_budget`, `extraordinary_budget`, `total_budget`, `monthly_provision_amount` (la DB est déjà `DECIMAL(12,2)`). `monthly_provision = total_budget / dec!(12)`.

## Consequences

**Positives**
- Conformité loi 2019 (réserve 5% + roulement, comptes distincts).
- Provisions mensuelles exactes (plus de dérive IEEE 754 type `6249.999`).
- Renonciation 4/5 modélisée et auditée.

**Négatives / coûts**
- Migration `acps` (3 colonnes funds).
- MVP : pas de moteur comptable complet (régularisations annuelles, ventilation analytique) — différé v0.2.0 ; on fournit le modèle + la validation + les appels typés.
- Le calcul des « charges ordinaires N-1 » suppose un historique d'exercice ; en l'absence (copropriété récente), le check est neutre la 1re année.

## Alternatives rejetées

- **Fonds sur le building** : la loi vise l'ACP (personne morale) ; rejeté (cohérent avec [ADR 0010](0010-acte-de-base-conformite-copropriete.md)).
- **Garder `f64` dans budget** : viole ADR-0007 (exactitude PCMN) ; rejeté.
- **Seuil 5% en constante non renonçable** : ignore la possibilité de renonciation 4/5 ; rejeté (champ `reserve_fund_waived`).
