ALTER TABLE magic_links DROP CONSTRAINT IF EXISTS magic_links_recipient_identified_check;

ALTER TABLE magic_links DROP CONSTRAINT IF EXISTS magic_links_scope_kind_check;
ALTER TABLE magic_links
    ADD CONSTRAINT magic_links_scope_kind_check CHECK (
        scope_kind IN ('ticket', 'quote', 'invoice', 'contractor_evaluation')
    );

ALTER TABLE magic_links DROP COLUMN IF EXISTS single_use;
ALTER TABLE magic_links DROP COLUMN IF EXISTS recipient_label;

ALTER TABLE magic_links ALTER COLUMN subject_user_id SET NOT NULL;
