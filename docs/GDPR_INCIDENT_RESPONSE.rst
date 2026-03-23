================================================================================
GDPR Article 33 - Security Incident Response & 72-hour APD Notification
================================================================================

**Document Version**: 1.0
**Date**: March 2026
**Responsibility**: Data Protection Officer / Security Team / SuperAdmin

Legal Basis
================================================================================

GDPR Article 33 requires organizations to notify the supervisory authority
(Autorité de Protection des Données - APD in Belgium) of any personal data
breach **within 72 hours** of discovery (unless the breach is unlikely to
result in risk to the rights and freedoms of natural persons).

Belgium's APD (Autorité de Protection des Données):
  - Email: violation@autoriteprotectiondonnees.be
  - Website: https://www.autoriteprotectiondonnees.be
  - Official Language: French (FR) and Dutch (NL)

Security Incident Classification
================================================================================

**CRITICAL** (Immediate Escalation - 0 hours)
  - Unauthorized access to encrypted backup keys or password hashes
  - Complete database breach exposing payment information
  - Loss of encryption keys preventing data recovery
  - Ransomware attack affecting production systems
  - Malicious insider breach
  - Zero-day vulnerability exploitation in production
  - **APD Notification**: Required within 72 hours
  - **Data Subject Notification**: Required (Art. 34) - immediate communication
  - **Response Time**: Full-time incident response team activation
  - **Communications**: Public disclosure may be required

**HIGH** (Urgent - 4 hours)
  - Unauthorized access to owner personal data (limited exposure, <100 records)
  - Exposure of building addresses and contact information
  - Compromise of API authentication tokens
  - Brute-force attack succeeding in multiple accounts
  - Database backup accessible without authorization
  - **APD Notification**: Required within 72 hours
  - **Data Subject Notification**: Likely required (context-dependent)
  - **Response Time**: Containment within 4 hours, investigation within 24 hours
  - **Communications**: Vendor notification, affected organization notification

**MEDIUM** (Important - 24 hours)
  - Unsuccessful brute-force attack on login endpoint (rate-limited)
  - Exposure of non-sensitive metadata (building names, unit numbers)
  - Unauthorized API calls using valid but compromised credentials
  - Cross-site scripting (XSS) vulnerability allowing session hijacking
  - **APD Notification**: Likely not required (low risk if quickly resolved)
  - **Data Subject Notification**: Generally not required
  - **Response Time**: Containment within 24 hours, patch deployment within 72 hours
  - **Communications**: Internal security team, development team

**LOW** (Monitor - 72 hours)
  - Failed login attempts from multiple IPs (rate-limited by fail2ban)
  - SQL injection attempt blocked by WAF
  - Port scanning detected by Suricata IDS
  - GDPR data request from legitimate data subject
  - Configuration change without authorization (audit-logged but non-critical)
  - **APD Notification**: Not required
  - **Data Subject Notification**: Not required
  - **Response Time**: Analysis within 72 hours, patch in next release cycle
  - **Communications**: Log review, system monitoring, monthly security report

Internal Escalation Procedure
================================================================================

**Step 1: Detection (0 minutes)**
  - Monitoring system alert (Prometheus, Suricata, CrowdSec, fail2ban)
  - Manual security report from team member
  - APD inquiry about potential breach
  - Customer complaint about data access
  - **Action**: Immediate alert to Security Team + SuperAdmin

**Step 2: Triage (0-15 minutes)**
  - Assess incident classification (CRITICAL/HIGH/MEDIUM/LOW)
  - Identify affected data categories (payment data, addresses, emails, etc.)
  - Estimate number of affected data subjects
  - Determine containment approach
  - **Decision**: Proceed to investigation or escalate to CRITICAL

**Step 3: Containment (15-60 minutes)**
  - Isolate affected systems (revoke API keys, reset passwords if compromised)
  - Stop ongoing breach (block IP, patch vulnerability, rotate credentials)
  - Preserve evidence (copy logs, database snapshots, network traffic)
  - Document timeline of discovery and containment
  - **Notification**: Inform organization leadership (SuperAdmin)

