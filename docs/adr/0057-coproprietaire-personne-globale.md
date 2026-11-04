# ADR 0057 : le copropriétaire est une personne globale, reliée aux ACP par ses lots

- **Status**: Proposed (à trancher par @gilmry)
- **Date**: 2026-10-31
- **Track**: Software / GDPR
- **Authors**: @gilmry (décision) + Claude (rédaction)
- **Related**: #1046, [ADR 0053](0053-acces-acp-refus-par-defaut.md), lots 3 et 4 de la lettre de mission du 2026-10-31

## Contexte

`owners` porte `organization_id` (migration `20250127000000`, index unique `(email, organization_id)`). La fiche d'un copropriétaire appartient donc au syndic, ni à la personne ni à l'ACP. Au transfert d'une ACP, la fiche reste chez l'ancien syndic et les copropriétaires perdent l'accès à leur propre ACP. La table `unit_owners` existe déjà et relie une personne à un lot.

## Options

1. **Personne globale, reliée par `unit_owners`.** Une fiche par personne, sans `organization_id` ; ses droits découlent de ses lots. Une personne propriétaire dans deux ACP a une seule fiche. Le transfert ne touche pas aux personnes. RGPD : chaque ACP est responsable du traitement des données qui la concernent (quotes-parts, dettes, PV) ; la plateforme tient l'identité minimale (nom, courriel) comme sous-traitant de chaque ACP, et le contrat de sous-traitance le dit.
2. **Une fiche par ACP.** Portabilité triviale (la fiche suit l'ACP), frontière RGPD simple (tout appartient à l'ACP), mais une même personne existe en plusieurs exemplaires, avec autant de consentements, de corrections et de droits d'accès à exercer.
3. **Statu quo par organisation.** Écarté : contredit la décision et casse les critères 1 et 2.

## Recommandation

**Option 1.** C'est le modèle que le domaine suppose déjà (`unit_owners`) et le seul où le critère « les copropriétaires gardent l'accès après le transfert » tient sans copie de données. Le point à trancher par le PO n'est pas technique : c'est **qui est responsable de traitement** de la fiche d'identité commune. La rédaction propose : l'ACP pour tout ce qui la concerne, l'exploitant de la plateforme pour l'identité partagée, sur une base légale que le PO choisit (contrat avec l'ACP, ou intérêt légitime). Cette base légale est réservée au PO et n'est pas tranchée ici.

## Réversibilité

Moyenne. Fusionner les fiches en doublon (même courriel dans plusieurs organisations) est l'opération irréversible : elle se prépare par un rapport de doublons relu avant la migration, et un `down` qui redistribue par organisation reste possible tant que l'historique de `unit_owners` est conservé.
