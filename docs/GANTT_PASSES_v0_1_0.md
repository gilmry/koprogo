---
livrable: Gantt en passes d'agent (pilotage — persona chef de projet)
projet: KoproGo
jalon: v0.1.0
genere_par: scripts/gantt-passes.py
signature_humaine:
  date: 2026-09-12
  nom: Gilles Maury
  role: Product Owner / superviseur
  etat: VALIDÉ — avec amendement d'orchestration multiagent
  amendement: >-
    Le ratio_supervision ne bride plus l'éventail. Le répondre-de est porté
    par la revue de promotion de branche, instruite par les gates et la
    vitrine. Orchestration retenue : fan-out en CI.
  reserve_inscrite: >-
    Le mécanisme de preuve n'est pas encore opérationnel (e2e et doc-vivante
    rouges, #872). Et aucune voie d'authentification ne donne du parallélisme
    gratuit : l'API se paie en argent, l'abonnement en débit partagé.
---

# Gantt de la v0.1.0 — en passes d'agent

*Généré par `scripts/gantt-passes.py` : ne pas éditer à la main, la
prochaine génération écraserait la correction.*

> **L'axe n'est pas un calendrier.** Une *passe d'agent* est un tour de
> boucle supervisé : l'agent produit, le superviseur relit, le tour est
> clos. Dater un plan qu'on ne tiendra pas produit un document que plus
> personne ne lit au troisième glissement ; ordonner sans dater reste
> vrai plus longtemps.

## Ce que le diagramme dit, et ce qu'il ne dit pas

Il **ordonne**, il ne **prédit** pas. Les dépendances sont déclarées
dans `scripts/gantt-passes.py`, chacune avec sa raison écrite, à partir
de ce que la story de l'issue dit d'elle-même — jamais déduites du
code. Une dépendance oubliée ici devient un parallélisme optimiste dans
le résultat : le graphe est une **hypothèse falsifiable**, à corriger
dès qu'une passe réelle la contredit.

Le plafond de parallélisme retenu est **3 chantiers
de front** par binôme — le `ratio_supervision` de l'abaque. Ce n'est pas
une variable budgétaire mais le plafond du *répondre-de* : au-delà, on
ne peut plus répondre de ce qui est produit. C'est un **prior**
`[caler]`, à remplacer par du mesuré dès qu'on aura tourné quelques
passes réelles.

## Le diagramme — ce qui est *possible*

Colonnes : couches 1 à 7. `█` = la capacité est ouvrable,
`·` = elle attend.

```text
capacité  rang moscow  1234567
--------- ---- ------- -------
C7.3         0 Must    ███····
C7.1         1 Must    ···██··
C4.1         2 Must    ···██··
C4.2         2 Must    ···█···
C4.3         2 Must    ···█···
C10.1        3 Must    ···█···
C5.1         4 Should  ···█···
C5.2         4 Must    ···█···
C1.1         5 Must    ···██··
C1.3         5 Must    ···██··
C1.2         6 Should  ···███·
C1.5         6 Should  ···█···
C2.1         6 Should  ···█···
C2.2         6 Should  ···█···
C3.1         6 Should  ···██··
C4.4         6 Should  ···█···
C4.5         6 Should  ···██··
C6.2         6 Should  ···██··
C9.1         6 Should  ···██··
C9.3         6 Should  ···█···
C6.1         6 Should  ····██·
C7.2         6 Should  ····█··
C2.3         7 Could   ···█···
C3.3         7 Could   ···███·
C5.3         7 Could   ···██··
C8.1         7 Could   ···███·
C8.4         7 Could   ···█···
C9.2         7 Could   ···██··
C9.4         7 Could   ···█···
C1.4         7 Could   ····██·
C3.2         7 Could   ····██·
C8.2         7 Could   ····██·
C8.3         7 Could   ······█
```

## Points de concours et de divergence

**Divergence** — ce qui débloque le plus de chantiers en aval. Ce sont
les issues à ne pas laisser traîner : chacune tient une file.

