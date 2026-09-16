-- Retour aux quatre portées d'origine.
--
-- ⚠️ Cette descente ÉCHOUERA si des liens `contractor_report` existent — et
-- c'est voulu. Les supprimer silencieusement ferait disparaître des accès
-- que des prestataires détiennent peut-être encore ; la contrainte doit
-- refuser plutôt que le faire à notre place.
ALTER TABLE magic_links DROP CONSTRAINT IF EXISTS magic_links_scope_kind_check;

ALTER TABLE magic_links ADD CONSTRAINT magic_links_scope_kind_check CHECK (
    scope_kind IN ('ticket', 'quote', 'invoice', 'contractor_evaluation')
);
