=======================
domain/entities/poll.rs
=======================

:Fichier: ``backend/src/domain/entities/poll.rs``
:Type: RUST
:Lignes de Code: 402
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant un **sondage** pour consultations entre assemblées générales. Implémente la logique métier pour les 4 types de votes (Oui/Non, Choix Multiple, Notation, Texte libre) avec validation des règles métier belges.

API Publique
============

Structures
----------

- ``Poll``
- ``PollOption``

Énumérations
------------

- ``PollType``
- ``PollStatus``

Fonctions
---------

- ``new()``
- ``publish()``
- ``close()``
- ``cancel()``
- ``is_active()``
- ``is_ended()``
- ``participation_rate()``
- ``get_winning_option()``
- ``record_vote()``
- ``auto_close_if_ended()``
- ``new()``

Code Source
===========

Voir: ``backend/src/domain/entities/poll.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