**Step 4: Investigation (1-24 hours)**
  - Root cause analysis (how did breach occur, how long was it active)
  - Scope determination (which data was actually exposed)
  - Affected records count and data categories
  - Impact assessment (was data exfiltrated or just accessed)
  - **Output**: Security incident record in database with investigation notes

**Step 5: Notification (24-72 hours)**
  - **APD Notification**: If required (CRITICAL/HIGH incidents with personal data exposure)
  - **Data Subject Notification**: If required (Art. 34) and high risk
  - **Customer Notification**: Immediately if organization's data was affected
  - **Sub-processor Notification**: If breach involved external processor (Stripe, AWS, etc.)
  - **Public Disclosure**: Only if legally required or high-profile incident

**Step 6: Remediation (48-168 hours)**
  - Deploy patches for root cause vulnerability
  - Update security configurations (WAF rules, fail2ban jails, Suricata signatures)
  - Reset compromised credentials
  - Audit access logs for other signs of exploitation
  - **Communication**: Technical details to development team, timeline to leadership

**Step 7: Post-Incident Review (7-30 days)**
  - Document all findings in security_incidents table
  - Update security procedures based on lessons learned
  - Audit insurance coverage (cyber liability)
  - Communicate findings to board (if organization is co-property)
  - **Output**: Incident report filed with APD (if applicable)

APD Notification Template (French)
================================================================================

**Subject**: Notification de violation de données — Article 33 RGPD

**Body Template** (fill in specific details):

