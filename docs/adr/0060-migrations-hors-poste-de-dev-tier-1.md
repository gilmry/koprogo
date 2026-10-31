# ADR 0060 : toute migration hors du poste de dev est lancée par le PO

- **Status**: Proposed (à trancher par @gilmry)
- **Date**: 2026-10-31
- **Track**: Software / Governance
- **Authors**: @gilmry (décision) + Claude (rédaction)
- **Related**: #1046, [ADR 0050](0050-pile-de-recette-sur-le-vps-ports-decales.md), lettre de mission du 2026-10-31 §3.6 et §5

## Contexte

Les lots 2 à 5 comportent des migrations de données (mandats reconstitués, plan comptable copié par ACP, fiches de copropriétaires fusionnées, `acp_id` rempli sur une vingtaine de tables). Revenir à l'image précédente ne défait pas une migration : un retour arrière après migration aggrave la panne au lieu de la réparer.

## Options

1. **Le PO lance lui-même toute migration hors poste de dev** (recette comme production), le samedi de 9 h à 12 h, après sauvegarde, avec le `down` testé sur une copie. Un agent prépare la migration, son `down`, le script de sauvegarde et la vérification ; il ne l'exécute que sur la base jetable de la CI et du poste de dev.
2. **Un agent peut migrer la recette, le PO garde la production.** Plus rapide pour éprouver les migrations, mais la recette tourne sur le même hôte que les démos, et une erreur y coûte une matinée.
3. **Migration automatique au déploiement.** Écarté tant que les `down` ne sont pas prouvés et que le modèle bouge.

## Recommandation

**Option 1.** Chaque migration de données livre quatre pièces dans la pull request : `up`, `down`, un script de sauvegarde et de restauration, et un test qui joue `up` puis `down` sur un jeu de données réaliste. Le PO exécute, un agent regarde. Le créneau du samedi et le retour arrière à midi s'appliquent.

## Réversibilité

C'est l'objet de la décision : rendre chaque migration réversible avant de l'exécuter, et confier l'exécution à la seule personne qui répond du résultat.
