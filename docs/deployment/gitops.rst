
GitOps Auto-Update
==================

KoproGo se met à jour automatiquement depuis GitHub grâce à un service systemd.

----

🔄 Fonctionnement
-----------------

Vue d'ensemble
^^^^^^^^^^^^^^

Le système GitOps vérifie toutes les **3 minutes** s'il y a de nouveaux commits sur la branche ``main``. Si oui, il met à jour automatiquement l'application.

.. code-block::

   ┌─────────────────────────────────────────────────┐
   │         Service systemd (continu)              │
   │                                                 │
   │  ┌──────────────────────────────────────────┐ │
   │  │  Toutes les 3 minutes:                   │ │
   │  │                                           │ │
   │  │  1. git fetch origin                     │ │
   │  │  2. Compare HEAD local vs remote         │ │
   │  │  3. Si différent → Update                │ │
   │  │  4. Pull images Docker                   │ │
   │  │  5. docker compose up -d --pull always   │ │
   │  │  6. Health check HTTPS                   │ │
   │  │  7. Fix permissions .git/                │ │
   │  │  8. Rollback si health check échoue      │ │
   │  └──────────────────────────────────────────┘ │
   └─────────────────────────────────────────────────┘

----

📝 Composants GitOps
--------------------

1. Service systemd
^^^^^^^^^^^^^^^^^^

Fichier : ``/etc/systemd/system/koprogo-gitops.service``

.. code-block:: ini

   [Unit]
   Description=KoproGo GitOps Auto-Update
   After=network.target docker.service

   [Service]
   Type=simple
   User=root
   WorkingDirectory=/home/koprogo/koprogo
   ExecStart=/home/koprogo/koprogo/deploy/production/gitops-deploy.sh monitor
   Restart=always
   RestartSec=180

   [Install]
   WantedBy=multi-user.target

**Caractéristiques** :


* **Type** : ``simple`` (processus continu)
* **User** : ``root`` (nécessaire pour docker et chown)
* **Restart** : Automatique si crash
* **RestartSec** : 180 secondes (3 minutes entre chaque vérification)

2. Script GitOps
^^^^^^^^^^^^^^^^

Fichier : ``/home/koprogo/koprogo/deploy/production/gitops-deploy.sh``

**Modes d'exécution** :

.. code-block:: bash

   # Mode monitor (utilisé par systemd)
   ./gitops-deploy.sh monitor

   # Mode deploy manuel (force une mise à jour immédiate)
   ./gitops-deploy.sh deploy

3. Workflow de Mise à Jour
^^^^^^^^^^^^^^^^^^^^^^^^^^

Étape 1 : Détection de changements
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   cd /home/koprogo/koprogo
   git fetch origin main

   LOCAL=$(git rev-parse HEAD)
   REMOTE=$(git rev-parse origin/main)

   if [ "$LOCAL" != "$REMOTE" ]; then
       echo "📦 Nouveau commit détecté: $REMOTE"
       # Continuer vers update
   fi

Étape 2 : Pull du code
~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   git pull origin main

   if [ $? -ne 0 ]; then
       echo "❌ Erreur git pull"
       exit 1
   fi

Étape 3 : Pull des images Docker
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   cd deploy/production
   docker compose pull

   # Pull depuis GitHub Container Registry:
   # - ghcr.io/gilmry/koprogo-backend:latest
   # - ghcr.io/gilmry/koprogo-frontend:latest

Étape 4 : Rebuild et redémarrage
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   docker compose up -d --pull always

   # Flags:
   # -d : Détaché (background)
   # --pull always : Force pull des images avant up

Étape 5 : Health Check
~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   HEALTH_URL="https://api.${DOMAIN}/api/v1/health"

   for i in {1..10}; do
       RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" -k "$HEALTH_URL")

       if [ "$RESPONSE" = "200" ]; then
           echo "✅ Health check réussi"
           break
       fi

       sleep 10
   done

   if [ "$RESPONSE" != "200" ]; then
       echo "❌ Health check échoué après 10 tentatives"
       # Rollback
       git reset --hard HEAD^
       docker compose up -d --force-recreate
       exit 1
   fi

**Pourquoi ``-k`` (insecure SSL) ?**


