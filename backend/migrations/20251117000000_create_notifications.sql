-- Create notifications and notification_preferences tables
-- Issue #86 - Phase 2: Multi-Channel Notifications System

-- Notifications table
CREATE TABLE IF NOT EXISTS notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notification_type VARCHAR(50) NOT NULL,
    channel VARCHAR(20) NOT NULL,
    priority VARCHAR(20) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'Pending',
    title VARCHAR(200) NOT NULL,
    message TEXT NOT NULL,
    link_url TEXT,
    metadata JSONB,
    sent_at TIMESTAMPTZ,
    read_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    error_message TEXT,

    -- Constraints
    CONSTRAINT notifications_title_not_empty CHECK (LENGTH(TRIM(title)) > 0),
    CONSTRAINT notifications_message_not_empty CHECK (LENGTH(TRIM(message)) > 0),
    CONSTRAINT notifications_type_valid CHECK (notification_type IN (
        'ExpenseCreated', 'MeetingConvocation', 'PaymentReceived', 'TicketResolved',
        'DocumentAdded', 'BoardMessage', 'PaymentReminder', 'BudgetApproved',
        'ResolutionVote', 'System'
    )),
    CONSTRAINT notifications_channel_valid CHECK (channel IN ('Email', 'InApp', 'Push')),
    CONSTRAINT notifications_priority_valid CHECK (priority IN ('Low', 'Medium', 'High', 'Critical')),
    CONSTRAINT notifications_status_valid CHECK (status IN ('Pending', 'Sent', 'Failed', 'Read')),
    CONSTRAINT notifications_sent_at_set_when_sent CHECK (
        (status IN ('Sent', 'Read') AND sent_at IS NOT NULL) OR
        (status NOT IN ('Sent', 'Read') AND sent_at IS NULL)
    ),
    CONSTRAINT notifications_read_at_only_for_inapp CHECK (
        (channel = 'InApp' AND status = 'Read' AND read_at IS NOT NULL) OR
        (status != 'Read' AND read_at IS NULL)
    )
);

-- Notification preferences table
CREATE TABLE IF NOT EXISTS notification_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notification_type VARCHAR(50) NOT NULL,
    email_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    in_app_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    push_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Constraints
    CONSTRAINT notification_preferences_type_valid CHECK (notification_type IN (
        'ExpenseCreated', 'MeetingConvocation', 'PaymentReceived', 'TicketResolved',
        'DocumentAdded', 'BoardMessage', 'PaymentReminder', 'BudgetApproved',
        'ResolutionVote', 'System'
    )),
    -- One preference per user per notification type
    CONSTRAINT notification_preferences_unique UNIQUE (user_id, notification_type)
);

-- Indexes for notifications
CREATE INDEX idx_notifications_user ON notifications(user_id);
CREATE INDEX idx_notifications_organization ON notifications(organization_id);
CREATE INDEX idx_notifications_type ON notifications(notification_type);
CREATE INDEX idx_notifications_channel ON notifications(channel);
CREATE INDEX idx_notifications_status ON notifications(status);
CREATE INDEX idx_notifications_priority ON notifications(priority);
CREATE INDEX idx_notifications_created_at ON notifications(created_at DESC);
CREATE INDEX idx_notifications_sent_at ON notifications(sent_at DESC) WHERE sent_at IS NOT NULL;

-- Composite indexes for common queries
CREATE INDEX idx_notifications_user_status ON notifications(user_id, status);
CREATE INDEX idx_notifications_user_unread ON notifications(user_id, read_at)
    WHERE channel = 'InApp' AND status = 'Sent' AND read_at IS NULL;
CREATE INDEX idx_notifications_pending ON notifications(status, created_at)
    WHERE status = 'Pending';
CREATE INDEX idx_notifications_failed ON notifications(status, created_at)
    WHERE status = 'Failed';

-- Indexes for notification preferences
CREATE INDEX idx_notification_preferences_user ON notification_preferences(user_id);

-- Comments
COMMENT ON TABLE notifications IS 'Multi-channel notifications (Email, InApp, Push)';
COMMENT ON COLUMN notifications.notification_type IS 'Type of event: ExpenseCreated, MeetingConvocation, etc.';
COMMENT ON COLUMN notifications.channel IS 'Delivery channel: Email, InApp, Push';
COMMENT ON COLUMN notifications.priority IS 'Urgency level: Low, Medium, High, Critical';
COMMENT ON COLUMN notifications.status IS 'Workflow state: Pending → Sent/Failed → Read (InApp only)';
COMMENT ON COLUMN notifications.metadata IS 'JSON metadata for rich notifications (links, actions, etc.)';
COMMENT ON COLUMN notifications.link_url IS 'Optional deep link to related resource';
COMMENT ON COLUMN notifications.read_at IS 'Timestamp when notification was read (InApp only)';
COMMENT ON COLUMN notifications.error_message IS 'Error details if status = Failed';

COMMENT ON TABLE notification_preferences IS 'User preferences for notification channels';
COMMENT ON COLUMN notification_preferences.email_enabled IS 'User wants email notifications for this type';
COMMENT ON COLUMN notification_preferences.in_app_enabled IS 'User wants in-app notifications for this type';
COMMENT ON COLUMN notification_preferences.push_enabled IS 'User wants push notifications for this type';
