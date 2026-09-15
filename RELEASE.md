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
- **Reprise par une session neuve** :
  [lettre de mission du 2026-09-13](docs/plans/2026-09-13-lettre-de-mission-reprise.md)
  — où travailler, ce qui bloque, et les sept pièges qui coûtent des heures
  quand on les redécouvre
- **Archétype** : full-stack *(Rust hexagonal + Astro/Svelte 5 en îlots, PostgreSQL)*
- **Substrat d'exécution** : conteneur — `~/bin/kcargo` pour Rust, jamais `cargo` sur l'hôte
- **Démarré le** : 2026-09-12
- **Dernière mise à jour** : 2026-09-15 (par : Claude — le Gantt DÉROULÉ,
  cinq créneaux, 26 branches d'agent, socle vert et promu sur `main`)

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
  - **#876** — la vitrine **ne suit pas le moule Foyer**. Les quatre éléments
    sont livrés (parcours partagé, cadence nommée, narration incrustée,
    chapitres, galerie, invariant anti-dette). **Mais elle filmait son propre
    échec** : le run `34707123590` a rendu 307 specs vertes et une rouge — le
    parcours de référence, sur « TestWorld not found ». L'artefact `vitrine`
    s'est bien publié et téléchargé (93 Mo, 316 vidéos, galerie, chapitres) en
    montrant un écran de connexion qui ne s'ouvre pas.

    **Cause** : `playwright.config.ts` ne déclare **aucun `globalSetup`**. Le
    `tests/e2e/global-setup.ts` existe, écrit un TestWorld de 500 lignes, et
    rien ne l'exécute — les 307 autres specs ne s'en apercevaient pas, aucune
    ne s'en sert. Corrigé par `597b3eeb` : le parcours amorce son monde
    lui-même. **En attente de la preuve CI.**
  - **#874** — le fan-out **n'a jamais tourné** : permissions Actions à
    `read`, secret absent, label absent, zéro run.

- **✅ ADR 0050 exécutée** (`0e3036d8`) — les quatre ports sont décalés
  (8090 / 8091 / 15432 / 19000-19001), `make test-e2e` et
  `make docs-with-videos` visent `$(RECETTE)` dont le défaut est
  `http://localhost:8090`, et `docs/E2E_TESTING_GUIDE.rst` ne présente plus la
  commande dangereuse comme la normale. `garde-piles-compose-distinctes`
  interdit désormais qu'une pile suivie reprenne le nom, le conteneur ou le
  **port** d'une autre — trois témoins de rougeur vérifiés.

  Deux choses à en retenir, qui ne sont pas dans l'ADR :

  - `PUBLIC_API_URL` du frontend valait `http://localhost/api/v1`. Lue par le
    **navigateur**, elle aurait envoyé chaque appel de la recette à la démo
    malgré le décalage, en silence.
  - `docker-compose.mcp.yml`, suivi par git, ne posait **aucun `name:`** :
    ses conteneurs héritaient du projet `koprogo`, et un `down` dessus aurait
    emporté la démo. Le défaut de #872 était encore armé dans un troisième
    fichier, invisible parce que la garde n'en connaissait que deux.
  - L'ADR attribuait 5432/9000/9001 à la démo. Mesuré : ce sont
    `elevia-postgres` et `derniere-chance-minio`, des projets **voisins**. La
    décision reste bonne, la garde de ce dépôt ne peut simplement pas voir
    ces collisions-là.

- **✅ #872, livrable 4 fermé** — `garde-identifiants-de-recette.test.ts`
  (#870) ne s'arrêtait que sur « aucun identifiant choisi ». Le cas restant,
  nommé par #872, est l'inverse et plus dangereux : un identifiant qui
  **fonctionne** contre un hôte distant enchaînerait écritures,
  `seed-reset`, `reset-db` sur des données vivantes sans qu'aucun message ne
  l'ait jamais demandé. `verifieLesIdentifiants()` exige désormais une
  seconde variable disjointe, `KOPROGO_CONFIRME_HOTE_DISTANT`, avant de
  laisser passer un hôte distant — que l'identifiant soit correct ou non.
  Deux tests `@happy` existants ont été adaptés (pas supprimés, cf.
  commentaire dans le fichier) pour exiger ce second choix ; le guide E2E
  documente la variable.

- **✅ La pile de recette tourne** (autorisée par le PO le 2026-09-12). Cinq
  conteneurs `koprogo-dev-*` sur 8090 / 8091 / 15432 / 19000-19001. Isolation
  vérifiée à chaque étape : les quatre conteneurs de la démo sont restés
  identiques au caractère près et `api.koprogo.com` a répondu 200 tout du long.
  `Host(localhost)` ne sert que nos deux conteneurs — contrôlé par l'API de
  Traefik, pas supposé.

- **Prochaine action attendue** : **relire les sept branches vertes et
  chronométrer** (#875). Tout le reste est débloqué.

  Les deux promotions sont **fusionnées** : `main` porte les correctifs et
  `feature/dev` n'a plus un seul commit d'avance. `main` connaît désormais le
  jeton du fan-out et l'épinglage du `checkout`.

  | Gate | État |
  |---|---|
  | `integration` | 🟢 mesuré, et vert en CI sur `main` |
  | `e2e` parcours | 🟢 308 ✓ / 0 ✘, code 0 |

  Il reste **trois 🔴, tous du ressort du PO** : `ACTIX_WORKERS: 1` sur la
  démo, le banc de recette en hot reload (#880), et le type du jeton
  `FANOUT_GITHUB_TOKEN`.

- **À noter, sans conséquence aujourd'hui** : le Traefik de la recette voit les
  **18 routeurs des projets voisins** de l'hôte (derniere-chance, elevia, n8n),
  parce que `--providers.docker=true` regarde tout le socket. Ils sont tous
  `disabled` faute d'entrypoint correspondant, et aucun ne matche
  `Host(localhost)`. C'est la même famille que #731, et ça mériterait une
  contrainte Traefik plutôt qu'une chance de plus.
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

> **84 → 87.** Trois stories habilitantes ajoutées le même jour sur instruction
> du PO. Le champ `ecart_depuis_signature` du livrable le trace : le périmètre
> a bougé **sous** une signature qui attestait 84. La préparation reste close —
> les trois nouvelles portent les huit éléments d'emblée. 74,75 j / 299 tours. Si le PO conteste
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
| `integration` | 🟢 | suites `e2e_*.rs` (testcontainers) | `storage_s3` rend `1 passed`, code 0, contre `quay.io`. **Mesuré en local le 2026-09-13**. ⚠️ #877 reste OUVERTE : son premier critère dit « vert EN CI », et la CI ne l'a pas vu — les commits ne sont pas poussés |
| `bdd` | 🟢 | suites `bdd_*.rs` | |
| `e2e` parcours | 🟢 | `make test-e2e` | **308 ✓ / 0 ✘ / 14 sautés — CODE 0**, le 2026-09-13 après le correctif de #718 (`40eb8edd`). Aucun redémarrage pendant (`SIGTERM` 4 avant, 4 après). Même chiffre qu'en CI |
| `visuel` | ⚪ | — | pas de goldens |
| `doc-vivante` | 🟢 | `make vitrine` | le PARCOURS : complet, 10 chapitres, 81 s, `interrompu: None`, artefact de 79 Mo publié (run 34764114133). ⚠️ Les douze `.scenario.ts` du même job rendent **10 ✓ / 2 ✘** et ne peuvent PAS rougir le job : `continue-on-error: true` depuis le 2026-06-15, avec une condition de retrait jamais rouverte |
| front typecheck | 🟢 | `npx svelte-check --threshold error` | 0 erreur |
| front tests | 🟢 | `npx vitest run` | 659 tests, 120 fichiers |

**Le socle est vert le 2026-09-13**, `plancher migrations` et `visuel` mis à
part, qui restent ⚪ faute de gate. Il l'est sur des exécutions, pas sur des
lectures de diff : chaque 🟢 du tableau ci-dessus porte un code de sortie.

Le dernier à céder aura été `e2e`, et il n'a pas cédé parce qu'on a réparé
dix specs — il a cédé parce qu'on a trouvé **une** cause à ses dix échecs.
C'est la leçon de la journée, et elle vaut d'être écrite : instruire dix
symptômes un par un aurait coûté des jours et n'aurait rien réparé.

**Ce que le premier passage a coûté, et qui reste vrai.** La pile de
recette a tourné pour la première fois de son existence le 2026-09-12, et
chaque ligne du tableau ci-dessus repose désormais sur une exécution, plus sur
une lecture de diff.

Ce que le premier passage a coûté, et qui est le vrai résultat de la journée :

1. **Le backend ne démarrait pas** — `JWT_SECRET` absent de la pile de dev.
2. **57 specs sur 106 échouaient** — `PLAYWRIGHT_API_BASE` retombait sur le
   port 80 dans **93 fichiers**, donc sur le Traefik de la démo. Rien n'a été
   écrit là-bas : le 80 rend un `301 → https://localhost` qui n'aboutit pas.
   Ce n'était pas une garde, c'était une chance. C'en est une maintenant.
3. **Le gate `integration` était déclaré 🟢 et ne l'était pas** — #877.

Aucun de ces trois défauts n'était visible avant l'exécution. C'est la
définition d'un gate jamais lancé : il ne dit rien, et son silence se lit
comme un accord.

Reste 8 échecs sur 322. Quatre sont **#718**, reproduite hors production pour
la première fois — le backend ne meurt pas, il cesse de répondre sous rafale
(0 redémarrage, aucune recompilation, routage vérifié vers nos seuls
conteneurs). Les quatre autres sont à instruire un par un, et c'est #832.

Le 🟢 se posera quand `make test-e2e` rendra 0. Pas avant.

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

**`ACTIX_WORKERS: 1` sur la démo** — posé le 2026-09-13, avec sa preuve.

`docker-compose.prod.yml:101` pose `ACTIX_WORKERS: ${ACTIX_WORKERS:-1}`, et
ce n'est pas qu'une valeur par défaut dans un fichier : vérifié sur le
conteneur qui tourne, `docker inspect koprogo-backend` rend bien
`ACTIX_WORKERS=1`. Mesuré, pas supposé.

La mesure de #718 établit que `hash`/`verify` bloquaient le worker 1,69 s en
médiane : avec un seul worker, **une connexion bloquait toute l'API**. Le
correctif `40eb8edd` retire le blocage ; il ne rend pas un worker unique
défendable pour un produit où des dizaines de copropriétaires se connectent
dans la même minute d'une AG.

Ce n'est pas à l'agent de trancher : c'est un arbitrage de dimensionnement,
avec un coût en RAM sur un VPS qui porte trente conteneurs.

**Le banc de mesure de la recette** — posé le 2026-09-13, #880.

Le backend de la recette tourne sous `cargo-watch`. Toute édition de Rust
pendant une campagne la coupe, et rien dans les artefacts ne le dit. L'ADR
0050 a choisi une pile unique ; en changer se pose au PO. Trois voies : un
banc à binaire figé, la sérialisation explicite de l'accès, ou l'acceptation
du défaut avec un témoin d'interruption (le minimum, déjà décrit dans #880).

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
| **#877 — l'image MinIO** | **quay.io, épinglé par tag, partout** — test d'intégration, trois composes et Helm. `:latest` retiré | Gilles Maury | 2026-09-13 | #877, `72d719e6` |
| **Voie du fan-out** | **abonnement (`CLAUDE_CODE_OAUTH_TOKEN`), `max_parallel` à 2** pour la première vague. À monter une fois mesurée, pas avant | Gilles Maury | 2026-09-13 | ce registre |
| **PR #879** | **relancer pour qu'elle ait ses gates** avant la revue. La chronométrer sans preuve mesurerait autre chose que ce que #875 cherche | Gilles Maury | 2026-09-13 | #875, run `34764114133` |

## Journal (chronologie courte)

- 2026-09-15 — **Décision du PO : relire les promotions à l'ENVERS de la
  pyramide des tests** — #913.

  La vitrine d'abord, puis chaque couche en descendant, et seulement si la
  précédente laisse un doute.

  ```
     vitrine (doc-vivante)     ← on commence ICI
          e2e parcours
            bdd
         integration
        unit · contrat
           plancher            ← on finit ici, si on y arrive
  ```

  La pyramide est un guide de **construction** — beaucoup d'unitaires en bas
  parce que le rapport coût/couverture l'impose. Elle n'a jamais prétendu
  dire dans quel ordre on **relit**, et pour relire l'ordre inverse est le
  bon : seule la couche haute répond à la question d'une promotion, qui
  n'est pas « le code est-il correct » mais « la chose demandée est-elle là ».

  **Le rang atteint devient une mesure.** Une branche relue au rang 1 coûte
  une minute ; une branche relue au rang 7 — le diff — coûte une heure et dit
  que le harnais n'a pas fait son travail. C'est le prolongement de #875, où
  le signal était binaire (« a-t-il ouvert le diff ») : il devient une
  profondeur.

  **Ce que l'ordre protège, c'est l'ATTENTION.** Précision du PO, et elle
  change le sens de la décision :

  - **le *code smell* se voit dans la preuve de valeur.** Un parcours qui
    demande sept clics là où trois suffiraient dit une responsabilité mal
    placée ; une capacité qu'on ne sait pas filmer sans l'expliquer dit
    qu'elle n'a pas de frontière. La vitrine n'est pas qu'une preuve de
    valeur, c'est **le premier détecteur de conception** — et il opère sur ce
    que le produit FAIT, pas sur ce qu'il est écrit ;
  - **l'expertise se réserve au non-trivial** : la frontière, l'invariant,
    l'arbitrage, et le non-dit. Aucune de ces quatre questions ne se lit dans
    un diff. Les vingt-sept branches de la vague 4 le confirment : tous leurs
    défauts ont été trouvés par des gardes, aucun n'exigeait un œil humain ;
  - **avec une pyramide exigeante et une boucle courte, le modèle se corrige
    seul.** `garde_taxonomie_des_tests` a refusé `story/781`,
    `garde_harnais_executes` a refusé `story/867` — sans humain, et en
    nommant le geste correctif.

  D'où la règle qui en découle, et qui est exigeante :

  > **Chaque garde ajoutée déplace une question du rang 7 vers le rang 1.**
  > Et un défaut trouvé en lisant le diff est une garde MANQUANTE : il ne
  > suffit pas de le corriger, il faut poser le cliquet qui empêchera le
  > suivant.

  Sans quoi le rang 7 se repeuple, et l'expertise retourne faire le travail
  des machines.

  ⚠️ **Le rang 1 est inatteignable aujourd'hui** : aucune des 27 branches
  d'agent n'a de vitrine, leurs gates ayant été déclenchés à la main. Corrigé
  le 2026-09-15 ; `story/579` est la première dont les gates partent seuls.
  C'est ce qui rend le protocole nécessaire MAINTENANT — sans lui, la vitrine
  resterait ce qu'elle a été pendant vingt-sept branches : un artefact que
  personne n'ouvre.

- 2026-09-15 — **Les vagues 5 à 7 ne peuvent PAS s'ouvrir, et ce n'est pas
  une prudence : c'est la règle du Gantt, vérifiée.**

  Le plan écrit : « une vague ne s'ouvre qu'une fois l'amont **fusionné** ».
  Mesuré sur la table de dépendances de `gantt-passes.py` : **les vingt-deux
  stories des vagues 5 à 7 attendent toutes une story de la vague 4**, sans
  une seule exception.

  ```
  #880 attend [872]      #818 attend [797, 802, 803]
  #718 attend [872]      #820 attend [797, 802, 803]
  #845 attend [855]      #823 attend [797, 802, 803]
  #581 attend [576]      #427 attend [872]       … 22 sur 22
  ```

  Et l'état de la vague 4 : **27 PR ouvertes, 0 fusionnée.**

  Le déroulé du Gantt est donc allé jusqu'à son butoir. Ce qui l'arrête
  n'est pas un défaut du harnais — c'est le **gate humain** que le plan
  prévoit : la revue de promotion. Sept branches l'attendent, toutes gates
  verts.

- 2026-09-15 — **Sixième défaut du harnais : la reprise ressuscitait les
  instruments périmés.**

  V4.10 a refusé #867 sur « ne porte pas les huit éléments ». Mesurée depuis
  `feature/dev`, elle les porte. Le journal donne la vraie cause :

  ```
  File ".../scripts/backlog-pret.py", line 61, in issues_ouvertes
  FileNotFoundError: No such file or directory: '/home/ubuntu/koprogo'
  ```

  Ligne 61, dans `issues_ouvertes` — le chemin de `main()`, pas celui de
  `--issue`. Le script exécuté n'était donc pas celui de `feature/dev` :
  l'étape « Ouvrir la branche » venait AVANT la mesure, et `story/867`
  datait de la première vague, quand `DEPOT` était encore codé en dur.

  **Le harnais se mesurait avec ses propres instruments d'hier.**

  Règle désormais écrite dans le fichier : *tout ce qui MESURE tourne sur
  `feature/dev`, jamais sur la branche mesurée. Une branche peut contenir du
  code faux — c'est même pour ça qu'on la mesure.*

  Quatrième défaut de la même famille, après « ce n'est pas cette copie-là
  qui s'exécute », « deux instruments mesurent la même chose » et « le jeton
  est posé là où git regarde en dernier ». Le dispositif était juste à
  chaque fois ; le fil était mal branché.

- 2026-09-15 — **Le correctif du rapport porte ses fruits le jour même, et
  il révèle un DEUXIÈME type de silence.**

  Hier, on avait établi que le silence d'un agent était de la variance —
  rejouer la passe produisait du travail. Aujourd'hui, les rapports enfin
  lisibles montrent autre chose :

  > **#802** — « Le travail demandé est déjà fait et fusionné dans `main`.
  > Je m'arrête plutôt que de produire un doublon. » *(avec ses preuves,
  > dont `docs/refonte/CONTRAT_DE_TESTS.md`, commit `39e97277`)*
  >
  > **#803** — « Tout est déjà en place — je m'arrête ici plutôt que de
  > refaire ou dupliquer ce travail. »

  Il y a donc **deux silences**, et ils n'appellent pas le même geste :

  | Silence | Ce qu'il faut faire |
  |---|---|
  | variance | **rejouer la passe** |
  | « déjà fait » | **vérifier, puis fermer l'issue** |

  ⚠️ **Et le second est une AFFIRMATION, pas un constat.** Vérifié pour
  #802 : le fichier cité existe et son commit porte bien `(#802)`. Mais
  l'issue s'intitule « adapter les tests sans en supprimer les règles
  produit qu'ils encodent » — écrire le contrat en est UN livrable, adapter
  les tests en est un autre. L'agent a peut-être conclu trop vite, et ce
  n'est pas à lui d'en juger.

  Les deux issues restent donc **ouvertes**. « Déjà fait » est exactement le
  genre de verdict que la revue de promotion doit trancher, pas le harnais.

- 2026-09-15 — **LE HARNAIS EST COMPLET. Les gates partent seuls.** Mesuré
  sur `story/579`, première branche poussée après le correctif :

  ```
  gh run list --branch story/579
  push  in_progress  CI Pipeline      ← déclenché SEUL
  ```

  Vingt-six branches l'avaient précédée sans jamais y arriver.

  **Le jeton du PO n'a jamais été en cause.** Vérifié sur sa page : PAT à
  portée fine, `gilmry/koprogo`, Contents + Issues + Pull requests +
  **Workflows** en écriture. Le défaut était le CÂBLAGE.

  `actions/checkout` persiste ses identifiants dans un en-tête HTTP —
  `http.https://github.com/.extraheader` — et **cet en-tête prime sur les
  identifiants écrits dans l'URL du remote**. L'étape de poussée faisait
  pourtant `git remote set-url origin "https://x-access-token:$PAT@..."`,
  si bien que git authentifiait avec le `GITHUB_TOKEN` du checkout. Le
  journal annonçait « Jeton dédié présent » : c'était vrai, et inutile — le
  jeton était présent, il n'était pas EMPLOYÉ.

  Le jeton passe désormais à `checkout` par `token:`.

  **C'est la troisième fois de cette campagne qu'un dispositif correct est
  inopérant par un détail de câblage**, après « ce n'est pas cette copie-là
  qui s'exécute » et « deux instruments mesurent la même chose ». Le motif
  est stable, et il mérite d'être nommé comme tel : **ce qui manque n'est
  jamais l'intention, c'est le fil.**

  Conséquence pour la revue de promotion : la **vitrine** va enfin être
  produite. Elle manquait aux vingt-six branches précédentes, et c'est la
  preuve de VALEUR — non bloquante, non facultative — sans laquelle le
  relecteur doit rouvrir le diff.

- 2026-09-15 — **Les deux promotions sont fusionnées.** `main` est à
  `b073fe98` et l'écart avec `feature/dev` est de **zéro commit**. Elle
  porte donc les correctifs qui la bloquaient elle-même — #877 et #718 — et
  ses neuf contrôles requis sont passés verts *parce qu'ils sont réparés*,
  pas parce qu'on les a contournés.

  Conséquence immédiate pour le harnais : le `fanout-stories.yml` de la
  branche par défaut connaît enfin `FANOUT_GITHUB_TOKEN` (8 occurrences) et
  l'épinglage `ref: feature/dev` (2 occurrences). Le premier défaut de la
  vague V4.1 — « ce n'est pas cette copie-là qui s'exécute » — est éteint.

- 2026-09-15 — **État du fan-out après cinq créneaux** : V4.1 à V4.5, **26
  PR d'agent ouvertes**.

  | Verdict | Nombre |
  |---|---:|
  | tous gates verts | **7** |
  | au moins un gate rouge | 14 |
  | gates pas encore déclenchés | 5 |

  Les quatorze rouges se lisent en deux familles, et la revue n'a pas à les
  démêler :

  - **le socle et rien d'autre** — `story/872`, `story/425` : un
    `Integration Tests` hérité de `main`, corrigé depuis ;
  - **leur propre défaut** — Prettier (`731`, `868`, `841`, `798`), tests
    sans catégorie ou unitaires rouges (`576`, `781`, `805`, `867`),
    contrat OpenAPI dérivé (`852`, `635`), BDD (`882`, `635`).

  Aucun de ces défauts n'aurait été refusé par une relecture de diff : du
  code non formaté se lit très bien, et un test sans catégorie aussi.

- 2026-09-14 — **Six branches d'agent passent TOUS les gates.** C'est le
  premier résultat du fan-out directement promouvable, et il vaut d'être lu
  comme tel : la revue n'a plus à ouvrir un diff pour savoir si une branche
  mérite d'être regardée.

  | Branche | PR | Volume |
  |---|---|---|
  | `story/864` | #892 | 2 fichiers, 93 l. |
  | `story/432` | #895 | 2 fichiers, 273 l. |
  | `story/429` | #896 | 5 fichiers, 360 l. |
  | `story/847` | #899 | 1 fichier, 289 l. |
  | `story/453` | #901 | 3 fichiers, 406 l. |
  | `story/595` | #902 | 3 fichiers, 102 l. |

  Bilan sur les vingt branches produites : **6 toutes vertes**, 11 avec au
  moins un gate rouge, 3 en cours. Les rouges se répartissent en deux
  familles que la revue n'a pas à démêler elle-même — les sept de V4.1
  portent un `Integration Tests` qui n'est pas le leur (socle `main`,
  corrigé depuis), les autres portent leurs propres défauts : Prettier,
  tests sans catégorie, contrat OpenAPI dérivé, BDD.

  **Le modèle de promotion tient.** « Le relecteur regarde le film et les
  gates, pas le code » : sur six branches il n'a rien à lire, et sur les
  autres le gate nomme le défaut.

  ⚠️ Ce qui manque toujours : **la vitrine**. Les gates ont été déclenchés à
  la main faute de jeton opérant, ce qui produit les tests mais pas le
  parcours filmé. La preuve de VALEUR — non bloquante et non facultative —
  n'est au rendez-vous d'aucune de ces vingt branches.

- 2026-09-14 — **Le silence des agents était de la VARIANCE, pas un jugement
  sur la story.** C'est le résultat le plus important sur le harnais, et il
  contredit ce qu'on supposait.

  V4.4 a été lancée deux fois, même prompt, mêmes issues, à une heure
  d'intervalle :

  | Story | run `34789291390` | run `34793052029` |
  |---|---|---|
  | #868 | rien | **8 fichiers, 484 l.** |
  | #848 | rien | **17 fichiers, 920 l.** |
  | #731 | rien | **3 fichiers, 200 l.** |
  | #835 | rien | rien |

  Trois agents sur quatre qui s'étaient tus ont produit du travail
  substantiel à la seconde tentative, **sans qu'une ligne du prompt ni de la
  story ne change**.

  On en tirait jusqu'ici la conclusion inverse — « l'agent juge la story
  infaisable et s'arrête, comme le prompt le lui demande ». C'est faux au
  moins trois fois sur quatre. Le bon réflexe devant un silence n'est donc
  pas d'aller relire l'issue, c'est de **rejouer la passe**.

  Conséquence pour le CSI : un taux de passes muettes de 10 sur 30 n'est pas
  un taux de stories mal écrites, c'est un taux de reprise. Le Gantt
  provisionnait justement un facteur **×1,5 pour les reprises** ; il avait
  raison de le faire, et pour une raison qu'il n'avait pas anticipée.

  Corrigé au passage (`c4b3382d`) : le rapport de l'agent partait au seul
  `$GITHUB_STEP_SUMMARY`, que l'API REST n'expose pas — il fallait ouvrir
  trente pages à la main. Il va désormais au journal, à un artefact
  `rapport-<issue>`, et les 400 premiers caractères sont collés dans
  l'avertissement de silence lui-même.

- 2026-09-13 — **Le registre CSI se remplit, et le prior du Gantt est
  CALIBRÉ pour la première fois.** 21 passes, dont 15 avec télémétrie fine
  (V4.2 et V4.3, Sonnet). Le job « Récapitulatif » de la version corrigée du
  workflow fonctionne — il ne fonctionnait pas dans celle de `main`.

  | | Prior du Gantt | Mesuré (médiane) |
  |---|---|---|
  | cache lu / story | ~4 500 000 | **4 312 532** |
  | sortie / story | ~80 000 | **45 256** |
  | coût / story | ~2,70 $ | **1,86 $** |
  | 84 stories, reprises ×1,5 | **340 $** | **234 $** |

  **Le prior tient, et il était prudent dans le bon sens** — c'est ce qu'une
  borne haute doit faire. L'écart vient surtout de la sortie, surestimée de
  près du double.

  La dispersion, elle, est le vrai enseignement : le coût va de **0 à 15 $**
  et les tours de **14 à 233** selon la story. Une médiane par story est donc
  un mauvais instrument de planification — c'est la TAILLE déclarée (S/M/L)
  qu'il faudrait croiser, et le registre le permet désormais.

  Le Gantt écrivait que ses chiffres sont « des bornes hautes de première
  passe, à resserrer story après story sur le réel observé ». C'est la
  première fois qu'on peut le faire.

- 2026-09-13 — **Deux défauts dans MES scripts, trouvés par le fan-out.**
  Quatre agents de V4.4 refusés sur « #868 ne porte pas les huit éléments ».
  C'était faux : le script était mort sur
  `FileNotFoundError: '/home/ubuntu/koprogo'`. **`DEPOT` valait le chemin de
  mon poste** dans `backlog-pret.py` et `backlog-structure.py` ;
  `gantt-passes.py` et `rice-produit.py` le dérivaient déjà. Deux sur quatre,
  et rien ne signalait l'écart.

  Le second défaut est le plus grave : **le workflow confondait une panne et
  un verdict.** `if ! script` traite tous les codes non nuls pareil, si bien
  que le harnais a prononcé un jugement sur quatre stories qu'il n'avait pas
  pu lire. Le script rend désormais **2 pour « je n'ai PAS PU mesurer »**,
  distinct de 1 pour « mesuré et incomplet », et le workflow dit alors
  « corrigez l'instrument, pas l'issue ».

  C'est la règle du registre — l'absence de mesure s'écrit `null`, jamais
  `0` — portée aux codes de sortie.

- 2026-09-13 — **L'épinglage du `checkout` est prouvé PAR CONTRASTE**, et
  c'est la mesure la plus propre de la journée sur le harnais :

  | Vague | Socle | `Integration Tests` |
  |---|---|---|
  | V4.1 | `main` | 🔴 sur **7 branches sur 7** |
  | V4.2 | `feature/dev` | ✅ sur `story/841` |

  Même gate, même code de produit, deux socles. `story/841` ne porte plus que
  **son propre** défaut — Prettier sur deux fichiers. Le bruit du socle a
  disparu, et il ne reste que ce que la revue de promotion doit juger.

  C'est la démonstration que le modèle tient : « le relecteur regarde le film
  et les gates, pas le code » n'est vrai que si les gates parlent de la
  branche. Pendant V4.1, ils parlaient d'autre chose.

- 2026-09-13 — **Vague V4.2 déroulée**, run `34783546008`, lancée
  `--ref feature/dev` pour que ce soit la version CORRIGÉE du workflow qui
  s'exécute. Neuf stories, six branches :

  | Story | Produit |
  |---|---|
  | #635 | 15 fichiers, 1558 l. |
  | #850 | 3 fichiers, 470 l. |
  | #429 | 5 fichiers, 360 l. |
  | #432 | 2 fichiers, 273 l. |
  | #841 | 3 fichiers, 215 l. |
  | #864 | 2 fichiers, 93 l. — **par-dessus le travail de la session** |
  | #870, #585, #854 | aucun changement |

  **#864 est le cas intéressant** : l'agent a branché sur `feature/dev`, donc
  sur le cliquet à 73 et ses 33 exceptions que je venais de poser, et il l'a
  étendu de 61 lignes. Le fan-out capitalise au lieu de refaire — c'est
  exactement ce que l'épinglage du `checkout` devait rendre possible.

  **Des deux correctifs, un seul marche, et c'est mesuré :**

  | | Verdict |
  |---|---|
  | `checkout` épinglé | ✅ `story/841` part de `1d261413`, sommet de `feature/dev` |
  | jeton dédié | ❌ **aucun run `ci.yml` sur push**, malgré « Jeton dédié présent » au journal |

  Les réglages Actions sont permissifs (`allowed_actions=all`,
  `default_workflow_permissions=write`). Le secret existe et le workflow
  l'utilise. **Le jeton ne réveille donc pas les workflows**, ce qui est le
  comportement du `GITHUB_TOKEN`, pas celui d'un PAT. Sa valeur ne peut pas
  être relue par un agent : c'est au PO de vérifier son type.

- 2026-09-13 — **🔴 `main` est bloqué par des défauts que seul `main` peut
  recevoir.** La protection de `main` exige **neuf** contrôles verts —
  protection classique, plus large que le ruleset. Deux sont rouges :

  | Requis | Cause | Corrigé sur `feature/dev` |
  |---|---|---|
  | `Integration Tests (API)` | #877 | `72d719e6` |
  | `Playwright E2E Tests` | #718 | `40eb8edd` |

  La PR #890, qui promeut le seul fichier de workflow, ne peut pas les
  porter. Deux sorties : `--admin`, qui contourne les contrôles que `main` a
  précisément pour empêcher ça, ou **promouvoir la branche entière** — PR
  #894 — où les deux deviennent verts *parce qu'ils sont réparés*.

  #894 est ouverte et **non fusionnée** : elle dépasse ce qui a été validé,
  et la promotion vers `main` reste un geste humain.

- 2026-09-13 — **Les sept branches de V4.1 ont leur verdict de gates**, et il
  se lit en deux colonnes — ce qui est le but du dispositif.

  | Branche | Défaut du SOCLE | Défaut de l'AGENT |
  |---|---|---|
  | `story/872` | Integration (#877) | — |
  | `story/425` | Integration (#877) | — |
  | `story/798` | Integration (#877) | Frontend Check — Prettier, 2 fichiers |
  | `story/576` | Integration (#877) | Unit Tests |
  | `story/781` | Integration (#877) | `garde_taxonomie_des_tests` — tests sans leur catégorie |
  | `story/805` | Integration (#877) | Unit Tests |
  | `story/852` | Integration (#877) | Lint, Contract Types, Unit Tests |

  **`Integration Tests` rouge sur les SEPT** : c'est #877, corrigé à 15h sur
  `feature/dev` et absent du socle depuis lequel les agents ont branché. Pas
  un seul agent n'y est pour quelque chose. C'est le quatrième défaut de la
  vague, corrigé par `b094778d` — le `checkout` du job agent est désormais
  épinglé à `feature/dev`.

  **Deux branches sur sept sont propres** une fois le socle mis de côté.
  Cinq portent des défauts réels, tous attrapés par des gardes du dépôt :
  du code non formaté, des tests sans leur catégorie
  `@happy/@negative/@edge/@security`, un contrat OpenAPI dérivé.

  C'est exactement ce que le Gantt promettait — « le relecteur regarde le
  film et les gates, pas le code » — et c'est la première fois que la
  promesse est vérifiable sur du réel. Aucune de ces cinq branches n'aurait
  été refusée par une relecture de diff : du code non formaté se lit très
  bien.

  ⚠️ **La vitrine manque encore.** Les gates ont été déclenchés à la main
  (`gh workflow run ci.yml --ref story/<n>`), ce qui produit les tests mais
  pas le parcours filmé. La preuve de VALEUR, non bloquante et non
  facultative, n'est pas au rendez-vous de cette première vague.

- 2026-09-13 — **LE GANTT TOURNE. Vague V4.1 déroulée en réel**, run
  `34776213266` : dix stories, deux agents simultanés (`max-parallel: 2`
  tenu, mesuré sur les horodatages), abonnement, Sonnet.

  | Story | Domaine | Produit |
  |---|---|---|
  | #872 | harnais | 4 fichiers, 176 l. — **le livrable 4 qui manquait** : le refus explicite d'un hôte non amorcé |
  | #798 | front/composants | 10 fichiers, 977 l. |
  | #576 | back/copropriete | 14 fichiers, 919 l. |
  | #781 | back/communaute | 18 fichiers, 752 l. |
  | #805 | docs-vivante | 9 fichiers, 953 l. |
  | #425 | meta | 1 fichier, 345 l. |
  | #852 | back/comptabilite | 5 fichiers, 196 l. |
  | #694, #515, #869 | — | **aucun changement**, 26 à 30 s chacun |

  Sept PR en brouillon, #883 à #889.

  **Trois défauts que seule l'exécution pouvait montrer :**

  1. **Le fan-out a tourné avec une version périmée de lui-même.**
     `workflow_dispatch` exécute le fichier de la branche PAR DÉFAUT, et
     `main` portait un `fanout-stories.yml` antérieur de **238 lignes**,
     sans aucune mention de `FANOUT_GITHUB_TOKEN`. Les sept branches sont
     donc parties avec le `GITHUB_TOKEN` — zéro gate, zéro vitrine —
     pendant que le corps des PR annonçait « les gates partent seuls ».
     C'est le motif dominant du dépôt sous une forme nouvelle : un
     dispositif corrigé, présent, et inopérant parce que **ce n'est pas
     cette copie-là qui s'exécute**. Promotion validée par le PO et en
     cours (PR #890). Gates déclenchés à la main entre-temps.
  2. **Trois agents sur dix n'ont rien produit**, et on ne sait pas
     pourquoi : leur rapport part au résumé d'étape, que l'API GitHub
     n'expose pas. C'est un trou du dispositif, pas une conclusion.
  3. **Le registre CSI n'a reçu aucune ligne.** Le job « Récapitulatif »
     n'existe pas dans la version de `main`. Dix passes réelles, télémétrie
     perdue — pour la deuxième fois, après le run `34739504910`.

- 2026-09-13 — **`feature/dev` poussée après les seize gardes vertes.** Le
  barrage est passé, les images `sha-69b915f7` sont publiées : le correctif
  de #718 et le passage de MinIO sur quay.io partent en production.

- 2026-09-13 — **#864 : le cliquet CLASSE, et le relevé passe de 95 à 73.**
  Vingt-deux routes cloisonnées dans la journée. Les dix transitions d'état
  (budget, état daté), puis les quatre POST d'assemblée — annuler, clôturer,
  reporter, valider le quorum, c'est-à-dire l'Art. 3.87 — les quatre
  transitions de dépense, `assign_owner`, et `create_quote` dont l'identité
  était nommée `_auth`.

  **Deux d'une forme que ni l'issue ni le cliquet ne cherchaient** :
  `list_call_for_funds` et `get_contributions_by_owner` cloisonnaient
  correctement dans leur branche nominale et **pas du tout** dans leur
  branche filtrée. Un paramètre facultatif — `building_id`, `owner_id` —
  contournait le chemin protégé, et les deux fois sur la même donnée : qui
  doit combien. Aucun cliquet qui compte des *gestionnaires* ne voit cette
  forme-là.

  Le cliquet porte désormais **33 exceptions, chacune lue, chacune avec sa
  raison écrite**, et un second compteur : **40 routes ni corrigées ni
  lues**, qui ne peut que descendre. Un troisième test garde la liste
  honnête — une exception qui ne correspond plus à rien fait rougir.

  Témoin de rougeur, gardes retirées : `POST /meetings/{id}/cancel` rend
  **200 OK** au syndic d'une autre organisation, et annule son AG.

  **Une catégorie manquait à l'énoncé de l'issue, et c'est la plus
  dangereuse** : « rôle contrôlé, périmètre NON ». `check_syndic_role`,
  `check_accountant_role`, `check_unit_ownership_permission` — un syndic du
  cabinet A y approuve la facture du cabinet B. Le cliquet les compte **par
  accident**, parce que le nom du helper n'est dans aucun motif. Renommer
  l'un d'eux `verify_*` les sortirait du compte **sans les corriger**.

- 2026-09-13 — **LE GATE `e2e` EST VERT.** `make test-e2e` rend
  **308 ✓ / 0 ✘ / 14 sautés, code 0**, sur la pile de recette, sans
  redémarrage du backend pendant. C'est le même chiffre que la CI, ce qui
  clôt l'écart entre les deux bancs — il venait entièrement de #718.

  Trois campagnes, même code sauf un correctif :

  | Campagne | Résultat |
  |---|---|
  | contaminée (deux recompilations pendant) | 213 ✓ / 95 ✘ |
  | propre | 298 ✓ / 10 ✘ — **dix 502** |
  | après `40eb8edd` | **308 ✓ / 0 ✘ — code 0** |

  La latence de `register` n'a PAS bougé — médiane 1,66 s contre 1,69 s — et
  c'est la preuve que le correctif ne triche pas : le coût bcrypt est
  intact, c'est la file d'attente qui a disparu. Le maximum tombe de 3,84 s
  à 2,63 s, et plus aucun appel ne dépasse 3 s.

- 2026-09-13 — **#718 EXPLIQUÉ, et corrigé.** C'est le résultat de la
  journée.

  La campagne propre a rendu **298 ✓ / 10 ✘ / 14 sautés**, sans un seul
  redémarrage du backend pendant (`SIGTERM` à 2 avant comme après). **Les dix
  échecs sont des 502, sans exception** — huit portent littéralement
  `seed:org: HTTP 502 — Bad Gateway`. Aucun n'est un défaut produit.

  La cause tient en deux lignes. `auth_use_cases.rs:141` et `:79` appelaient
  `hash` et `verify` **synchrones, dans des `async fn`, sans
  `spawn_blocking`**. Relevé sur 161 643 lignes de journal :

  | Route | Appels > 1 s |
  |---|---:|
  | `POST /auth/register` | **722** — médiane 1,69 s, p90 2,02 s, max 3,84 s |
  | `POST /auth/login` | **281** |

  Pendant ces 1,7 s le thread de travail Actix ne rend la main à rien. Avec
  deux workers, deux inscriptions simultanées consomment toute la capacité ;
  la troisième requête attend et Traefik rend 502.

  ⚠️ **La démo tourne avec `ACTIX_WORKERS: 1`** (`docker-compose.prod.yml:101`).
  UNE connexion y bloque toute l'API pendant 1,7 s. Sur un produit fait pour
  des assemblées générales, le seuil est franchi au premier usage réel. Le
  correctif ne touche pas ce réglage : **il reste à trancher, et c'est une
  modalité du PO.**

  Corrigé par `40eb8edd`, avec un témoin **déterministe** plutôt que
  chronométré : sur un runtime `current_thread`, une tâche témoin qui
  `yield_now()` ne progresse que si le hachage libère le thread. Blocage
  réarmé → FAILED ; correctif → 12 passed. Un test chronométré aurait
  clignoté sur un hôte à trente conteneurs, et un test qui clignote finit
  désactivé — c'est ainsi que ce blocage a survécu.

- 2026-09-13 — **#832 est répondu, et la réponse tient en une ligne.** La
  question était de départager « cascade d'un 502 » et « défaut réel » pour
  chaque spec rouge. Les dix sont du premier type. Il n'y a pas de défaut
  réel à instruire dans ce lot.

- 2026-09-13 — **#879 relancée : les gates disent en vingt minutes ce que la
  relecture du diff n'aurait pas vu.** Run `34764114133`. La branche
  `story/867` n'apporte qu'un fichier : `backend/tests/e2e_stats_owner_dues.rs`,
  412 lignes. Verdict :

  | Gate | Résultat |
  |---|---|
  | Unit Tests | 🔴 `chaque_harnais_est_execute_quelque_part` |
  | Integration Tests | 🔴 #877 (corrigé sur `feature/dev`, pas sur cette branche) |
  | Playwright E2E | 🟢 **308 ✓ / 0 ✘** |
  | vitrine | 🟢 artefact de 79 Mo, publié |
  | lint, BDD, contrat, front | 🟢 |

  Le message du garde se suffit à lui-même :

  > Ces harnais ne sont cités par AUCUN workflow, donc ne s'exécutent jamais :
  > `e2e_stats_owner_dues`. Ils compilent, ils passent en local, et la CI
  > reste verte sans les avoir vus.

  **L'agent a livré un harnais dormant.** C'est le motif dominant du dépôt,
  reproduit par la première passe de fan-out, et c'est un garde du dépôt qui
  l'a arrêté. Sans le jeton, la PR aurait été relue sur son diff et ce défaut
  serait passé : un fichier de test qui compile et que rien n'exécute ne se
  voit pas à la lecture.

  ⚠️ **Le résultat le plus utile du run n'est pas là.** La campagne Playwright
  passe à **308 ✓ / 0 ✘** en CI, sur le même code qui rendait 12 échecs sur
  la pile de recette de cet hôte. L'écart n'est pas dans le produit, il est
  dans le banc : quatre cœurs, trente conteneurs, et un backend en hot
  reload. C'est la matière de #718 et de #880.

- 2026-09-13 — **#877 est VERT, mesuré.** `kcargo test --test storage_s3` :
  `1 passed`, code 0, contre `quay.io/minio/minio`.

- 2026-09-13 — **#864 : dix transitions d'état cloisonnées, cliquet 95 → 85.**
  Les cinq `PUT /budgets/{id}/*` et les cinq `PUT /etats-dates/{id}/*`
  prenaient `AuthenticatedUser` sans s'en servir pour décider, dans des
  fichiers où la LECTURE cloisonne correctement depuis toujours. Le test est
  rouge sans le correctif, et c'est démontré : handler remis dans son état
  d'avant, `PUT /budgets/{id}` rend **200 OK** au syndic d'une autre
  organisation.

  **Le compteur n'a pas bougé au premier essai**, et c'est le fait
  intéressant. Les helpers s'appelaient `cloisonner_*`, que le détecteur ne
  connaît pas : dix trous bouchés, instrument aveugle. Allonger `DECISION`
  aurait fait tomber le chiffre par une modification de sa définition. Les
  helpers portent désormais `verify_*`, l'idiome que le dépôt emploie déjà
  — le compteur suit le travail, pas le barème.

- 2026-09-13 — **#880 ouverte : le gate e2e mesure contre un backend en hot
  reload.** Découvert en le subissant. Deux éditions de fichiers Rust
  pendant une campagne ont déclenché deux recompilations (`cargo-watch`,
  `Dockerfile.dev:71`), coupant le service à 14:54:29 puis 14:59:02.
  Résultat : **213 ✓ / 95 ✘**, dont **83 échecs ayant visé un backend mort**.
  Aucun artefact ne distingue les deux populations ; le code de sortie vaut 2
  dans les deux cas. Ce n'est pas l'étourderie qui compte, c'est que le banc
  est unique et que le fan-out y fera tourner N agents qui écrivent du Rust.

  Deux défauts d'outillage trouvés en enquêtant, tous deux corrigés :

  - le **rapport JSON** de Playwright avait son chemin en dur : l'exécution
    ciblée lancée pour instruire les échecs a effacé les messages d'erreur
    qu'elle servait à expliquer (`5ce48d93`). Le rapport HTML, lui, était
    paramétré depuis #873 — pour exactement cette raison ;
  - **`make seed-reset` annonçait ✅ sur un refus de l'API** (`0713da2b`).
    `{"error":"Scenario world already exists"}` et « ✅ Seed world reset » sur
    deux lignes consécutives, code de sortie 0 — le `| head -c 200` rendait
    le statut de `head`. Une précondition de recette qui ment fait démarrer
    la campagne suivante sur un état inconnu. `seed-clear` ajoutée, témoin de
    rougeur vérifié.

- 2026-09-13 — **le gate `doc-vivante` mérite une nuance qu'il n'avait pas.**
  Le *parcours* (vitrine) est bien vert et son artefact fait 79 Mo. Mais les
  douze `.scenario.ts` du même job rendent **10 ✓ / 2 ✘**
  (`meeting-vote`, `sel-exchange`) et le job reste `success` : l'étape porte
  `continue-on-error: true` depuis une décision du 2026-06-15, assortie d'une
  condition de retrait — « toutes les sub-tasks C-Scen DONE et 0 flake sur
  3 runs » — que personne n'a rouverte depuis. Une concession datée qu'on
  ne réexamine pas devient un gate qui ne dit plus ce qu'on croit.

- 2026-09-13 — **#877 tranchée sur une preuve qui a corrigé son diagnostic.**
  L'issue disait « l'éditeur a retiré CE tag ». Mesuré : c'est **tout**
  `docker.io/minio/minio` qui a disparu, `:latest` compris, et `minio/mc`
  avec. « Rafraîchir le tag » sortait donc de l'arbitrage — il n'y avait
  plus rien à rafraîchir. Le PO a tranché **quay.io, épinglé par tag,
  partout** (`72d719e6`).

  **Ce que la recherche a trouvé au passage est plus grave que le gate.** La
  DÉMO tourne sur `minio/minio:latest`, non tirable depuis. Elle ne
  fonctionnait que par le cache d'images de cet hôte : un `compose pull`, un
  `image prune` ou une reconstruction ailleurs, et le stockage objet ne
  revenait pas. Rien ne l'aurait annoncé avant le redémarrage. Le correctif
  ne déplace aucun bit, et c'est vérifié : `koprogo-minio` et
  `quay.io/minio/minio:RELEASE.2025-09-07T16-13-09Z` portent le **même**
  `sha256:14cea493d9a3`. Le `:latest` du cache ÉTAIT cette release.
  ⚠️ **La protection ne prend effet qu'au prochain déploiement de la démo.**

- 2026-09-13 — **les trois verrous de #874 sont tombés.**
  `FANOUT_GITHUB_TOKEN` posé par le PO à 14:48 UTC,
  `default_workflow_permissions` passé de `read` à `write`, label `agent`
  présent. Le fan-out peut produire des branches instruites par leurs gates.
  Voie retenue : **abonnement, `max_parallel` à 2**.

- 2026-09-13 — **#879 relancée pour ses gates** (run `34764114133`,
  `workflow_dispatch` sur `story/867`). Elle avait été poussée AVANT le
  jeton : seul CodeQL s'était déclenché, ni `ci.yml` ni la vitrine. La
  relire ainsi aurait chronométré une revue sur diff, c'est-à-dire
  exactement ce que le modèle de promotion existe pour éviter.

- 2026-09-13 — **l'hôte a redémarré à 14:14 UTC et la pile de recette n'en
  est pas revenue.** Les cinq conteneurs `koprogo-dev-*` étaient en
  `Exited (137)` ; la démo, elle, était remontée seule. Cause : la pile de
  recette n'a pas de `restart:`, ce qui est cohérent avec « jetable » mais
  n'était écrit nulle part. **Après chaque redémarrage du VPS, la recette
  est à relever à la main** — `docker compose -p koprogo-dev up -d`.
  Isolation revérifiée au relevé : les quatre conteneurs de la démo
  identiques au caractère près, `api.koprogo.com` à 200.
- 2026-09-13 — **#872, dernier livrable (4) fermé côté agent.** Le garde
  d'identifiants (#870) laissait passer un hôte distant dès qu'un mot de
  passe — n'importe lequel — était choisi. Exigence supplémentaire :
  `KOPROGO_CONFIRME_HOTE_DISTANT`, une confirmation disjointe de
  l'identifiant, pour le cas que le garde ne peut pas juger sans se
  connecter — un mot de passe qui fonctionne. Guide E2E mis à jour. **Non
  exécuté par l'agent** : les commandes `docker compose` de vérification ont
  été refusées par le mode de permission de la session ; à faire tourner
  humainement (`cd frontend && npm run test -- garde-identifiants`).
- 2026-09-12 — **La pile de recette a tourné pour la première fois.** Trois
  défauts que seule l'exécution pouvait montrer : `JWT_SECRET` absent
  (`489a5473`), `PLAYWRIGHT_API_BASE` retombant sur le port 80 dans 93 fichiers
  (`fc6251ad`, centralisé dans `helpers/adresses.ts` + garde), et le gate
  `integration` déclaré vert alors qu'il est rouge (#877). Puis le gate :
  300 ✓ / 8 ✗, dont 4 en 502 sous rafale — **#718 reproduite hors production
  pour la première fois**.
- 2026-09-12 — **ADR 0050 exécutée (`0e3036d8`), #872.** Quatre ports décalés,
  `RECETTE` nommée dans le Makefile, guide E2E corrigé, garde étendue aux
  ports et à toutes les paires de piles. Trois trouvailles au passage :
  `PUBLIC_API_URL` visait encore le 80, `docker-compose.mcp.yml` n'avait
  toujours pas de `name:`, et les 5432/9000/9001 de l'hôte appartiennent à des
  projets voisins, pas à la démo.
- 2026-09-12 — **#876 corrigé (`597b3eeb`).** La vitrine se publiait et se
  téléchargeait en filmant son propre échec : le parcours lisait un TestWorld
  qu'aucun `globalSetup` n'écrit, la configuration n'en déclarant aucun. Le
  parcours amorce désormais son monde. Preuve CI en attente.
- 2026-09-12 — **#876 : la vitrine au moule Foyer.** #873 rend les vidéos
  téléchargeables, pas lisibles. `skills/documentation-vivante.md` pose quatre
  éléments non optionnels et `kit-actix` en donne le patron pour cette pile ;
  KoproGo n'en a **aucun**. La barrière d'habilitation passe à trois couches :
  #873 → #876 → #874. 87 issues, 74,75 j, 299 tours.
- 2026-09-12 — **CI débloquée** : `playwright` était **sauté depuis cinq runs**,
  `frontend-check` échouant sur un Prettier dans un fichier non touché par le
  pilote. Le gate `e2e` n'était pas rouge — il n'était pas exécuté.
- 2026-09-12 — **l'arbitre de promotion tracé, des deux côtés.** #875 (hors
  jalon) relève le coût réel d'un arbitrage sur la première vague ; l'issue
  `gilmry/foyer#7` porte la persona `arbitre-de-promotion`, **qui n'existe
  pas** dans la méthode — Foyer a des rôles qui produisent et qui mesurent,
  personne qui juge sur preuve. L'ordre est écrit dans les deux : on mesure
  sur le dogfood, on généralise ensuite.

  Ce que la mesure doit établir : le `ratio_supervision` de l'abaque ne
  disparaît pas quand on déplace la supervision vers la promotion — **il
  change d'unité**, et devient pour la première fois mesurable (minutes par
  branche). Le signal le plus important n'est pas la durée mais **le nombre
  de fois où l'arbitre a dû ouvrir le diff** : chaque occurrence est un échec
  de la preuve, pas de l'arbitre.
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
