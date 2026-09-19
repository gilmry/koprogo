-- `contractor_report` rejoint les portées de lien magique acceptées.
--
-- ── Le défaut, et pourquoi il a survécu ───────────────────────────────────
--
-- #835 a ajouté la variante `MagicLinkScopeKind::ContractorReport` au
-- domaine Rust, pour absorber le second système de liens magiques qui
-- existait en parallèle. La contrainte SQL, elle, est restée aux quatre
-- portées de 2026-06-05 :
--
--     CHECK (scope_kind IN ('ticket','quote','invoice','contractor_evaluation'))
--
-- Résultat, mesuré en CI le 2026-09-16 :
--
--     new row for relation "magic_links" violates check constraint
--     "magic_links_scope_kind_check"
--
-- Le code compile, les tests unitaires passent — la variante existe bien
-- côté Rust — et c'est la BASE qui refuse à l'exécution. Rien dans la revue
-- d'une branche ne peut voir ça : la branche qui ajoute la variante ne
-- touche pas aux migrations, et rien ne relie les deux.
--
-- C'est le QUATRIÈME défaut de jonction de la fusion des vingt-huit
-- branches, après le DTO sans `ToSchema`, le test BDD qui n'avait pas suivi
-- un changement de signature, et la table `funds` qui n'existait pas. Le
-- motif est stable : **une énumération Rust et une contrainte CHECK sont
-- deux copies de la même vérité, et rien ne les tient ensemble.**
--
-- ── Ce que cette migration ne répare pas ──────────────────────────────────
--
-- Elle rattrape UNE variante. La prochaine se perdra pareil. Le vrai
-- remède est une garde qui compare les variantes de `MagicLinkScopeKind` à
-- la contrainte — le dépôt sait faire, `garde_schema_mort` lit déjà le SQL.
-- Elle mérite son issue plutôt qu'un commentaire ici.

ALTER TABLE magic_links DROP CONSTRAINT IF EXISTS magic_links_scope_kind_check;

ALTER TABLE magic_links ADD CONSTRAINT magic_links_scope_kind_check CHECK (
    scope_kind IN (
        'ticket',
        'quote',
        'invoice',
        'contractor_evaluation',
        -- #835 — le rapport d'intervention rejoint l'écran unifié : même
        -- page, même paramètre `t`, plus de second système parallèle.
        'contractor_report'
    )
);