* Pendant les premières secondes après déploiement, le certificat Let's Encrypt peut ne pas être encore généré
* Le health check valide quand même que l'API répond
* Le certificat sera généré par Traefik dans les minutes suivantes

Étape 6 : Fix permissions Git
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   chown -R koprogo:koprogo /home/koprogo/koprogo/.git

   # Évite les erreurs:
   # error: cannot open .git/FETCH_HEAD: Permission denied
   # error: cannot open .git/index: Permission denied

Étape 7 : Rollback automatique
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Si le health check échoue :

.. code-block:: bash

   echo "🔄 Rollback vers version précédente..."

   # Revenir au commit précédent
   git reset --hard HEAD^

   # Forcer recréation containers avec ancienne version
   docker compose up -d --force-recreate

   # Re-check health
   # Si toujours échec, alerter (TODO: notification Discord/Slack)

----

🎛️ Gestion du Service
---------------------

Commandes systemd
^^^^^^^^^^^^^^^^^

.. code-block:: bash

   # Status du service
   sudo systemctl status koprogo-gitops.service

   # Démarrer
   sudo systemctl start koprogo-gitops.service

   # Arrêter
   sudo systemctl stop koprogo-gitops.service

   # Redémarrer
   sudo systemctl restart koprogo-gitops.service

   # Activer au démarrage (déjà fait par Ansible)
   sudo systemctl enable koprogo-gitops.service

   # Désactiver au démarrage
   sudo systemctl disable koprogo-gitops.service

Voir les logs
^^^^^^^^^^^^^

.. code-block:: bash

   # Logs en temps réel
   sudo journalctl -u koprogo-gitops.service -f

   # Logs depuis aujourd'hui
   sudo journalctl -u koprogo-gitops.service --since today

   # Dernières 100 lignes
   sudo journalctl -u koprogo-gitops.service -n 100

   # Filtrer par mot-clé
   sudo journalctl -u koprogo-gitops.service | grep "Health check"
   sudo journalctl -u koprogo-gitops.service | grep "Nouveau commit"

----

🚀 Forcer une Mise à Jour Manuelle
----------------------------------

Si vous ne voulez pas attendre les 3 minutes :

.. code-block:: bash

   # Sur le VPS
   ssh ubuntu@VPS_IP

   # Exécuter deploy manuel
   sudo /home/koprogo/koprogo/deploy/production/gitops-deploy.sh deploy

----

🔒 Sécurité GitOps
------------------

Permissions fichiers
^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   # Script GitOps
   -rwxr-xr-x 1 koprogo koprogo  gitops-deploy.sh

   # Repository Git
   drwxr-xr-x 8 koprogo koprogo  .git/

   # Service systemd
   -rw-r--r-- 1 root root  koprogo-gitops.service

Accès GitHub
^^^^^^^^^^^^

Le service utilise HTTPS public :

.. code-block:: bash

   git remote -v
   # origin  https://github.com/gilmry/koprogo.git (fetch)
   # origin  https://github.com/gilmry/koprogo.git (push)

**Pas de clé SSH** requise pour ``git pull`` (repository public).

**Pour push** (dev uniquement) : Clé SSH configurée pour l'utilisateur ``koprogo``.

----

📊 Monitoring GitOps
--------------------

Vérifier dernière mise à jour
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   # Sur le VPS
   cd /home/koprogo/koprogo

   # Dernier commit local
   git log -1 --oneline

   # Dernier commit remote
   git fetch origin
   git log origin/main -1 --oneline

Vérifier fréquence de vérification
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   # Voir les derniers checks
   sudo journalctl -u koprogo-gitops.service --since "10 minutes ago" | grep "Checking for updates"

   # Devrait montrer des checks toutes les 3 minutes

Statistiques
^^^^^^^^^^^^

.. code-block:: bash

   # Nombre de mises à jour aujourd'hui
   sudo journalctl -u koprogo-gitops.service --since today | grep "Nouveau commit détecté" | wc -l

   # Uptime du service
   systemctl status koprogo-gitops.service | grep "Active:"

----

🔧 Configuration Avancée
------------------------

Changer la fréquence de vérification
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Par défaut : **3 minutes** (\ ``RestartSec=180``\ )

Pour modifier :