::

    Madame, Monsieur,

    Nous vous notifions une violation de données personnelles en vertu de l'article 33
    du Règlement Général sur la Protection des Données (RGPD).

    **Détails de la violation:**

    Date de découverte: [DATE]
    Nature de la violation: [NATURE: accès non autorisé, perte, modification, etc.]
    Date de la violation (estimée): [DATE]

    **Données affectées:**

    Catégories de données: [LISTE: noms, emails, adresses, données financières, etc.]
    Nombre de personnes affectées: [NOMBRE]
    Catégories de personnes: [LISTE: propriétaires d'unités, syndics, occupants, etc.]

    **Mesures prises:**

    1. Containment: [MESURES DE CONFINEMENT]
    2. Notification: Les personnes affectées ont été informées le [DATE]
    3. Prévention: [MESURES PRÉVENTIVES]

    **Persistance de la violation:**

    Risque résiduel: [FAIBLE/MOYEN/ÉLEVÉ]
    Violation toujours active: [OUI/NON]

    **Contact:**

    Délégué à la protection des données: [NOM, EMAIL, TÉLÉPHONE]

    Nous restons à votre disposition pour toute question.

    Cordialement,
    [ORGANISATION]

APD Notification Template (Dutch)
================================================================================

**Onderwerp**: Melding van gegevensbreuk — Artikel 33 AVG

**Body Template** (fill in specific details):

::

    Mevrouw, Meneer,

    Wij melden u een persoonlijke gegevensbreuk onder artikel 33
    van de Verordening (EU) 2016/679 (AVG).

    **Details van de inbreuk:**

    Datum van ontdekking: [DATUM]
    Aard van de inbreuk: [AARD: ongeautoriseerde toegang, verlies, wijziging, enz.]
    Datum van de inbreuk (geschat): [DATUM]

    **Getroffen gegevens:**

    Gegevenscategorieën: [LIJST: namen, e-mailadressen, adressen, financiële gegevens, enz.]
    Aantal getroffen personen: [AANTAL]
    Categorieën personen: [LIJST: eigenaren van wooneenheden, syndici, bewoners, enz.]

    **Genomen maatregelen:**

    1. Beperking: [BEPERKINGSMAATREGELEN]
    2. Melding: De getroffen personen zijn op [DATUM] geïnformeerd
    3. Voorkoming: [PREVENTIEVE MAATREGELEN]

    **Voortduring van de inbreuk:**

    Residueel risico: [LAAG/GEMIDDELD/HOOG]
    Inbreuk nog actief: [JA/NEE]

    **Contact:**

    Functionaris voor gegevensbescherming: [NAAM, E-MAIL, TELEFOONNUMMER]

    Wij staan u graag voor vragen ter beschikking.

    Met vriendelijke groeten,
    [ORGANISATIE]

Database Schema for Security Incidents
================================================================================

The `security_incidents` table tracks all incidents with APD notification status:

::

    CREATE TABLE security_incidents (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        organization_id UUID NOT NULL REFERENCES organizations(id),
        severity TEXT NOT NULL CHECK (severity IN ('critical', 'high', 'medium', 'low')),
        incident_type TEXT NOT NULL, -- "data_breach", "unauthorized_access", "malware", etc.
        title TEXT NOT NULL,
        description TEXT NOT NULL,
        data_categories_affected TEXT[] NOT NULL, -- array: "payment_data", "personal_info", etc.
        affected_subjects_count INTEGER,
        discovery_at TIMESTAMPTZ NOT NULL,
        notification_at TIMESTAMPTZ, -- when APD was notified
        apd_reference_number TEXT, -- APD's acknowledgment number
        status TEXT NOT NULL CHECK (status IN ('detected', 'investigating', 'contained', 'reported', 'closed')),
        reported_by UUID NOT NULL REFERENCES users(id),
        investigation_notes TEXT,
        root_cause TEXT,
        remediation_steps TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
    );

    CREATE INDEX idx_security_incidents_organization ON security_incidents(organization_id);
    CREATE INDEX idx_security_incidents_severity ON security_incidents(severity);
    CREATE INDEX idx_security_incidents_status ON security_incidents(status);
    CREATE INDEX idx_security_incidents_created_at ON security_incidents(created_at DESC);
    CREATE INDEX idx_security_incidents_not_reported ON security_incidents(notification_at IS NULL) WHERE status IN ('detected', 'investigating', 'contained');

Automated Compliance Checks
================================================================================

**Background Job**: Check for incidents overdue for APD notification (>72 hours old)

::

    -- Pseudo-code (implemented as scheduled task)
    SELECT * FROM security_incidents
    WHERE (severity IN ('critical', 'high')
           OR affected_subjects_count > 10)
      AND status IN ('detected', 'investigating', 'contained')
      AND discovery_at < NOW() - INTERVAL '72 hours'
      AND notification_at IS NULL;

    -- Alert: Log warning for any matching incidents
    -- Response: Trigger APD notification immediately

**Metrics Dashboard** (for SuperAdmin):

- Total incidents this month (by severity)
- Average time to containment (target: 4 hours for HIGH)
- Average time to APD notification (target: 48 hours for required incidents)
- Incidents closed without escalation (MEDIUM/LOW incidents)
- APD reference numbers tracked

Post-Incident Review Checklist
================================================================================

After containment and investigation:

- [ ] Incident logged in security_incidents table with all details
- [ ] Root cause identified and documented
- [ ] Number of affected data subjects confirmed
- [ ] APD notification sent (if required) within 72 hours
- [ ] Data subjects notified (if required) with high-risk assessment
- [ ] Remediation steps completed (patches deployed, credentials rotated)
- [ ] Lessons learned documented for security team
- [ ] Monitoring rules updated to detect similar incidents
- [ ] Security procedure updated to prevent recurrence
- [ ] Incident report provided to organization leadership
- [ ] Insurance provider notified (if cyber liability policy exists)
- [ ] Board decision log updated (if organization is co-property with board)

Compliance Deadlines Summary
================================================================================

- **72 hours**: APD notification deadline (from discovery)
- **30 days**: Post-incident security review (from incident closure)
- **90 days**: Incident report retention in database (for audit trail)
- **1 year**: Incident records archive (compliance/legal hold)
- **Immediately**: Data subject notification (if high risk, Art. 34)

Contact Information
================================================================================

**APD (Autorité de Protection des Données - Belgium)**
  - Email: violation@autoriteprotectiondonnees.be
  - Website: https://www.autoriteprotectiondonnees.be
  - Phone: +32 2 274 48 00
  - Address: Rue de Trèves 61, 1040 Bruxelles, Belgium
  - Languages: FR, NL, DE, EN

**KoproGo Security Team**
  - On-Call: [CONTACT INFO]
  - Email: security@koprogo.com
  - Escalation: Contact SuperAdmin immediately

**Legal Counsel**
  - [NAME, EMAIL, PHONE for GDPR legal advisor]

Revision History
================================================================================

- **2026-03-23**: Initial document (v1.0) - Incident classification, APD procedure, templates
