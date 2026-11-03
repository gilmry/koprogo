========================================
Bienvenue dans la Communauté KoproGo !
========================================

**Vous découvrez KoproGo ? Vous êtes au bon endroit !**

Ce guide vous accompagne pas à pas dans votre découverte du projet et vous aide à faire votre première contribution. Que vous soyez développeur débutant ou expérimenté, partenaire technique, ou simplement curieux du projet, vous trouverez ici le parcours adapté à votre profil.

.. note::
   **Temps de lecture total** : Environ 2-3 heures pour comprendre le projet et être prêt à contribuer

   **Aucune connaissance préalable requise** : Ce guide part du principe que vous découvrez KoproGo pour la première fois.

Qui Êtes-Vous ?
===============

KoproGo accueille différents types de contributeurs. Choisissez le profil qui vous correspond le mieux :

.. list-table::
   :header-rows: 1
   :widths: 25 50 25

   * - Profil
     - Intérêt Principal
     - Temps Estimé
   * - 👨‍💻 **Développeur Débutant**
     - Apprendre Rust, architecture hexagonale, open-source
     - 3-4 heures
   * - 👩‍💻 **Développeur Expérimenté**
     - Contribuer code, architecture, reviews
     - 1-2 heures
   * - 🏢 **Partenaire Technique**
     - Comprendre l'architecture, intégrations possibles
     - 2 heures
   * - 📊 **Organisme Public**
     - Impact sociétal, gouvernance, modèle économique
     - 1 heure
   * - 🏘️ **ASBL de Copropriété**
     - Utilisation pratique, bénéfices concrets
     - 30 min

Tous les profils sont les bienvenus ! Le parcours ci-dessous s'adapte à vos besoins.

Parcours Guidé en 5 Étapes
===========================

Étape 1 : Comprendre le Projet (15-30 min)
-------------------------------------------

**Objectif** : Comprendre pourquoi KoproGo existe et quel problème il résout.

📖 **Lectures Essentielles**

1. Lisez :doc:`vision-strategie/pourquoi-koprogo` (10 min)

   * Le problème sociétal des copropriétés belges
   * Les coûts prohibitifs (200-500€/mois) et leur impact
   * Comment KoproGo propose une alternative à 5€/mois
   * L'impact écologique (96% réduction CO₂)

2. Découvrez :doc:`vision-strategie/vision` (10 min)

   * Vision à long terme : 5 000 copropriétés
   * Modèle de démocratie tarifaire (prix voté en AG)
   * Paliers de progression mesurables
   * Impact social et économique attendu

3. Explorez :doc:`vision-strategie/mission` (10 min optionnel)

   * Les 6 piliers de la mission
   * Pratiques technologiques à la pointe
   * Développement collaboratif et opensource

📺 **Ressource Vidéo** (optionnel)

* Regardez la `vidéo de présentation KoproGo <https://www.youtube.com/@koprogo>`_ (si disponible)

✅ **Checkpoint 1**

Après cette étape, vous devriez pouvoir répondre à ces questions :

* Pourquoi KoproGo existe-t-il ?
* Quelle est la différence entre une solution propriétaire (200€/mois) et KoproGo (5€/mois) ?
* Quel est l'impact écologique de KoproGo ?
* Qu'est-ce que la "démocratie tarifaire" ?

Étape 2 : Explorer la Roadmap (20-30 min)
------------------------------------------

**Objectif** : Comprendre où en est le projet et où il va.

🗺️ **Comprendre la Progression**

1. Consultez :doc:`roadmap/roadmap-2025-2030` (15 min)

   * Vision 2025-2030 : De 100 à 5 000 copropriétés
   * Les 3 phases : VPS MVP, K3s, K8s Production
   * Évolution juridique : ASBL → Coopérative
   * Modules PropTech 2.0 (IoT, IA, Blockchain)

