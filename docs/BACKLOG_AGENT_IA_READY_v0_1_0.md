---
livrable: Epics & User Stories (BMAD phase E — TOGAF Solutions)
projet: KoproGo
jalon: v0.1.0
version: 1
signature_humaine:
  date: null
  nom: null
  role: null
  etat: NON SIGNÉ — en attente de validation du superviseur
---

# Backlog Agent IA Ready — v0.1.0

*Livrable du Scrum Master de conception, gabarit
`foyer/bmad/livrables/epics-stories.template.md`. Il ne remplace pas
`WBS_v0_1_0.md`, qui reste l'inventaire du périmètre : il dit **dans quel état
de préparation** ce périmètre se trouve, et ce qu'il faut pour qu'un tour de
boucle puisse commencer.*

Le frontmatter est **délibérément vide**. BMAD pose qu'un livrable non relu par
l'humain est une faute au sens *répondre-de* ; le déclarer signé sans l'être
serait précisément la faute que la signature doit empêcher.

---

## 1. Le fait, mesuré

```
$ python3 scripts/backlog-pret.py
84 issues ouvertes en release:0.1.0

    4/84   Récit — En tant que… je veux… afin de…
    2/84   Critères Gherkin — Étant donné / Quand / Alors
   21/84   Classe @happy — le chemin nominal
   22/84   Classe @negative — entrées invalides, échecs attendus
   21/84   Classe @edge — bornes : vide, max, concurrence
   23/84   Classe @security — abus, injection, autorisation
    2/84   Couche — Domain / Application / Infra / Frontend / IaC
    2/84   Taille — S (0,5 j) / M (0,75 j) / L (1 j)

   20/84   portent les QUATRE classes de tests
    2/84   Agent IA Ready — les huit éléments
```

**Deux sur quatre-vingt-quatre.** Le relevé initial, avant ce travail, donnait
**zéro**.

Le chiffre n'est pas écrit ici à la main : `scripts/backlog-pret.py` le relève
à chaque exécution. Un taux de préparation recopié dans un markdown vieillit en
trois jours sans que personne s'en aperçoive — c'est déjà arrivé à la section
« Ordre d'exécution » du WBS, dont les quatre premières étapes désignent
aujourd'hui des issues toutes fermées.

### Ce que ce chiffre ne dit pas

Le script cherche des **marqueurs de forme**. Une issue qui écrit `@security`
au-dessus d'un critère creux est comptée comme portant sa classe de tests.
C'est une **borne haute** de la préparation, jamais un verdict. Le jugement
reste humain : c'est le point entier du *répondre-de*.

Il mesure aussi le **corps** de l'issue, pas ses commentaires. Ce n'est pas un
détail de mise en œuvre : c'est le corps qu'un agent lit quand il reprend un
ticket, et un commentaire se fait enterrer par les quarante suivants. Les deux
premières stories ont d'abord été postées en commentaire, et le mesureur ne les
voyait pas — il avait raison.

---

## 2. Ce que « Agent IA Ready » veut dire ici

Les huit éléments du gabarit BMAD. Le Scrum Master de conception l'écrit sans
détour : *« aucune story n'est prête sans elles »*.

| Élément | Ce qu'il pré-engage |
|---|---|
| Récit *En tant que… je veux… afin de…* | **à qui** ça sert, et donc ce qu'on peut retirer |
| Critères Gherkin | le critère, **avant** la génération |
| `@happy` | le chemin nominal |
| `@negative` | les entrées invalides et les échecs attendus |
| `@edge` | les bornes : vide, max, concurrence |
| `@security` | abus, injection, autorisation |
| Couche(s) | où le code atterrit, donc quelles gardes s'appliquent |
| Taille + tours | le coût, sur les deux axes : wall-clock et tokens |

L'ordre compte. La boucle de craft pose le test d'abord *parce qu'il pré-engage
le critère* : une story sans critère falsifiable ne peut pas commencer par du
rouge, donc ne peut pas boucler.

