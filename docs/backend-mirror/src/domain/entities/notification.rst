===============================
domain/entities/notification.rs
===============================

:File: ``domain/entities/notification.rs``
:Lines of Code: 434
:Layer: Domain
:Has Tests: ✅ Yes

Public API
==========

Structures
----------

- ``Notification``
- ``NotificationPreference``

Enumerations
------------

- ``NotificationType``
- ``NotificationChannel``
- ``NotificationStatus``
- ``NotificationPriority``

Functions
---------

- ``new()``
- ``with_link()``
- ``with_metadata()``
- ``mark_sent()``
- ``mark_failed()``
- ``mark_read()``
- ``is_unread()``
- ``is_pending()``
- ``retry()``
- ``new()``

*... and 2 more functions*

Source Code
===========

See: ``backend/src/domain/entities/notification.rs``

Related Documentation
=====================

.. seealso::

   - :doc:`/ARCHITECTURE`
   - :doc:`/CLAUDE`

