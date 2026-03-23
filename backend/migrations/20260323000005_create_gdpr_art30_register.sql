-- Migration: GDPR Article 30 - Processing Activities and Sub-Processors Registry
-- Issue #316 - DPA with sub-processors
-- Date: 2026-03-23

-- Table: Data Processing Activities (Art. 30 register)
-- Tracks all data processing activities and their legal bases
CREATE TABLE IF NOT EXISTS data_processing_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    activity_name VARCHAR(255) NOT NULL,
    controller_name VARCHAR(255) NOT NULL,
    purpose TEXT NOT NULL,
    legal_basis VARCHAR(50) NOT NULL, -- Art. 6(1)(a) through (f), Art. 6(1)(c) for legal obligations
    data_categories TEXT[] NOT NULL, -- array: "personal_identifiers", "financial_data", "contact_info", etc.
    data_subjects TEXT[] NOT NULL, -- array: "building_owners", "syndics", "occupants", etc.
    recipients TEXT[] NOT NULL, -- array: "stripe", "aws_s3", "smtp_provider", etc.
    retention_period VARCHAR(100) NOT NULL, -- e.g. "7 years", "90 days", "until contract termination"
    security_measures TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_data_processing_activities_controller ON data_processing_activities(controller_name);
CREATE INDEX idx_data_processing_activities_created_at ON data_processing_activities(created_at DESC);

-- Table: Data Processor Agreements (Sub-Processors)
-- Tracks DPA status with all sub-processors
CREATE TABLE IF NOT EXISTS data_processor_agreements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    processor_name VARCHAR(255) NOT NULL UNIQUE,
    service_description TEXT NOT NULL,
    dpa_signed_at TIMESTAMPTZ,
    dpa_url VARCHAR(500),
    transfer_mechanism VARCHAR(100), -- "EU_SCC", "Standard_Contractual_Clauses", "EU_only", "Deferred"
    data_categories TEXT[] NOT NULL, -- array: "payment_data", "backup_data", "email_addresses", etc.
    certifications TEXT[], -- array: "ISO_27001", "SOC2_Type_II", "C5", etc.
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_data_processor_agreements_processor_name ON data_processor_agreements(processor_name);
CREATE INDEX idx_data_processor_agreements_dpa_signed_at ON data_processor_agreements(dpa_signed_at);
CREATE INDEX idx_data_processor_agreements_created_at ON data_processor_agreements(created_at DESC);

-- Seed default processing activities
INSERT INTO data_processing_activities (
    activity_name,
    controller_name,
    purpose,
    legal_basis,
    data_categories,
    data_subjects,
    recipients,
    retention_period,
    security_measures
) VALUES
    (
        'General Co-property Management',
        'KoproGo Organization',
        'Manage building data, units, ownership relationships, and expenses',
        'Art. 6(1)(b) - Contractual necessity',
        ARRAY['personal_identifiers', 'contact_information', 'financial_data', 'ownership_data'],
        ARRAY['building_owners', 'syndics', 'occupants'],
        ARRAY['internal', 'stripe', 'aws_s3', 'smtp_provider'],
        '7 years',
        'Encryption at rest (AES-256), TLS in transit, role-based access control'
    ),
    (
        'GDPR Compliance & Audit Logging',
        'KoproGo Organization',
        'Maintain GDPR compliance records and detect security incidents',
        'Art. 6(1)(c) - Legal obligation, Art. 6(1)(f) - Legitimate interest',
        ARRAY['event_logs', 'ip_addresses', 'user_actions'],
        ARRAY['system_users'],
        ARRAY['internal_superadmin_only'],
        '1 year',
        'Immutable PostgreSQL logging, encrypted backup, access control'
    ),
    (
        'Financial Reporting & Accounting',
        'KoproGo Organization',
        'Generate financial statements, VAT records, and audit trails',
        'Art. 6(1)(c) - Legal obligation (Belgian accounting law)',
        ARRAY['financial_transactions', 'payment_methods', 'account_codes', 'vat_data'],
        ARRAY['organization_users', 'building_owners'],
        ARRAY['internal', 'authorized_accountants', 'aws_s3'],
        '7 years',
        'PCI-DSS compliance, encryption, audit trail tracking'
    ),
    (
        'Notifications & Communication',
        'KoproGo Organization',
        'Send meeting invitations, payment reminders, and voting notifications',
        'Art. 6(1)(b) - Contractual necessity',
        ARRAY['names', 'email_addresses', 'phone_numbers', 'meeting_information'],
        ARRAY['building_owners', 'syndics', 'unit_occupants'],
        ARRAY['smtp_provider', 'twilio_sms_optional'],
        '90 days (email logs), 30 days (SMS logs)',
        'SPF/DKIM/DMARC authentication, TLS encryption'
    ),
    (
        'Security Monitoring & Incident Response',
        'KoproGo Organization',
        'Detect and respond to security incidents (Art. 33)',
        'Art. 6(1)(f) - Legitimate interest, Art. 6(1)(c) - Legal obligation',
        ARRAY['ip_addresses', 'timestamps', 'error_messages', 'system_logs'],
        ARRAY['all_system_users'],
        ARRAY['internal_security_team'],
        '1 year (incident records), 90 days (audit logs)',
        'Immutable logging, access control, intrusion detection systems'
    );

-- Seed default sub-processor agreements
INSERT INTO data_processor_agreements (
    processor_name,
    service_description,
    dpa_signed_at,
    dpa_url,
    transfer_mechanism,
    data_categories,
    certifications
) VALUES
    (
        'Stripe',
        'Payment processing and transaction management for building expenses and owner contributions',
        NOW(),
        'https://stripe.com/be/legal/dpa',
        'EU_SCC',
        ARRAY['payment_details', 'card_metadata', 'billing_information'],
        ARRAY['ISO_27001', 'SOC2_Type_II', 'PCI_DSS_Level_1']
    ),
    (
        'AWS S3',
        'Encrypted backups and document storage for buildings, meetings, and financial records',
        NOW(),
        'https://aws.amazon.com/legal/data-processing-addendum/',
        'EU_only',
        ARRAY['backup_data', 'documents', 'financial_records', 'building_data'],
        ARRAY['ISO_27001', 'SOC2_Type_II', 'C5_German_BSI']
    ),
    (
        'SMTP Email Provider',
        'Transactional email delivery for notifications, convocations, and reminders',
        NULL,
        NULL,
        'Deferred',
        ARRAY['email_addresses', 'names', 'meeting_information', 'payment_details'],
        ARRAY[]
    ),
    (
        'Twilio',
        'Optional SMS delivery for urgent notifications to building owners and syndics',
        NOW(),
        'https://www.twilio.com/legal/data-protection-addendum',
        'EU_SCC',
        ARRAY['phone_numbers', 'sms_content'],
        ARRAY['ISO_27001', 'SOC2_Type_II']
    );
