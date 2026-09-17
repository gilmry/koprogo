-- Story 4.6 (#581) — la colonne `resolutions.kind` que le code écrivait déjà.
--
-- `ResolutionKind` a été livré avec son entité, son mapping
-- (`kind_to_string` / `parse_kind`) et son INSERT — mais SANS migration.
-- Le dépôt de résolutions emploie des requêtes vérifiées à l'exécution :
-- le compilateur ne pouvait donc rien dire, et TOUTE création de résolution
-- échouait en 400 « column "kind" of relation "resolutions" does not exist ».
--
-- Relevé par la campagne e2e du 2026-09-16 (10 échecs), pas par un gate :
-- c'est précisément le trou que #940 décrit.
--
-- `standard` est le `#[default]` de l'enum : les résolutions existantes
-- reçoivent donc l'exacte valeur que le code leur attribuait déjà en
-- mémoire. La migration décrit l'existant, elle ne le redéfinit pas.

ALTER TABLE resolutions
    ADD COLUMN IF NOT EXISTS kind VARCHAR(50) NOT NULL DEFAULT 'standard';

-- Les valeurs doivent rester alignées sur l'enum Rust `ResolutionKind`
-- (`domain/copropriete/resolution.rs`). La paire est inscrite dans
-- `garde_enum_contre_contrainte`, qui compare les deux listes dans les DEUX
-- sens — c'est ce qui rend cet alignement vérifié plutôt que promis.
ALTER TABLE resolutions
    ADD CONSTRAINT resolutions_kind_check CHECK (
        kind IN ('standard', 'evaluation_contractors_auto')
    );
