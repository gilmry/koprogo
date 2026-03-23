================================================================================
GDPR Article 28 - Data Processing Agreement (DPA) Registry
================================================================================

**Document Version**: 1.0
**Date**: March 2026
**Responsibility**: Data Protection Officer / SuperAdmin

Legal Basis
================================================================================

Article 28 of the General Data Protection Regulation (GDPR) requires organizations
to ensure that any external processor handling personal data on their behalf has
entered into a Data Processing Agreement (DPA).

All KoproGo sub-processors must:

1. Provide a signed Data Processing Agreement (DPA)
2. Offer Standard Contractual Clauses (SCCs) for non-EU transfers
3. Implement appropriate technical and organizational security measures
4. Maintain a register of processing activities (Art. 30)

Sub-Processor Register
================================================================================

**Stripe** (Payments Integration)
  - **Service**: Payment processing and transaction management
  - **DPA Status**: Signed ✓
  - **DPA URL**: https://stripe.com/be/legal/dpa
  - **Data Categories**: Payment details, card metadata, billing information
  - **Data Subjects**: Building owners, syndics, organization users
  - **Transfer Mechanism**: EU Standard Contractual Clauses (Stripe Europe)
  - **Certifications**: ISO 27001, SOC 2 Type II
  - **Retention**: 7 years (Belgian legal requirement)
  - **Security**: AES-256 encryption, PCI-DSS Level 1

**AWS S3** (Backups and Document Storage)
  - **Service**: Encrypted backups, PDF document storage
  - **DPA Status**: Signed (AWS Data Processing Addendum) ✓
  - **DPA URL**: https://aws.amazon.com/legal/data-processing-addendum/
  - **Data Categories**: Building data, financial records, PDFs, encrypted backups
  - **Data Subjects**: All organization members, owners
  - **Transfer Mechanism**: EU region (eu-west-1) - No transfers
  - **Certifications**: ISO 27001, SOC 2 Type II, C5 (German BSI)
  - **Retention**: 7 days (local), lifecycle-based S3 policy
  - **Security**: LUKS encryption at rest (AES-XTS-512), TLS in transit

**SMTP/Email Provider** (Notifications, Invitations)
  - **Service**: Transactional email delivery (convocations, reminders, etc.)
  - **DPA Status**: DPA Pending - Review Required
  - **Service Details**: Email authentication (SPF, DKIM, DMARC), bounce handling
  - **Data Categories**: Email addresses, names, meeting dates, payment information
  - **Data Subjects**: Building owners, syndics, unit occupants
  - **Transfer Mechanism**: To be determined
  - **Certifications**: TBD
  - **Retention**: 90 days (server logs), 30 days (transactional history)
  - **Action Required**: Execute DPA with email provider (Sendgrid, AWS SES, or equivalent)

**Twilio/SMS** (Optional SMS Alerts)
  - **Service**: SMS delivery for urgent notifications
  - **DPA Status**: Signed ✓ (Optional/Beta Feature)
  - **DPA URL**: https://www.twilio.com/legal/data-protection-addendum
  - **Data Categories**: Phone numbers, SMS content (brief messages)
  - **Data Subjects**: Owners who opt-in to SMS notifications
  - **Transfer Mechanism**: EU Standard Contractual Clauses
  - **Certifications**: ISO 27001, SOC 2 Type II
  - **Retention**: 30 days
  - **Note**: Only active if organization enables SMS notifications
  - **Security**: TLS encryption, authentication via API tokens

Processing Activities Register (Art. 30)
================================================================================

**Activity**: General Co-property Management
  - **Purpose**: Manage building data, units, ownership relationships, expenses
  - **Legal Basis**: Contractual necessity (Art. 6(1)(b))
  - **Data Categories**: Name, email, phone, address, ownership %, payment history
  - **Data Subjects**: Building owners, syndics, occupants
  - **Recipients**: Stripe (payments), AWS (backups), SMTP (emails)
  - **Retention**: 7 years (Belgian law - copropriété archives)
  - **Security**: Encryption at rest + in transit, access control (role-based)

**Activity**: GDPR Compliance & Audit Logging
  - **Purpose**: Maintain GDPR compliance records, detect incidents
  - **Legal Basis**: Legal obligation (Art. 6(1)(c)), legitimate interest (Art. 6(1)(f))
  - **Data Categories**: Event logs, IP addresses, timestamps, user actions
  - **Data Subjects**: All system users
  - **Recipients**: Internal (SuperAdmin only)
  - **Retention**: 90 days (logs), 1 year (incident records)
  - **Security**: Encrypted logs, immutable storage via PostgreSQL

