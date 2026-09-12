========================================================================
RFC 0003: Ranger la documentation éparpillée
========================================================================

:RFC: 0003
:Auteur: Claude Opus 5 (rédaction) / gilmry <gilmry@gmail.com> (commande)
:Date: 2026-09-10
:Statut: Draft
:Type: Gouvernance
:Équipes: docs, gouvernance
:Jalon: 0 (avant la release v0.1.0)

.. contents:: Table des matières
   :depth: 2
   :local:

Résumé (TL;DR)
==============

**Vingt-neuf fichiers markdown vivent à la racine de** ``docs/`` **et le site
Sphinx n'en référence aucun.** Huit n'ont zéro référence entrante dans tout le
dépôt. Six datent du 11 mars 2026 et n'ont pas bougé depuis.

Ce RFC propose une disposition pour chacun : ce qui est **tranché** devient une
ADR ou rejoint ``docs/architecture/``, ce qui est **encore ouvert** devient un
RFC ou une issue, ce qui est **périmé** part en archive, et ce qui est un
**guide vivant** rejoint la section qui le concerne avec une entrée dans le
``toctree``.

Le motif
========

La consigne qui a déclenché ce RFC
-----------------------------------

  « Quand tu me poses une question : mets-le dans le registre des RFC ou dans
  une issue, ou les deux, pas dans des markdown éparpillés. »

Elle m'a été donnée après que j'ai regroupé sept arbitrages dans un fichier
``docs/ARBITRAGES_EN_ATTENTE.md`` que j'avais inventé. Ce fichier avait déjà,
en deux jours d'existence, les trois défauts du genre :

- son en-tête annonçait « six décisions » alors qu'il en portait sept ;
- son contenu sur l'Art. 3.89 § 1er était en retard sur le code, qui
  implémentait déjà le plafond de trois ans ;
- rien ne notifiait sa mise à jour, et rien ne signalait sa péremption.

Il a été supprimé, son contenu réparti en ADR 0046, 0047 et 0048.

Ce que la mesure montre
------------------------

Aucun des vingt-neuf ``.md`` de la racine n'apparaît dans ``index.rst``, qui ne
liste que des ``.rst``. Ils ne sont donc pas publiés, pas navigables, et pas
soumis à la revue que le site impose.

Références entrantes, tout le dépôt confondu :

.. list-table::
   :header-rows: 1
   :widths: 10 55 20

   * - Réfs
     - Fichier
     - Dernière écriture
   * - 0
     - ``MODELE_ACP_CENTRE.md``
     - 2026-09-02
   * - 0
     - ``AUDIT_COUVERTURE_2026-09-02.md``
     - 2026-09-02
   * - 0
     - ``ACCESSIBILITY.md``
     - 2026-03-11
   * - 0
     - ``BOARD_OF_DIRECTORS_GUIDE.md``
     - 2026-03-11
   * - 0
     - ``EMAIL_TEMPLATES.md``
     - 2026-03-11
   * - 0
     - ``FRONTEND_COMPONENTS.md``
     - 2026-03-11
   * - 0
     - ``INTEGRATION_GUIDES.md``
     - 2026-03-11
   * - 0
     - ``PRODUCTION_MIGRATION_RECOVERY.md``
     - 2026-03-11

Un document sans référence entrante et sans entrée au ``toctree`` n'est pas
« de la documentation » : c'est une note privée dans un dépôt public.

Le second symptôme : les dossiers en double
--------------------------------------------

``docs/`` compte trente-six sous-dossiers, dont trois paires qui se recouvrent :
``archive/`` et ``archives/``, ``ci/`` et ``ci-cd/``, ``ops/`` et
``operations/``. Une paire de ce genre garantit que la moitié du contenu sera
cherchée au mauvais endroit.

Disposition proposée
====================

Règle de tri
------------

.. list-table::
   :header-rows: 1
   :widths: 25 75

   * - Nature
     - Destination
   * - Décision d'architecture **tranchée**
     - ADR numérotée dans ``docs/adr/``
   * - Analyse d'architecture tranchée, non décisionnelle
     - ``docs/architecture/`` avec entrée au ``toctree``
   * - Question **encore ouverte**
     - RFC numéroté, ou issue GitHub, ou les deux
   * - Constat daté à traiter
     - Issue GitHub, jamais un fichier
   * - Guide vivant
     - Sous-dossier thématique + entrée au ``toctree``
   * - Périmé
     - ``docs/archive/`` avec la date de péremption en tête

