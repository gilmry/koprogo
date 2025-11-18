====================================
domain/entities/two_factor_secret.rs
====================================

:File: ``domain/entities/two_factor_secret.rs``
:Lines of Code: 320
:Layer: Domain
:Has Tests: ✅ Yes

Public API
==========

Structures
----------

- ``TwoFactorSecret``

Functions
---------

- ``new()``
- ``with_backup_codes()``
- ``enable()``
- ``disable()``
- ``mark_used()``
- ``regenerate_backup_codes()``
- ``remove_backup_code()``
- ``backup_codes_low()``
- ``needs_reverification()``

Source Code
===========

See: ``backend/src/domain/entities/two_factor_secret.rs``

Related Documentation
=====================

.. seealso::

   - :doc:`/ARCHITECTURE`
   - :doc:`/CLAUDE`

