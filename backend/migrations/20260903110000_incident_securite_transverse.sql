-- Un incident de sécurité peut ne relever d'aucune organisation.
--
-- RGPD, article 33 : la notification de violation incombe au **responsable du
-- traitement**. Pour une violation qui touche une seule copropriété, c'est son
-- syndic ; pour une violation de la plateforme elle-même — une fuite de base,
-- une compromission d'infrastructure — c'est l'exploitant, qui n'est rattaché
-- à aucune organisation.
--
-- Le domaine le modélise correctement depuis toujours :
-- `SecurityIncident.organization_id` est un `Option<Uuid>`, et le dépôt
-- distingue les deux cas dans ses requêtes. C'est la colonne qui n'a jamais
-- suivi.
--
-- Conséquence mesurée : un superadmin — seul rôle autorisé sur ces endpoints,
-- et le seul dont le jeton porte `organization_id = NULL` — déclenchait une
-- violation de contrainte NOT NULL à chaque opération. Les endpoints de
-- notification de violation de données étaient donc inutilisables par la seule
-- personne censée s'en servir.
--
-- Voir #722.

ALTER TABLE security_incidents ALTER COLUMN organization_id DROP NOT NULL;

COMMENT ON COLUMN security_incidents.organization_id IS
    'Organisation concernée, ou NULL pour un incident transverse à la plateforme (RGPD art. 33).';