| Issue | Capacité | Couche | Chantiers débloqués |
|---|---|---:|---:|
| `#803` | C5.2 | 4 | **11** |
| `#805` | C8.1 | 4 | **10** |
| `#797` | C5.1 | 4 | **9** |
| `#802` | C5.2 | 4 | **9** |
| `#872` | C7.1 | 4 | **5** |
| `#780` | C1.1 | 5 | **4** |

**Concours** — ce qui attend plusieurs chemins. Ce sont les points où
un retard sur *n'importe laquelle* des amont décale l'aval.

| Issue | Attend |
|---|---|
| `#813` | `#806`, `#807`, `#808`, `#809`, `#815`, `#816`, `#810`, `#811`, `#812`, `#817` |
| `#818` | `#797`, `#802`, `#803` |
| `#820` | `#797`, `#802`, `#803` |
| `#821` | `#797`, `#802`, `#803` |
| `#822` | `#797`, `#802`, `#803` |
| `#823` | `#797`, `#802`, `#803` |

## Ce que la largeur révèle

**3 des 7 couches dépassent le plafond de 3** : L4 (39 chantiers), L5 (36 chantiers), L6 (8 chantiers).

C'est le résultat le plus utile du diagramme, et il est
contre-intuitif : **les dépendances ne sont pas le goulot.** Les
chaînes sont courtes — 7 couches seulement — et l'essentiel
du travail est parallélisable. Ce qui borne la release n'est donc
pas l'ordre des choses, c'est la capacité à *répondre de* ce qui est
produit.

Deux issues, et une seule est bonne :

- **Étaler** — le plan sous contrainte ci-dessous. C'est le défaut,
  et il est honnête.
- **Ajouter un pair** — coût en marche d'escalier, pas en pente.
  C'est ce que l'abaque appelle un investissement wall-clock dans la
  transmissibilité, pas un surcoût à raboter.

Ce qu'il ne faut **pas** faire est réduire le binôme à une personne
seule pour tenir la largeur : on gagne du wall-clock et on rachète
du *bus factor* 1.

## Le plan d'orchestration — multiagent, parallélisme maximal

**Amendement du 2026-09-12.** Le `ratio_supervision` ne bride plus
l'éventail : le *répondre-de* est porté par la **revue de promotion de
branche**, instruite par les gates et la vitrine. On ne supervise plus
des agents en direct, on relit une preuve attachée à une branche.

Ce qui bride encore, et qui est **physique** :

0. **La story habilitante** — `C7.3` est une **barrière**, pas une
   dépendance parmi d'autres. Tant qu'elle n'est pas close, le fan-out
   n'a ni preuve à produire ni mécanisme prouvé. La Méthode Foyer :
   « elle bloque le reste du backlog tant qu'elle n'est pas fermée ».
1. **Les dépendances** — une vague ne s'ouvre qu'une fois l'amont
   fusionné.
2. **Les conflits d'écriture** — deux agents dans le même domaine se
   marchent dessus. Un agent par domaine et par créneau, chacun dans
   son *worktree*.
3. **La concurrence de l'hôte** — `min(16, CPU-2)` = **2** sur cette machine. Mesurée, pas supposée.

Résultat : **7 vagues**, **26 créneaux**, largeur
maximale **10 agents simultanés** — contre 29
passes en séquentiel supervisé.

> ⚠️ **Le goulot n'est plus le plan, c'est l'hôte.** La largeur
> demandée est 10 ; la machine en tient 2. Un
> créneau large s'exécutera donc en plusieurs vagues réelles, ou
> ailleurs — agents distants, ou hôte plus gros. C'est le premier
> chiffre à caler avant de lancer l'expérimentation.

### Vague 1 — **habilitation**

> **Exécutée en session, pas par le fan-out.** Les habilitantes
> sont toutes dans le domaine `harnais` : le fan-out les
> sérialiserait sans gain. Et c'est un œuf et une poule — la
> valeur du fan-out est que les gates et la vitrine instruisent
> la revue, et ce sont précisément eux qu'on construit ici.

| Créneau | Agents | Domaines |
|---|---|---|
| V1.1 | 1 — #873 | `harnais` |

### Vague 2 — **habilitation**

