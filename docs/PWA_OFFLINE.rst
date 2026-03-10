=====================================================
Progressive Web App (PWA) avec Mode Hors Ligne
=====================================================

.. contents:: Table des matieres
   :depth: 3

Vue d'Ensemble
==============

KoproGo est une Progressive Web App (PWA) installable sur mobile et desktop,
avec support du mode hors ligne via Service Worker et IndexedDB.

Fonctionnalites PWA
====================

+----------------------------------+-------------------------------------------+
| Fonctionnalite                   | Implementation                            |
+==================================+===========================================+
| Installable                      | Web App Manifest + Install Prompt         |
+----------------------------------+-------------------------------------------+
| Mode hors ligne                  | Service Worker + Cache API                |
+----------------------------------+-------------------------------------------+
| Stockage local                   | IndexedDB (buildings, owners, expenses)   |
+----------------------------------+-------------------------------------------+
| Synchronisation                  | Background Sync + Sync Queue              |
+----------------------------------+-------------------------------------------+
| Notifications push               | Push API + VAPID                          |
+----------------------------------+-------------------------------------------+
| Mise a jour automatique          | SW update detection + UI notification     |
+----------------------------------+-------------------------------------------+
| Indicateur connectivite          | Online/Offline status en temps reel       |
+----------------------------------+-------------------------------------------+

Architecture
============

::

    Layout.astro
      |-- <link rel="manifest"> (manifest.json)
      |-- initializePWA() (pwa.ts)
      |      |-- registerServiceWorker()
      |      |-- setupInstallPrompt()
      |      |-- setupConnectivityListeners()
      |-- <PWAInstallPrompt /> (install banner)
      |-- <SyncStatus /> (online/offline indicator)

    Service Worker (service-worker.js)
      |-- Install: precache static assets
      |-- Activate: cleanup old caches
      |-- Fetch: routing strategies
      |      |-- API: Network First + cache fallback
      |      |-- Static: Cache First + network fallback
      |      |-- HTML: Network First + offline.html fallback
      |-- Push: notification display
      |-- Sync: background data sync

    IndexedDB (db.ts)
      |-- buildings, owners, units, expenses, users
      |-- sync_queue (pending offline mutations)

    SyncService (sync.ts)
      |-- Online: fetch fresh data from API
      |-- Offline: serve from IndexedDB
      |-- Reconnect: replay sync_queue mutations

Web App Manifest
================

Fichier : ``frontend/public/manifest.json``

- **Nom** : KoproGo - Gestion de Copropriete
- **Display** : standalone (plein ecran sans barre navigateur)
- **Theme** : #10b981 (vert emeraude)
- **Orientation** : portrait-primary
- **Langue** : fr-BE
- **Icones** : 8 tailles (72x72 a 512x512), maskable
- **Shortcuts** : Dashboard, Immeubles, Documents
- **Screenshots** : Dashboard (wide), Buildings (narrow)

Meta tags PWA dans ``Layout.astro`` :

- ``theme-color``
- ``mobile-web-app-capable``
- ``apple-mobile-web-app-capable``
- ``apple-mobile-web-app-status-bar-style``
- Apple Touch Icons (152x152, 192x192)

Service Worker
==============

Fichier : ``frontend/public/service-worker.js``

Strategies de Cache
-------------------

**Network First** (requetes API) :

1. Tenter la requete reseau
2. Si succes : mettre en cache et retourner la reponse
3. Si echec : retourner la version en cache
4. Header ``X-From-Cache: true`` ajoute sur les reponses du cache

**Cache First** (assets statiques) :

1. Chercher dans le cache
2. Si trouve : retourner immediatement
3. Si absent : fetcher, mettre en cache, retourner

Assets pre-caches a l'installation :

- ``/``, ``/dashboard``, ``/buildings``, ``/documents``
- ``/offline.html``
- Icones PWA (192x192, 512x512)

Endpoints API mis en cache :

- ``/api/v1/buildings``
- ``/api/v1/units``
- ``/api/v1/owners``
- ``/api/v1/documents``

Page Hors Ligne
---------------

Fichier : ``frontend/public/offline.html``

Affichee quand une page HTML est demandee sans connexion reseau et sans cache.
Inclut :

- Indicateur de statut en temps reel (en ligne / hors ligne)
- Bouton de retry
- Liste des fonctionnalites disponibles hors ligne
- Auto-redirect au retour en ligne (apres 1s)

Notifications Push
------------------

- Reception et affichage via Service Worker
- Icone et badge KoproGo
- Vibration pattern [200, 100, 200]
- Click: ouverture de l'URL associee
- Configuration VAPID via ``PUBLIC_VAPID_KEY``

Background Sync
---------------

