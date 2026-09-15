-- Issue #855 (ADR 0048) — le notaire consulte un état daté via un lien signé
-- émis par le syndic, pas via un compte KoproGo dédié. Le mécanisme générique
-- de magic_links (Story 3.2) est réutilisé plutôt que d'en créer un troisième
-- (cf. #835 sur la prolifération des liens magiques).
--
-- Deux sous-questions tranchées ici (issue #855) :
-- 1. La portée `etat_date` se relit jusqu'à expiration plutôt que de se
--    consommer à la première lecture (`single_use = false`) : un notaire
--    reconsulte l'état daté pendant l'instruction d'une vente.
-- 2. Le notaire n'a pas de compte : `subject_user_id` devient nullable, et
--    `recipient_label` porte son identité pour l'audit quand il est absent.

ALTER TABLE magic_links
    ALTER COLUMN subject_user_id DROP NOT NULL;

ALTER TABLE magic_links
    ADD COLUMN recipient_label VARCHAR(255);

ALTER TABLE magic_links
    ADD COLUMN single_use BOOLEAN NOT NULL DEFAULT true;

ALTER TABLE magic_links
    DROP CONSTRAINT IF EXISTS magic_links_scope_kind_check;

ALTER TABLE magic_links
    ADD CONSTRAINT magic_links_scope_kind_check CHECK (
        scope_kind IN ('ticket', 'quote', 'invoice', 'contractor_evaluation', 'etat_date')
    );

-- Défense en profondeur : la même règle existe déjà dans `MagicLink::issue`
-- (domain/plateforme/magic_link.rs), mais un lien sans compte ET sans
-- étiquette ne serait plus auditable — la contrainte l'interdit aussi côté
-- base, au cas où un futur appelant contournerait le domaine.
ALTER TABLE magic_links
    ADD CONSTRAINT magic_links_recipient_identified_check CHECK (
        subject_user_id IS NOT NULL OR recipient_label IS NOT NULL
    );

COMMENT ON COLUMN magic_links.recipient_label IS
    'Identité du destinataire quand il n''a pas de compte KoproGo (ex. notaire). Issue #855.';
COMMENT ON COLUMN magic_links.single_use IS
    'Copié de MagicLinkScopeKind::is_single_use() à l''émission (issue #855). false pour etat_date : relecture bornée par expires_at seul.';