2. Voyez :doc:`roadmap/jalons-atteints` (5 min)

   * Jalon 0-1 : Fondations techniques (73 endpoints API ✅)
   * Architecture hexagonale opérationnelle
   * Tests E2E automatisés avec Playwright
   * Infrastructure VPS OVH en place

3. Explorez :doc:`roadmap/jalons-a-venir` (10 min)

   * Jalon 2 : Conformité légale belge (État daté, PCMN)
   * Jalon 3 : Features différenciantes (SEL, Partage objets)
   * Jalons 4-6 : Automation, Mobile, PropTech 2.0

✅ **Checkpoint 2**

Après cette étape, vous devriez comprendre :

* Où en est le projet aujourd'hui (jalons atteints)
* Quelle est la prochaine grande étape (Jalon 2)
* Comment le projet progresse par capacités, pas par dates
* Les 3 moteurs d'acquisition (Gestion, Communauté, Valeurs)

Étape 3 : Comprendre l'Architecture (30-45 min)
-----------------------------------------------

**Objectif** : Comprendre comment KoproGo est construit techniquement.

🏗️ **Architecture & Technologies**

1. Lisez :doc:`architecture/vue-ensemble` (15 min)

   * Architecture hexagonale (Ports & Adapters)
   * Couches : Domain, Application, Infrastructure
   * Flux de données : HTTP → Handlers → Use Cases → Repositories
   * Séparation frontend/backend

2. Découvrez :doc:`architecture/choix-technologiques` (15 min)

   * **Pourquoi Rust ?** (Performance 10x, sécurité mémoire, éco-responsabilité)
   * **Pourquoi PostgreSQL ?** (Fiabilité, ACID, JSONB)
   * **Pourquoi Astro + Svelte ?** (Performance, Islands architecture, SEO)
   * **Pourquoi Architecture Hexagonale ?** (Testabilité, évolutivité)

3. Parcourez :doc:`backend/index` (10 min optionnel)

   * Structure du code backend
   * Entités du domaine (Building, Unit, Owner, Expense, etc.)
   * Repositories et Use Cases
   * Tests (unit, integration, BDD, E2E)

📖 **Documentation Technique** (optionnel pour développeurs)

* :doc:`PROJECT_STRUCTURE` - Structure détaillée du projet
* :doc:`MULTI_OWNER_SUPPORT` - Support multi-propriétaires
* :doc:`MULTI_ROLE_SUPPORT` - Support multi-rôles utilisateurs

✅ **Checkpoint 3**

Après cette étape, vous devriez comprendre :

* Qu'est-ce que l'architecture hexagonale ?
* Pourquoi Rust permet des économies de 99% sur l'infrastructure ?
* Comment les couches Domain, Application, Infrastructure sont organisées
* Quelles sont les entités principales du domaine ?

Étape 4 : Préparer Votre Environnement (1-2h)
----------------------------------------------

**Objectif** : Installer le projet localement et vérifier que tout fonctionne.

🛠️ **Installation Pas-à-Pas**

1. Suivez :doc:`contribuer/premiers-pas` (15 min)

   * Vérifier les prérequis système (Rust, Node.js, PostgreSQL, Docker)
   * Cloner le repository GitHub
   * Lire CONTRIBUTING.md
   * Explorer les "good first issues"

2. Installez le projet : :doc:`contribuer/installer-projet` (45-90 min)

   * Installation Rust (rustup)
   * Installation Node.js et npm
   * Installation PostgreSQL
   * Installation Docker (optionnel)
   * Configurer les variables d'environnement
   * Lancer `make setup`
   * Démarrer le backend (`make dev`)
   * Démarrer le frontend (`cd frontend && npm run dev`)
   * Vérifier `http://localhost:4321`

3. Lancez les tests (15 min)

   .. code-block:: bash

      # Tests unitaires
      make test

      # Tests E2E
      make test-e2e

      # Build du frontend
      cd frontend && npm run build

