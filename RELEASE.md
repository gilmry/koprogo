<!--
  Registre d'état Foyer — SOURCE DE VÉRITÉ PARTAGÉE (PO ↔ dev).
  Tout agent LE LIT avant d'agir et LE MET À JOUR à chaque étape conclusive.
  Markdown volontairement lisible : le PO doit le comprendre sans outil.

  Méthode : submodule `.foyer` (épinglé). Commandes : /foyer-status, /foyer-next,
  /foyer-bascule. Si `.foyer/pilote/` est vide : `git submodule update --init --recursive`.
-->
# État Foyer — KoproGo

- **Porte active** : `release`
- **Parcours** : `.foyer/pilote/journeys/planification-release.md`
- **Archétype** : full-stack *(Rust hexagonal + Astro/Svelte 5 en îlots, PostgreSQL)*
- **Substrat d'exécution** : conteneur — `~/bin/kcargo` pour Rust, jamais `cargo` sur l'hôte
- **Démarré le** : 2026-09-12
- **Dernière mise à jour** : 2026-09-12 (par : Claude — story habilitante posée, rang 0)

## Répartition des rôles

| Rôle | Qui | Depuis |
|---|---|---|
| **PO** (tranche les modalités, valide sur preuve, signe) | Gilles Maury | 2026-09-12 |
| **Dev** (exécute, assemble la preuve, ne tranche jamais) | Gilles Maury + agent | 2026-09-12 |

**Les deux chapeaux sont portés par la même personne, et c'est temporaire.** Un
partenaire agent IPI en exercice reprendra le rôle de PO une fois le pilotage
stabilisé. D'ici là, le dogfooding ne vaut que si la séparation reste **écrite** :
un arbitrage se pose en 🔴 dans ce registre avec sa preuve **avant** d'être
tranché, même quand celui qui pose et celui qui tranche sont la même personne.
Se l'épargner reviendrait à valider ce qu'on vient de produire, ce que la méthode
appelle une signature et non une validation.

## Position courante

- **Phase / étape** : **Phase A close.** Livrable BMAD **SIGNÉ** le 2026-09-12
  par Gilles Maury. Passage à la **phase B — vérifier le socle avant d'ajouter**.