> **Exécutée en session, pas par le fan-out.** Les habilitantes
> sont toutes dans le domaine `harnais` : le fan-out les
> sérialiserait sans gain. Et c'est un œuf et une poule — la
> valeur du fan-out est que les gates et la vitrine instruisent
> la revue, et ce sont précisément eux qu'on construit ici.

| Créneau | Agents | Domaines |
|---|---|---|
| V2.1 | 1 — #876 | `harnais` |

### Vague 3 — **habilitation**

> **Exécutée en session, pas par le fan-out.** Les habilitantes
> sont toutes dans le domaine `harnais` : le fan-out les
> sérialiserait sans gain. Et c'est un œuf et une poule — la
> valeur du fan-out est que les gates et la vitrine instruisent
> la revue, et ce sont précisément eux qu'on construit ici.

| Créneau | Agents | Domaines |
|---|---|---|
| V3.1 | 1 — #874 | `harnais` |

### Vague 4

| Créneau | Agents | Domaines |
|---|---|---|
| V4.1 | 10 — #872, #694, #798, #576, #515, #869, #781, #852, #805, #425 | `harnais`, `back/plateforme`, `front/composants`, `back/copropriete`, `iac`, `front/mobile-a11y`, `back/communaute`, `back/comptabilite`, `docs-vivante`, `meta` |
| V4.2 | 10 — #870, #864, #841, #850, #432, #871, #585, #429, #635, #854 | `harnais`, `back/plateforme`, `front/composants`, `back/copropriete`, `iac`, `front/mobile-a11y`, `back/communaute`, `meta`, `back/comptabilite`, `docs-vivante` |
| V4.3 | 6 — #842, #847, #835, #453, #556, #595 | `front/composants`, `back/copropriete`, `back/plateforme`, `iac`, `meta`, `docs-vivante` |
| V4.4 | 4 — #868, #848, #762, #731 | `front/composants`, `back/copropriete`, `back/plateforme`, `iac` |
| V4.5 | 3 — #856, #579, #354 | `front/composants`, `back/copropriete`, `iac` |
| V4.6 | 2 — #803, #855 | `front/composants`, `back/copropriete` |
| V4.7 | 1 — #802 | `front/composants` |
| V4.8 | 1 — #797 | `front/composants` |
| V4.9 | 1 — #834 | `front/composants` |
| V4.10 | 1 — #867 | `front/composants` |

### Vague 5

| Créneau | Agents | Domaines |
|---|---|---|
| V5.1 | 8 — #696, #845, #780, #865, #718, #779, #807, #818 | `harnais`, `back/plateforme`, `back/copropriete`, `front/mobile-a11y`, `iac`, `back/communaute`, `docs-vivante`, `front/composants` |
| V5.2 | 8 — #832, #577, #555, #866, #587, #808, #355, #820 | `harnais`, `back/copropriete`, `back/plateforme`, `front/mobile-a11y`, `back/communaute`, `docs-vivante`, `iac`, `front/composants` |
| V5.3 | 6 — #581, #427, #586, #809, #823, #466 | `back/copropriete`, `harnais`, `back/communaute`, `docs-vivante`, `front/composants`, `iac` |
| V5.4 | 4 — #846, #811, #824, #590 | `back/copropriete`, `docs-vivante`, `front/composants`, `back/communaute` |
| V5.5 | 3 — #812, #583, #825 | `docs-vivante`, `back/copropriete`, `front/composants` |
| V5.6 | 2 — #815, #826 | `docs-vivante`, `front/composants` |
| V5.7 | 2 — #816, #821 | `docs-vivante`, `front/composants` |
| V5.8 | 2 — #817, #822 | `docs-vivante`, `front/composants` |
| V5.9 | 1 — #827 | `front/composants` |

### Vague 6

| Créneau | Agents | Domaines |
|---|---|---|
| V6.1 | 4 — #578, #592, #806, #591 | `back/copropriete`, `front/mobile-a11y`, `docs-vivante`, `back/communaute` |
| V6.2 | 3 — #810, #582, #588 | `docs-vivante`, `back/copropriete`, `back/communaute` |
| V6.3 | 1 — #589 | `back/communaute` |

