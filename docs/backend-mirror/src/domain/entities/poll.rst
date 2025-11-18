=======================
domain/entities/poll.rs
=======================

:File: ``domain/entities/poll.rs``
:Lines of Code: 402
:Layer: Domain
:Has Tests: ✅ Yes

Public API
==========

Structures
----------

- ``Poll``
- ``PollOption``

Enumerations
------------

- ``PollType``
- ``PollStatus``

Functions
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

*... and 1 more functions*

Source Code
===========

See: ``backend/src/domain/entities/poll.rs``

Related Documentation
=====================

.. seealso::

   - :doc:`/ARCHITECTURE`
   - :doc:`/CLAUDE`

