# Branch Cleanup Scripts

Scripts pour nettoyer les branches distantes obsolètes du dépôt KoproGo.

## Scripts Disponibles

### 1. `cleanup-merged-branches.sh`

Supprime les **12 branches Claude** qui sont complètement mergées dans `main`.

**Branches concernées:**
- `claude/add-ci-workflows-011CUMhvUnFsKBJoJ9rbWXoN`
- `claude/analyze-koprogov-gaps-011CUh7ioKWdZhjJ1f9FuUsA`
- `claude/board-bdd-tests-011CUoTtHbDHEVcjxuBomuXa`
- `claude/coproperty-management-api-011CUMdbsxYD4gaY6CunHTxJ`
- `claude/implement-feature-011CUu1Thc9FFvC2kV5Qptem`
- `claude/install-make-commands-011CUXqV27esaJtLT3uMgCWw`
- `claude/invoice-system-issue-73-011CUoTtHbDHEVcjxuBomuXa`
- `claude/koprogo-agile-documentation-01CCjEWRbpxLjdDLdteKs58A`
- `claude/koprogo-docs-update-011CV4VuR2WmjfjGkYqnABrp`
- `claude/market-study-priorities-011CUQaaCd44rswjFhufsWVX`
- `claude/restructure-koprogo-docs-015UntXtYRrUE5XMV9CyKRBx`
- `chore/dependabot-updates-2025-11-03`

**Utilisation:**
```bash
./scripts/cleanup-merged-branches.sh
```

### 2. `cleanup-dependabot-branches.sh`

Supprime les **branches Dependabot obsolètes** (anciennes versions de dépendances).

**Stratégie:** Garde uniquement la version la plus récente de chaque dépendance.

**Exemples de branches supprimées:**
- `astro-5.15.1`, `astro-5.15.3` (garde `5.15.4`)
- `svelte-5.41.3`, `5.42.2`, `5.43.2` (garde `5.43.5`)
- `jsonwebtoken-10.1.0` (garde `10.2.0`)
- etc.

**Total:** ~16 branches Dependabot obsolètes

**Utilisation:**
```bash
./scripts/cleanup-dependabot-branches.sh
```

## Ordre Recommandé

1. **D'abord:** Exécuter `cleanup-merged-branches.sh`
   - Supprime les branches Claude mergées (safe, pas de risque)
   - Libère 12 branches

2. **Ensuite:** Exécuter `cleanup-dependabot-branches.sh`
   - Supprime les anciennes versions de dépendances
   - Libère ~16 branches
   - **Note:** Vérifier qu'aucune PR Dependabot importante n'est ouverte avant

3. **Après nettoyage:**
   ```bash
   git fetch --prune
   ```

## Impact Total

**Avant nettoyage:** 77 branches distantes

**Après nettoyage:** ~49 branches (77 - 12 - 16 = 49)

**Répartition finale estimée:**
- 3 branches principales (main, testing, integration251118)
- 13 branches Claude actives (25 - 12 = 13)
- 26 branches Dependabot actives (42 - 16 = 26)
- 7 autres branches (feat, docs, chore, hotfix)

## Sécurité

Les deux scripts:
- ✅ Vérifient que les branches sont bien mergées avant suppression
- ✅ Demandent confirmation avant d'agir
- ✅ Affichent un résumé détaillé
- ✅ Gèrent les erreurs (branches déjà supprimées)
- ✅ Utilisent `set -e` pour arrêter en cas d'erreur

## Rollback

Si vous supprimez une branche par erreur, vous pouvez la restaurer:

```bash
# Trouver le SHA du dernier commit de la branche
git reflog | grep "branch-name"

# Recréer la branche
git push origin <SHA>:refs/heads/branch-name
```

## Vérification Post-Cleanup

Après exécution, vérifier sur GitHub:
https://github.com/gilmry/koprogo/branches

Ou localement:
```bash
git fetch --prune
git branch -r | wc -l  # Devrait afficher ~49
```