### Vague 7

| Créneau | Agents | Domaines |
|---|---|---|
| V7.1 | 1 — #813 | `docs-vivante` |

## Parallélisme maximal — par rapport à l'hôte

Mesuré le 2026-09-12 sur `ecosolva`. Ce ne sont pas des ordres de
grandeur : ce sont les chiffres de la machine qui orchestrerait.

| Ressource | Mesure | Agents qu'elle permet |
|---|---|---:|
| CPU | 4 cœurs, charge 1.97 (~2.0 libres) | **2** |
| RAM | 10 Go disponibles sur 14 | confortable |
| Disque | 16 Go libres, 600 Mo par worktree | ~27 |
| Build Rust | volume docker PARTAGÉ (rustbuild-target-koprogo) | **1 à la fois** |

**Le plafond est le CPU, et il vaut 2** : Claude Code borne les agents concurrents à
`min(16, CPU − 2)`, soit `min(16, 4 − 2)`. Le disque en
permettrait ~27, la RAM aussi — ils ne servent à rien.

Deux aggravations que la formule ne voit pas :

- **L'hôte n'est pas dédié.** Il porte 31 conteneurs
  pour 10 projets, dont la démo KoproGo. La
  charge est déjà à 1.97 sur 4 cœurs :
  la moitié de la machine est prise avant qu'un seul agent démarre.
- **Le `target` Rust est un volume Docker partagé.** Deux agents qui
  compilent en même temps se bloquent sur le verrou de `cargo`, quel
  que soit le nombre de worktrees. Le parallélisme backend est donc
  **de 1** tant que chaque agent n'a pas son propre `target`.

### Le verdict, et il est inconfortable

Le plan demande une largeur de **10**. L'hôte en tient
**2**. L'expérimentation s'exécuterait donc à **un
cinquième** de la largeur pour laquelle elle est conçue :
~44 créneaux réels au lieu de 26.

À 2 de front, le parallélisme n'apporte presque rien :
le gain vient alors de la **suppression de l'attente humaine entre
passes**, pas du parallélisme lui-même. C'est un vrai gain — mais ce
n'est pas l'expérience qu'on voulait mener.

**Pour tenir la largeur demandée**, trois voies, par coût croissant :

| Voie | Ce qu'il faut | Ce que ça coûte |
|---|---|---|
| Hôte plus gros | ≥ 12 cœurs (`min(16, n−2) ≥ 10`) | une machine |
| Agents distants | orchestration en nuage | facturation à l'usage |
| Fan-out en CI | un job par story | temps de CI, pas de worktree |

La troisième est **retenue et implémentée** :
`.github/workflows/fanout-stories.yml`. Elle ne demande pas de machine,
isole naturellement les `target` Rust, et produit déjà les artefacts —
gates et vitrine — que la revue de promotion attend. Le parallélisme y
est borné par les *runners*, pas par cet hôte.

### Ce que le fan-out coûte, et c'est un choix

Deux voies d'authentification, et **aucune ne donne du parallélisme
gratuit** :

| Voie | Facturation | Ce qui borne |
|---|---|---|
| `CLAUDE_CODE_OAUTH_TOKEN` *(défaut)* | l'abonnement | les limites de débit, **partagées avec les sessions interactives** |
| `ANTHROPIC_API_KEY` | à l'usage | le budget |

Ordre de grandeur pour les 84 stories, reprises comprises — hypothèses
visibles : ~5 M tokens d'entrée par story dont ~90 % en lecture de
cache, ~80 k en sortie. **À caler sur la première vague réelle.**

| Modèle | par story | 84 stories | avec reprises (×1,5) |
|---|---:|---:|---:|
| Haiku 4.5 | ~1,4 $ | ~115 $ | **~170 $** |
| Sonnet 5 | ~2,7 $ | ~227 $ | **~340 $** |
| Opus 5 | ~6,8 $ | ~567 $ | **~850 $** |

Le jeton d'abonnement évite la facture mais pas la contrainte : un
fan-out large consomme les limites de débit et **ralentit le travail
humain en cours**. Le défaut de `max_parallel` est donc **2**, à monter
une fois la première vague mesurée — pas avant.

