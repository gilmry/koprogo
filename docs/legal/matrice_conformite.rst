Matrice de Conformite Code-Loi
================================

.. contents:: Table des matieres
   :local:
   :depth: 2

Introduction
------------

Cette matrice lie chaque exigence legale belge au code qui l'implemente,
au scenario de test BDD qui le prouve, et au statut de conformite.

**Fichier de tracabilite BDD** : ``backend/tests/features/legal_compliance.feature``

Ce fichier Gherkin centralise toutes les exigences legales sous forme de scenarios
executables. Chaque ligne de cette matrice correspond a un scenario dans ce fichier.

Droit de la copropriete (Art. 3.84-3.94 CC)
---------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 20 10 20 20 15 15

   * - Exigence legale
     - Article
     - Fichier code
     - Scenario BDD
     - Tracabilite
     - Statut
   * - Quotes-parts = 100%
     - Art. 3.84
     - ``unit_owner.rs``, migration ``20251120230000``
     - ``multitenancy.feature``
     - ``legal_compliance.feature`` @conforme
     - CONFORME
   * - Convocation 15j tous types
     - Art. 3.87 §3
     - ``convocation.rs:23-30``
     - ``convocations.feature:19-32``
     - ``legal_compliance.feature`` @conforme @corrige
     - CONFORME
   * - Convocation contenu (agenda)
     - Art. 3.87 §2
     - ``meeting.rs`` (champ agenda)
     - ``meetings.feature``
     - ``legal_compliance.feature`` @conforme
     - CONFORME
   * - Lien agenda-resolutions
     - Art. 3.87 §2
     - —
     - —
     - ``legal_compliance.feature`` @manquant
     - MANQUANT
   * - Quorum 50%+50%
     - Art. 3.87 §5
     - —
     - —
     - ``legal_compliance.feature`` @manquant
     - MANQUANT
   * - Quorum 3/4 (decisions 3/4)
     - Art. 3.87 §5
     - —
     - —
     - ``legal_compliance.feature`` @manquant
     - MANQUANT
   * - 2eme convocation si quorum KO
     - Art. 3.87 §5
     - —
     - —
     - ``legal_compliance.feature`` @manquant
     - MANQUANT
   * - Procuration mandataire
     - Art. 3.87 §4
     - ``vote.rs`` (proxy_owner_id)
     - ``resolutions.feature``
     - ``legal_compliance.feature`` @conforme
     - CONFORME
   * - Procurations max 3
     - Art. 3.87 §7
     - —
     - —
     - ``legal_compliance.feature`` @manquant
     - MANQUANT
   * - Procurations exception 10%
     - Art. 3.87 §7
     - —
     - —
     - ``legal_compliance.feature`` @manquant
     - MANQUANT
   * - Majorite simple (50%+1 exprimes)
     - Art. 3.87 §8
     - ``resolution.rs`` (Simple)
     - ``resolutions.feature``
     - ``legal_compliance.feature`` @conforme
     - CONFORME
   * - Majorite absolue (50%+1 tous)
     - Art. 3.87 §8
     - ``resolution.rs`` (Absolute)
     - ``resolutions.feature``
     - ``legal_compliance.feature`` @conforme
     - CONFORME
   * - Majorite 2/3 (travaux extra.)
     - Art. 3.88 al.1
     - ``resolution.rs`` (Qualified 0.667)
     - ``resolutions.feature``
     - ``legal_compliance.feature`` @conforme
     - CONFORME
   * - Majorite 3/4 (jouissance)
     - Art. 3.88 al.2
     - ``resolution.rs`` (Qualified 0.75)
     - —
     - ``legal_compliance.feature`` @conforme
     - CONFORME
   * - Majorite 4/5 (statuts)
     - Art. 3.88 al.3
     - ``resolution.rs`` (Qualified 0.80)
     - —
     - ``legal_compliance.feature`` @conforme
     - CONFORME
   * - Unanimite (quotes-parts)
     - Art. 3.88 al.4
     - ``resolution.rs`` (Qualified 1.0)
     - —
     - ``legal_compliance.feature`` @conforme
     - CONFORME
   * - PV distribution 30j
     - Art. 3.87 §10
     - —
     - —
     - ``legal_compliance.feature`` @manquant
     - MANQUANT
   * - Syndic mandat max 3 ans
     - Art. 3.89
     - —
     - —
     - ``legal_compliance.feature`` @manquant
     - MANQUANT
   * - Info publique syndic
     - Art. 3.89
     - ``building.rs`` (7 champs syndic)
     - ``public_syndic.feature``
     - ``legal_compliance.feature`` @conforme
     - CONFORME
   * - Conseil copropriete >=20 lots
     - Art. 3.90
     - ``board_member.rs``
     - ``board_decisions.feature``
     - ``legal_compliance.feature`` @conforme
     - CONFORME (partiel)
   * - Etat date delai 15j
     - Art. 3.94
     - ``etat_date.rs:286-297``
     - ``etat_date.feature``
     - ``legal_compliance.feature`` @conforme @corrige
     - CONFORME (corrige)
   * - Etat date contenu 16 sections
     - Art. 3.94
     - ``etat_date.rs`` (champs + additional_data)
     - ``etat_date.feature``
     - ``legal_compliance.feature`` @conforme
     - CONFORME

Comptabilite (AR 12/07/2012)
------------------------------

