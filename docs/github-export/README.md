# GitHub Export to RST

Ce dossier contient l'export automatique des données GitHub du projet KoproGo au format RST (ReStructuredText), compatible avec Sphinx.

## 📦 Contenu

- **[index.rst](index.rst)** : Vue d'ensemble du projet GitHub
- **[issues/](issues/)** : Toutes les issues (54 issues au total)
  - `issues/issue-*.rst` : Chaque issue exportée individuellement
  - `issues/by-phase/` : Issues classées par phase (VPS, K3s, K8s)
  - `issues/by-priority/` : Issues classées par priorité (critical, high, medium, low)
  - `issues/by-label/` : Issues classées par labels (top 10)
- **[milestones/](milestones/)** : Les 3 milestones du projet
  - Phase 1: VPS MVP + Legal Compliance (Nov 2025 - Mar 2026)
  - Phase 2: K3s + Automation (Mar - Juin 2026)
  - Phase 3: K8s Production (Juin - Sept 2026)
- **[projects/](projects/)** : Les 2 GitHub Projects
  - KoproGo - Software Roadmap
  - KoproGo - Infrastructure Roadmap
- **[labels/](labels/)** : Les 32 labels du projet

## 🔄 Mise à jour

Pour mettre à jour l'export avec les dernières données GitHub :

```bash
# Via Makefile (recommandé)
make docs-export-github

# Ou directement
./scripts/export-github-to-rst.sh
```

## 📚 Intégration Sphinx

L'export est automatiquement intégré dans la documentation Sphinx via `docs/index.rst` :

```rst
.. toctree::
   :maxdepth: 2
   :caption: 📊 GitHub Project Management

   github-export/index
```

Pour générer la documentation Sphinx avec l'export GitHub :

```bash
# Build Sphinx docs
make docs-sphinx

# Ou avec live reload
make docs-serve
```

La documentation sera accessible à : `http://localhost:8000` (section "GitHub Project Management")

## 🤖 Automatisation

Pour automatiser l'export GitHub quotidien, vous pouvez :

1. **Via cron** (serveur) :
```bash
# Chaque jour à 6h du matin
0 6 * * * cd /path/to/koprogo && ./scripts/export-github-to-rst.sh
```

2. **Via GitHub Actions** (CI/CD) :
```yaml
name: Export GitHub to RST
on:
  schedule:
    - cron: '0 6 * * *'  # Tous les jours à 6h UTC
  workflow_dispatch:  # Déclenchement manuel

jobs:
  export:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install gh CLI
        run: |
          sudo apt-get update
          sudo apt-get install -y gh
      - name: Export GitHub data
        run: ./scripts/export-github-to-rst.sh
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      - name: Commit changes
        run: |
          git config user.name "GitHub Actions"
          git config user.email "actions@github.com"
          git add docs/github-export
          git commit -m "docs: auto-update GitHub export [skip ci]" || echo "No changes"
          git push
```

## 🔍 Structure des fichiers générés

```
docs/github-export/
├── index.rst                    # Vue d'ensemble
├── stats.rst                    # Statistiques rapides
├── README.md                    # Ce fichier
├── issues/
│   ├── index.rst               # Index de toutes les issues
│   ├── issue-1.rst             # Issue #1
│   ├── issue-2.rst             # Issue #2
│   ├── ...
│   ├── by-phase/
│   │   ├── index.rst
│   │   ├── phase-vps.rst       # Issues Phase VPS
│   │   ├── phase-k3s.rst       # Issues Phase K3s
│   │   └── phase-k8s.rst       # Issues Phase K8s
│   ├── by-priority/
│   │   ├── index.rst
│   │   ├── critical.rst        # Issues critiques
│   │   ├── high.rst            # Issues haute priorité
│   │   ├── medium.rst          # Issues priorité moyenne
│   │   └── low.rst             # Issues basse priorité
│   └── by-label/
│       ├── index.rst
│       └── label-*.rst         # Top 10 labels
├── milestones/
│   ├── index.rst
│   ├── milestone-1-*.rst       # Milestone 1 (Phase VPS)
│   ├── milestone-2-*.rst       # Milestone 2 (Phase K3s)
│   └── milestone-3-*.rst       # Milestone 3 (Phase K8s)
├── projects/
│   ├── index.rst
│   ├── project-2-*.rst         # Software Roadmap
│   └── project-3-*.rst         # Infrastructure Roadmap
└── labels/
    └── index.rst               # Liste complète des labels
```

## 📊 Statistiques actuelles

**Dernière mise à jour** : 2025-11-04 20:55:40 CET

- **Issues totales** : 54 (47 ouvertes, 7 fermées)
- **Milestones** : 3
- **Labels** : 32
- **Projects** : 2

## 💡 Utilisation pour Claude Code (web)

Ce dossier permet à Claude Code Web (qui n'a pas accès direct à GitHub) d'avoir une vue complète du projet en clonant simplement le dépôt :

```bash
git clone https://github.com/gilmry/koprogo.git
cd koprogo
# Toutes les données GitHub sont dans docs/github-export/
```

Claude Code Web peut alors naviguer dans :
- Les issues par phase, priorité ou label
- Les milestones avec leurs issues associées
- Les projets GitHub
- Les statistiques du projet

## 🛠️ Dépendances

Le script d'export nécessite :

- **gh CLI** : GitHub CLI officiel (`make install-deps` pour l'installer)
- **jq** : Parser JSON (généralement pré-installé sur Linux/macOS)
- **Authentification GitHub** : `gh auth login` si pas déjà connecté

## 📝 Format RST

Le format RST (ReStructuredText) est le format natif de Sphinx. Avantages :

- ✅ Conversion automatique vers HTML/PDF
- ✅ Liens croisés automatiques entre documents
- ✅ Table des matières générée automatiquement
- ✅ Syntaxe lisible en texte brut
- ✅ Support complet dans Sphinx

## 🔗 Liens utiles

- [GitHub Repository](https://github.com/gilmry/koprogo)
- [Sphinx Documentation](https://www.sphinx-doc.org/)
- [ReStructuredText Primer](https://www.sphinx-doc.org/en/master/usage/restructuredtext/basics.html)
- [GitHub CLI Documentation](https://cli.github.com/manual/)

---

*Export généré automatiquement par `scripts/export-github-to-rst.sh`*
