# ADR 0047 : une migration initiale unique jusqu'à la première release, puis plus jamais

- **Status**: Accepted (tranché par @gilmry le 2026-09-10)
- **Date**: 2026-09-10
- **Track**: Software / Data / Release
- **Authors**: @gilmry (décision) + Claude Opus 5 (rédaction)
- **Related**: issues #846, #840, [ADR 0008](0008-numeric-vs-double-precision-postgresql.md)

## Contexte

Deux arbitrages distincts posaient, sans qu'on l'ait vu, **la même question** :

**Les onze tables muettes (#846).** Onze tables et une vue existent en base et
ne sont lues par aucun code. Fallait-il les supprimer par migration, au risque
d'une opération irréversible, ou les documenter comme réservées ?

**Les résolutions invotables (#840).** La migration qui ajoute
`agenda_item_index` ne reprend pas les données : toute résolution antérieure
porte `NULL` et devient invotable dès le déploiement du refus de l'Art. 3.87
§ 2. Fallait-il écrire une reprise qui rattache chaque résolution au point 0 —
en inscrivant une **supposition** dans des données que la loi rend opposables ?

J'avais traité les deux comme des décisions lourdes, parce que je raisonnais
sous une contrainte qui n'existe pas encore.

## Décision

**Jusqu'à la première release, le schéma peut être refondu librement.** Une
migration initiale unique sera écrite pour la release, remplaçant l'empilement
actuel. Aucune reprise de données n'est à écrire d'ici là.

**À partir de la première release, l'inverse devient vrai** : chaque migration
devra préserver les données existantes, sans exception.

### La raison

> « Pour le moment il n'y a rien en prod, il n'y a personne qui dépend de nous,
> donc on peut repartir sur une base propre. Les vraies migrations importantes,
> c'est à partir de la première release, car il faudra faire gaffe à
> l'auto-hébergé. »

Aucun utilisateur tiers n'héberge KoproGo. La contrainte de compatibilité
ascendante que je m'appliquais était **empruntée à un futur qui n'est pas
arrivé**. La respecter aujourd'hui coûte du travail réel — reprises de données,
suppositions inscrites, migrations de suppression pesées — pour protéger
personne.

### Pourquoi la borne est la release et pas une date

Le produit est destiné à l'auto-hébergement. Le jour où quelqu'un installe la
`v0.1.0` chez lui, ses données deviennent les siennes et cessent d'être
jetables. Ce n'est pas un délai qui ferme la fenêtre, c'est un **fait** : la
première installation hors de notre contrôle.

## Conséquences

### Ce qui se simplifie immédiatement

- **#846 devient sans objet.** Les onze tables et la vue disparaîtront avec la
  refonte du schéma. Rien à supprimer par migration, rien à documenter comme
  réservé. Le cas `proxy_mandate_stats` — une vue qui se déclare outil de
  conformité et rend toujours zéro — se règle de même.
- **#840 ne demande aucune reprise.** Les résolutions à `agenda_item_index`
  NULL disparaîtront avec le schéma. Le refus de `cast_vote` peut être déployé
  tel quel.
- Plus généralement : aucune migration d'ici la release n'a à se soucier de ce
  qu'elle casse en base.

### Ce qui devient une dette datée

**La migration initiale unique est un livrable de la release**, pas un
nettoyage optionnel. Si elle n'est pas écrite, l'empilement actuel devient le
socle de tous les auto-hébergés, avec ses onze tables mortes.

### La nuance à ne pas perdre

Le VPS ecosolva porte une démo, et la consigne courante est de le traiter comme
de la production. Les deux se concilient sans peine : **rien de ce qui y vit
n'a besoin d'être préservé** lors d'une remise à zéro de schéma, mais on ne
l'efface pas par surprise pour autant. Une remise à zéro reste une opération
annoncée, pas un effet de bord.

### Ce que cette ADR n'autorise pas

Elle ne dit pas que les données de démonstration sont sans valeur — elles
servent aux recettes navigateur, qui ont trouvé plus de défauts que n'importe
quel test. Elle dit que leur **schéma** n'est pas un contrat.
