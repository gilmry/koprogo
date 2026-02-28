==========================================
Multi-Channel Notification System (Issue #86)
==========================================

Overview
========

KoproGo supports multi-channel notifications (Email, SMS, Push, In-App)
with per-user preference management. 22 notification types cover all
platform events from payment reminders to meeting invitations.

Channels
========

- **Email**: Primary channel for formal communications
- **SMS**: Urgent notifications (payment overdue, emergency)
- **Push**: Mobile app notifications (future)
- **InApp**: Web dashboard notifications (badge + dropdown)

Notification Types (22)
=======================

+-----------------------+-------------------------------+
| Category              | Types                         |
+=======================+===============================+
| Meetings              | MeetingReminder,              |
|                       | MeetingCreated,               |
|                       | MeetingCancelled              |
+-----------------------+-------------------------------+
| Payments              | PaymentDue,                   |
|                       | PaymentReceived,              |
|                       | PaymentOverdue                |
+-----------------------+-------------------------------+
| Documents             | DocumentShared,               |
|                       | DocumentUploaded              |
+-----------------------+-------------------------------+
| Tickets               | TicketUpdate,                 |
|                       | TicketAssigned,               |
|                       | TicketResolved                |
+-----------------------+-------------------------------+
| System                | SystemAlert,                  |
|                       | SystemMaintenance             |
+-----------------------+-------------------------------+

User Preferences
================

Each user can configure notification preferences per type and per channel:

.. code-block:: json

    {
        "notification_type": "PaymentDue",
        "enabled": true,
        "email_enabled": true,
        "sms_enabled": false,
        "push_enabled": true
    }

API Endpoints
=============

**Notifications**:

- ``POST /notifications`` - Create notification
- ``GET /notifications/my`` - List my notifications
- ``GET /notifications/unread`` - List unread
- ``PUT /notifications/:id/read`` - Mark as read
- ``PUT /notifications/read-all`` - Mark all as read
- ``DELETE /notifications/:id`` - Delete
- ``GET /notifications/stats`` - Statistics

**Preferences**:

- ``GET /notification-preferences/:user_id`` - Get all preferences
- ``GET /notification-preferences/:user_id/:type`` - Get specific
- ``PUT /notification-preferences/:user_id/:type`` - Update

Architecture
============

- **Domain**: ``notification.rs``, ``notification_preference.rs``
- **Use Cases**: ``notification_use_cases.rs`` (13 methods)
- **Migration**: ``20251117000001_create_notifications.sql``
- **BDD Tests**: ``notifications.feature`` (14 scenarios)