🎯 **Objectif** : Voir la page d'accueil s'afficher sur `http://localhost:4321`

✅ **Checkpoint 4**

Après cette étape, vous devriez avoir :

* ✅ Backend Rust qui tourne sur `http://localhost:8080`
* ✅ Frontend Astro qui tourne sur `http://localhost:4321`
* ✅ Base de données PostgreSQL opérationnelle
* ✅ Tests qui passent (`make test`)

.. note::
   **Problèmes d'Installation ?**

   * Consultez :doc:`contribuer/installer-projet` pour le troubleshooting
   * Demandez de l'aide sur `GitHub Discussions <https://github.com/gilmry/koprogo/discussions>`_
   * Rejoignez le Discord (lien dans le README)

Étape 5 : Faire Votre Première Contribution (1-2h)
---------------------------------------------------

**Objectif** : Contribuer au projet et ouvrir votre première pull request.

🚀 **Votre Première Contribution**

1. Lisez :doc:`contribuer/faire-premiere-contribution` (20 min)

   * Workflow Git : branches, commits, pull requests
   * TDD (Test-Driven Development) : tests d'abord
   * Standards de code (rustfmt, clippy)
   * Conventions de commits (`feat:`, `fix:`, `docs:`)
   * DCO (Developer Certificate of Origin) : `git commit -s`

2. Trouvez une "Good First Issue" (10 min)

   * Allez sur https://github.com/gilmry/koprogo/issues
   * Filtrez par label "good first issue"
   * Choisissez une issue qui vous intéresse
   * Commentez sur l'issue pour indiquer que vous travaillez dessus

3. Créez une branche et codez (30-60 min)

   .. code-block:: bash

      # Créer une branche
      git checkout -b feat/mon-feature

      # Écrire les tests d'abord (TDD)
      # Puis implémenter la feature

      # Vérifier que tout passe
      make test
      make lint

      # Commit avec DCO
      git commit -s -m "feat: ajouter ma feature"

4. Ouvrez une Pull Request (20 min)

   * Poussez votre branche : `git push origin feat/mon-feature`
   * Ouvrez une PR sur GitHub
   * Remplissez le template de PR
   * Attendez les reviews et répondez aux commentaires

✅ **Checkpoint 5**

Après cette étape, vous devriez avoir :

* ✅ Une pull request ouverte sur GitHub
* ✅ Des tests qui passent en CI
* ✅ Code conforme aux standards (rustfmt, clippy)
* ✅ Commit signé avec DCO

.. tip::
   **Félicitations ! Vous êtes maintenant un contributeur KoproGo !** 🎉

   Votre contribution, quelle que soit sa taille, est précieuse pour la communauté.

Parcours Alternatifs
====================

Selon votre profil, vous pouvez adapter le parcours ci-dessus :

Pour les Non-Développeurs
--------------------------

**Si vous n'êtes pas développeur**, vous pouvez contribuer autrement :

* **Documentation** : Améliorez les docs, corrigez les fautes, ajoutez des exemples
* **Traductions** : Traduisez la documentation en NL, DE, EN
* **Tests** : Rejoignez le programme beta et testez la plateforme
* **Feedback** : Partagez vos idées sur GitHub Discussions
* **Promotion** : Parlez de KoproGo autour de vous

**Parcours recommandé** :

