-- Migration: Create skills table for Skills Directory (Issue #49 - Phase 3)
-- Date: 2025-11-20 18:00:00

-- Create custom ENUM types for skills
CREATE TYPE skill_category AS ENUM (
    'HomeRepair',     -- Home repair and maintenance (plumbing, electrical, carpentry, etc.)
    'Languages',      -- Languages (teaching, translation, conversation practice)
    'Technology',     -- Technology (IT support, web development, software, hardware)
    'Education',      -- Education and tutoring (math, science, music lessons, etc.)
    'Arts',           -- Arts and crafts (painting, sewing, woodworking, etc.)
    'Sports',         -- Sports and fitness (personal training, yoga, martial arts, etc.)
    'Cooking',        -- Cooking and baking
    'Gardening',      -- Gardening and landscaping
    'Health',         -- Health and wellness (massage, physiotherapy, counseling, etc.)
    'Legal',          -- Legal and administrative (tax preparation, document assistance, etc.)
    'Financial',      -- Financial (accounting, budgeting advice, etc.)
    'PetCare',        -- Pet care and training
    'Other'           -- Other skills
);

CREATE TYPE expertise_level AS ENUM (
    'Beginner',       -- Beginner (< 1 year experience)
    'Intermediate',   -- Intermediate (1-3 years experience)
    'Advanced',       -- Advanced (3-10 years experience)
    'Expert'          -- Expert (10+ years experience or professional certification)
);

-- Create skills table
CREATE TABLE skills (
    id UUID PRIMARY KEY,
    owner_id UUID NOT NULL REFERENCES owners(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    skill_category skill_category NOT NULL,
    skill_name VARCHAR(100) NOT NULL CHECK (LENGTH(skill_name) >= 3),
    expertise_level expertise_level NOT NULL,
    description VARCHAR(1000) NOT NULL CHECK (LENGTH(description) > 0),
    is_available_for_help BOOLEAN NOT NULL DEFAULT TRUE,
    hourly_rate_credits INT CHECK (hourly_rate_credits >= 0 AND hourly_rate_credits <= 100),
    years_of_experience INT CHECK (years_of_experience >= 0 AND years_of_experience <= 50),
    certifications TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for efficient queries

-- Index for building-based queries (primary use case: list all building skills)
CREATE INDEX idx_skills_building ON skills(building_id, is_available_for_help DESC, expertise_level DESC, skill_name ASC);

-- Partial index for available skills (marketplace view: skills directory)
CREATE INDEX idx_skills_available ON skills(building_id, expertise_level DESC, skill_name ASC)
WHERE is_available_for_help = TRUE;

-- Index for filtering by category (e.g., find all technology skills)
CREATE INDEX idx_skills_category ON skills(building_id, skill_category, expertise_level DESC, skill_name ASC);

-- Index for filtering by expertise level (e.g., find all experts)
CREATE INDEX idx_skills_expertise ON skills(building_id, expertise_level, skill_name ASC);

-- Index for owner-based queries (my skills)
CREATE INDEX idx_skills_owner ON skills(owner_id, created_at DESC);

-- Partial index for free/volunteer skills (hourly_rate_credits IS NULL OR = 0)
CREATE INDEX idx_skills_free ON skills(building_id, expertise_level DESC, skill_name ASC)
WHERE hourly_rate_credits IS NULL OR hourly_rate_credits = 0;

-- Partial index for professional skills (Expert level OR has certifications)
CREATE INDEX idx_skills_professional ON skills(building_id, skill_name ASC)
WHERE expertise_level = 'Expert' OR certifications IS NOT NULL;

-- Add column comments for documentation
COMMENT ON TABLE skills IS 'Skills directory for community members to share expertise and offer help';
COMMENT ON COLUMN skills.skill_category IS 'Skill category: HomeRepair, Languages, Technology, Education, Arts, Sports, Cooking, Gardening, Health, Legal, Financial, PetCare, Other';
COMMENT ON COLUMN skills.expertise_level IS 'Expertise level: Beginner (< 1 year), Intermediate (1-3 years), Advanced (3-10 years), Expert (10+ years or certified)';
COMMENT ON COLUMN skills.skill_name IS 'Name of the skill (3-100 characters)';
COMMENT ON COLUMN skills.description IS 'Detailed description of the skill (max 1000 characters)';
COMMENT ON COLUMN skills.is_available_for_help IS 'Whether the owner is currently available to help others with this skill';
COMMENT ON COLUMN skills.hourly_rate_credits IS 'Hourly rate in SEL credits (0-100, NULL or 0 = free/volunteer)';
COMMENT ON COLUMN skills.years_of_experience IS 'Years of experience (0-50, optional)';
COMMENT ON COLUMN skills.certifications IS 'Professional certifications or qualifications (optional)';
