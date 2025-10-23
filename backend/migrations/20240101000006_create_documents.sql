-- Create documents table
CREATE TYPE document_type AS ENUM ('meeting_minutes', 'financial_statement', 'invoice', 'contract', 'regulation', 'works_quote', 'other');

CREATE TABLE documents (
    id UUID PRIMARY KEY,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    document_type document_type NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    file_path TEXT NOT NULL,
    file_size BIGINT NOT NULL CHECK (file_size > 0),
    mime_type VARCHAR(100) NOT NULL,
    uploaded_by UUID NOT NULL,
    related_meeting_id UUID REFERENCES meetings(id) ON DELETE SET NULL,
    related_expense_id UUID REFERENCES expenses(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_documents_building ON documents(building_id);
CREATE INDEX idx_documents_type ON documents(document_type);
CREATE INDEX idx_documents_meeting ON documents(related_meeting_id);
CREATE INDEX idx_documents_expense ON documents(related_expense_id);
CREATE INDEX idx_documents_created_at ON documents(created_at);