1. Étape 1 : Comprendre le projet
2. Étape 2 : Explorer la roadmap
3. Contribuer à la documentation (pas besoin d'Étape 4-5)

Pour les Développeurs Expérimentés
-----------------------------------

**Si vous êtes un développeur Rust expérimenté**, vous pouvez accélérer :

* Étape 1 : Lecture rapide (10 min)
* Étape 2 : Parcourir la roadmap (10 min)
* Étape 3 : Architecture (20 min)
* Étape 4 : Installation (30 min)
* Étape 5 : Contribuer directement sur des issues complexes

**Issues recommandées** : Cherchez les labels "help wanted" ou "architecture" sur GitHub.

Pour les Partenaires Techniques
--------------------------------

**Si vous représentez une organisation partenaire** :

* Étape 1 : Comprendre le projet (focus sur l'impact sociétal)
* Étape 2 : Roadmap (focus sur les jalons et capacités)
* Étape 3 : Architecture (focus sur les intégrations possibles)
* **Étape 6** : Lisez :doc:`economic-model/modele-economique` (modèle OpenCore, viabilité)
* **Étape 7** : Lisez :doc:`gouvernance/modele-asbl` (gouvernance, transparence)

**Contact** : contact@koprogo.com pour discuter de partenariats.

Ressources Complémentaires
===========================

Gouvernance & RFC/ADR
---------------------

Si vous voulez comprendre comment les décisions sont prises dans le projet :

* :doc:`contribuer/comprendre-rfc-adr` - Qu'est-ce qu'une RFC ? Un ADR ?
* :doc:`gouvernance/index` - Gouvernance du projet
* :doc:`gouvernance/modele-asbl` - Modèle de la future ASBL

Documentation Technique Approfondie
------------------------------------

Pour aller plus loin techniquement :

* :doc:`backend/index` - Documentation backend complète
* :doc:`frontend/index` - Documentation frontend complète
* :doc:`infrastructure/index` - Infrastructure et déploiement
* :doc:`PERFORMANCE_TUNING` - Optimisation des performances
* :doc:`DATABASE_ADMIN` - Administration PostgreSQL

Guides Utilisateurs
--------------------

Si vous voulez comprendre l'usage de la plateforme :

* :doc:`user-guides/syndic-guide` - Guide du syndic
* :doc:`user-guides/owner-guide` - Guide du copropriétaire
* :doc:`user-guides/accountant-guide` - Guide du comptable
* :doc:`user-guides/board-member-guide` - Guide du conseil de copropriété

Communauté & Support
=====================

**Besoin d'Aide ?**

* **GitHub Discussions** : https://github.com/gilmry/koprogo/discussions
* **GitHub Issues** : https://github.com/gilmry/koprogo/issues
* **Discord** : (lien à venir)
* **Email** : contact@koprogo.com

**Suivez-Nous**

* **YouTube** : https://www.youtube.com/@koprogo (tutoriels vidéo)
* **GitHub** : https://github.com/gilmry/koprogo

**Contribuez**

* **Code** : Pull requests bienvenues
* **Documentation** : Améliorez les docs
* **Traductions** : NL, DE, EN, ES, IT
* **Tests** : Beta testing
* **Feedback** : Partagez vos idées

Prochaines Étapes
=================

Maintenant que vous avez terminé le parcours, voici les prochaines étapes recommandées :

1. **Rejoignez la communauté** : GitHub Discussions, Discord
2. **Choisissez une issue** : "good first issue" pour commencer
3. **Lisez les standards de code** : :doc:`contribuer/standards-code`
4. **Faites votre première PR** : Suivez le guide :doc:`contribuer/faire-premiere-contribution`
5. **Explorez la roadmap** : Voyez où vous pouvez aider le plus

.. important::
   **Votre Contribution Compte !**

   Que vous corrigiez une faute de frappe dans la doc ou que vous ajoutiez une feature majeure, chaque contribution fait avancer le projet. KoproGo est un projet communautaire : **vous êtes la communauté**.

   Merci de contribuer à un projet qui a un impact sociétal réel : 70M€/an économisés, 840 tonnes CO₂/an évitées, 40-60 copropriétaires aidés financièrement chaque année.

   **Ensemble, nous changeons la donne pour les copropriétés belges.**

----

*Guide du Parcours Contributeur KoproGo*

*Dernière mise à jour : 2025-01-19*

*Contact : contact@koprogo.com - GitHub : github.com/gilmry/koprogo*
