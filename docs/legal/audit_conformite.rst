Audit de Conformite Juridique — KoproGo v0.1.0
==================================================

.. contents:: Table des matieres
   :local:
   :depth: 2

Informations generales
-----------------------

:Date: 2026-02-28
:Portee: Droit belge de la copropriete (Art. 3.84-3.94 CC), RGPD, Comptabilite belge (PCMN), Securite technique
:Score global: **65% — NON PRET POUR PRODUCTION sans corrections Phase 1**

.. note::

   Cet audit a ete realise en comparant le code source de KoproGo avec les textes
   de loi officiels belges. Trois erreurs factuelles ont ete corrigees dans cette release :

   1. Delai convocations : 8 jours (Extraordinary) → **15 jours pour tous les types** (Art. 3.87 §3)
   2. Taux d'interet de retard : 8% → **4.5%** (taux legal civil 2026, Moniteur belge)
   3. Delai etat date : 10 jours → **15 jours** (Art. 3.94)
   4. 3 devis obligatoires : "exigence legale" → **bonne pratique professionnelle** (aucun article de loi)

Score de conformite par domaine
---------------------------------

.. list-table::
   :header-rows: 1
   :widths: 35 15 50

   * - Domaine
     - Score
     - Commentaire
   * - Droit copropriete belge (Art. 3.84-3.94)
     - **70%**
     - Solide mais lacunes AG critiques (quorum, proxies)
   * - Assemblees generales specifiquement
     - **55%**
     - Convocations OK, quorum/proxy/PV incomplets
   * - RGPD
     - **65%**
     - Articles 15-21 OK, documentation/cookies absents
   * - Comptabilite belge (PCMN)
     - **95%**
     - Quasi complet
   * - Securite technique
     - **90%**
     - Excellence infrastructure, gaps mineurs
   * - Documentation legale
     - **15%**
     - Documentation technique OK, legale absente
   * - **GLOBAL**
     - **65%**
     - **NON PRET sans corrections Phase 1**

Conformite — Droit de la copropriete
--------------------------------------

Convocations — CONFORME
~~~~~~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 40 15 45

   * - Exigence
     - Statut
     - Implementation
   * - Delai 15j tous types d'AG
     - OK
     - ``convocation.rs:23-30``
   * - Tracking email (envoi/ouverture)
     - OK
     - ``convocation_recipient.rs``
   * - Rappels J-3 automatiques
     - OK
     - ``should_send_reminder()``
   * - Support procuration
     - OK
     - ``proxy_owner_id`` field
   * - Multi-langue FR/NL/DE/EN
     - OK
     - Validation dans ``Convocation::new()``

Systeme de vote — LACUNES CRITIQUES
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Implemente** :

- 3 types de majorite (Simple, Absolute, Qualified)
- Tantiemes/milliemes (0-1000) avec ``voting_power: f64``
- Vote par procuration (``proxy_owner_id``)
- Audit trail complet
- 27 tests unitaires

**Lacunes** :

.. list-table::
   :header-rows: 1
   :widths: 40 20 40

   * - Lacune
     - Risque
     - Ref. legale
   * - Pas de validation du quorum
     - CRITIQUE
     - Art. 3.87 §5
   * - Pas de workflow 2eme convocation
     - CRITIQUE
     - Art. 3.87 §5
   * - Pas de limite de procurations
     - CRITIQUE
     - Art. 3.87 §7 (max 3 mandats)
   * - Pas de lien agenda-resolutions
     - ELEVE
     - Art. 3.87 §2
   * - Pas de presets majorite/type decision
     - MOYEN
     - Art. 3.88
   * - Pas de fenetre temporelle de vote
     - MOYEN
     - Pratique courante
   * - Pas de snapshot tantiemes
     - MOYEN
     - Tracabilite

Etat date — CONFORME (corrige)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- Delai 15 jours (Art. 3.94) : CORRIGE (etait 10 jours)
- 16 sections legales : OK
- Validite 90 jours : pratique professionnelle

