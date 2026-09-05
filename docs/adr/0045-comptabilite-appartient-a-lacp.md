# ADR 0045 : le dossier de gestion appartient à l'ACP, pas au syndic

- **Status**: Proposed (rédigé 2026-09-02, en attente de validation @gilmry)
- **Date**: 2026-09-02
- **Track**: Software / Legal-compliance / Multi-tenancy
- **Authors**: Claude Opus 5 (rédaction) + @gilmry sign-off
- **Related**: [ADR 0010](0010-acte-de-base-conformite-copropriete.md) (l'acte de base est au niveau ACP), [ADR 0002](0002-hexagonal-architecture.md)

## Contexte

### Ce que dit la loi

L'**ACP est une personne morale** (Art. 3.86 CC) : elle a son propre patrimoine,
son fonds de roulement et son fonds de réserve. Les comptes qu'un syndic tient
pour elle sont les comptes de l'ACP, tenus *pour le compte d'autrui*.

Le **syndic est un mandataire à durée bornée** : son mandat ne peut excéder trois
ans (Art. 3.89 § 1er), renouvelable par décision expresse de l'assemblée
générale. Le mandat est donc, par construction, un état transitoire.

À la fin du mandat, l'Art. 3.89 § 5, 7° impose au syndic sortant de transmettre
au successeur, **dans les trente jours**, *« l'ensemble du dossier de la gestion
de l'immeuble, y compris la comptabilité et les archives »*. La loi qualifie donc
elle-même ces données : elles constituent un dossier **attaché à l'immeuble**,
qui suit l'ACP d'un syndic à l'autre.

### Ce que fait le modèle actuel

Les tables opérationnelles portent `organization_id`, et `organizations` désigne
le **cabinet de syndic**. Le filtre de cloisonnement multi-tenant est donc
`WHERE organization_id = <syndic connecté>`.

Cela revient à écrire dans chaque pièce comptable *qui la détient* plutôt que
*à qui elle appartient*. Or les deux faits n'ont pas la même durée de vie :

| Fait | Stabilité |
|---|---|
| ce lot appartient à cette ACP | fixé par l'acte de base, ne change qu'avec lui |
| cet immeuble appartient à cette ACP | quasi immuable (une modification supposerait un nouvel acte de base) |
| cette ACP est gérée par ce syndic | change à chaque AG, au plus tous les trois ans |

Le modèle a inscrit le fait le plus volatil dans la colonne qui sert de clé
d'accès aux enregistrements les plus durables.

### Ce que ça produit, constaté sur le VPS de démonstration

En simulant une passation de mandat entre deux cabinets sur une même ACP :

- le syndic **entrant** ouvre le dossier et voit zéro écriture comptable, zéro
  budget, zéro quote-part, zéro copropriétaire — l'obligation de transmission de
  l'Art. 3.89 § 5, 7° n'a aucune traduction technique ;
- le syndic **sortant** continue de tout voir, y compris les noms, adresses et
  arriérés des copropriétaires, sans base légale pour ce traitement une fois son
  mandat éteint.

Une partie du chemin est déjà faite : `expenses`, `owners`, `charge_distributions`
portent leur rattachement à l'ACP, et l'entité `SyndicMandate` modélise le mandat
borné. Le reste du dossier de gestion — budgets, appels de fonds, écritures,
états datés, réunions, convocations — n'est pas encore rattaché.

## Décision

**Le rattachement d'une donnée de gestion à une ACP est la colonne qui porte la
propriété. Le rattachement à une organisation ne subsiste que là où il désigne
l'auteur d'un acte, jamais comme filtre d'accès.**

1. Toute table qui enregistre un acte de gestion d'une ACP porte `acp_id`,
   directement ou par une jointure stable vers l'acte de base
   (`unit → building → acp`).
2. Le cloisonnement multi-tenant se lit
   `acp_id IN (SELECT id FROM acps WHERE organization_id = $syndic)`, et non
   `organization_id = $syndic`. Le syndic voit ce que son mandat lui confie ;
   ce qu'il voit est dérivé du mandat, il n'est pas gravé dans la donnée.
3. `organization_id` est **conservé en horodatage d'auteur** là où il documente
   qui a posé l'acte (utile en audit et en cas de litige de passation), et
   **retiré de tout prédicat d'autorisation**.
4. Le mandat (`SyndicMandate`) est la seule pièce qui porte la relation
   ACP ↔ syndic, avec ses bornes. Une passation se traduit par la clôture d'un
   mandat et l'ouverture du suivant, pas par une réécriture de données.

### Portée

Restent à rattacher : `budgets`, `call_for_funds`, `journal_entries`,
`etats_dates`, `meetings`, `convocations`, `payment_reminders`,
`owner_contributions`, et la vue d'agrégat `account_balances`.

### Hors-scope

Le **droit d'accès du syndic sortant à ses propres archives** (conservation
légale, défense en cas de litige) n'est pas tranché ici : la présente décision se
borne à couper l'accès opérationnel. Une RFC devra dire ce qu'un mandataire
sortant peut encore consulter, et pendant combien de temps.

## Conséquences

**Positives**

- La passation devient une opération de gouvernance et non une migration de
  données ; l'obligation des trente jours devient exprimable.
- Le cloisonnement cesse de dépendre d'une donnée que l'AG peut changer.
- Une ACP reste lisible même sans syndic en fonction (période entre deux mandats).

**Négatives / coûts**

- Une colonne à rattacher sur neuf tables, et autant de filtres de lecture à
  reprendre. Le projet n'ayant encore rien livré à de vrais copropriétaires, le
  schéma peut être repris directement plutôt que migré par incréments
  (arbitrage @gilmry, 2026-09-02).
- `account_balances` est une vue d'agrégat sans rattachement possible en l'état :
  elle devra être recalculée par ACP, ce qui est un travail à part entière.

## Alternatives écartées

- **Garder `organization_id` et le réécrire à chaque passation.** Rejeté : cela
  réécrit l'histoire comptable pour exprimer un changement de mandataire, et
  perd l'information d'auteur. Une écriture passée par le cabinet A porterait le
  nom du cabinet B.
- **Doubler le filtre (`organization_id` OU `acp_id`).** Rejeté : deux sources de
  vérité pour la même question d'accès, donc deux occasions de diverger. Le point
  de la décision est précisément de n'en avoir qu'une.
- **Ne rien rattacher et filtrer par jointure à la volée** (`... JOIN buildings
  JOIN acps`). Rejeté pour les seules tables sans chemin stable vers un
  immeuble ; retenu au contraire là où le chemin existe et ne bouge pas
  (cf. `owners`, cloisonné par ses détentions).
