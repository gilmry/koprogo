# ADR 0011: Quorum double (têtes + quotités) & suspension du droit de vote des lots démembrés/indivis

- **Status**: Proposed (acceptation @gilmry au merge)
- **Date**: 2026-06-15
- **Track**: Software / Legal-compliance / Governance
- **Authors**: Claude Opus 4.8 (drafting) + @gilmry sign-off
- **Related**: [ADR 0010](0010-acte-de-base-conformite-copropriete.md), [ADR 0007](0007-decimal-vs-f64-for-money.md) ; issue [#618](https://github.com/gilmry/koprogo/issues/618) ; stories CL3 (H9/H10/H17)

## Context

Revue domain 2026-06-15 sur la gouvernance d'assemblée générale (Code civil Livre 3) :

- **Art. 3.87 §5 CC — DOUBLE quorum** : l'AG délibère valablement si les présents/représentés réunissent **(a) plus de la moitié des copropriétaires (têtes)** ET **(b) au moins la moitié des quotités** des parties communes. À défaut, une 2e assemblée se tient après ≥ 15 jours et délibère sans quorum. *([source droitbelge](https://www.droitbelge.be/fiches_detail.asp?idcat=9&id=623))*
- **Art. 3.87 §1 CC** — lorsqu'un lot appartient à plusieurs titulaires (**indivision**) ou est **démembré** (usufruit/nue-propriété, emphytéose, superficie), *« le droit de participation aux délibérations est **suspendu jusqu'à ce que les intéressés désignent** celui qui exercera le droit »* (mandataire/représentant unique). Pas de vote commun automatique ; en cas de désaccord usufruitier/nu-propriétaire, le juge peut trancher. *([source copropriete-ejuris](https://copropriete-ejuris.be/assemblee-generale-des-coproprietaires/))*
- **Art. 3.87 §7 CC** — un mandataire ne peut accepter plus de **3 procurations**, sauf si le total des voix (les siennes + mandants) **≤ 10%** des voix de la copropriété.

**Problèmes constatés dans le code** :
1. `meeting.rs validate_quorum` ne teste **que les quotités** (`>50%`) — le critère des têtes (Art. 3.87 §5 a) est **absent** → une AG peut être validée à tort (ex. 1 gros copropriétaire = 50% des quotités mais bien moins de la moitié des têtes).
2. Aucune notion de **représentant de vote** ni de **suspension** pour les lots démembrés/indivis (Art. 3.87 §1). `unit_owners.is_primary_contact` existe mais c'est un contact administratif, pas le représentant légal de vote.
3. `vote.rs validate_proxy_mandate()` (3/10%) existe mais **n'est jamais appelé** par un use-case.

## Decision

1. **Quorum double** : `Meeting::validate_quorum` prend désormais en paramètres les **quotités** (`present_quotas`, `total_quotas`) **ET les têtes** (`present_owners`, `total_owners`). Quorum valide ⟺ `present_quotas > total_quotas/2` **ET** `present_owners*2 > total_owners` (majorité stricte). Colonnes `meetings.present_owners_count` / `total_owners_count`. `MeetingCompletionChecklist` gagne les têtes ; nouvel invariant `MissingInvariant::HeadCountQuorumNotReached`.
2. **Représentant de vote / suspension** : `unit_owners` gagne `ownership_type` (`full_owner | usufruct | bare_owner | indivisaire | emphyteote | superficiaire`) et `is_voting_representative bool`. `Unit::voting_right_status(owners) -> Active | Suspended` : **Suspended** si lot démembré/indivis (≥2 titulaires OU type ≠ full_owner) **sans** représentant désigné. Erreur typée `VotingRightSuspended { unit_id }` → 422 `VOTING_RIGHT_SUSPENDED`. Un lot suspendu ne compte **ni en têtes ni en quotités** pour le quorum.
3. **Gates votes** : le use-case d'enregistrement de vote appelle `meeting.check_quorum_for_voting()?` (déjà présent, non branché) **et** `vote.validate_proxy_mandate()?` (3/10%).

## Consequences

**Positives**
- AG juridiquement valides (double quorum) ; nullité évitée.
- Lots démembrés/indivis traités conformément à Art. 3.87 §1 ; vote correctement actif ou suspendu.
- Méthodes domaine existantes enfin câblées.

**Négatives / coûts**
- Migrations : `meetings` (2 colonnes têtes), `unit_owners` (`ownership_type`, `is_voting_representative`).
- Comptage des têtes : `COUNT(DISTINCT owner_id)` via `unit_owners` ; présents dérivés des présences AG.
- Adaptation des seeds de tests AG (lots multi-titulaires doivent désigner un représentant sinon vote suspendu).
- `present_quotas` reste `DOUBLE PRECISION` en DB (dette ADR-0008) — **non aggravée** (têtes = `i32`) ; migration Decimal traitée séparément.

## Alternatives rejetées

- **Quorum quotités seul** (statu quo) : non conforme Art. 3.87 §5 ; rejeté.
- **Vote commun automatique des indivisaires** : interdit par Art. 3.87 §1 (désignation obligatoire) ; rejeté.
- **Stocker la suspension en colonne** : drift garanti ; calcul à la volée depuis `unit_owners` préféré.