Devis entrepreneurs — CONFORME
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- Workflow 7 etats : OK
- Scoring automatique (prix 40%, delai 30%, garantie 20%, reputation 10%) : OK
- TVA belge (6%, 12%, 21%) : OK
- Garantie decennale : OK
- 3 devis pour >5000 EUR : bonne pratique professionnelle (PAS une obligation legale)

Conformite — RGPD
-------------------

**Implemente** (Articles 15, 16, 17, 18, 21, 30) : Voir :doc:`rgpd_conformite`

**Lacunes** :

- Politique de confidentialite : ABSENT (Art. 13-14)
- Consentement cookies : ABSENT (ePrivacy)
- Notification violation : ABSENT (Art. 33)
- DPA sous-traitants : ABSENT (Art. 28)
- Chiffrement colonnes sensibles : ABSENT (Art. 32)

Conformite — Comptabilite (PCMN)
-----------------------------------

**Score : 95%** — Voir :doc:`pcmn_ar_12_07_2012`

- ~90 comptes PCMN pre-seedes : OK
- Comptabilite partie double : OK
- TVA belge 3 taux : OK
- Rapports financiers (bilan, compte resultats) : OK
- Conservation 7 ans : PARTIEL (cron manquant)

Securite technique
-------------------

**Score : 90%**

**Points forts** :

- JWT + refresh tokens + 2FA TOTP
- Rate limiting login (5 tentatives/15min)
- Headers securite (HSTS, CSP, X-Frame-Options)
- Prevention injection SQL (sqlx parametree)
- LUKS AES-XTS-512 (chiffrement disque)
- Backups chiffres GPG RSA-4096
- Suricata IDS + CrowdSec WAF
- fail2ban (SSH, Traefik, API, PostgreSQL)

**Lacunes** :

- CSP ``unsafe-inline`` (requis Svelte) — FAIBLE
- Pas de scanning images Docker — MOYEN
- Pas de gestion secrets (Vault) — MOYEN
- Pas de pentest tiers — MOYEN

Plan d'action
--------------

Voir :doc:`risques_juridiques` pour le detail des risques et priorites.

Phase 1 — Critique (avant production)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

1. Validation du quorum (~3 jours)
2. Limitation procurations (~2 jours)
3. Workflow 2eme convocation (~3 jours)
4. Lien agenda-resolutions (~2 jours)
5. Documentation legale (~5 jours)

**Total : ~15 jours**

Phase 2 — Elevee (avant 100 utilisateurs)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

6. Distribution PV sous 30 jours (~3 jours)
7. Presets majorite par type de decision (~1 jour)
8. Procedure notification violation RGPD (~2 jours)
9. Nettoyage automatique logs d'audit (~1 jour)

**Total : ~7 jours**

Phase 3 — Moyenne (avant 500 utilisateurs)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

10. Snapshot tantiemes au debut de l'AG (~2 jours)
11. Fenetre temporelle de vote (~1 jour)
12. Consentement cookies frontend (~2 jours)
13. Test de penetration tiers (externe)

**Total : ~5 jours + interventions externes**

Verification
-------------

.. code-block:: bash

   # Tests unitaires domain (convocation, payment_reminder, etat_date)
   cargo test --lib

   # Tests BDD (5 fichiers par domaine, 752 scenarios)
   cargo test --test bdd --test bdd_governance --test bdd_financial --test bdd_operations --test bdd_community

   # Compilation sans warnings
   cargo clippy -- -D warnings

.. important::

   Les 752 scenarios BDD ont ete ecrits mais n'ont pas tous ete
   executes dans un environnement d'integration complet. Des correctifs seront
   necessaires lors des premieres executions CI.

   Une revue manuelle de la documentation legale par un juriste belge specialise
   en copropriete est **fortement recommandee** avant la mise en production.
