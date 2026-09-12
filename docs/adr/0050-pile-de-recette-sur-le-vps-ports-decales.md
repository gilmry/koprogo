# ADR 0050 : la pile de recette tourne sur le VPS, à ports décalés

- **Status**: Accepted (tranché par @gilmry le 2026-09-12)
- **Date**: 2026-09-12
- **Track**: Infrastructure / Testing
- **Authors**: @gilmry (décision) + Claude Opus 5 (rédaction)
- **Related**: #872, #832, `cap:C7.1` (rang 1), [ADR 0049](0049-perimetre-v0-1-0-integral.md)

## Contexte

`cap:C7.1` — « la recette peut se connecter et s'exécuter » — est au rang 1 de
l'ordre de la release : sans elle, aucune capacité ne peut être *déclarée*
tenue. Elle est bloquée, et #832 avec elle.

La cause était double. La première est fermée : `docker-compose.yml` ne posait
pas de `name:`, la pile de développement résolvait donc vers les conteneurs
**et le volume** de la démo vivante. Le commit `d35332de` a posé
`name: koprogo-dev` face à `name: koprogo`, avec la garde
`garde-piles-compose-distinctes`.

La seconde reste : `make test-e2e` vise `http://localhost`, qui sur cet hôte
traverse Traefik jusqu'à `Host(api.koprogo.com)` — le backend de la démo.

### Pourquoi la recette ne peut pas viser la démo

Trois mesures, prises le 2026-09-12 :

| Fait | Mesure |
|---|---|
| La suite **écrit** | 480 appels `POST`/`PUT`/`DELETE`, sur 56 specs et 12 scénarios |
| Elle **se reconnecte sans cesse** | 130 connexions sur `/api/v1/auth/login`, limité à 5 req/min — un bannissement CrowdSec de l'adresse source le 2026-09-01 |
| Elle **exige un état initial connu** | `make seed-reset` POSTe sur `/api/v1/seed/scenario/world` ; `make reset-db` annonce « SUPPRIME TOUTES LES DONNÉES » |

Le troisième est décisif et n'est pas mitigeable : la **précondition de
reproductibilité** de la recette est exactement ce qui détruirait la démo. Ce
n'est pas un risque qu'on entoure de précautions, c'est la procédure elle-même.

L'hôte `ecosolva` porte par ailleurs 31 conteneurs pour cinq projets
(`koprogo`, `derniere-chance`, `sluis`, `klaar`, `taginy`) : un bannissement ou
un épuisement mémoire se paie en partie chez les voisins.

## Décision

**La pile de recette tourne sur le VPS, à côté de la démo, sous le projet
`koprogo-dev` déjà séparé, avec ses propres volumes et des ports décalés.**

Ports retenus — vérifiés libres sur l'hôte le 2026-09-12 (occupés : 22, 25, 53,
80, 443, 5432, 8095, 9000, 9001, 9090, 9093, 9100) :

| Service | Démo | Recette |
|---|---|---|
| HTTP (Traefik) | 80 | **8090** |
| Dashboard Traefik | — | **8091** |
| PostgreSQL | 5432 (localhost) | **15432** (localhost) |
| MinIO S3 / console | 9000 / 9001 (localhost) | **19000 / 19001** (localhost) |

`make test-e2e` cesse de viser `http://localhost` et vise `http://localhost:8090`.

## Pourquoi celle-là plutôt qu'une autre

**Parce que le gros du travail est déjà commité.** Le choix se présentait comme
« remettre deux piles sur un hôte partagé » ; c'était une lecture périmée. La
séparation des projets, des volumes et des réseaux est faite et gardée. Ce qui
restait n'était pas une architecture à inventer, mais **quatre numéros de
ports**.

**Parce que la recette a besoin d'être rejouable à la main.** #832 demande de
départager, sur quinze specs en échec, « la page ne charge pas » de « la
condition d'affichage n'est pas remplie » de « c'est une question de délai ».
Cela se tranche en rejouant, pas en relisant des artefacts de build. Une pile
persistante donne cette prise ; une pile de CI seule ne la donne pas.

**Parce que la doc vivante a besoin d'une URL.** Le Track T4 — 14 issues,
11,25 j — filme un parcours de référence. Il lui faut une cible stable.

### Ce qui a été écarté

- **CI seulement** : aucune ressource prise à l'hôte, mais pas d'URL persistante,
  donc pas de rejouabilité manuelle pour #832 ni de cible pour T4.
- **Hôte séparé** : isolation franche, mais coûte une machine à provisionner
  alors que `cap:C9.2` (IaC) est classée `Could`, donc en dernier ([ADR 0049](0049-perimetre-v0-1-0-integral.md)).
- **VPS + CI** : le plus complet et deux fois la maintenance — deux définitions
  à garder en phase, donc un endroit de plus où la dérive s'installe. Rien
  n'interdit d'y venir plus tard, une fois la pile du VPS stable.

## Conséquences

### À faire (livrables 2, 3 et 5 de #872)

- Décaler les quatre ports dans `docker-compose.yml`.
- `make test-e2e` et `make docs-with-videos` visent `http://localhost:8090`.
- `docs/E2E_TESTING_GUIDE.rst` corrigé : il présente aujourd'hui la commande
  dangereuse comme la façon normale de lancer la suite.
- Étendre `garde-piles-compose-distinctes` — ou lui adjoindre une garde — pour
  qu'aucune pile suivie ne réclame un port déjà tenu par une autre.

### Le coût assumé

**RAM et CPU partagés avec 31 conteneurs.** C'est la contrepartie réelle de ce
choix, et elle n'est pas gardée : rien ne mesure aujourd'hui la contention. Si
la recette dégrade la démo ou ses voisins, c'est ce qui rouvrira cette décision
— et l'option « hôte séparé » sera alors la suite naturelle.

### Ce que cela ne résout pas

La suite continuera de marteler `/api/v1/auth/login`. Sur la pile de recette
c'est sans conséquence pour la démo, mais 130 connexions contre une limite de
5 req/min restent 130 connexions : le limiteur devra être desserré sur la
recette, ou les scénarios devront partager leurs sessions. À traiter dans #832.
