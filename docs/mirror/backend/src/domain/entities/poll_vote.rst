============================
domain/entities/poll_vote.rs
============================

:Fichier: ``backend/src/domain/entities/poll_vote.rs``
:Type: RUST
:Lignes de Code: 173
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant un **sondage** pour consultations entre assemblées générales. Implémente la logique métier pour les 4 types de votes (Oui/Non, Choix Multiple, Notation, Texte libre) avec validation des règles métier belges.

API Publique
============

Structures
----------

- ``PollVote``

Fonctions
---------

- ``new()``
- ``is_anonymous()``

Code Source
===========

Voir: ``backend/src/domain/entities/poll_vote.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