Cas par cas
-----------

**Tranché → architecture ou ADR**

``MODELE_ACP_CENTRE.md``
  Analyse d'écart du 2026-09-02 qui a produit l'ADR-0045 et, indirectement,
  l'ADR-0046. Sa partie « modèle métier » est de l'architecture tranchée :
  elle rejoint ``docs/architecture/``. Sa partie « chemin de résorption » est
  du travail restant, déjà suivi par #779 et voisines.

``JWT_REFRESH_TOKENS.md``
  585 lignes qui décrivent un choix de sécurité tranché et son implémentation.
  La décision mérite une ADR ; le mode d'emploi reste un guide.

**Ouvert → RFC ou issue**

``AUDIT_COUVERTURE_2026-09-02.md``
  Un audit est un relevé de constats à traiter. Chaque constat encore valide
  devient une issue ; le fichier part en archive avec un renvoi vers elles.

``AUDIT_CONFORMITE_JURIDIQUE.md``
  Daté du 11 mars, six mois avant le registre légal exécutable
  (``registre_legal.rs``) qui couvre trente obligations avec leurs tests. À
  confronter au registre : ce qu'il porte de plus devient des issues, le reste
  part en archive.

``rapport-tests-e2e-koprogo.md``
  Rapport de recette manuelle du 11 mai. Les recettes suivantes en ont produit
  cinq autres. Archive, avec renvoi vers les issues nées de chacune.

**Guides vivants → sous-dossier + toctree**

``ACCESSIBILITY.md``, ``I18N_GUIDE.md``, ``FRONTEND_COMPONENTS.md``,
``MIGRATION_SVELTE5_RUNES.md``, ``TESTING_SVELTE5.md``,
``TEST_COVERAGE_MATRIX.md`` → ``docs/frontend/``

``DATABASE_ADMIN.md``, ``PERFORMANCE_TUNING.md``, ``JWT_REFRESH_TOKENS.md``,
``EMAIL_TEMPLATES.md``, ``INTEGRATION_GUIDES.md`` → ``docs/backend/``

``K3S_GITOPS_DEPLOYMENT.md``, ``K3S_QUICKSTART.md``,
``PRODUCTION_MIGRATION_RECOVERY.md``, ``RUNBOOK_VPS_PRODUCTION.md`` →
``docs/deployment/`` (et fusionner ``ops/`` dans ``operations/``)

``GDPR_ADDITIONAL_RIGHTS.md``, ``GDPR_COMPLIANCE_CHECKLIST.md`` →
``docs/legal/``

``BOARD_OF_DIRECTORS_GUIDE.md``, ``MULTI_OWNER_SUPPORT.md``,
``MULTI_ROLE_SUPPORT.md`` → ``docs/user-guides/``

``AGENT_RECIPES.md``, ``RELEASE_PROCESS.md`` → ``docs/governance/``

**Restent à la racine**

``WBS_v0_1_0.md``
  Se déclare « seule vérité courante » et l'est. Il porte l'état du périmètre
  de release, pas une connaissance durable.

``HUMAN_REVIEW_PLAN_v0.1.0.md`` et ``HUMAN_REVIEW_REPORT_v0.1.0.md``
  Matériel du jalon G1, non délégable
  (``docs/governance/RESPONSABILITE.md``). Dix références entrantes pour le
  second : c'est un document vivant du processus de release.

Ce que ce RFC ne tranche pas
=============================

**L'ordre d'exécution.** Déplacer vingt-cinq fichiers casse toute référence
relative non mise à jour. La mesure ci-dessus donne le nombre de références
entrantes par fichier ; les huit à zéro peuvent bouger sans risque, les
autres demandent une passe de réécriture des liens.

**Le sort des trois paires de dossiers en double.** Fusionner ``archive/`` et
``archives/`` est mécanique ; décider laquelle survit ne l'est pas, et les deux
contiennent du contenu.

**S'il faut convertir les guides en reStructuredText.** Le site est en ``.rst``
et les guides sont en ``.md``. Sphinx lit les deux via MyST si on l'active.
Convertir vingt-cinq fichiers a un coût ; les laisser en markdown hors du
``toctree`` a le coût qu'on mesure aujourd'hui.

Décision attendue
==================

Trois questions, à trancher ensemble ou séparément :

1. La règle de tri ci-dessus est-elle la bonne ?
2. Faut-il activer MyST pour publier les guides markdown, ou les convertir ?
3. Les trois paires de dossiers en double : laquelle survit dans chaque paire ?
