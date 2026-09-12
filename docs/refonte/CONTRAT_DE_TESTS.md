# Contrat de tests de la refonte UX

> « Le contrat `data-testid` n'est pas négociable. »
> — Revue de design frontend du 2026-09-06, Partie 6

Livrable du lot **#802**. À lire avant de toucher un composant.

Ce document existe parce que la suite de tests encode des **règles produit** —
RBAC, invariants légaux, cloisonnement — que les maquettes ne peuvent pas
exprimer. Renommer un identifiant, c'est effacer une règle sans s'en
apercevoir. La refonte (#797 à #803, #818, #820–#827) réécrit précisément les
écrans qui les portent.

---

## 1. Ce qui doit survivre verbatim

**956 identifiants littéraux et 26 préfixes construits**, figés dans
`frontend/src/lib/__tests__/data-testid.contrat.json` et gardés par
`garde-data-testid.test.ts`.

Le contrat ne peut que **grandir**. En retirer un exige de modifier le fichier
JSON dans le même commit, ce qui se voit en revue et appelle une
justification.

Pourquoi un fichier et pas les tests eux-mêmes : le code offre 956
identifiants, les tests n'en interrogent qu'une partie. **Les autres
disparaîtraient sans un bruit** — ils sont posés pour un usage futur ou pour
la recette humaine.

La dette d'ancrage, elle, est suivie séparément par
`garde-couverture-testid.test.ts` (#803) : figer ne sert à rien là où il n'y a
rien.

---

## 2. Les règles produit encodées, et où elles vivent

Chaque ligne est une règle que **seul un test énonce**. Si la refonte la
contredit, c'est `permissions.ts` qui change, avec le test mis à jour, un
commentaire disant la nouvelle règle et sa référence d'issue. **Jamais
l'assertion supprimée.**

| Règle | Où elle est écrite | Fondement |
|---|---|---|
| Le comptable n'a **jamais** le menu Communauté | `Navigation.test.ts @security`, `permissions.test.ts @security` | #589, Story 5.5 |
| Le copropriétaire n'a **jamais** le menu Gestion | `permissions.test.ts @security` | RBAC, ADR-0012 |
| Un rôle nul, un périmètre nul ou un menu inconnu rendent `false` — jamais d'exception | `permissions.test.ts @negative` | fail-closed |
| Aucune escalade par un rôle inférieur | `permissions.test.ts @security` | RBAC strict |
| Le syndic voit **exactement cinq** menus métier | `Navigation.test.ts @happy` | ADR-0012 |
| Un utilisateur sans aucun `UserRoleAssignment` voit un état vide explicite | `Navigation.test.ts @negative` | #814 |
| Un rôle servi par le serveur voit un menu **ou** figure au registre des rôles sans interface | `garde-roles.test.ts` | #814 |
| Aucun rôle n'est remplacé par un autre au passage du store | `garde-roles-non-travestis.test.ts` | #836 |
| Le copropriétaire ne voit **pas** le sélecteur de périmètre | `building-selector.spec.ts @security` | cloisonnement |
| Le membre du conseil voit Mes lots, Communauté et Gouvernance — et rien d'autre | `permissions.ts` `BOARD_MENUS` | #816, Art. 3.90 §§ 1er et 2 |
| Le prestataire ne voit **aucun** menu | `ROLES_SANS_INTERFACE` | #815, décision du 2026-09-07 |

**98 assertions `@security`, 96 `@negative`, 111 `@happy`, 113 `@edge`**
réparties sur 36 fichiers de test unitaire.

### Le conflit que la revue signale elle-même

Sa barre latérale du comptable inclut un groupe Communauté réduit. Le test dit
l'inverse, et la revue tranche : « The test wins unless the product owner says
otherwise ».

**Le test a raison, et pas seulement par défaut** : #589 (Story 5.5) pose que
le comptable, encodeur comme émetteur, reçoit un 403 sur `/community/*`. La
maquette contredit une story du périmètre, pas seulement une assertion.

---

## 3. Les deux entrées épinglées ne sont pas des menus

`Navigation.test.ts @happy` affirme que le syndic voit **exactement cinq**
menus métier. Les entrées épinglées de la nouvelle barre latérale ne doivent
donc pas recevoir de `navigation-menu-*` (#799) : elles feraient échouer une
règle qu'elles ne contredisent pas.

Un identifiant mal choisi ne casse pas seulement un test — il fait dire au
contrat quelque chose de faux.

---

## 4. La zone la plus risquée : le passage à l'ACP (#798)

| Surface | Fichiers |
|---|---|
| Unitaire | `BuildingSelector.test.ts`, `ContextBanner.test.ts`, `permissions.test.ts`, `Navigation.test.ts` |
| Périmètre | `stores/scope.svelte.ts` |
| e2e | `slice-1-acp-refacto/` (1), `slice-2-selector-banner/` (2), `fix-admin-buttons-acp/` (2), `track-h/` (3) |

**Stratégie de transition, en trois temps :**

1. garder les deux périmètres côte à côte ;
2. livrer `AcpSelector` avec des identifiants **neufs** — pas de réemploi de
   `building-selector-*`, dont le contrat doit rester intact tant que le
   chemin par immeuble existe ;
3. retirer le chemin par immeuble dans un **commit séparé**, où la suppression
   des identifiants correspondants se voit.

Réemployer les identifiants existants ferait passer les tests pendant la
transition tout en changeant ce qu'ils vérifient. C'est le pire des deux
mondes : vert et faux.

---

## 5. Définition de « fini »

Un lot de refonte est fini quand **toutes** ces conditions tiennent :

- `garde-data-testid` verte, ou le contrat JSON modifié dans le même commit
  avec une justification écrite ;
- `garde-couverture-testid` **en baisse** — celui qui touche un écran l'ancre
  (#803) ;
- `garde-libelles-en-dur` **en baisse** — celui qui touche un écran le traduit
  (#834) ;
- `garde-roles` et `garde-roles-non-travestis` vertes ;
- `svelte-check --fail-on-warnings` à 0 erreur **et** 0 warning ;
- les assertions `@security` du périmètre touché toujours présentes, ou
  modifiées avec la nouvelle règle écrite dans `permissions.ts` et une
  référence d'issue.

---

## 6. La règle de méthode

> « À chaque étape : jouer les suites, adapter les spécifications, et ajouter
> les tests manquants **dans le même commit que le changement** — pas en passe
> de nettoyage après coup. »

C'est la règle du dépôt, et elle vaut ici plus qu'ailleurs : une refonte qui
reporte l'adaptation des tests avance en aveugle sur les règles qu'elle
déplace.
