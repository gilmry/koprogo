-- Story 4.2 (#48) — la colonne `votes.auth_method` que le code lisait déjà.
--
-- Deuxième occurrence du même défaut que #581 (`resolutions.kind`), trouvée
-- dans la même campagne : l'enum `VoteAuthMethod`, son encodage
-- (`to_db_str` / `from_db_string`), l'INSERT et le SELECT ont été livrés —
-- sans migration. Le dépôt de votes emploie des requêtes vérifiées à
-- l'exécution : rien ne pouvait le dire avant un appel réel.
--
-- `GET /resolutions/{id}/votes` rendait **500** :
--   Database error finding votes by resolution:
--   column "auth_method" does not exist
--
-- `presence` est ce que `Vote::new` attribue par défaut (cf. le commentaire
-- de `Vote::auth_method`). Les votes existants ont été émis avant que la
-- modalité ne soit tracée, donc en présence : la migration leur donne la
-- valeur que le code leur prêtait déjà. Elle décrit l'existant.

ALTER TABLE votes
    ADD COLUMN IF NOT EXISTS auth_method VARCHAR(20) NOT NULL DEFAULT 'presence';

-- Aligné sur l'enum Rust `VoteAuthMethod`
-- (`domain/copropriete/vote.rs`), paire inscrite dans
-- `garde_enum_contre_contrainte` qui compare les deux sens.
--
-- `itsme` et `eid` sont les deux authentifications FORTES au sens de
-- `is_strong()` — Art. 3.87 §1er. La contrainte est donc aussi ce qui
-- empêche d'inventer une cinquième modalité sans le dire au domaine.
ALTER TABLE votes
    ADD CONSTRAINT votes_auth_method_check CHECK (
        auth_method IN ('presence', 'proxy', 'itsme', 'eid')
    );