- **⚠️ Une story habilitante bloque désormais le backlog.** `cap:C7.3`
  (#873, #874) est le **rang 0** : elle livre « la capacité de boucler ». La
  Méthode Foyer est catégorique — *« sur un projet existant sans ce harnais,
  c'est une story de correction structurelle dédiée qui bloque le reste du
  backlog tant qu'elle n'est pas fermée »*. Les vagues du fan-out n'ouvrent
  pas avant.

  - **#873** — la CI enregistre toutes les vidéos et **ne les téléverse nulle
    part** : la vitrine n'existe pas comme artefact de branche, donc la revue
    de promotion n'a rien à relire.
  - **#874** — le fan-out **n'a jamais tourné** : permissions Actions à
    `read`, secret absent, label absent, zéro run.

- **Prochaine action attendue** : exécuter
  [ADR 0050](docs/adr/0050-pile-de-recette-sur-le-vps-ports-decales.md) —
  décaler les quatre ports, faire viser `http://localhost:8090` à
  `make test-e2e` et `make docs-with-videos`, corriger
  `docs/E2E_TESTING_GUIDE.rst`. C'est le premier pas de la phase B : tant que
  `e2e` est 🔴, le parcours interdit d'empiler la release.
- **Le 🔴 sur les dialectes est éteint par le travail** : les 12 issues
  restantes ont été traduites au rang 7. L'instrument de mesure n'a pas été
  touché — c'est le travail qui a fait monter le chiffre, pas sa définition.
- **La fabrication n'ouvre pas encore.** Le PO a choisi une signature unique
  **après** la préparation complète ; ni #872 (harnais, ADR 0050) ni #855 ni
  #802 n'entrent en fabrication avant. L'ADR 0050 attend, elle n'est pas
  perdue.
- **Rôle à jouer** : `.foyer/pilote/roles/gate-runner.md`.

### Le pilotage — Gantt en passes d'agent

`docs/GANTT_PASSES_v0_1_0.md`, généré par `scripts/gantt-passes.py`
(persona `chef-de-projet`). **VALIDÉ le 2026-09-12**, avec amendement.

**L'amendement.** Le `ratio_supervision` ne bride plus l'éventail : le
*répondre-de* est porté par la **revue de promotion de branche**, instruite
par les gates et la **vitrine** (parcours filmé). On ne supervise plus des
agents en direct — on relit une preuve attachée à une branche. Objectif :
paralléliser au maximum, orchestration Claude Code multiagent.

**Ce que le plan établit** : 4 vagues, 23 créneaux, largeur demandée
**10 agents** — contre 29 passes en séquentiel supervisé. Depuis le rang 0,
le diagramme compte **six couches** : deux de barrière (l'habilitante, exécutée
en session) puis les quatre du travail. Les dépendances
ne sont pas le goulot : les chaînes ne font que quatre couches. Quatre
issues tiennent chacune une file — **#803** en débloque 11, **#805** dix,
**#797** et **#802** neuf chacune.

**Deux réserves inscrites au livrable, non résolues :**

1. **Le filet n'est pas tendu.** Le mécanisme choisi pour porter le
   répondre-de — gates + vitrine — **n'est pas opérationnel** : `e2e` et
   `doc-vivante` sont 🔴, tous deux sur #872. Lancer 84 chantiers avant que
   la preuve existe produirait 84 branches que rien ne permet de relire.
   V1 n'est donc pas une formalité : c'est ce qui rend le reste légitime.
2. **L'hôte plafonne à 2 agents** quand le plan en demande 10 — `min(16,
   CPU−2)` avec 4 cœurs, dont la moitié déjà prise par 31 conteneurs, et un
   `target` Rust en volume Docker **partagé** qui sérialise les compilations.
   L'expérimentation tournerait au cinquième de sa largeur. Trois voies :
   hôte ≥ 12 cœurs, agents distants, ou **fan-out en CI** — cette dernière
   isole naturellement les `target` et produit déjà les artefacts que la
   revue attend.

### Ce qui est déjà produit de la phase A

- **Analyste / cadrage** : `docs/WBS_v0_1_0.md` — le périmètre, sa provenance.
- **Product Manager / epics-stories** : `docs/BACKLOG_STRUCTURE_v0_1_0.md` —
  10 épopées, 32 capacités, 85 issues, classement **exhaustif et exclusif**
  (`scripts/backlog-structure.py --verifier` échoue sur toute issue non classée).
- **Architecte** : les quatre contextes bornés sont déclarés ET gardés en CI
  (`backend/tests/architecture.rs`), les dépendances croisées sont interdites.
- **Chiffrage** : 73,25 j de wall-clock superviseur · 293 tours. **Bornes hautes
  de première passe**, à resserrer sur le réel par le CSI.
- **Validateur** : ✅ **fait côté agent**. **86 issues sur 86** portent les huit
  éléments d'une story prête, et **86 sur 86** portent les quatre classes de
  tests (`scripts/backlog-pret.py`, mesuré le 2026-09-12). Le livrable porte
  **`SIGNÉ`** — Gilles Maury, 2026-09-12, Product Owner / superviseur.

> **84 → 86.** Deux stories habilitantes ajoutées le même jour sur instruction
> du PO. Le champ `ecart_depuis_signature` du livrable le trace : le périmètre
> a bougé **sous** une signature qui attestait 84. La préparation reste close —
> les deux nouvelles portent les huit éléments d'emblée. Si le PO conteste
> l'ajout, c'est la signature qu'il reprend, pas ce champ.

> **Ce que la signature atteste, et ce qu'elle n'atteste pas.** Sa `portee`
> l'écrit dans le livrable : le classement, le chiffrage en bornes hautes, et la
> clôture de la préparation. **Pas** la qualité de chaque story.
>
> Le script préserve désormais ce bloc à la régénération : une signature est un
> fait humain, tout le reste est généré, et l'écraser obligerait le superviseur
> à re-signer un document qu'il a déjà relu.

> **Les huit rangs sont couverts**, orphelines comprises — #578, #579 (C1.2) et
> #427 (C7.2) ont leur story même si leur **rang** attend encore un arbitrage.
> Préparer ne préjuge pas d'ordonnancer.
>
> ⚠️ **Ce que 84/84 ne dit pas.** Le contrôle est **de forme** : le script
> cherche des marqueurs, pas du sens, et l'écrit lui-même — « une borne haute de
> la préparation, jamais un verdict ». Une story creuse sous un `@security`
> compterait. **84/84 mesure la couverture du gabarit, pas la qualité du
> contenu.** C'est exactement ce que la relecture et la signature humaines
> doivent trancher — et c'est le sujet de #427, qui demande que la taxonomie
> soit mesurée sur le **code livré**, pas sur l'intention écrite.

> **84, et non 85.** Le backlog structuré a été généré à 85 issues ouvertes ;
> #840 a été fermée le même jour, après la génération. Le document n'est pas
> faux, il est daté — il se régénère par `scripts/backlog-structure.py`.

## Gates (dernier statut — mesuré le 2026-09-12)

| Gate | Statut | Commande | Note |
|---|---|---|---|
| `plancher` secrets | 🟢 | `.claude/hooks/stop-leak-scan.sh` | bloquant, 3 hooks sur 8 bloquent vraiment |
| `plancher` migrations | ⚪ | — | réversibilité `down.sql` jamais vérifiée par un gate |
| `verify` structurel | 🟢 | `kcargo test --test architecture` + 15 gardes | 16 suites vertes |
| `contrat` anti-drift | 🟢 | gate OpenAPI + `oasdiff` en CI | #765 fermée |
| `unit` domaine | 🟢 | `kcargo test --lib` | 1989 tests |
| `integration` | 🟢 | suites `e2e_*.rs` (testcontainers) | |
| `bdd` | 🟢 | suites `bdd_*.rs` | |
| `e2e` parcours | 🔴 | `make test-e2e` | **vise `api.koprogo.com` sur le VPS** — #872 |
| `visuel` | ⚪ | — | pas de goldens |
| `doc-vivante` | 🔴 | `make docs-with-videos` | bloqué par #872, non bloquant par nature |
| front typecheck | 🟢 | `npx svelte-check --threshold error` | 0 erreur |
| front tests | 🟢 | `npx vitest run` | 652 tests, 119 fichiers |

**Le socle n'est pas vert.** La phase B du parcours exige de partir d'un vert
connu ; `e2e` est rouge et sa cause est nommée. Empiler la release avant de la
corriger serait la déviation que le parcours interdit.

## Périmètres / backlog

Le backlog vit dans **`docs/BACKLOG_STRUCTURE_v0_1_0.md`**, régénéré par
`scripts/backlog-structure.py`. Ne pas le recopier ici : deux backlogs sont déjà
un défaut de structure. Seul l'ordre des capacités est repris.

| Rang | Capacité | État |
|---|---|---|
| 1 | C7.1 — la recette peut se connecter et s'exécuter | débloquée — ADR 0050 à exécuter |
| 2 | C4.1 / C4.2 / C4.3 — identité, périmètre, RGPD | en cours (#845 à 1 route) |
| 3 | C10.1 — arbitrage du groupe « Communauté » | ✅ tranché (ADR 0052) · story prête |
| 4 | C5.2 puis C5.1 — contrat de tests, socle visuel | ✅ 4/4 stories prêtes |
| 5 | C1.1 / C1.3 — le noyau légal | ✅ 8/8 stories prêtes · #840 fermée |
| 6 | les `Should`, parallélisables **+ C1.2 et C7.2** | ✅ 20/20 stories prêtes |
| 7 | les `Could` | ✅ 39/39 stories prêtes |
| 8 | G1 puis G2 — revue humaine signée, puis le tag | hors périmètre agent |

## Arbitrages

### 🔴 En attente (le PO doit trancher une MODALITÉ)

**Aucun.** Tous les arbitrages ouverts ont été tranchés le 2026-09-12.

### ✅ Tranchés

| Point | Décision (modalité) | Par | Le | ADR |
|---|---|---|---|---|
| Accès notaire à l'état daté *(destination)* | lien signé à durée limitée, émis par le syndic, révocable, journalisé | Gilles Maury | 2026-09-12 | [ADR 0051](docs/adr/0051-lien-notaire-sept-jours-renouvelable.md) |
| Mot de passe superadmin de la démo | `admin123` posé dans l'environnement, pour survivre à l'upsert du seed ; coût assumé : il est publié dans le dépôt | Gilles Maury | 2026-09-12 | #870 |
| Porte du pilote Foyer | `release` — le produit tourne, on cadre un gros incrément | Gilles Maury | 2026-09-12 | ce registre |
| **Découpage de la release v0.1.0** | **option A — périmètre intégral** : les 84 issues ouvertes restent au tag, aucun report en 0.2.0. `Must/Should/Could` ordonne l'exécution, ne retire rien | Gilles Maury | 2026-09-12 | [ADR 0049](docs/adr/0049-perimetre-v0-1-0-integral.md) |
| **#872 — pile de recette** | **sur le VPS, ports décalés** : 8090 HTTP, 8091 dashboard, 15432 PG, 19000/19001 MinIO. La séparation `koprogo-dev` ≠ `koprogo` étant déjà commitée, il ne restait que des ports | Gilles Maury | 2026-09-12 | [ADR 0050](docs/adr/0050-pile-de-recette-sur-le-vps-ports-decales.md) |
| **#855 — lien du notaire** | **sept jours, plusieurs lectures, renouvelable** par le syndic ; chaque consultation journalisée | Gilles Maury | 2026-09-12 | [ADR 0051](docs/adr/0051-lien-notaire-sept-jours-renouvelable.md) |
| **#856 — « Communauté » et le comptable** | **la maquette est corrigée** ; `permissions.ts` et le test `@security` ne bougent pas. Débloque #802 intégralement | Gilles Maury | 2026-09-12 | [ADR 0052](docs/adr/0052-le-comptable-ne-voit-pas-communaute.md) |
| **Signature du livrable BMAD** | **une seule signature, après** que les stories portent les huit éléments. Elle attestera que la fabrication peut commencer, pas seulement que le classement tient | Gilles Maury | 2026-09-12 | ce registre |
| **Livrable BMAD — SIGNÉ** | signé au terme de la préparation (84/84). Portée explicite : classement + chiffrage + clôture de la préparation, **pas** la qualité story par story | Gilles Maury | 2026-09-12 | `docs/BACKLOG_STRUCTURE_v0_1_0.md` |
| **#694 — accès d'un collaborateur à une ACP** | **refus par défaut**, accès explicite par table d'association. Rend #694 structurellement requise : sa mention « non bloquant » est périmée et son rang 2 confirmé | Gilles Maury | 2026-09-12 | [ADR 0053](docs/adr/0053-acces-acp-refus-par-defaut.md) |
| **Rang des 3 orphelines** | C1.2 (#578, #579) et C7.2 (#427) rejoignent le **rang 6**, avec les autres `Should`. Leurs stories étaient déjà prêtes | Gilles Maury | 2026-09-12 | ce registre |
| **Dialecte des stories** | **option A — traduire**, jamais modifier le compteur. Les 12 restantes traduites au rang 7 | Gilles Maury | 2026-09-12 | ce registre |

## Journal (chronologie courte)

- 2026-09-12 — **story habilitante posée** : #873 (vitrine en artefact CI) et
  #874 (fan-out prouvé) forment `cap:C7.3`, rang **0**, et bloquent le reste
  du backlog. Le Gantt gagne une **barrière** : rien ne se fanne-out avant
  qu'elle soit close. 86 issues, 73,75 j, 295 tours.
- 2026-09-12 — **Gantt VALIDÉ avec amendement multiagent** : le répondre-de
  passe du direct à la revue de promotion de branche. Deux réserves mesurées
  et inscrites — le filet (gates + vitrine) est rouge, et l'hôte tient 2
  agents pour 10 demandés.
- 2026-09-12 — **Gantt en passes d'agent** produit et publié
  (`docs/GANTT_PASSES_v0_1_0.md`, au sommaire Sphinx).
- 2026-09-12 — **PHASE A CLOSE.** Livrable BMAD **signé** par Gilles Maury.
  #694 tranchée en **refus par défaut** (ADR 0053), ce qui rend la table
  d'association structurellement requise et périme sa mention « non bloquant ».
  Les 3 orphelines placées au rang 6. **Plus aucun arbitrage 🔴.**
- 2026-09-12 — **le script de backlog préserve désormais la signature** à la
  régénération, et #840 (fermée) retirée des capacités — sa propre garde
  l'exigeait. Totaux : 84 issues, 72,50 j, 290 tours.
- 2026-09-12 — **étape 4 close côté agent : 84/84.** Les trois orphelines
  (#578, #579, #427) préparées sans préjuger de leur rang. Le backlog v0.1.0
  est intégralement outillé pour la fabrication ; reste la signature du PO,
  qui est un acte humain et le seul qui atteste du **fond**.
- 2026-09-12 — **rang 7 porté à « Agent IA Ready »** : les 39 `Could`.
  **81/84**, et **84/84 portent les quatre classes de tests**. Les 12 issues en
  dialecte Maury ont été traduites plutôt que de modifier le compteur. Les 3
  restantes sont les orphelines sans rang.
- 2026-09-12 — **rang 6 porté à « Agent IA Ready »** : les 17 `Should` de
  C2.1, C2.2, C3.1, C4.4, C4.5, C6.1, C6.2, C9.1 et C9.3 (#855 l'était déjà).
  **42/84 — la moitié.** En les listant, découverte que le rang 6 en comptait
  **18 et non 23** : mon compte précédent était faux, et trois issues (#578,
  #579, #427) ne figurent dans **aucun** rang. Inscrit en 🔴.
- 2026-09-12 — **rang 5, le noyau légal, porté à « Agent IA Ready »** : #780,
  #848, #850, #576, #577, #581 (C1.1) et #847, #846 (C1.3). 25/84. #840 étant
  de C1.1, sa fermeture explique le 85 → 84.
- 2026-09-12 — **deux dialectes de story découverts** : 16 issues portent les
  quatre classes dans le vocabulaire Maury d'origine, que le compteur ne lit
  pas. Inscrit en 🔴 — l'option « apprendre les deux dialectes » modifierait
  l'instrument qui conditionne la signature.
- 2026-09-12 — **rang 4 porté à « Agent IA Ready »** : #803 (C5.2) et #834
  (C5.1) ; #802 et #797 l'étaient déjà. Les deux stories partagent une même
  règle de séquencement — **celui qui touche un écran l'ancre et le traduit dans
  le même commit**, plutôt qu'une correction de masse invérifiable. 17/84.
- 2026-09-12 — **rang 3 porté à « Agent IA Ready »** : #856 (C10.1). Sa story
  exécute l'ADR 0052 et nomme la lecture qui concilie tout — l'accès
  communautaire d'un comptable viendrait d'une **désignation** de community
  manager (ADR 0046), jamais de son rôle. 15/84.
- 2026-09-12 — **rang 2 porté à « Agent IA Ready »** : sept stories écrites
  sur #845, #864 (C4.1), #868, #841, #798, #694 (C4.2) et #842 (C4.3). La
  préparation passe de 7/84 à **14/84**, mesuré. #772 étant fermée, la
  précondition de #798 est levée.
- 2026-09-12 — **les quatre arbitrages restants tranchés** : #872 recette sur
  le VPS à ports décalés (ADR 0050), #855 lien notaire sept jours renouvelable
  (ADR 0051), #856 la maquette cède au test `@security` (ADR 0052), signature
  BMAD unique et différée. Plus aucun 🔴 ouvert.
- 2026-09-12 — **une mesure a corrigé le registre** : `make test-e2e` ne « vise »
  pas `api.koprogo.com` par configuration — il vise `http://localhost`, que
  Traefik route vers la démo sur cet hôte. Et la raison décisive de ne pas
  l'y laisser n'est pas la pollution : c'est que `make seed-reset` et
  `make reset-db`, préconditions de toute recette reproductible, effaceraient
  la démo.
- 2026-09-12 — **découpage de la release tranché : option A**, périmètre
  intégral confirmé face à son chiffrage (ADR 0049). La décision du 2026-09-06
  n'est plus seulement héritée, elle est opposée à son coût.
- 2026-09-12 — pilote Foyer installé : submodule `.foyer` épinglé à `f6fe69c`,
  quatre commandes dans `.claude/commands/`, ce registre créé.
- 2026-09-12 — backlog restructuré par capacité (`1a14a8e4`), chiffré, et
  matérialisé sur GitHub en étiquettes `epic:*` / `cap:*`.
- 2026-09-12 — #872 ouverte puis son risque de destruction fermé (`d35332de`).
- 2026-09-12 — #840 fermée (Art. 3.87 §2, décision hors ordre du jour nulle).
- 2026-09-12 — huit commits poussés, `ca7de310..0adb9b8e`.
