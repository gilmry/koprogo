Matrice de Conformite Code-Loi
================================

.. contents:: Table des matieres
   :local:
   :depth: 2

Introduction
------------

Cette matrice lie chaque exigence legale belge au code qui l'implemente,
au scenario de test BDD qui le prouve, et au statut de conformite.

Droit de la copropriete (Art. 3.84-3.94 CC)
---------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 25 15 25 20 15

   * - Exigence legale
     - Article
     - Fichier code
     - Scenario BDD
     - Statut
   * - Quotes-parts = 100%
     - Art. 3.84
     - ``unit_owner.rs``, migration ``20251120230000``
     - ``multitenancy.feature``
     - CONFORME
   * - Convocation 15j tous types
     - Art. 3.87 §3
     - ``convocation.rs:23-30``
     - ``convocations.feature:19-32``
     - CONFORME
   * - Convocation contenu (agenda)
     - Art. 3.87 §2
     - ``meeting.rs`` (champ agenda)
     - ``meetings.feature``
     - CONFORME
   * - Lien agenda-resolutions
     - Art. 3.87 §2
     - —
     - —
     - MANQUANT
   * - Quorum 50%+50%
     - Art. 3.87 §5
     - —
     - —
     - MANQUANT
   * - Quorum 3/4 (decisions 3/4)
     - Art. 3.87 §5
     - —
     - —
     - MANQUANT
   * - 2eme convocation si quorum KO
     - Art. 3.87 §5
     - —
     - —
     - MANQUANT
   * - Procuration mandataire
     - Art. 3.87 §4
     - ``vote.rs`` (proxy_owner_id)
     - ``resolutions.feature``
     - CONFORME
   * - Procurations max 3
     - Art. 3.87 §7
     - —
     - —
     - MANQUANT
   * - Procurations exception 10%
     - Art. 3.87 §7
     - —
     - —
     - MANQUANT
   * - Majorite simple (50%+1 exprimes)
     - Art. 3.87 §8
     - ``resolution.rs`` (Simple)
     - ``resolutions.feature``
     - CONFORME
   * - Majorite absolue (50%+1 tous)
     - Art. 3.87 §8
     - ``resolution.rs`` (Absolute)
     - ``resolutions.feature``
     - CONFORME
   * - Majorite 2/3 (travaux extra.)
     - Art. 3.88 al.1
     - ``resolution.rs`` (Qualified 0.667)
     - ``resolutions.feature``
     - CONFORME
   * - Majorite 3/4 (jouissance)
     - Art. 3.88 al.2
     - ``resolution.rs`` (Qualified 0.75)
     - —
     - CONFORME
   * - Majorite 4/5 (statuts)
     - Art. 3.88 al.3
     - ``resolution.rs`` (Qualified 0.80)
     - —
     - CONFORME
   * - Unanimite (quotes-parts)
     - Art. 3.88 al.4
     - ``resolution.rs`` (Qualified 1.0)
     - —
     - CONFORME
   * - PV distribution 30j
     - Art. 3.87 §10
     - —
     - —
     - MANQUANT
   * - Syndic mandat max 3 ans
     - Art. 3.89
     - —
     - —
     - MANQUANT
   * - Info publique syndic
     - Art. 3.89
     - ``building.rs`` (7 champs syndic)
     - ``public_syndic.feature``
     - CONFORME
   * - Conseil copropriete >=20 lots
     - Art. 3.90
     - ``board_member.rs``
     - ``board_decisions.feature``
     - CONFORME (partiel)
   * - Etat date delai 15j
     - Art. 3.94
     - ``etat_date.rs:286-297``
     - ``etat_date.feature``
     - CONFORME (corrige)
   * - Etat date contenu 16 sections
     - Art. 3.94
     - ``etat_date.rs`` (champs + additional_data)
     - ``etat_date.feature``
     - CONFORME

Comptabilite (AR 12/07/2012)
------------------------------

.. list-table::
   :header-rows: 1
   :widths: 25 15 25 20 15

   * - Exigence
     - Article AR
     - Fichier code
     - Scenario BDD
     - Statut
   * - Plan comptable PCMN
     - Art. 3
     - ``account.rs``, ``account_use_cases.rs``
     - —
     - CONFORME
   * - Comptabilite partie double
     - Art. 2
     - ``journal_entry.rs``
     - ``journal_entries.feature``
     - CONFORME
   * - Pieces justificatives
     - Art. 4
     - ``document.rs``, ``journal_entry.rs`` (document_ref)
     - ``documents.feature``
     - CONFORME
   * - Conservation 7 ans
     - Art. 5
     - Backups S3 chiffres
     - —
     - PARTIEL
   * - Bilan annuel
     - Art. 6
     - ``financial_report_use_cases.rs``
     - —
     - CONFORME
   * - Compte de resultats
     - Art. 6
     - ``financial_report_use_cases.rs``
     - —
     - CONFORME
   * - TVA belge (6/12/21%)
     - Legislation TVA
     - ``invoice_line_item.rs``, ``quote.rs``
     - ``invoices.feature``
     - CONFORME

RGPD
------

.. list-table::
   :header-rows: 1
   :widths: 25 15 25 20 15

   * - Exigence
     - Article
     - Fichier code
     - Scenario BDD
     - Statut
   * - Droit d'acces
     - Art. 15
     - ``gdpr_use_cases.rs``
     - ``gdpr.feature``
     - CONFORME
   * - Droit de rectification
     - Art. 16
     - ``gdpr_handlers.rs`` (PUT /gdpr/rectify)
     - ``gdpr.feature``
     - CONFORME
   * - Droit a l'effacement
     - Art. 17
     - ``gdpr_use_cases.rs``
     - ``gdpr.feature``
     - CONFORME
   * - Droit a la limitation
     - Art. 18
     - ``gdpr_handlers.rs``
     - ``gdpr.feature``
     - CONFORME
   * - Droit d'opposition
     - Art. 21
     - ``gdpr_handlers.rs``
     - ``gdpr.feature``
     - CONFORME
   * - Registre des traitements
     - Art. 30
     - ``audit.rs``, table ``audit_logs``
     - —
     - CONFORME
   * - Information personnes
     - Art. 13-14
     - —
     - —
     - MANQUANT
   * - DPA sous-traitants
     - Art. 28
     - —
     - —
     - MANQUANT
   * - Notification violation
     - Art. 33
     - —
     - —
     - MANQUANT
   * - Securite du traitement
     - Art. 32
     - LUKS, bcrypt, HSTS
     - —
     - PARTIEL

Recapitulatif
--------------

.. list-table::
   :header-rows: 1
   :widths: 30 20 20 30

   * - Domaine
     - Conforme
     - Manquant
     - Priorite
   * - Copropriete (AG)
     - 12/19
     - 7/19
     - Phase 1 critique
   * - Comptabilite (PCMN)
     - 6/7
     - 1/7
     - Phase 2
   * - RGPD
     - 6/10
     - 4/10
     - Phases 1-2
   * - **Total**
     - **24/36 (67%)**
     - **12/36**
     - —

Taux d'interet legal
----------------------

.. list-table::
   :header-rows: 1
   :widths: 25 15 25 20 15

   * - Exigence
     - Source
     - Fichier code
     - Scenario BDD
     - Statut
   * - Taux legal civil 4.5% (2026)
     - Moniteur belge
     - ``payment_reminder.rs:87``
     - ``payment_recovery.feature``
     - CONFORME (corrige)
