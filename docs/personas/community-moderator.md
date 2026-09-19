---
persona: "Modérateur communauté (community.moderator)"
role_code: "community.moderator"
statut: brouillon
population_recette: 0
eprouve: false
date: "2026-09-13"
version: "0.1"
superviseur: null
signature_humaine: null
issue_cadre: 805
issue_parcours: null
videos: []
---

# Modérateur communauté

Modère les modules communautaires (SEL, partage d'objets, annonces,
compétences, réservations). **Non éprouvé** : aucun compte réel n'a ce
rôle en base au 2026-09-13.

À la différence de `admin.md`, ce rôle n'a **pas non plus d'interface
distincte** au 2026-09-13 : aucune ancre `data-testid` de modération n'a
été trouvée dans le contrat gelé (`frontend/src/lib/__tests__/data-testid.contrat.json`).
Le code ne fait, aujourd'hui, que le mentionner comme équivalent au rôle
`owner` pour la visibilité du menu communauté
(`frontend/src/lib/auth/permissions.ts` : « `community.moderator` — comme
owner pour `communaute` ») et prévoit une évolution future (« Sub-rôle
`community.moderator` aura des actions modération en plus »). C'est donc un
rôle **prévu, non construit** — moins avancé que `admin`, qui a au moins
son interface.

## Parcours nominal

Non éprouvé et sans interface de modération dédiée au 2026-09-13 : aucun
parcours n'est affirmé ici, ni même les ancres qui le délimiteraient. Le
document sera complété quand une story lui donnera un contenu réel
(cf. « Ce qui ne marche pas encore »).

## Ce que ce rôle ne peut pas faire, et pourquoi

- Ne dispose d'**aucune action de modération distincte** de celles d'un
  `owner` aujourd'hui — le rôle existe dans le type `Role` mais son
  différentiel fonctionnel n'est pas encore implémenté.

  **Correction du 2026-09-19, et elle est plus sévère.** Cette phrase était
  optimiste : il ne dispose pas non plus des actions d'un `owner`. Mesuré en
  écrivant son parcours filmé (#805), sur un compte réel :

  ```
  [RouteGuard] Access denied to /notices for role community.moderator
  ```

  **Les sept routes du groupe `communaute` le refusent** — `/exchanges`,
  `/polls`, `/notices`, `/bookings`, `/sharing`, `/skills`,
  `/energy-campaigns`. Vérifié une par une par
  `moderation.spec.ts::@security`.

  `permissions.ts:212` annonce pourtant « community.moderator → comme owner
  pour `communaute` », et pilote les MENUS. `guards.ts` pilote l'ACCÈS, et
  ses sept entrées ne listent que superadmin, syndic, comptable et
  copropriétaire. **Le rôle voit une porte qu'il ne peut pas franchir**
  (#962).

  Ce n'est donc pas « un rôle prévu, non construit » : c'est un rôle
  construit dans un fichier et oublié dans l'autre. Le backend, lui, le
  délivre bien — inscription en 201.

## Références légales

Aucune : la modération des modules communautaires est une fonctionnalité
produit, pas une qualité reconnue par le Code civil de la copropriété.

## Ce qui ne marche pas encore

Rôle non éprouvé et sans interface de modération construite. Nommer une
étape rouge précise supposerait une capacité qui n'existe pas encore à
casser ; ce n'est donc pas une étape rouge au sens de #805, mais une
capacité **non commencée**. À suivre par la story qui l'implémentera
(#806-#809, #815 ou #816, association à confirmer par le PO).

## Comptes de recette

Aucun persona nommé dans `docs/specs/00-personas-et-seed.rst` ne porte ce
rôle. Les personas communautaires du seed (Ahmed Mansouri, Sophie Martin,
Lucas Martin, Fatima El Amrani) participent à la communauté mais n'en
modèrent aucun module. Un compte de recette dédié reste à créer avant
toute vidéo ou capture publiée.