- Tag ``sync-data`` pour relancer les mutations en attente
- IndexedDB ``pendingRequests`` comme file d'attente
- Replay automatique au retour en ligne

IndexedDB
=========

Fichier : ``frontend/src/lib/db.ts``

Base de donnees locale ``koprogo_db`` avec les object stores :

+----------------+------------------------------------------+
| Store          | Contenu                                  |
+================+==========================================+
| users          | Profil utilisateur connecte              |
+----------------+------------------------------------------+
| buildings      | Liste des immeubles                      |
+----------------+------------------------------------------+
| owners         | Liste des coproprietaires                |
+----------------+------------------------------------------+
| units          | Liste des lots                           |
+----------------+------------------------------------------+
| expenses       | Liste des charges                        |
+----------------+------------------------------------------+
| sync_queue     | Mutations en attente de synchronisation  |
+----------------+------------------------------------------+

Operations disponibles :

- ``get(store, id)`` : Lire un enregistrement
- ``getAll(store)`` : Lister tous les enregistrements
- ``put(store, data)`` : Creer ou mettre a jour
- ``delete(store, id)`` : Supprimer
- ``clear(store)`` : Vider le store
- ``addToSyncQueue(action, entity, data)`` : Ajouter a la file de sync
- ``markSynced(id)`` / ``clearSyncedItems()`` : Gerer la file de sync

Synchronisation
===============

Fichier : ``frontend/src/lib/sync.ts``

Le ``SyncService`` (singleton) gere la synchronisation bidirectionnelle :

**Mode en ligne** :

1. Verifier la file ``sync_queue`` pour les mutations en attente
2. Rejouer chaque mutation (POST/PUT/DELETE) vers l'API
3. Marquer comme synchronisees
4. Fetcher les donnees fraiches du serveur
5. Stocker dans IndexedDB

**Mode hors ligne** :

1. Les lectures sont servies depuis IndexedDB
2. Les ecritures sont mises en file dans ``sync_queue``
3. Un enregistrement temporaire (``temp-xxx``) est cree localement

**Transition offline -> online** :

1. Evenement ``online`` detecte par le navigateur
2. ``SyncService.sync()`` est appele automatiquement
3. File de sync rejouee, puis donnees rafraichies

Composants UI
=============

PWAInstallPrompt
----------------

Fichier : ``frontend/src/components/PWAInstallPrompt.svelte``

Banniere d'installation en bas d'ecran (mobile-friendly) :

- Apparait quand le navigateur emet ``beforeinstallprompt``
- Bouton "Installer" : declenche le prompt natif
- Bouton "Plus tard" : dismiss avec memorisation 7j (localStorage)
- Auto-masquage si deja installe (``display-mode: standalone``)

Notification de mise a jour en haut d'ecran (bleu) :

- Apparait quand un nouveau Service Worker est detecte
- Bouton "Actualiser" : recharge la page
- Bouton "Plus tard" : dismiss temporaire

SyncStatus
----------

Fichier : ``frontend/src/components/SyncStatus.svelte``

Affiche dans le footer :

- Indicateur vert (pulse) / rouge : en ligne / hors ligne
- Bouton "Sync" : synchronisation manuelle
- Spinner pendant la synchronisation

Donnees Disponibles Hors Ligne
==============================

+----------------------------------+--------+--------+
| Donnee                           | Lire   | Ecrire |
+==================================+========+========+
| Immeubles (liste)                | Oui    | Oui*   |
+----------------------------------+--------+--------+
| Coproprietaires                  | Oui    | Non    |
+----------------------------------+--------+--------+
| Charges                          | Oui    | Non    |
+----------------------------------+--------+--------+
| Documents recents                | Oui    | Non    |
+----------------------------------+--------+--------+
| Notifications                    | Oui    | Non    |
+----------------------------------+--------+--------+
| Profil utilisateur               | Oui    | Non    |
+----------------------------------+--------+--------+

``*`` Les creations hors ligne sont mises en file et synchronisees au retour en ligne.

Fichiers
========

- Manifest : ``frontend/public/manifest.json``
- Service Worker : ``frontend/public/service-worker.js``
- Page offline : ``frontend/public/offline.html``
- Icones : ``frontend/public/icons/`` (72 a 512px)
- PWA lib : ``frontend/src/lib/pwa.ts``
- IndexedDB : ``frontend/src/lib/db.ts``
- Sync service : ``frontend/src/lib/sync.ts``
- Install prompt : ``frontend/src/components/PWAInstallPrompt.svelte``
- Sync status : ``frontend/src/components/SyncStatus.svelte``
- Layout : ``frontend/src/layouts/Layout.astro`` (PWA init + meta tags)