.. code-block:: bash

   # Éditer le service
   sudo nano /etc/systemd/system/koprogo-gitops.service

   # Changer RestartSec=180 vers autre valeur (en secondes)
   # Exemples:
   # RestartSec=60   → 1 minute
   # RestartSec=300  → 5 minutes
   # RestartSec=600  → 10 minutes

   # Recharger systemd
   sudo systemctl daemon-reload

   # Redémarrer service
   sudo systemctl restart koprogo-gitops.service

Ajouter notifications
^^^^^^^^^^^^^^^^^^^^^

Modifier le script ``gitops-deploy.sh`` pour envoyer notifications Discord/Slack :

.. code-block:: bash

   # Après health check réussi
   if [ "$RESPONSE" = "200" ]; then
       # Notifier Discord
       curl -X POST "$DISCORD_WEBHOOK_URL" \
           -H "Content-Type: application/json" \
           -d "{\"content\":\"✅ KoproGo mis à jour vers $NEW_COMMIT\"}"
   fi

   # En cas d'échec
   if [ "$RESPONSE" != "200" ]; then
       curl -X POST "$DISCORD_WEBHOOK_URL" \
           -H "Content-Type: application/json" \
           -d "{\"content\":\"❌ Échec mise à jour KoproGo, rollback effectué\"}"
   fi

Changer de branche
^^^^^^^^^^^^^^^^^^

Par défaut : branche ``main``

Pour suivre une autre branche (ex: ``staging``\ ) :

.. code-block:: bash

   # Éditer script
   sudo nano /home/koprogo/koprogo/deploy/production/gitops-deploy.sh

   # Remplacer toutes les occurrences de "main" par "staging"
   # Ligne: git fetch origin main → git fetch origin staging
   # Ligne: git pull origin main → git pull origin staging
   # etc.

   # Redémarrer service
   sudo systemctl restart koprogo-gitops.service

----

❌ Désactiver GitOps
--------------------

Si vous voulez gérer les mises à jour manuellement :

.. code-block:: bash

   # Arrêter le service
   sudo systemctl stop koprogo-gitops.service

   # Désactiver au démarrage
   sudo systemctl disable koprogo-gitops.service

   # Vérifier qu'il est bien arrêté
   sudo systemctl status koprogo-gitops.service
   # Devrait afficher: Active: inactive (dead)

Vous devrez ensuite mettre à jour manuellement :

.. code-block:: bash

   cd /home/koprogo/koprogo
   git pull origin main
   cd deploy/production
   docker compose pull
   docker compose up -d

----

🐛 Debugging GitOps
-------------------

Le service ne démarre pas
^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   # Vérifier les erreurs
   sudo systemctl status koprogo-gitops.service
   sudo journalctl -u koprogo-gitops.service -n 50

   # Vérifier que le script existe et est exécutable
   ls -la /home/koprogo/koprogo/deploy/production/gitops-deploy.sh

   # Tester le script manuellement
   sudo /home/koprogo/koprogo/deploy/production/gitops-deploy.sh monitor

Le service tourne mais ne met pas à jour
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   # Vérifier les logs pour erreurs
   sudo journalctl -u koprogo-gitops.service -f

   # Erreurs possibles:
   # - "Permission denied" sur .git/ → Fix: chown -R koprogo:koprogo .git/
   # - "docker compose: command not found" → Réinstaller Docker Compose
   # - "Health check échoué" → Vérifier manuellement: curl https://api.domain.com/api/v1/health

Les mises à jour sont trop fréquentes
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Si vous poussez beaucoup de commits en dev et que GitOps update trop souvent :

**Option 1** : Augmenter ``RestartSec`` (voir Configuration Avancée)

**Option 2** : Désactiver temporairement GitOps pendant le dev

**Option 3** : Utiliser une branche séparée (ex: ``production``\ ) et merger seulement quand prêt

----

📚 Ressources
-------------


* **Script GitOps** : ``deploy/production/gitops-deploy.sh``
* **Service systemd** : ``/etc/systemd/system/koprogo-gitops.service``
* **Logs** : ``journalctl -u koprogo-gitops.service``

----

🔗 Prochaine Étape
------------------

Problème de déploiement ? Consulter **\ `Troubleshooting <troubleshooting.md>`_\ **

----

**Dernière mise à jour** : Octobre 2025

**KoproGo ASBL** 🚀
