=====================================================
Convocations AG Automatiques (Issue #88)
=====================================================

:Date: 2026-03 (initial) — màj 2026-06 (correction délais légaux Story H14)
:Version: 1.1.0
:Issue GitHub: #88 ; correction #618 (Track H, Story H14)
:Statut: Production-ready (Backend + Frontend complets — #88 fermée)

Vue d'ensemble
==============

Systeme de convocations automatiques pour assemblees generales avec conformite legale belge stricte. Gestion des delais legaux, tracking email, procurations, et rappels automatiques J-3.

Conformite legale belge
=======================

Delais minimaux obligatoires
-----------------------------

Le delai minimum de convocation est de **15 jours pour TOUTES les assemblees
generales** (ordinaire comme extraordinaire), conformement a l'**Art. 3.87 §3 du
Code civil** (Livre 3, ex-Art. 577-6 §2). Il n'existe **pas** de delai reduit a
8 jours pour les AG extraordinaires : cette distinction etait erronee.

.. list-table:: Delais minimaux de convocation (Art. 3.87 Code civil)
   :header-rows: 1
   :widths: 45 15 40

   * - Type de convocation
     - Delai minimum
     - Base legale
   * - AG ordinaire
     - 15 jours
     - Art. 3.87 §3 Code civil
   * - AG extraordinaire
     - 15 jours
     - Art. 3.87 §3 Code civil
   * - Seconde convocation (quorum non atteint)
     - 15 jours
     - Art. 3.87 §5 Code civil

**Exception — urgence.** L'Art. 3.87 §3 reserve le cas de l'urgence (« Sauf
dans les cas d'urgence, la convocation est communiquee quinze jours au moins
avant la date de l'assemblee... »). En cas d'urgence justifiee, le syndic peut
convoquer dans un delai plus court : la loi **ne fixe aucun seuil minimal** et,
en particulier, **aucun delai forfaitaire de 8 jours** n'est prevu par le Code
civil. Le reglement de copropriete peut imposer un delai **plus long** que
15 jours, jamais plus court (hors urgence).

La validation des delais est effectuee a trois niveaux :

1. **Domain entity** : calcul de ``minimum_send_date`` et validation ``respects_legal_deadline``
2. **Repository** : verification en base
3. **Database** : contraintes CHECK dans la migration

Workflow
--------

.. code-block:: text

   Draft --> Scheduled --> Sent --> (termine)
     |                      |
     '--> Cancelled         '--> Reminders J-3

Tracking email
--------------

- ``email_sent_at`` : horodatage d'envoi
- ``email_opened_at`` : tracking pixel / click link
- ``email_failed`` : gestion des bounces
- ``reminder_sent_at`` : rappel J-3 automatique

Suivi de presence
-----------------

.. code-block:: text

   Pending --> WillAttend --> Attended
                |              |
                '--> WillNotAttend --> DidNotAttend

Procuration belge
-----------------

Support de la delegation de pouvoir de vote via ``proxy_owner_id`` (mandataire).

Endpoints API (14)
==================

- ``POST /convocations`` - Creer une convocation (validation delais legaux)
- ``GET /convocations/:id`` - Obtenir une convocation
- ``GET /convocations/meeting/:meeting_id`` - Par reunion
- ``GET /buildings/:id/convocations`` - Par immeuble
- ``DELETE /convocations/:id`` - Supprimer
- ``PUT /convocations/:id/schedule`` - Planifier (valide avant deadline legale)
- ``POST /convocations/:id/send`` - Envoyer (genere PDF, cree destinataires, emails)
- ``PUT /convocations/:id/cancel`` - Annuler
- ``GET /convocations/:id/recipients`` - Liste destinataires avec tracking
- ``GET /convocations/:id/tracking-summary`` - Statistiques agregees
- ``PUT /convocation-recipients/:id/email-opened`` - Marquer email ouvert
- ``PUT /convocation-recipients/:id/attendance`` - Mettre a jour presence
- ``PUT /convocation-recipients/:id/proxy`` - Definir procuration
- ``POST /convocations/:id/reminders`` - Envoyer rappels J-3

Metriques de tracking
=====================

Le ``RecipientTrackingSummary`` fournit 8 metriques :

- total, opened, will_attend, will_not_attend
- attended, did_not_attend, pending, failed

Fichiers sources
================

- Domain: ``backend/src/domain/entities/convocation.rs`` (440 lignes), ``convocation_recipient.rs`` (260 lignes)
- Use Cases: ``backend/src/application/use_cases/convocation_use_cases.rs`` (430 lignes, 21 methods)
- Repositories: ``backend/src/infrastructure/database/repositories/convocation_repository_impl.rs``, ``convocation_recipient_repository_impl.rs``
- Handlers: ``backend/src/infrastructure/web/handlers/convocation_handlers.rs``
- Migration: ``backend/migrations/20251119000000_create_convocations.sql``

Total : ~3,650 lignes de code, 14 endpoints REST.
