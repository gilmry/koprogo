-- Create convocations tables for automatic AG (General Assembly) invitations
-- Issue #88: Automatic AG Convocations with Legal Deadline Verification
-- Belgian copropriété legal requirements:
--   - Ordinary AG: 15 days minimum notice
--   - Extraordinary AG: 8 days minimum notice
--   - Second convocation: 8 days minimum notice

-- Create convocation_type ENUM
CREATE TYPE convocation_type AS ENUM (
    'ordinary',           -- Assemblée Générale Ordinaire (15 days notice)
    'extraordinary',      -- Assemblée Générale Extraordinaire (8 days notice)
    'second_convocation'  -- Second convocation after quorum not reached (8 days notice)
);

-- Create convocation_status ENUM
CREATE TYPE convocation_status AS ENUM (
    'draft',      -- Not yet sent
    'scheduled',  -- Will be sent at scheduled time
    'sent',       -- Emails dispatched
    'cancelled'   -- Meeting cancelled
);

-- Create attendance_status ENUM for recipients
CREATE TYPE attendance_status AS ENUM (
    'pending',            -- No response yet
    'will_attend',        -- Will attend the meeting
    'will_not_attend',    -- Will not attend
    'attended',           -- Attended (marked after meeting)
    'did_not_attend'      -- Did not attend (marked after meeting)
);

-- Create convocations table
CREATE TABLE convocations (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    meeting_id UUID NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,

    -- Meeting details
    meeting_type convocation_type NOT NULL,
    meeting_date TIMESTAMPTZ NOT NULL,
    status convocation_status NOT NULL DEFAULT 'draft',

    -- Legal deadline tracking
    minimum_send_date TIMESTAMPTZ NOT NULL,  -- Latest date to send (meeting_date - minimum_notice_days)
    actual_send_date TIMESTAMPTZ,            -- When actually sent
    scheduled_send_date TIMESTAMPTZ,         -- When scheduled to be sent

    -- PDF generation
    pdf_file_path TEXT,                      -- Path to generated PDF
    language VARCHAR(2) NOT NULL CHECK (language IN ('FR', 'NL', 'DE', 'EN')),

    -- Tracking
    total_recipients INTEGER NOT NULL DEFAULT 0 CHECK (total_recipients >= 0),
    opened_count INTEGER NOT NULL DEFAULT 0 CHECK (opened_count >= 0),
    will_attend_count INTEGER NOT NULL DEFAULT 0 CHECK (will_attend_count >= 0),
    will_not_attend_count INTEGER NOT NULL DEFAULT 0 CHECK (will_not_attend_count >= 0),

    -- Reminders
    reminder_sent_at TIMESTAMPTZ,            -- J-3 reminder

    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),

    -- Constraints
    CONSTRAINT convocations_unique_meeting UNIQUE (meeting_id),
    CONSTRAINT convocations_counts_valid CHECK (
        opened_count <= total_recipients AND
        will_attend_count <= total_recipients AND
        will_not_attend_count <= total_recipients
    ),
    CONSTRAINT convocations_minimum_send_before_meeting CHECK (minimum_send_date < meeting_date),
    CONSTRAINT convocations_actual_send_before_meeting CHECK (
        actual_send_date IS NULL OR actual_send_date <= meeting_date
    ),
    CONSTRAINT convocations_scheduled_send_before_meeting CHECK (
        scheduled_send_date IS NULL OR scheduled_send_date <= meeting_date
    ),
    CONSTRAINT convocations_pdf_when_sent CHECK (
        status != 'sent' OR (status = 'sent' AND pdf_file_path IS NOT NULL)
    )
);

