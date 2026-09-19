🛠️ Guide des Commandes Make
===========================

.. warning::

   **Ce guide n'est pas la liste des commandes. Le Makefile l'est.**

   Ce fichier a listé pendant des mois treize cibles qui n'existaient plus —
   ``make docker-up``, ``make test-e2e-ui``, ``make dev-all``… Un lecteur qui
   les tapait recevait ``No rule to make target``, et rien dans le dépôt ne
   s'en apercevait. Recopier le Makefile dans un ``.rst`` fabrique une
   seconde source de vérité qui dérive par construction.

   La liste complète et à jour est donc **toujours** :

   .. code-block:: bash

      make help

   Ce qui suit n'en est qu'une **entrée en matière** : les quelques commandes
   du quotidien, avec ce qu'il faut savoir avant de les taper. La garde
   ``garde-commandes-documentees`` refuse désormais toute citation d'une
   cible qui n'existe pas, dans ce fichier comme dans les autres.

----

🚀 Démarrer
-----------

.. code-block:: bash

   make setup   # première fois : dépendances, navigateurs, base, migrations
   make up      # la pile de développement en hot reload (alias de `make dev`)
   make ps      # ce qui tourne
   make down    # tout arrêter

``make up`` démarre Traefik, le backend et le frontend. Le backend tourne
sous ``cargo watch`` : une modification d'un fichier de ``backend/`` le
recompile et le redémarre.

.. note::

   Les documents envoyés ne doivent jamais atterrir dans l'arbre surveillé :
   ``UPLOAD_DIR`` pointe vers un volume dédié, faute de quoi chaque envoi
   redémarre le backend au milieu d'une campagne (#880).

----

🧪 Tester
---------

.. code-block:: bash

   make test-unit          # unitaires backend (domaine)
   make test-e2e-backend   # intégration backend (testcontainers)
   make test-bdd           # scénarios Cucumber
   make test-e2e           # Playwright, encadré du témoin d'interruption (#880)
   make test               # les trois premiers

``make test-e2e`` vise la **pile de recette**, pas la démo :

.. code-block:: bash

   make test-e2e                              # défaut : http://localhost:8090
   make test-e2e RECETTE=http://localhost:3000

.. danger::

   Ne jamais viser ``http://localhost`` nu. Le port 80 est le Traefik de la
   **démo** ; la recette est sur 8090 (ADR 0050). Les deux variables
   ``PLAYWRIGHT_BASE_URL`` et ``PLAYWRIGHT_API_BASE`` sont posées ensemble par
   la cible — les dissocier enverrait le navigateur d'un côté et l'amorçage du
   monde de l'autre.

.. note::

   Sans ``CI=1``, Playwright tourne en parallèle et la mesure **n'est pas
   comparable** à celle de la CI. Pour un chiffre qu'on inscrit quelque part :
   ``CI=1 make test-e2e``.

----

🎬 La vitrine — la preuve de valeur
-----------------------------------

.. code-block:: bash

   make vitrine

Harnais **séparé** du gate : il rejoue le parcours de référence en cadence
(``CADENCE_MS``), incruste la narration à l'écran, et assemble une galerie
autonome à chapitres horodatés dans
``frontend/tests/e2e/journeys/vitrine/index.html``.

Il ne touche **aucun** fichier du gate, et c'est tout le sujet de #876 :
ralentir les tests en les modifiant confiait au gate une responsabilité qui
n'est pas la sienne.

----

🗃️ Base de données
------------------

.. code-block:: bash

   make migrate      # appliquer les migrations
   make seed         # données de test
   make seed-clear   # vider le monde de scénario
   make seed-reset   # le vider puis le recréer (échoue si l'API refuse)
   make reset-db     # ⚠️ SUPPRIME TOUTES LES DONNÉES

----

🔍 Avant de pousser
-------------------

.. code-block:: bash

   make format
   make lint
   make ci            # les vérifications CI, en conteneurs

Le barrage de déploiement lance en plus **les gardes du dépôt** et le gate
OpenAPI, qui ne sont pas des cibles ``make`` :

.. code-block:: bash

   cd backend && ~/bin/kcargo test --no-fail-fast --test architecture --test 'garde_*'
   ./scripts/check-openapi-coverage.sh

Ces gardes portent les **cliquets de dette** : elles n'exigent pas la
perfection, elles refusent l'aggravation. Une seule qui rougit suffit à ce
que rien ne soit construit ni déployé.

----

📚 Documentation
----------------

.. code-block:: bash

   make docs-sphinx        # build Sphinx
   make docs-serve         # avec live reload
   make docs-with-videos   # vitrine + gate + galerie + Sphinx
   make docs-guard         # refuse un markdown non listé à la racine de docs/ (#854)
   make rfc-new TITLE="mon-titre"
   make adr-new TITLE="mon-titre"

.. note::

   Une question se pose en **RFC** ou en **issue**, jamais dans un markdown
   créé pour l'occasion.

----

🔐 Sécurité et garde-fous
-------------------------

.. code-block:: bash

   make secret-scan            # gitleaks sur le diff et l'arbre de travail
   make secret-scan-history    # tout l'historique (lent)
   make audit                  # cargo-audit + npm audit
   make claude-check           # config des garde-fous Claude Code
   make test-guardrail-hooks   # les hooks bloquent-ils vraiment ? (#429)
   make iac-lint

----

Et pour tout le reste
---------------------

.. code-block:: bash

   make help