**Activity**: Financial Reporting & Accounting
  - **Purpose**: Generate financial statements, VAT records, audit trails
  - **Legal Basis**: Legal obligation (Art. 6(1)(c)) - Belgian accounting law
  - **Data Categories**: Financial transactions, payment methods, account codes
  - **Data Subjects**: Organization users, building owners
  - **Recipients**: AWS (backups), authorized accountants
  - **Retention**: 7 years (Belgian law)
  - **Security**: PCI-DSS compliance for payment data, encryption

**Activity**: Notifications & Communication
  - **Purpose**: Send meeting invitations, payment reminders, voting notifications
  - **Legal Basis**: Contractual necessity (Art. 6(1)(b))
  - **Data Categories**: Names, emails, phone numbers, meeting information
  - **Data Subjects**: Building owners, syndics, unit occupants
  - **Recipients**: SMTP provider, Twilio (optional)
  - **Retention**: 90 days (email logs), 30 days (SMS logs)
  - **Security**: SPF/DKIM/DMARC, TLS encryption

**Activity**: Security Monitoring & Incident Response
  - **Purpose**: Detect and respond to security incidents (Art. 33)
  - **Legal Basis**: Legitimate interest (Art. 6(1)(f)), legal obligation (Art. 6(1)(c))
  - **Data Categories**: IP addresses, timestamps, error messages, system logs
  - **Data Subjects**: All system users
  - **Recipients**: Internal (SuperAdmin/Security team)
  - **Retention**: 1 year (incident records), 90 days (audit logs)
  - **Security**: Immutable logging, access control

Standard Contractual Clauses (SCCs)
================================================================================

All non-EU data transfers are governed by EU Standard Contractual Clauses
per the European Commission decisions and GDPR Article 46:

- **Stripe SCCs**: Embedded in Stripe DPA (https://stripe.com/be/legal/dpa)
- **AWS SCCs**: Embedded in AWS Data Processing Addendum
- **Twilio SCCs**: Embedded in Twilio DPA (https://www.twilio.com/legal/dpa)

Organizations using KoproGo in EU regions are protected by:

1. **No transfers for EU data**: AWS S3 eu-west-1 (Ireland), Stripe EU region
2. **Redundancy**: Local PostgreSQL backups stored in EU (optional S3 lifecycle)
3. **Supplementary measures**: Encryption at rest/in transit, access controls

Non-EU Organizations
================================================================================

If deploying KoproGo outside the EU:

1. **Data transfers** must comply with local data protection laws
2. **SCCs** are the default transfer mechanism (GDPR Art. 46)
3. **Supplementary measures** (encryption, access control) reduce transfer risks
4. **Risk assessments** required per EDPB Guidelines 05/2020 (SCCs adequacy)

Data Breach Notification
================================================================================

In case of a personal data breach affecting sub-processor data:

1. **Notify processor immediately** (within 24 hours if possible)
2. **Log incident** in security_incidents table (Art. 33)
3. **Notify APD** (Autorité de Protection des Données) within 72 hours
4. **Notify data subjects** if high risk (Art. 34)

See **GDPR_INCIDENT_RESPONSE.rst** for detailed procedures.

Contact Information
================================================================================

**Data Protection Officer (DPO)**:
  - Role: Oversee GDPR compliance
  - Responsibility: Review DPA status, process subject requests
  - Escalation: Contact SuperAdmin for incident response

**APD (Autorité de Protection des Données - Belgium)**:
  - Email: violation@autoriteprotectiondonnees.be
  - Website: https://www.autoriteprotectiondonnees.be
  - Languages: FR, NL, DE, EN

**Stripe Support**:
  - DPA inquiries: legal@stripe.com
  - Technical support: https://support.stripe.com

**AWS DPA**:
  - Form: https://aws.amazon.com/legal/data-processing-addendum/
  - Support: https://console.aws.amazon.com/support

Compliance Checklist
================================================================================

- [ ] All DPAs are signed and current
- [ ] Sub-processor list is up to date
- [ ] SCCs are in place for non-EU transfers
- [ ] Data subjects were notified of sub-processors (Art. 14(2)(e))
- [ ] Security measures are documented and reviewed annually
- [ ] Incident response procedure is tested
- [ ] Audit logs are retained per retention schedules
- [ ] Access controls limit processor visibility to necessary data only

Revision History
================================================================================

- **2026-03-23**: Initial document (v1.0) - Stripe, AWS S3, SMTP, Twilio registered
