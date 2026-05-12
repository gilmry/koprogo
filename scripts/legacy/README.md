# scripts/legacy/

Scripts hérités déplacés depuis le root du dépôt lors du cleanup #426 (2026-04-29).

Politique : **les nouveaux scripts ne vont PAS ici**. Cette directory contient uniquement des scripts ad-hoc historiques préservés pour référence.

Pour de nouveaux scripts utilitaires : `scripts/` (root des scripts maintenus). Pour des scripts agent-specific : `.claude/scripts/`.

## Contenu

| Script | Origine | Statut |
|---|---|---|
| `get_building_id.py` | root du dépôt | déprécié, peut être supprimé après vérif d'usage |
| `test_post.py` | root du dépôt | déprécié, fixture de test ad-hoc |

Si tu n'utilises pas ces scripts dans ton workflow actuel, propose leur suppression via une PR.
