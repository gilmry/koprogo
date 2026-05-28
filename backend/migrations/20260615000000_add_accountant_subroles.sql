-- Migration: Add Story 3.1 sous-rôles métier (accountant.encodeur, accountant.emetteur,
--            community.moderator, lawyer, notary, amo, architect, bet, warden).
-- Date: 2026-06-15
-- Story: 3.1 — Sous-rôles métier (FR21 séparation des pouvoirs comptables, INV-10).
-- Refs: docs/maury/refonte-ux-multi-role-acp/stories.md L302-324

-- 1. Étendre la CHECK constraint sur user_roles (table user_role_assignments).
ALTER TABLE user_roles DROP CONSTRAINT IF EXISTS user_roles_role_check;
ALTER TABLE user_roles ADD CONSTRAINT user_roles_role_check
    CHECK (role IN (
        'superadmin',
        'syndic',
        'accountant',
        'accountant.encodeur',
        'accountant.emetteur',
        'board_member',
        'contractor',
        'owner',
        'community.moderator',
        'lawyer',
        'notary',
        'amo',
        'architect',
        'bet',
        'warden'
    ));

-- 2. Étendre la CHECK constraint sur la table users (rôle primaire).
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_role_check;
ALTER TABLE users ADD CONSTRAINT users_role_check
    CHECK (role IN (
        'superadmin',
        'syndic',
        'accountant',
        'accountant.encodeur',
        'accountant.emetteur',
        'board_member',
        'contractor',
        'owner',
        'community.moderator',
        'lawyer',
        'notary',
        'amo',
        'architect',
        'bet',
        'warden'
    ));

-- 3. Indexes pour requêtes filtrées par sous-rôle comptable (FR21 séparation pouvoirs).
CREATE INDEX IF NOT EXISTS idx_user_roles_accountant_encodeur
    ON user_roles(role, organization_id)
    WHERE role = 'accountant.encodeur';

CREATE INDEX IF NOT EXISTS idx_user_roles_accountant_emetteur
    ON user_roles(role, organization_id)
    WHERE role = 'accountant.emetteur';

CREATE INDEX IF NOT EXISTS idx_user_roles_community_moderator
    ON user_roles(role, organization_id)
    WHERE role = 'community.moderator';
