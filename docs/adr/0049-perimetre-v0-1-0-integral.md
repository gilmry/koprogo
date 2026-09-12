# ADR 0049 : la v0.1.0 garde son périmètre intégral — `Must / Should / Could` ordonne, ne retire rien

- **Status**: Accepted (tranché par @gilmry le 2026-09-12)
- **Date**: 2026-09-12
- **Track**: Pilotage / Release
- **Authors**: @gilmry (décision) + Claude Opus 5 (rédaction)
- **Related**: `RELEASE.md` (registre d'état Foyer), `docs/BACKLOG_STRUCTURE_v0_1_0.md`,
  `docs/WBS_v0_1_0.md`, décision du 2026-09-06

## Contexte

La décision du 2026-09-06 avait mis l'intégralité des issues ouvertes au
périmètre du tag v0.1.0. Elle a été prise **avant** que le chiffrage existe.

Depuis, le backlog a été restructuré par capacité et chiffré : 10 épopées,
32 capacités, **73,25 j de wall-clock superviseur** et **293 tours**, en bornes
hautes de première passe. Le périmètre était donc engagé sur une intuition, et
rien ne l'avait réexaminé une fois le coût sur la table. Le pilote Foyer a
inscrit ce réexamen comme arbitrage 🔴 au registre d'état.

Trois modalités ont été présentées :

| Option | Périmètre | Issues | Jours | Tours |
|---|---|---:|---:|---:|
| **A** | tout | 84 | 73,25 | 293 |
| B | sans les `Could` | 45 | 39,75 | 159 |
| C | les `Must` seuls | 22 | 19,75 | 79 |

*(Le backlog a été généré à 85 issues ; #840 a été fermée le même jour. Les
jours et tours sont ceux du document généré, non recalculés.)*

## Décision

**Option A — le périmètre reste intégral.** Les 84 issues ouvertes restent au
tag v0.1.0. Aucune capacité n'est reportée en 0.2.0.

La décision du 2026-09-06 est donc **confirmée, et non plus seulement héritée** :
elle a maintenant été opposée à son coût.

## Pourquoi

**Le classement MoSCoW était déjà conçu pour ordonner, pas pour retirer.** Le
backlog l'écrit dans ses propres termes : « un `Could` ici veut dire *en
dernier*, pas *hors release* ». Sortir les `Could` reviendrait à utiliser un
instrument de séquencement comme un instrument de coupe — et à perdre
l'information que le rang portait.

**L'ordre rend le périmètre intégral tenable.** Le rang 1 à 8 place le harnais
de recette d'abord, puis ce qui expose des données, puis le noyau légal, et
laisse les `Could` en dernier. Un périmètre intégral **exécuté dans cet ordre**
n'a pas le même risque qu'un périmètre intégral exécuté à plat : si le temps
manque, ce qui reste à faire est déjà ce qui compte le moins, sans qu'aucune
décision de coupe ait eu à être prise à l'avance.

**Couper maintenant coûterait un revirement plus tard.** Les 73,25 j sont des
**bornes hautes de première passe**, que le CSI doit resserrer story après
story sur le réel observé. Retirer 39 issues sur la foi d'un chiffrage qu'on
sait non resserré serait trancher avec la donnée la plus faible dont on
disposera jamais.

## Conséquences

### Ce qui est engagé

- Les 84 issues restent étiquetées `release:0.1.0`. Le WBS et le backlog
  structuré n'ont pas à être régénérés : ils décrivaient déjà ce périmètre.
- L'ordre des rangs 1 à 8 devient la séquence d'exécution de la release, et
  non une suggestion.

### Ce qui reste ouvert

- **Le backlog n'est toujours pas « Agent IA Ready ».** 7 issues sur 84 portent
  les huit éléments (`scripts/backlog-pret.py`) ; 25 portent les quatre classes
  de tests. Le livrable BMAD porte `etat: NON SIGNÉ`. Trancher le périmètre ne
  prépare pas les stories : l'étape 4 de la phase A reste due.
- **Le socle n'est pas vert.** `e2e` est 🔴 (#872). La phase B du parcours exige
  un vert connu avant d'empiler la release ; le périmètre intégral ne change pas
  cette précédence, il l'aggrave.

### Ce qui rouvrirait cette décision

Le resserrement du chiffrage par le CSI. Si le réel observé écarte
significativement les bornes hautes, la question du périmètre se repose — avec,
cette fois, une donnée solide. Cette ADR n'interdit pas ce réexamen : elle
enregistre qu'à la date du 2026-09-12, avec les bornes hautes pour seule
information, la coupe n'était pas justifiée.

---

*Arbitrage conduit selon `.foyer/pilote/arbitrage.md` : l'agent a assemblé la
preuve et présenté les modalités, l'humain a tranché.*
