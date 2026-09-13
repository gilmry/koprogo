-- La reprise supprime les incidents transverses, qui n'ont pas d'organisation
-- à laquelle les rattacher. C'est destructif et c'est assumé : ce sont
-- précisément les lignes que la contrainte interdisait.
DELETE FROM security_incidents WHERE organization_id IS NULL;
ALTER TABLE security_incidents ALTER COLUMN organization_id SET NOT NULL;
