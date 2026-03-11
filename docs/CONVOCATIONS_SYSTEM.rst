=============================================
AG Convocation System (Issue #88)
=============================================

Overview
========

Automatic generation and sending of general assembly (AG) invitations
with strict compliance to Belgian copropriété law deadlines.

Belgian Legal Requirements
==========================

Article 577-6 §2 of the Belgian Civil Code mandates minimum notice
periods for general assembly convocations:

+-------------------+-------------------+-------------------------+
| Meeting Type      | Minimum Notice    | Legal Basis             |
+===================+===================+=========================+
| Ordinary AG       | 15 calendar days  | Art. 577-6 §2 al. 1    |
+-------------------+-------------------+-------------------------+
| Extraordinary AG  | 8 calendar days   | Art. 577-6 §2 al. 2    |
+-------------------+-------------------+-------------------------+
| Second Convoc.    | 8 calendar days   | After quorum failure    |
+-------------------+-------------------+-------------------------+

The system automatically calculates the ``minimum_send_date`` based on
the meeting date and type, and blocks sending if the legal deadline
would not be respected.

Workflow
========

::

    Draft → Scheduled → Sent → (Cancelled)

Features
========

- **Legal deadline validation**: Prevents sending too late
- **Email tracking**: sent_at, opened_at (tracking pixel), failed (bounce)
- **Reminder automation**: J-3 automatic reminders for unopened emails
- **Attendance tracking**: Pending → WillAttend/WillNotAttend → Attended/DidNotAttend
- **Proxy delegation**: Belgian "procuration" support
- **Multi-language**: FR/NL/DE/EN PDF generation
- **Bulk operations**: Atomic creation of all recipients

Recipient Tracking
==================

Each recipient has individual tracking:

- ``email_sent_at``: When the convocation email was sent
- ``email_opened_at``: When the email was opened (tracking pixel)
- ``email_failed``: Whether delivery failed (bounce)
- ``reminder_sent_at``: When the J-3 reminder was sent
- ``attendance_status``: Current attendance status
- ``proxy_owner_id``: Delegate for vote proxy

Tracking Summary provides aggregate statistics:

- Total recipients, opened count, will_attend count
- Opening rate, attendance rate

API Endpoints (14)
==================

- ``POST /convocations`` - Create with legal deadline validation
- ``GET /convocations/:id`` - Get details
- ``GET /convocations/meeting/:meeting_id`` - By meeting
- ``GET /buildings/:id/convocations`` - Building convocations
- ``PUT /convocations/:id/schedule`` - Schedule send date
- ``POST /convocations/:id/send`` - Send to all owners
- ``PUT /convocations/:id/cancel`` - Cancel
- ``GET /convocations/:id/recipients`` - List recipients
- ``GET /convocations/:id/tracking-summary`` - Aggregate stats
- ``PUT /convocation-recipients/:id/email-opened`` - Mark opened
- ``PUT /convocation-recipients/:id/attendance`` - Update attendance
- ``PUT /convocation-recipients/:id/proxy`` - Set proxy
- ``POST /convocations/:id/reminders`` - Send J-3 reminders
- ``DELETE /convocations/:id`` - Delete

Architecture
============

- **Domain**: ``convocation.rs`` (440 lines), ``convocation_recipient.rs`` (260 lines)
- **Use Cases**: ``convocation_use_cases.rs`` (21 methods)
- **Migration**: ``20251119000000_create_convocations.sql``
- **BDD Tests**: ``convocations.feature`` (13 scenarios)
- **Total**: ~3,650 lines of code
