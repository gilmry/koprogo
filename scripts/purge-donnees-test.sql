-- Purge des organisations de test créées par la suite e2e.
--
-- POURQUOI
--   `loginAsSyndic` (tests/e2e/helpers/auth.ts) crée une organisation par
--   test, sous la forme `${prefix} Org ${timestamp}` / `${prefix}-${timestamp}`.
--   La campagne e2e tournant contre la production, 1637 organisations de test
--   s'y étaient accumulées. Effet mesuré le 2026-08-31 : /admin/users mettait
--   une vingtaine de secondes à rendre (~500 utilisateurs, ~1900 badges).
--
-- CRITÈRE
--   Slug suffixé d'un horodatage epoch-ms à 13 chiffres. Aucune organisation
--   réelle n'en porte : au 2026-08-31, 1637 correspondaient, 4 non
--   (Copropriété Bruxelles SPRL, Résidence Grand Place SPRL, Syndic Liège SA,
--   op). S'y ajoutent les comptes `login-test-<horodatage>@example.com`,
--   créés par des tests qui n'avaient pas créé d'organisation.
--
-- POURQUOI `session_replication_role = replica`
--   La hiérarchie compte plus de quatre-vingts tables, et plusieurs clés
--   étrangères ne cascadent pas : `acps.organization_id` est en SET NULL,
--   `buildings.acp_id` en NO ACTION. Un simple DELETE sur `organizations`
--   échoue (contrainte `fk_account` depuis `journal_entry_lines`) et, s'il
--   passait, laisserait des milliers d'orphelins. Désactiver les déclencheurs
--   le temps de la suppression permet de retirer l'arbre entier ; le balayage
--   final vérifie qu'aucun orphelin ne subsiste.
--
-- VALIDÉ AVANT APPLICATION
--   Exécuté le 2026-08-31 sur une restauration complète de la sauvegarde de
--   production, pas sur la production elle-même. Résultat : 4 organisations,
--   7 comptes seed et 4 immeubles conservés, quotas intacts, ZÉRO orphelin
--   sur l'ensemble des clés étrangères du schéma.
--
-- AVANT DE RELANCER
--   1. Sauvegarder :  docker exec koprogo-postgres pg_dump -U koprogo \
--                       -d koprogo_db --clean --if-exists | gzip > sauvegarde.sql.gz
--   2. Vérifier que la sauvegarde se restaure et que les comptes y sont.
--   3. Puis :  docker exec -i koprogo-postgres psql -U koprogo -d koprogo_db \
--                < scripts/purge-donnees-test.sql
--
--   Le tout est dans une transaction : en cas d'erreur, rien n'est appliqué.
--
-- NE PAS OUBLIER
--   Ceci traite le symptôme. La cause est que la suite e2e écrit dans la base
--   de production. Tant qu'elle y écrira, la purge sera à refaire.

\set ON_ERROR_STOP on
BEGIN;
SET session_replication_role = replica;

CREATE TEMP TABLE cibles_org ON COMMIT DROP AS
  SELECT id FROM organizations WHERE slug ~ '-[0-9]{13}$';
CREATE TEMP TABLE cibles_acp ON COMMIT DROP AS
  SELECT id FROM acps WHERE organization_id IN (SELECT id FROM cibles_org);
CREATE TEMP TABLE cibles_bat ON COMMIT DROP AS
  SELECT id FROM buildings WHERE acp_id IN (SELECT id FROM cibles_acp);
CREATE TEMP TABLE cibles_lot ON COMMIT DROP AS
  SELECT id FROM units WHERE building_id IN (SELECT id FROM cibles_bat);

