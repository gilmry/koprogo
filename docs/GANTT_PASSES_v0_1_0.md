---
livrable: Gantt en passes d'agent (pilotage — persona chef de projet)
projet: KoproGo
jalon: v0.1.0
genere_par: scripts/gantt-passes.py
signature_humaine:
  date: null
  nom: null
  role: null
  etat: NON SIGNÉ — en attente de validation du superviseur
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

Colonnes : couches 1 à 4. `█` = la capacité est ouvrable,
`·` = elle attend.

```text
capacité  rang moscow  1234
--------- ---- ------- ----
C7.1         1 Must    ██··
C4.1         2 Must    ██··
C4.2         2 Must    █···
C4.3         2 Must    █···
C10.1        3 Must    █···
C5.1         4 Should  █···
C5.2         4 Must    █···
C1.1         5 Must    ██··
C1.3         5 Must    ██··
C1.2         6 Should  ███·
C1.5         6 Should  █···
C2.1         6 Should  █···
C2.2         6 Should  █···
C3.1         6 Should  ██··
C4.4         6 Should  █···
C4.5         6 Should  ██··
C6.2         6 Should  ██··
C9.1         6 Should  ██··
C9.3         6 Should  █···
C6.1         6 Should  ·██·
C7.2         6 Should  ·█··
C2.3         7 Could   █···
C3.3         7 Could   ███·
C5.3         7 Could   ██··
C8.1         7 Could   ███·
C8.4         7 Could   █···
C9.2         7 Could   ██··
C9.4         7 Could   █···
C1.4         7 Could   ·██·
C3.2         7 Could   ·██·
C8.2         7 Could   ·██·
C8.3         7 Could   ···█
```

## Points de concours et de divergence

**Divergence** — ce qui débloque le plus de chantiers en aval. Ce sont
les issues à ne pas laisser traîner : chacune tient une file.

| Issue | Capacité | Couche | Chantiers débloqués |
|---|---|---:|---:|
| `#803` | C5.2 | 1 | **11** |
| `#805` | C8.1 | 1 | **10** |
| `#797` | C5.1 | 1 | **9** |
| `#802` | C5.2 | 1 | **9** |
| `#872` | C7.1 | 1 | **5** |
| `#780` | C1.1 | 2 | **4** |

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

**3 des 4 couches dépassent le plafond de 3** : L1 (39 chantiers), L2 (36 chantiers), L3 (8 chantiers).

C'est le résultat le plus utile du diagramme, et il est
contre-intuitif : **les dépendances ne sont pas le goulot.** Les
chaînes sont courtes — 4 couches seulement — et l'essentiel
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

## Le plan exécutable — dépendances *et* plafond de supervision

À 3 chantiers de front, la release demande
**28 passes**. C'est ce plan qui se pilote ; la couche
topologique ci-dessus dit seulement ce qui *pourrait* être mené de
front si la supervision était infinie.

L'ordre à l'intérieur d'une passe suit le rang, puis le MoSCoW, puis la
taille décroissante.

| Passe | Chantiers | Jours | Cumul | Capacités |
|---|---|---:|---:|---|
| **P1** | #872 (L), #870 (S), #694 (L) | 2.50 | 2.50 | C4.2, C7.1 |
| **P2** | #696 (M), #832 (M), #798 (L) | 2.50 | 5.00 | C4.2, C7.1 |
| **P3** | #864 (L), #841 (M), #842 (M) | 2.50 | 7.50 | C4.1, C4.2, C4.3 |
| **P4** | #868 (S), #856 (S), #802 (L) | 2.00 | 9.50 | C10.1, C4.2, C5.2 |
| **P5** | #803 (L), #797 (L), #834 (L) | 3.00 | 12.50 | C5.1, C5.2 |
| **P6** | #576 (L), #780 (L), #847 (L) | 3.00 | 15.50 | C1.1, C1.3 |
| **P7** | #848 (L), #850 (L), #581 (M) | 2.75 | 18.25 | C1.1 |
| **P8** | #577 (L), #846 (M), #427 (L) | 2.75 | 21.00 | C1.1, C1.3, C7.2 |
| **P9** | #515 (L), #579 (L), #718 (L) | 3.00 | 24.00 | C1.2, C9.1 |
| **P10** | #578 (L), #762 (L), #779 (L) | 3.00 | 27.00 | C1.2, C3.1, C4.5 |
| **P11** | #555 (L), #835 (L), #855 (L) | 3.00 | 30.00 | C1.5, C4.4, C4.5 |
| **P12** | #845 (L), #867 (L), #432 (M) | 2.75 | 32.75 | C2.2, C4.1, C9.3 |
| **P13** | #453 (M), #731 (M), #781 (M) | 2.25 | 35.00 | C3.1, C9.1 |
| **P14** | #865 (M), #869 (M), #852 (S) | 2.00 | 37.00 | C2.1, C6.1, C6.2 |
| **P15** | #592 (M), #866 (M), #871 (S) | 2.00 | 39.00 | C6.1, C6.2 |
| **P16** | #354 (L), #425 (L), #429 (L) | 3.00 | 42.00 | C9.2, C9.4 |
| **P17** | #355 (L), #556 (L), #583 (L) | 3.00 | 45.00 | C1.4, C5.3, C9.2 |
| **P18** | #585 (L), #635 (L), #818 (L) | 3.00 | 48.00 | C2.3, C3.3, C5.3 |
| **P19** | #820 (L), #823 (L), #824 (L) | 3.00 | 51.00 | C5.3 |
| **P20** | #825 (L), #826 (L), #466 (M) | 2.75 | 53.75 | C5.3, C9.2 |
| **P21** | #582 (M), #586 (M), #587 (M) | 2.25 | 56.00 | C1.4, C3.2, C3.3 |
| **P22** | #591 (L), #588 (M), #590 (M) | 2.50 | 58.50 | C3.2, C3.3 |
| **P23** | #805 (M), #821 (M), #822 (M) | 2.25 | 60.75 | C5.3, C8.1 |
| **P24** | #806 (L), #807 (L), #810 (L) | 3.00 | 63.75 | C8.1, C8.2 |
| **P25** | #808 (M), #809 (M), #811 (M) | 2.25 | 66.00 | C8.1, C8.2 |
| **P26** | #812 (M), #815 (M), #816 (M) | 2.25 | 68.25 | C8.1, C8.2 |
| **P27** | #817 (M), #827 (M), #854 (M) | 2.25 | 70.50 | C5.3, C8.2, C8.4 |
| **P28** | #813 (L), #589 (S), #595 (S) | 2.00 | 72.50 | C3.2, C8.3, C8.4 |

Le harnais (`C7.1`, rang 1) occupe **P1**, **P2** : rien ne se
**déclare** tenu avant lui, et la phase B du parcours Foyer exige un
socle vert avant d'empiler la release.

## Coût

| Axe | Valeur | Ce que ça mesure |
|---|---:|---|
| Issues | 84 | le périmètre, intégral (ADR 0049) |
| Passes | 28 | les tours de boucle supervisés |
| Jours | 72.50 | wall-clock **superviseur** |
| Tours | 290 | coût **tokens** |

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
