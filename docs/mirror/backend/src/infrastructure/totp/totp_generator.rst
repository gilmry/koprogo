=====================================
infrastructure/totp/totp_generator.rs
=====================================

:Fichier: ``backend/src/infrastructure/totp/totp_generator.rs``
:Type: RUST
:Lignes de Code: 445
:Couche: Infrastructure (Adaptateurs)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Module Rust **totp generator**. Fait partie de la couche Infrastructure (Adaptateurs).

API Publique
============

Structures
----------

- ``TotpGenerator``

Fonctions
---------

- ``generate_secret()``
- ``generate_qr_code()``
- ``verify_code()``
- ``generate_backup_codes()``
- ``hash_backup_code()``
- ``verify_backup_code()``
- ``encrypt_secret()``
- ``decrypt_secret()``
- ``generate_current_code()``

Code Source
===========

Voir: ``backend/src/infrastructure/totp/totp_generator.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

