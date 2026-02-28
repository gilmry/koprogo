=============================================
Payment Integration System (Issue #84)
=============================================

Overview
========

KoproGo integrates with Stripe Payment Intents and SEPA Direct Debit
for secure payment processing. The system handles the full transaction
lifecycle from creation through refund, with PCI-DSS compliance via
Stripe tokenization.

Belgian Context
===============

- **EUR only**: All transactions in Euros (Belgian market)
- **SEPA Direct Debit**: Primary payment method for Belgian bank accounts
- **Belgian IBAN validation**: BE + 14 digits format

Transaction Lifecycle
=====================

::

    Pending → Processing → RequiresAction → Succeeded
                                ↓              ↓
                              Failed      Refunded (partial/full)
                                ↓
                            Cancelled

Payment Methods
===============

+---------------+---------------------------+
| Type          | Description               |
+===============+===========================+
| Card          | Visa, Mastercard (Stripe) |
+---------------+---------------------------+
| SepaDebit     | Belgian IBAN mandate      |
+---------------+---------------------------+
| BankTransfer  | Manual wire transfer      |
+---------------+---------------------------+
| Cash          | In-person payment         |
+---------------+---------------------------+

Key Features
============

- **Idempotency keys**: Prevent duplicate charges on retry (min 16 chars)
- **PCI-DSS**: No raw card data stored, only Stripe tokens (``pm_xxx``)
- **Refunds**: Partial and full refunds with anti-over-refund validation
- **Default management**: One default payment method per owner (atomic)
- **Statistics**: Total paid, net amount, per owner/building/expense

API Endpoints (38 total)
========================

**Payments** (22 endpoints):

- ``POST /payments`` - Create payment
- ``GET /payments/:id`` - Get payment
- ``GET /owners/:id/payments`` - Owner payments
- ``GET /buildings/:id/payments`` - Building payments
- ``PUT /payments/:id/succeeded`` - Mark succeeded
- ``POST /payments/:id/refund`` - Process refund
- ``GET /owners/:id/payments/stats`` - Owner statistics

**Payment Methods** (16 endpoints):

- ``POST /payment-methods`` - Create
- ``GET /owners/:id/payment-methods`` - List owner methods
- ``PUT /payment-methods/:id/set-default`` - Set default
- ``PUT /payment-methods/:id/deactivate`` - Deactivate

Architecture
============

- **Domain**: ``payment.rs`` (530 lines), ``payment_method.rs`` (273 lines)
- **Use Cases**: ``payment_use_cases.rs`` (26 methods), ``payment_method_use_cases.rs`` (14 methods)
- **Migration**: ``20251118000000_create_payments.sql``
- **BDD Tests**: ``payments.feature`` + ``payment_methods.feature`` (28 scenarios)
- **Total**: ~5,500 lines of code, 38 REST endpoints
