# ADR 0058 : supprimer une organisation ne détruit jamais une donnée d'ACP

- **Status**: Proposed (à trancher par @gilmry)
- **Date**: 2026-10-31
- **Track**: Software / Security
- **Authors**: @gilmry (décision) + Claude (rédaction)
- **Related**: #1046, lot 2 de la lettre de mission du 2026-10-31

## Contexte

`budgets`, `journal_entries`, `call_for_funds` et `owners` portent `organization_id ... ON DELETE CASCADE`. Supprimer une organisation de syndic efface la comptabilité et les copropriétaires des ACP qu'elle gérait. Or ces données appartiennent à l'ACP, et un syndic qui cesse son activité ne doit rien emporter.

## Options

1. **`RESTRICT`.** Une organisation ne peut être supprimée tant qu'un mandat est actif ou qu'une ligne la référence. Il faut d'abord transférer ses ACP. Le refus est explicite, rien ne disparaît en silence.
2. **`SET NULL`.** La suppression passe, les ACP deviennent auto-gérées d'office. Les données restent, mais personne n'y a plus accès (une ACP sans organisation n'est accessible qu'au SuperAdmin tant que le lot 3 n'est pas fait), et le mandat n'est pas clos proprement.
3. **Archivage logique.** `organizations.archived_at`, aucune suppression physique. Les mandats sont clos à la date d'archivage. La ligne reste pour l'historique (un état daté à J-1 doit encore citer l'ancien syndic).

## Recommandation

**Option 1 sur toutes les clés étrangères qui portent une donnée d'ACP, et option 3 pour l'organisation elle-même.** Autrement dit : les `CASCADE` deviennent `RESTRICT` dès le lot 2, et « supprimer » une organisation dans l'interface signifie l'archiver, ce qui n'est permis qu'une fois tous ses mandats clos. Les colonnes `organization_id` de ces tables disparaissent ensuite au lot 4 au profit de `acp_id`, et la question se vide.

## Réversibilité

Élevée : changer une action référentielle est une migration avec `down`. L'archivage logique est réversible par nature. Aucune donnée n'est détruite par cette décision ; c'est précisément son objet.