-- Create convocation_recipients table
CREATE TABLE convocation_recipients (
    id UUID PRIMARY KEY,
    convocation_id UUID NOT NULL REFERENCES convocations(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES owners(id) ON DELETE CASCADE,
    email TEXT NOT NULL CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$'),

    -- Email tracking
    email_sent_at TIMESTAMPTZ,
    email_opened_at TIMESTAMPTZ,
    email_failed BOOLEAN NOT NULL DEFAULT FALSE,
    email_failure_reason TEXT,

    -- Reminder tracking
    reminder_sent_at TIMESTAMPTZ,
    reminder_opened_at TIMESTAMPTZ,

    -- Attendance tracking
    attendance_status attendance_status NOT NULL DEFAULT 'pending',
    attendance_updated_at TIMESTAMPTZ,

    -- Proxy delegation (if owner delegates voting power)
    proxy_owner_id UUID REFERENCES owners(id) ON DELETE SET NULL,

    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT convocation_recipients_unique_owner UNIQUE (convocation_id, owner_id),
    CONSTRAINT convocation_recipients_email_opened_after_sent CHECK (
        email_opened_at IS NULL OR (email_sent_at IS NOT NULL AND email_opened_at >= email_sent_at)
    ),
    CONSTRAINT convocation_recipients_reminder_after_email CHECK (
        reminder_sent_at IS NULL OR (email_sent_at IS NOT NULL AND reminder_sent_at >= email_sent_at)
    ),
    CONSTRAINT convocation_recipients_reminder_opened_after_sent CHECK (
        reminder_opened_at IS NULL OR (reminder_sent_at IS NOT NULL AND reminder_opened_at >= reminder_sent_at)
    ),
    CONSTRAINT convocation_recipients_no_self_proxy CHECK (
        proxy_owner_id IS NULL OR proxy_owner_id != owner_id
    )
);

-- Indexes for convocations
CREATE INDEX idx_convocations_organization_id ON convocations(organization_id);
CREATE INDEX idx_convocations_building_id ON convocations(building_id);
CREATE INDEX idx_convocations_meeting_id ON convocations(meeting_id);
CREATE INDEX idx_convocations_status ON convocations(status);
CREATE INDEX idx_convocations_meeting_date ON convocations(meeting_date);
CREATE INDEX idx_convocations_scheduled_send_date ON convocations(scheduled_send_date) WHERE scheduled_send_date IS NOT NULL;
CREATE INDEX idx_convocations_created_by ON convocations(created_by);

-- Indexes for convocation_recipients
CREATE INDEX idx_convocation_recipients_convocation_id ON convocation_recipients(convocation_id);
CREATE INDEX idx_convocation_recipients_owner_id ON convocation_recipients(owner_id);
CREATE INDEX idx_convocation_recipients_attendance_status ON convocation_recipients(attendance_status);
CREATE INDEX idx_convocation_recipients_email_sent ON convocation_recipients(email_sent_at) WHERE email_sent_at IS NOT NULL;
CREATE INDEX idx_convocation_recipients_email_opened ON convocation_recipients(email_opened_at) WHERE email_opened_at IS NOT NULL;
CREATE INDEX idx_convocation_recipients_email_failed ON convocation_recipients(email_failed) WHERE email_failed = TRUE;
CREATE INDEX idx_convocation_recipients_needs_reminder ON convocation_recipients(convocation_id)
    WHERE email_sent_at IS NOT NULL
    AND email_opened_at IS NULL
    AND reminder_sent_at IS NULL
    AND email_failed = FALSE;

-- Comments
COMMENT ON TABLE convocations IS 'Automatic AG (General Assembly) convocations with Belgian legal compliance';
COMMENT ON TABLE convocation_recipients IS 'Individual recipients of convocations with email tracking and attendance status';

COMMENT ON COLUMN convocations.minimum_send_date IS 'Latest date to send convocation (meeting_date - minimum_notice_days). Belgian law: 15d for ordinary AG, 8d for extraordinary/second convocation';
COMMENT ON COLUMN convocations.actual_send_date IS 'When convocation was actually sent. Must be <= minimum_send_date to respect legal deadline';
COMMENT ON COLUMN convocations.scheduled_send_date IS 'When convocation is scheduled to be sent automatically';
COMMENT ON COLUMN convocations.reminder_sent_at IS 'J-3 reminder sent to recipients who have not opened the convocation';

COMMENT ON COLUMN convocation_recipients.email_opened_at IS 'Email read receipt timestamp (tracking pixel or link click)';
COMMENT ON COLUMN convocation_recipients.attendance_status IS 'Attendance status: pending → will_attend/will_not_attend → attended/did_not_attend';
COMMENT ON COLUMN convocation_recipients.proxy_owner_id IS 'Owner to whom voting power is delegated (procuration)';