.. list-table::
   :header-rows: 1
   :widths: 20 10 20 20 15 15

   * - Exigence
     - Article AR
     - Fichier code
     - Scenario BDD
     - Tracabilite
     - Statut
   * - Plan comptable PCMN
     - Art. 3
     - ``account.rs``, ``account_use_cases.rs``
     - —
     - ``legal_compliance.feature`` @conforme
     - CONFORME
   * - Comptabilite partie double
     - Art. 2
     - ``journal_entry.rs``
     - ``journal_entries.feature``
     - ``legal_compliance.feature`` @conforme
     - CONFORME
   * - Pieces justificatives
     - Art. 4
     - ``document.rs``, ``journal_entry.rs`` (document_ref)
     - ``documents.feature``
     - ``legal_compliance.feature`` @conforme
     - CONFORME
   * - Conservation 7 ans
     - Art. 5
     - Backups S3 chiffres
     - —
     - ``legal_compliance.feature`` @partiel
     - PARTIEL
   * - Bilan annuel
     - Art. 6
     - ``financial_report_use_cases.rs``
     - —
     - ``legal_compliance.feature`` @conforme
     - CONFORME
   * - Compte de resultats
     - Art. 6
     - ``financial_report_use_cases.rs``
     - —
     - ``legal_compliance.feature`` @conforme
     - CONFORME
   * - TVA belge (6/12/21%)
     - Legislation TVA
     - ``invoice_line_item.rs``, ``quote.rs``
     - ``invoices.feature``
     - ``legal_compliance.feature`` @conforme
     - CONFORME

RGPD
------

.. list-table::
   :header-rows: 1
   :widths: 20 10 20 20 15 15

   * - Exigence
     - Article
     - Fichier code
     - Scenario BDD
     - Tracabilite
     - Statut
   * - Droit d'acces
     - Art. 15
     - ``gdpr_use_cases.rs``
     - ``gdpr.feature``
     - ``legal_compliance.feature`` @conforme
     - CONFORME
   * - Droit de rectification
     - Art. 16
     - ``gdpr_handlers.rs`` (PUT /gdpr/rectify)
     - ``gdpr.feature``
     - ``legal_compliance.feature`` @conforme
     - CONFORME
   * - Droit a l'effacement
     - Art. 17
     - ``gdpr_use_cases.rs``
     - ``gdpr.feature``
     - ``legal_compliance.feature`` @conforme
     - CONFORME
   * - Droit a la limitation
     - Art. 18
     - ``gdpr_handlers.rs``
     - ``gdpr.feature``
     - ``legal_compliance.feature`` @conforme
     - CONFORME
   * - Droit d'opposition
     - Art. 21
     - ``gdpr_handlers.rs``
     - ``gdpr.feature``
     - ``legal_compliance.feature`` @conforme
     - CONFORME
   * - Registre des traitements
     - Art. 30
     - ``audit.rs``, table ``audit_logs``
     - —
     - ``legal_compliance.feature`` @conforme
     - CONFORME
   * - Information personnes
     - Art. 13-14
     - —
     - —
     - ``legal_compliance.feature`` @manquant
     - MANQUANT
   * - DPA sous-traitants
     - Art. 28
     - —
     - —
     - ``legal_compliance.feature`` @manquant
     - MANQUANT
   * - Notification violation
     - Art. 33
     - —
     - —
     - ``legal_compliance.feature`` @manquant
     - MANQUANT
   * - Securite du traitement
     - Art. 32
     - LUKS, bcrypt, HSTS
     - —
     - ``legal_compliance.feature`` @partiel
     - PARTIEL

Taux d'interet legal
----------------------

.. list-table::
   :header-rows: 1
   :widths: 20 10 20 20 15 15

   * - Exigence
     - Source
     - Fichier code
     - Scenario BDD
     - Tracabilite
     - Statut
   * - Taux legal civil 4.5% (2026)
     - Moniteur belge
     - ``payment_reminder.rs:90``
     - ``payment_recovery.feature``
     - ``legal_compliance.feature`` @conforme @corrige
     - CONFORME (corrige)

Recapitulatif
--------------

.. list-table::
   :header-rows: 1
   :widths: 25 15 15 15 30

   * - Domaine
     - Conforme
     - Manquant
     - Partiel
     - Priorite
   * - Copropriete (AG)
     - 12/19
     - 7/19
     - 0
     - Phase 1 critique
   * - Comptabilite (PCMN)
     - 6/7
     - 0
     - 1/7
     - Phase 2
   * - RGPD
     - 6/10
     - 3/10
     - 1/10
     - Phases 1-2
   * - Taux d'interet
     - 1/1
     - 0
     - 0
     - Corrige
   * - **Total**
     - **25/37 (67%)**
     - **10/37**
     - **2/37**
     - —

Comment utiliser ce suivi
--------------------------

Le fichier ``backend/tests/features/legal_compliance.feature`` est le point central.
Pour voir l'etat de conformite en un coup d'oeil :

.. code-block:: bash

   # Compter les exigences par statut
   grep -c "@conforme" backend/tests/features/legal_compliance.feature
   grep -c "@manquant" backend/tests/features/legal_compliance.feature
   grep -c "@partiel" backend/tests/features/legal_compliance.feature

   # Lister les exigences critiques manquantes
   grep -A1 "@manquant.*@critique" backend/tests/features/legal_compliance.feature

   # Lister les corrections effectuees
   grep -A1 "@corrige" backend/tests/features/legal_compliance.feature
