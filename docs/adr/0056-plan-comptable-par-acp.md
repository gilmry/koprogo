# ADR 0056 : un plan comptable par ACP

- **Status**: Proposed (à trancher par @gilmry)
- **Date**: 2026-10-31
- **Track**: Software / Finance
- **Authors**: @gilmry (décision) + Claude (rédaction)
- **Related**: #1046, lot 4 de la lettre de mission du 2026-10-31

## Contexte

Le plan comptable `accounts` est rattaché à l'organisation (`UNIQUE(code, organization_id)`, migration `20251107000000`). Les écritures (`journal_entries`) portent déjà `acp_id`, mais leurs lignes pointent vers des comptes du syndic. Au transfert d'une ACP, ses écritures citeraient des comptes qui restent chez l'ancien syndic : la portabilité (critère 1) est impossible dans ce modèle. En droit belge, c'est l'ACP qui tient une comptabilité, pas le syndic.

## Options

1. **Un plan par ACP** : `accounts` reçoit `acp_id`, clé `(acp_id, code)`. À la création d'une ACP, le PCMN de référence est copié depuis une table de gabarit. Chaque ACP peut ajouter ses sous-comptes. Cohérent avec des comptes séparés par ACP et avec le transfert : tout suit.
2. **Modèle commun et instance par ACP** : une table `account_models` (PCMN) et une table `acp_accounts` (instance, soldes, activations). Plus normalisé, mais deux tables à joindre partout, et une divergence du modèle commun affecte toutes les ACP d'un coup.
3. **Statu quo par organisation.** Écarté : casse le critère 1.

## Recommandation

**Option 1.** C'est la plus proche de la pratique comptable (un plan par entité) et de la décision « l'ACP possède sa comptabilité ». La migration : pour chaque ACP, copier les comptes de son organisation actuelle, puis repointer `journal_entry_lines` par correspondance de code dans la même transaction, avec un contrôle de somme (débit, crédit par compte, avant et après). Le gabarit PCMN vit dans une table de référence versionnée, pas dans le code.

## Réversibilité

Moyenne. Le `down` regroupe les comptes par organisation et par code, ce qui perd les sous-comptes propres à une ACP s'il y en a eu entre-temps. La migration de données se fait hors poste de dev par le PO (ADR 0060), avec sauvegarde avant.
