-- Quarantaine des pièces qui ne se rattachent à aucune ACP
-- ==========================================================
--
-- À jouer UNE FOIS sur une base existante, AVANT le déploiement qui applique
-- les migrations `20260903020000` et `20260903030000`. Sans cela, ces deux
-- migrations échouent — à raison — et le déploiement laisse la base en avance
-- sur le binaire, état dont le rollback ne sait pas revenir (drill F3 du
-- 2026-09-04, docs/ops/2026-09-04-drill-f3-restauration.md).
--
-- POURQUOI UN SCRIPT ET PAS UNE MIGRATION
--
-- Les deux migrations bloquantes portent des numéros déjà appliqués sur
-- certaines bases. En intercaler une nouvelle avant elles décalerait la liste
-- et ferait échouer la validation de sqlx sur les bases à jour. Ce nettoyage
-- est donc un acte d'exploitation, joué une fois, pas une migration.
--
-- POURQUOI DÉPLACER ET NON SUPPRIMER
--
-- Ces enregistrements sont sans objet, mais ce sont des écritures et des
-- quotes-parts. Les détruire serait irréversible et discutable au regard de
-- l'obligation de conservation comptable. Ils sont donc déplacés dans une
-- table de quarantaine, avec la raison et la date. Décider de leur sort
-- ensuite reste possible ; l'inverse ne l'est pas.
--
-- CE QUI EST DÉPLACÉ
--
-- 1. Les quotes-parts sans lot ET sans appel de fonds. Une quote-part naît de
--    la propriété d'un lot au prorata de ses tantièmes (Art. 3.86 § 3) : sans
--    lot ni appel, elle ne désigne aucune créance d'aucune ACP.
-- 2. Les écritures sans dépense ni quote-part de rattachement, et leurs
--    lignes. Rien ne permet de dire de quels livres elles relèvent.
--
-- Le script est idempotent : le rejouer ne déplace rien de plus.

BEGIN;

CREATE TABLE IF NOT EXISTS pieces_sans_acp_quarantaine (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_origine   TEXT        NOT NULL,
    identifiant     UUID        NOT NULL,
    contenu         JSONB       NOT NULL,
    raison          TEXT        NOT NULL,
    mis_en_quarantaine_le TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE pieces_sans_acp_quarantaine IS
    'Pièces retirées avant le recentrage ACP (ADR-0045) faute de copropriété résoluble. Rien n''est supprimé : leur sort reste à décider.';

-- ── 1. Les quotes-parts sans lot ni appel de fonds ────────────────────────
INSERT INTO pieces_sans_acp_quarantaine (table_origine, identifiant, contenu, raison)
SELECT 'owner_contributions', oc.id, to_jsonb(oc),
       'Ni lot ni appel de fonds : aucune ACP créancière résoluble (Art. 3.86 § 3)'
FROM owner_contributions oc
WHERE oc.unit_id IS NULL
  AND oc.call_for_funds_id IS NULL
  AND NOT EXISTS (
      SELECT 1 FROM pieces_sans_acp_quarantaine q
      WHERE q.table_origine = 'owner_contributions' AND q.identifiant = oc.id
  );

-- Les écritures qui pointaient vers ces quotes-parts partent avec elles,
-- sans quoi elles resteraient orphelines à leur tour.
INSERT INTO pieces_sans_acp_quarantaine (table_origine, identifiant, contenu, raison)
SELECT 'journal_entries', je.id, to_jsonb(je),
       'Rattachée à une quote-part mise en quarantaine'
FROM journal_entries je
WHERE je.contribution_id IN (
        SELECT identifiant FROM pieces_sans_acp_quarantaine
        WHERE table_origine = 'owner_contributions')
  AND NOT EXISTS (
      SELECT 1 FROM pieces_sans_acp_quarantaine q
      WHERE q.table_origine = 'journal_entries' AND q.identifiant = je.id
  );

-- ── 2. Les écritures sans aucun rattachement ──────────────────────────────
INSERT INTO pieces_sans_acp_quarantaine (table_origine, identifiant, contenu, raison)
SELECT 'journal_entries', je.id, to_jsonb(je),
       'Ni dépense ni quote-part : aucun livre d''ACP identifiable'
FROM journal_entries je
WHERE je.expense_id IS NULL
  AND je.contribution_id IS NULL
  AND NOT EXISTS (
      SELECT 1 FROM pieces_sans_acp_quarantaine q
      WHERE q.table_origine = 'journal_entries' AND q.identifiant = je.id
  );

-- Les lignes suivent leur écriture : une ligne sans écriture ne veut rien dire.
INSERT INTO pieces_sans_acp_quarantaine (table_origine, identifiant, contenu, raison)
SELECT 'journal_entry_lines', jel.id, to_jsonb(jel),
       'Ligne d''une écriture mise en quarantaine'
FROM journal_entry_lines jel
WHERE jel.journal_entry_id IN (
        SELECT identifiant FROM pieces_sans_acp_quarantaine
        WHERE table_origine = 'journal_entries')
  AND NOT EXISTS (
      SELECT 1 FROM pieces_sans_acp_quarantaine q
      WHERE q.table_origine = 'journal_entry_lines' AND q.identifiant = jel.id
  );

-- ── Retrait, dans l'ordre des dépendances ─────────────────────────────────
DELETE FROM journal_entry_lines
WHERE id IN (SELECT identifiant FROM pieces_sans_acp_quarantaine
             WHERE table_origine = 'journal_entry_lines');

DELETE FROM journal_entries
WHERE id IN (SELECT identifiant FROM pieces_sans_acp_quarantaine
             WHERE table_origine = 'journal_entries');

DELETE FROM owner_contributions
WHERE id IN (SELECT identifiant FROM pieces_sans_acp_quarantaine
             WHERE table_origine = 'owner_contributions');

-- ── Ce qui a été déplacé, à lire avant de valider ─────────────────────────
DO $$
DECLARE n_oc INT; n_je INT; n_jel INT;
BEGIN
    SELECT COUNT(*) INTO n_oc  FROM pieces_sans_acp_quarantaine WHERE table_origine = 'owner_contributions';
    SELECT COUNT(*) INTO n_je  FROM pieces_sans_acp_quarantaine WHERE table_origine = 'journal_entries';
    SELECT COUNT(*) INTO n_jel FROM pieces_sans_acp_quarantaine WHERE table_origine = 'journal_entry_lines';
    RAISE NOTICE 'Quarantaine : % quote(s)-part(s), % écriture(s), % ligne(s). Rien n''a été supprimé.', n_oc, n_je, n_jel;
END $$;

COMMIT;