## Le protocole — une branche, une preuve, une revue

Chaque agent travaille dans un **worktree isolé** et livre une branche
`story/<issue>`. La promotion vers `feature/dev` est le gate, et elle
exige **trois preuves attachées à la branche** :

| Preuve | Gate | Bloquant |
|---|---|---|
| Correctness | `e2e` sur la pile de recette | oui |
| Les quatre classes | `unit` + `integration` + `bdd` | oui |
| Valeur | **vitrine** — parcours filmé | non bloquant, **non facultatif** |

Le parcours Foyer est explicite : la doc vivante est « une preuve, pas
un verrou » — mais une story full-stack sans sa preuve de valeur **n'est
pas terminée**. C'est elle qui rend la revue de promotion possible sans
relire le diff ligne à ligne : le relecteur regarde le film et les
gates, pas le code.

## ⚠️ La précondition — le filet avant le saut

**Le mécanisme choisi pour porter le répondre-de n'est pas
opérationnel.** Au 2026-09-12 :

| Gate | État | Cause |
|---|---|---|
| `e2e` | 🔴 | vise la démo via Traefik — #872 |
| `doc-vivante` (vitrine) | 🔴 | `make docs-with-videos` — #872 |

Lancer 84 chantiers en parallèle avant que la preuve existe reviendrait
à produire 84 branches que **rien ne permet de relire**. Le parallélisme
n'est pas risqué en soi : il l'est quand son filet n'est pas tendu.

**V1 n'est donc pas une formalité, c'est ce qui rend le reste**
**légitime.** L'ADR 0050 — décaler les ports, pile de recette jetable —
est la condition d'existence de l'expérimentation, pas sa première
étape parmi d'autres.

## Coût

| Axe | Valeur | Ce que ça mesure |
|---|---:|---|
| Issues | 87 | le périmètre, intégral (ADR 0049) |
| Passes séquentielles | 29 | régime supervisé, 3 de front |
| Créneaux multiagent | 26 | régime parallèle, revue à la promotion |
| Jours | 74.75 | wall-clock **superviseur** |
| Tours | 299 | coût **tokens** |

L'abaque est formelle sur la lecture de ces deux dernières lignes : le
**poste dominant est le superviseur, pas le modèle**. Optimiser les
tokens ne déplace presque rien. Ce sont donc les jours qu'il faut
regarder, et le seul levier qui les réduise sans rien racheter est de
retirer du périmètre — ce que l'ADR 0049 a explicitement refusé de
faire.

**Ces chiffres sont des bornes hautes de première passe.** Le CSI doit
les resserrer story après story sur le réel observé. Les publier non
resserrés est le seul moyen d'avoir un point de départ falsifiable ;
les publier comme un engagement serait une faute.

## Cadre de delivery

Largeur maximale *possible* : **39 chantiers simultanés**. Largeur
*retenue* : **3**.

Cela tient dans **une seule équipe** : Scrum suffit, et Nexus serait
une cérémonie sans objet. Le passage à Nexus se justifierait à
partir de trois équipes sur le produit.

Le rescaling du barreau (Scrum → Nexus → SAFe) est un **point
irréversible** que la persona chef de projet porte en ADR, validé
par l'humain. Rien ne l'appelle aujourd'hui : ce qui manque n'est
pas de la coordination inter-équipes, c'est de la capacité de
supervision dans une seule.

## Ce que le Gantt ne couvre pas

- **La qualité des stories.** 84/84 « Agent IA Ready » est un contrôle
  de forme. Le plan ordonne ce qui est écrit, pas ce qui est bon.
- **Les deux gates humains** du rang 8 — revue signée puis tag — qui
  sont hors périmètre agent par construction.
- **Le réordonnancement.** Le Gantt se réévalue à chaque jalon, à mesure
  que le parallélisme réel se découvre. Celui-ci est la première passe
  du plan, pas le plan définitif.

---

*Dérivé du Manifeste Maury (CC BY-SA 4.0). Persona `chef-de-projet` de
la Méthode Foyer. Chiffrage : `skills/abaque-cout-capacite.md`.*
