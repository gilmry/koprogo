==============================
Guide du Contributeur
==============================

Cette section vous guide pas à pas pour contribuer à KoproGo, de l'installation à votre première pull request.

.. tip::
   **Nouveau contributeur ?** Suivez le :doc:`../parcours-contributeur` pour un parcours guidé complet.

Contenu de cette Section
=========================

.. toctree::
   :maxdepth: 2

   premiers-pas
   installer-projet
   faire-premiere-contribution
   comprendre-rfc-adr
   standards-code

Vue d'Ensemble
==============

**Étapes pour Contribuer**

1. **Comprendre le projet** : Lisez :doc:`../vision-strategie/pourquoi-koprogo`
2. **Installer localement** : Suivez :doc:`installer-projet`
3. **Choisir une issue** : "good first issue" sur GitHub
4. **Coder avec TDD** : Tests d'abord, puis implémentation
5. **Ouvrir une PR** : Suivez :doc:`faire-premiere-contribution`

**Prérequis Système**

* Rust 1.83+ (rustup)
* Node.js 18+ & npm
* PostgreSQL 15+
* Docker (optionnel)
* Git 2.30+

**Philosophie de Contribution**

✅ **Tests d'abord** (TDD)
✅ **Code propre** (rustfmt, clippy)
✅ **Documentation synchronisée**
✅ **Commits signés** (DCO)
✅ **Reviews constructives**

Documents
=========

:doc:`premiers-pas`
   Premiers pas pour contribuer : prérequis, cloner le repo, explorer les issues, rejoindre la communauté.

:doc:`installer-projet`
   Installation pas-à-pas du projet localement : Rust, PostgreSQL, Docker, configuration, lancement des tests.

:doc:`faire-premiere-contribution`
   Workflow Git complet : branches, TDD, commits, pull requests, reviews, checklist PR.

:doc:`comprendre-rfc-adr`
   Qu'est-ce qu'une RFC ? Un ADR ? Comment proposer une RFC ? Processus d'approbation.

:doc:`standards-code`
   Conventions de code : Rust (rustfmt, clippy), TypeScript (ESLint), commits (conventional commits), DCO.

**Ressources Complémentaires**

* :doc:`../PROJECT_STRUCTURE` - Structure détaillée du projet
* :doc:`../GIT_HOOKS` - Hooks Git (pre-commit, pre-push)
* :doc:`../MAKEFILE_GUIDE` - Guide du Makefile
* :doc:`../E2E_TESTING_GUIDE` - Guide des tests E2E

----

*Section Guide du Contributeur - Documentation KoproGo ASBL*
