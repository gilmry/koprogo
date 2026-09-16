-- Retour à `NOT NULL`.
--
-- ⚠️ Cette descente ÉCHOUERA si des liens sans sujet existent — et c'est
-- voulu. Leur inventer un utilisateur pour satisfaire la contrainte
-- fabriquerait une identité que personne n'a créée ; les supprimer
-- révoquerait des accès en cours. Les deux sont pires qu'un échec explicite.
ALTER TABLE magic_links ALTER COLUMN subject_user_id SET NOT NULL;