### Le témoin, qui ne figure pas au gabarit et que ce dépôt exige

Un test écrit après coup peut ne rien garder. Ce dépôt l'a vérifié à ses
dépens : une assertion d'un banc mobile comparait `document.scrollWidth` à
`window.innerWidth`, deux valeurs qui grandissent ensemble — elle **ne pouvait
pas échouer**, et cachait un vrai débordement.

Toute story livrée ici porte donc un neuvième point : **le défaut est remis, et
le test doit échouer**. Un témoin qui ne mord pas est une question, pas une
licence à inventer une explication.

---

## 3. Les stories prêtes

| # | Story | Couche | Taille | Tours |
|---|---|---|---|---|
| [#797](https://github.com/gilmry/koprogo/issues/797) | U1 — jetons de design et jeu d'icônes SVG | Frontend | L | 4 |
| [#802](https://github.com/gilmry/koprogo/issues/802) | U6 — adapter les tests sans effacer les règles produit | Frontend | L | 3 |

Ce sont les deux prochaines étapes réellement exécutables de l'ordre du WBS :
#797 est le préalable de tout le Track U, #802 est le relevé du contrat de
tests qu'il faut tenir **avant** de déplacer un écran.

---

## 4. Ce qui bloque, et où cela se tranche

La Méthode Foyer §1 est explicite : **une question ouverte devient une RFC**,
formalisée pour être discutée *avant* d'être tranchée, plutôt que tranchée en
silence dans le code.

Une issue du périmètre porte aujourd'hui l'étiquette `question` :
[#856](https://github.com/gilmry/koprogo/issues/856), le groupe « Communauté »
de la barre latérale du comptable. La remise de design le propose ; un test
`@security` de `Navigation.test.ts` l'interdit. La revue tranche elle-même en
faveur du test — *« The test wins unless the product owner says otherwise »* —
ce qui laisse la décision exactement là où elle doit être.

C'est aussi ce qui **borne** la story #802 : elle livre tout sauf ce point, et
le dit. Modifier `permissions.ts` pour faire passer une maquette serait
trancher une règle RBAC sans que personne n'en réponde.

**Le balayage des autres arbitrages en attente reste à faire.** Il n'est pas
fait ici et n'est pas prétendu fait : seule #856 porte l'étiquette, et rien ne
garantit qu'elle soit la seule question du périmètre.

---

## 5. Sprint 0 — la story habilitante

Le Scrum Master pose que pour un archétype *full-stack*, le Sprint 0 inclut
**obligatoirement** le harnais de contrat API, et que sur un projet existant
qui en manque, c'est une story de correction structurelle qui **bloque le reste
du backlog** tant qu'elle n'est pas fermée.

Vérifié : [#765](https://github.com/gilmry/koprogo/issues/765) est **fermée**.
Les dix-sept routes `/expenses` et `/invoices` hors contrat OpenAPI — celles
qui avaient laissé `line_items` se perdre en silence — sont au contrat. Le
backlog n'est donc pas bloqué à ce titre.

Le reste du Sprint 0 est en place et se relève : quinze gardes en CI, un
cliquet par dette, `fmt`, `clippy -D warnings`, et des suites e2e qui montent
une vraie base.

---

## 6. Ce qui reste à préparer

Quatre-vingt-deux issues sur quatre-vingt-quatre ne sont pas prêtes. Les rendre
prêtes d'un coup serait une correction de masse — exactement ce que la méthode
range parmi les gestes qui retirent à l'humain les moyens d'assumer.

La règle retenue : **une story est mise au gabarit au moment où elle devient la
prochaine étape**, pas trois semaines avant. Le taux de préparation se relève
alors à chaque tour, et il monte pour une raison qu'on peut nommer.

---

*Dérivé du Manifeste Maury (CC BY-SA 4.0). Mesure : `scripts/backlog-pret.py`.*
