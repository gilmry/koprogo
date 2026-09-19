-- Story 5.1 (#585) — registre des modules activables par ACP (ADR-0015).
--
-- Une ACP n'allume que les capacités dont elle a besoin : KoproGo est une
-- boîte à outils, pas un monolithe imposé.
--
-- INV-27 — la désactivation est RÉVERSIBLE et ne détruit rien. C'est pour
-- cela qu'il y a une ligne par couple (acp, module) et une colonne
-- `archived_at` que l'on éteint et rallume, plutôt qu'un DELETE : réactiver
-- un module doit retrouver les données telles qu'elles étaient. Un DELETE
-- aurait rendu la désactivation irréversible dans les faits, donc
-- inacceptable pour un utilisateur qui hésite.
--
-- `archived_at IS NULL` = module actif. C'est la seule lecture de l'état.

CREATE TABLE IF NOT EXISTS acp_enabled_modules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    acp_id UUID NOT NULL REFERENCES acps(id) ON DELETE CASCADE,
    module VARCHAR(50) NOT NULL,
    enabled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    archived_at TIMESTAMPTZ,
    -- Les valeurs doivent rester alignées sur l'enum Rust `Module`
    -- (`domain/copropriete/acp_enabled_module.rs`). La garde
    -- `garde_enum_contre_contrainte` compare les deux listes dans les DEUX
    -- sens et casse si l'une devance l'autre.
    CONSTRAINT acp_enabled_modules_module_check CHECK (
        module IN (
            'identity',
            'community',
            'ticketing',
            'accounting',
            'governance',
            'maintenance',
            'portfolio'
        )
    )
);

-- Un seul enregistrement par couple : le cycle activer/désactiver/réactiver
-- réutilise la ligne au lieu d'en empiler une par bascule. C'est ce qui rend
-- `archived_at` porteur de l'historique plutôt que d'un instantané.
CREATE UNIQUE INDEX IF NOT EXISTS acp_enabled_modules_acp_module_unique
    ON acp_enabled_modules (acp_id, module);

-- Lecture chaude : « quels modules sont actifs pour cette ACP ». Servie à
-- chaque requête passant par `ModuleGuard`, donc indexée.
CREATE INDEX IF NOT EXISTS acp_enabled_modules_actifs
    ON acp_enabled_modules (acp_id)
    WHERE archived_at IS NULL;

-- Remplissage des ACPs EXISTANTES avec **tous** les modules.
--
-- Ce n'est pas une facilité, c'est la seule valeur qui ne change rien. Une
-- ACP déjà en service a aujourd'hui accès à toutes les capacités : n'inscrire
-- que `identity` ferait de cette migration une extinction de masse le jour où
-- un garde lira ce registre. Le registre doit naître en décrivant l'existant,
-- pas en le redéfinissant.
--
-- Les ACPs créées APRÈS cette migration n'héritent de rien : c'est
-- l'assistant d'embarquement qui pose leur sélection, module par module.
INSERT INTO acp_enabled_modules (acp_id, module)
SELECT acps.id, m.module
FROM acps
CROSS JOIN (
    VALUES
        ('identity'),
        ('community'),
        ('ticketing'),
        ('accounting'),
        ('governance'),
        ('maintenance'),
        ('portfolio')
) AS m(module)
ON CONFLICT DO NOTHING;
