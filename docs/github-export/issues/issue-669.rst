===================================================================================================
Issue #669: [Story B5] TicketCreate Complaint refacto + Severity/Evidence/Witness atomiques (FR-B5)
===================================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: maury,slice-3
:Assignees: Unassigned
:Created: 2026-07-26
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/669>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Story rétro-documentée le 2026-07-26 suite à l'audit `docs/agent-activity/2026-07-26-sync-audit-feature-dev.md` §2.2 — cluster « Phase B FE » implémenté sans issue GitHub créée en amont (gap de traçabilité identifié, cf. règle CRITICAL.md #6 « tout dans GitHub »).
   
   Fait partie de l'épique #556 ([EPIC] Refonte UX multi-rôle + modèle ACP — pipeline Maury, 39 stories, 7 slices), volet frontend de la Slice 3 (sous-rôles + magic link + PWA + mandats + ticketing).
   
   ## Implémentation
   
   Refonte du formulaire de création de ticket pour le type plainte, avec composants atomiques Severity/Evidence/Witness.
   
   Commit : `f1c871db`
   
   ## Stabilisation e2e
   
   Les specs Playwright de cette phase sont documentées comme instables — suivi séparément dans #617 (Phase C — Stabilisation Documentation Vivante e2e, 8 specs Phase B FE). Cette issue certifie l'implémentation, pas la stabilité e2e.

.. raw:: html

   </div>

