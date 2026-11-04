# ADR 0055 : `acps.organization_id` devient un cache dérivé du mandat actif, puis disparaît

- **Status**: Proposed (à trancher par @gilmry)
- **Date**: 2026-10-31
- **Track**: Software
- **Authors**: @gilmry (décision) + Claude (rédaction)
- **Related**: #1046, [ADR 0053](0053-acces-acp-refus-par-defaut.md), lots 2 et 3 de la lettre de mission du 2026-10-31

## Contexte

`acps.organization_id` est aujourd'hui le seul lien entre une ACP et son syndic. Il est écrit directement par `PUT /acps/{id}` (`Acp::set_organization`), sans date d'effet ni historique. Le domaine modélise pourtant `SyndicMandate` (`holder_at`, `revoke`, échéance légale de trois ans) sans table ni dépôt. Le lot 2 crée `syndic_mandates` ; la question est ce que devient la colonne.

Mesuré le 2026-10-31 sur `dadf78d4` : 66 références à `acp.organization_id` ou `acps.organization_id` dans 12 fichiers de `backend/src`.

## Options

1. **Supprimer la colonne.** Le mandat actif est la seule source ; chaque garde et chaque requête passent par `syndic_mandates`. Propre, mais toutes les références ci-dessus changent d'un coup, avant même que la garde par mandat (lot 3) existe.
2. **Garder la colonne comme cache dérivé, jamais écrite directement.** Seul le use case `TransfererAcp` la met à jour, dans la même transaction que la clôture et l'ouverture des mandats. Un test d'invariant vérifie que `acps.organization_id = holder_at(now)` sur toute la base. Les lectures restent inchangées jusqu'au lot 3.
3. **Statu quo.** Écarté : contredit la décision du PO (mandat daté et révocable).

## Recommandation

**Option 2 pour la v0.1.0, puis option 1.** Le cache limite le rayon du lot 2 à l'écriture, et le lot 3 remplace les lectures une par une par `verify_acp_access`. Quand plus aucune lecture ne consulte la colonne, une migration la retire (fin du lot 4 ou lot 5). La colonne devient `NOT NULL` interdite à l'écriture par un trigger, pour que l'oubli se voie.

## Réversibilité

Élevée : le cache se reconstruit depuis `syndic_mandates` par une seule requête, et la suppression finale se fait par une migration avec `down`. Le point irréversible est la première migration de données qui crée les mandats depuis la colonne actuelle (un mandat ouvert par ACP, `started_at` inconnu, à poser à la date de la migration et à marquer comme reconstitué).