-- Du plus fin au plus large : lot, immeuble, ACP, organisation.
DO $$
DECLARE t text;
BEGIN
  FOR t IN SELECT c.table_name FROM information_schema.columns c
           JOIN information_schema.tables tb ON tb.table_name=c.table_name AND tb.table_schema=c.table_schema
           WHERE c.column_name='unit_id' AND c.table_schema='public' AND tb.table_type='BASE TABLE'
  LOOP EXECUTE format('DELETE FROM %I WHERE unit_id IN (SELECT id FROM cibles_lot)', t); END LOOP;

  FOR t IN SELECT c.table_name FROM information_schema.columns c
           JOIN information_schema.tables tb ON tb.table_name=c.table_name AND tb.table_schema=c.table_schema
           WHERE c.column_name='building_id' AND c.table_schema='public' AND tb.table_type='BASE TABLE'
  LOOP EXECUTE format('DELETE FROM %I WHERE building_id IN (SELECT id FROM cibles_bat)', t); END LOOP;

  FOR t IN SELECT c.table_name FROM information_schema.columns c
           JOIN information_schema.tables tb ON tb.table_name=c.table_name AND tb.table_schema=c.table_schema
           WHERE c.column_name='acp_id' AND c.table_schema='public' AND tb.table_type='BASE TABLE'
             AND c.table_name<>'buildings'
  LOOP EXECUTE format('DELETE FROM %I WHERE acp_id IN (SELECT id FROM cibles_acp)', t); END LOOP;

  FOR t IN SELECT c.table_name FROM information_schema.columns c
           JOIN information_schema.tables tb ON tb.table_name=c.table_name AND tb.table_schema=c.table_schema
           WHERE c.column_name='organization_id' AND c.table_schema='public' AND tb.table_type='BASE TABLE'
             AND c.table_name NOT IN ('organizations','acps')
  LOOP EXECUTE format('DELETE FROM %I WHERE organization_id IN (SELECT id FROM cibles_org)', t); END LOOP;
END $$;

DELETE FROM buildings     WHERE id IN (SELECT id FROM cibles_bat);
DELETE FROM acps          WHERE id IN (SELECT id FROM cibles_acp);
DELETE FROM organizations WHERE id IN (SELECT id FROM cibles_org);
DELETE FROM users WHERE email ~ '^login-test-[0-9]{13}@example\.com$';

-- Balayage des orphelins restants.
--
-- Plusieurs tables ne se rattachent ni à une organisation ni à un immeuble,
-- mais seulement par user_id, meeting_id ou campaign_id : refresh_tokens,
-- magic_links, resolutions, provider_offers... On supprime en boucle toute
-- ligne dont le parent a disparu, jusqu'à stabilisation. Une ligne rattachée
-- à une donnée réelle a par construction un parent existant : elle n'est
-- jamais touchée.
DO $$
DECLARE r record; n bigint; passe int := 0; supprimes bigint;
BEGIN
  LOOP
    passe := passe + 1; supprimes := 0;
    FOR r IN SELECT tc.table_name AS src, kcu.column_name AS col,
                    ccu.table_name AS ref, ccu.column_name AS refcol
             FROM information_schema.table_constraints tc
             JOIN information_schema.key_column_usage kcu ON tc.constraint_name=kcu.constraint_name
             JOIN information_schema.constraint_column_usage ccu ON tc.constraint_name=ccu.constraint_name
             WHERE tc.constraint_type='FOREIGN KEY' AND tc.table_schema='public'
    LOOP
      BEGIN
        EXECUTE format(
          'DELETE FROM %I s WHERE s.%I IS NOT NULL AND NOT EXISTS (SELECT 1 FROM %I p WHERE p.%I = s.%I)',
          r.src, r.col, r.ref, r.refcol, r.col);
        GET DIAGNOSTICS n = ROW_COUNT; supprimes := supprimes + n;
      EXCEPTION WHEN others THEN NULL; END;
    END LOOP;
    RAISE NOTICE 'balayage passe % : % orphelin(s) supprimé(s)', passe, supprimes;
    EXIT WHEN supprimes = 0 OR passe >= 10;
  END LOOP;
END $$;

SET session_replication_role = DEFAULT;
COMMIT;
