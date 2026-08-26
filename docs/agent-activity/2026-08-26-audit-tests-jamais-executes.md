# Agent activity — 2026-08-26 — Audit : les tests écrits mais jamais exécutés

**Persona :** audit qualité + outillage CI (Tier 2), branche `story/661-followup-payment-reminder-decimal`.

**Déclencheur :** question directe de @gilmry — « il y a d'autres tests qui sont écrits et jamais exécutés, fais le point ». Elle est partie d'un constat fait en traitant #661 : les assertions renforcées de `e2e_ag_sessions.rs` étaient « laissées à la CI », alors que la CI ne joue pas ce fichier.

## Constat : ~36 800 lignes de test n'ont jamais rien vérifié

| | Lignes | Jamais exécutées |
| --- | ---: | ---: |
| Harnais `backend/tests/*.rs` | 66 001 | **32 461 (49 %)** |
| — dont 59 fichiers `e2e_*.rs` | 27 134 | 27 134 (100 %) |
| — dont 11 harnais BDD/intégration | 5 327 | 5 327 (100 %) |
| Fichiers `.feature` | 10 497 | **4 378 (42 %)** |

Pour situer : le backend fait ~117 000 lignes de code de production. L'équivalent d'un tiers de cette base existe en tests qui n'ont jamais tourné — coût de maintenance intégral, bénéfice nul.

### Trois causes, toutes documentaires

Le point commun est frappant : **dans les trois cas, un commentaire ou un nom affirme une couverture que la commande ne fournit pas**. C'est pour cela que des relectures successives n'ont rien vu.

1. **Job BDD** — commentaire : *« --no-fail-fast: run ALL bdd\*.rs binaries […] so the full BDD picture surfaces in one CI run »*. La commande listait **5 harnais sur 13**.
2. **Job Playwright** — step intitulé *« Run Playwright smoke tests »*, exécutant `--project=chromium`, dont la config porte `testIgnore: [/smoke\//, …]`. Il **exclut** smoke. Les 98 tests du projet `smoke` n'ont jamais tourné.
3. **Job E2E** — `cargo test --test e2e` cible **un seul fichier** (`tests/e2e.rs`, 4 tests), pas les 59 `e2e_*.rs` (488 tests).

À quoi s'ajoutent deux commandes documentées qui échouent : `cargo test --test integration` (harnais inexistant) et `make coverage` (appelait `cargo tarpaulin`, jamais installé).

### Conséquence sur la lecture du WBS

Plusieurs items cochés « FAIT », dont des **bloqueurs légaux**, reposaient sur une exécution locale ponctuelle jamais rejouée : WP-H3 (`bdd_meeting_complete`, Art. 3.87 §3-5), WP-CL1 (`bdd_acp`, Art. 3.84), WP-H1 (`bdd_building_conformity`), WP-H2 (`bdd_validate_before_compute`).

## Mesure : il n'y avait pas de mur

Les 8 harnais BDD jamais exécutés ont été lancés un par un (séquentiel, `-j 1` — voir §Incident) :

| Harnais | Résultat | Correctifs |
| --- | --- | --- |
| `bdd_voting_right` | 5/5 | — |
| `bdd_meeting_complete` | 14/14 | — |
| `bdd_list_buildings_role_based` | 8/8 | — |
| `bdd_building_conformity` | 9/9 | — |
| `bdd_acp` | 17/17 | — |
| `bdd_portfolio` | 11/11 | — |
| `bdd_validate_before_compute` | 5/5 (1 skipped) | — |
| `bdd_iot` | 21/21 (1 skipped) | **4** |
| **Total** | **90 verts, 2 skipped, 0 échec** | |

**Sept harnais sur huit passent sans rien toucher.** Le coût de les brancher était nul ; il ne manquait qu'une ligne de commande.

### Les 4 défauts de `bdd_iot` — le prix de la non-exécution

1. `127.0.0.1` en dur au lieu de `container.get_host()` → `PoolTimedOut` sur 15 scénarios. **Le correctif existait déjà** dans `bdd_governance` et `bdd_financial`, avec un commentaire nommant ce symptôme exact. `bdd_iot` ne l'a jamais reçu, n'étant dans le périmètre d'aucune story.
2. Insertion d'une colonne `owners.name` supprimée depuis (refactoring du modèle Owner).
3. Colonnes `NOT NULL` ajoutées depuis (`address`, `city`, `postal_code`, `country`) et absentes de l'insertion.
4. Deux assertions comparant un **nom de variante** d'erreur à un **message d'affichage** (`InvalidTopic`, `TaskNotFound`) : structurellement incapables de passer.

Sur le point 4, `TaskNotFound` mérite d'être relevé : la variante n'est **pas observable** parce que `poll_task` renvoie `Result<_, String>` et aplatit l'erreur en texte. Le scénario a donc été rabattu sur le message, avec un commentaire renvoyant au critère GO « aucun `Result<_, String>` », toujours ouvert.

## État de la couverture (jamais mesurée avant ce jour)

Aucun job de couverture n'existait, et `cargo tarpaulin` n'était installé nulle part.

**Structurel** — la fracture est nette entre le domaine et l'infrastructure :

| Couche | Fichiers | Avec tests unitaires | Sans rien |
| --- | ---: | ---: | ---: |
| Entités domaine | 68 | **68 (100 %)** | 0 |
| Use cases | 67 | **64 (96 %)** | 3 |
| Handlers web | 73 | **12 (16 %)** | 22 |
| Repositories | 68 | **5 (7 %)** | ~63 |

L'effort de test pour l'infrastructure **existe** : il est dans les 27 134 lignes d'`e2e_*.rs`. Il n'est simplement pas branché. Sur 606 endpoints exposés, la CI n'en vérifiait directement que ceux traversés par `e2e.rs` (4 tests) et les scénarios BDD joués.

**Mesuré** — première valeur connue, frontend (vitest v8, 352 tests unitaires) :
`Statements 12,01 % · Branches 14,85 % · Functions 10,98 % · Lines 14,31 %`
Playwright s'exécutant contre un serveur, hors du processus, sa couverture n'y figure pas.

## Trancher : les « e2e » backend sont des tests d'intégration

Vérifié sur les **59 fichiers sans exception** : tous montent l'application Actix en mémoire (`test::init_service`), **aucun** ne passe par un socket réseau. Ils exercent handler → use case → repository → PostgreSQL réel, sans navigateur ni déploiement. C'est de l'**intégration API**. Le seul e2e réel du projet est Playwright.

La pyramide n'a donc pas besoin d'un étage supplémentaire — elle en a quatre, dont deux débranchés et un mal nommé :

| Étage | Existant | En CI avant | En CI après |
| --- | --- | --- | --- |
| Unitaire (domaine) | 1 669 tests | ✅ | ✅ |
| Intégration composant | 3 fichiers | ❌ | à brancher |
| Intégration API (« e2e\_\*.rs ») | 488 tests | 4/488 | à brancher par lots |
| BDD / acceptation | 1 108 scénarios | 597 | **687** |
| E2E réel (Playwright) | ~390 tests | ~260 | smoke à rebrancher |

**Renommage `e2e_*.rs` → `it_*.rs` : différé et assumé.** 59 renommages plus le `Cargo.toml` polluent l'historique sans rien apporter tant que rien ne tourne. Contre-argument dû : tant que le nom reste, le malentendu se reproduira.

## Livré

- **CI, job BDD** : 5 → 13 harnais, avec le commentaire rectifié sur place.
- **CI, nouveau job `coverage`** : `cargo-llvm-cov` (backend) + vitest v8 (frontend), artefacts uploadés. En `continue-on-error` **volontairement** : poser un seuil avant de connaître la valeur réelle bloque la CI sans rien apprendre. Le plancher se fixe une fois la pyramide branchée, juste sous la valeur constatée.
- **`cargo-llvm-cov`** ajouté au `Dockerfile.dev` — et non tarpaulin, qui instrumente via `ptrace`, ne couvre pas les binaires d'intégration, et ne tiendrait pas sur une machine de 8 Gio avec ce crate.
- **`Makefile`** : `coverage`, `coverage-backend`, `coverage-backend-lib`, `coverage-frontend`.
- **`vitest.config.ts`** : provider v8, rapports `text-summary`/`html`/`lcov`, sans `thresholds` (même raison).
- **`CLAUDE.md`** : les deux commandes fausses corrigées.
- **`bdd_iot`** : 4 correctifs, harnais vert.

## Incident — la mesure a saturé la machine

Première tentative : `cargo test --no-fail-fast` sur l'ensemble. Après **2 h**, zéro test exécuté. Diagnostic : swap saturé (2,0 / 2,0 Gio), 132 Mio de RAM libre sur 7,6 Gio, **61 % du temps CPU en attente d'I/O**. La machine ne compilait plus, elle swappait — et `koprogo.com`, hébergé sur cette même machine, répondait en 5,7 s au lieu de 0,06 s.

Compiler 75 binaires de test, chacun liant un crate de 117 000 lignes, ne tient pas dans 7,6 Gio. Reprise en séquentiel `-j 1`, un harnais à la fois : ~4 min de compilation chacun, puis **~10 s d'exécution**.

**C'est une contrainte à retenir pour la CI** : brancher les 488 tests d'intégration API demandera de découper les jobs, pas seulement d'allonger une liste.

## Ce que l'échantillon d'intégration API a révélé

Six harnais sur 59 lancés (les plus gros, les plus sensibles, et un témoin touché par #661). Deux défauts majeurs sont sortis dès les trois premiers — et aucun n'aurait été trouvé autrement.

### 1. Un endpoint cassé en production depuis des mois

`e2e_api_keys` : 3 passés, 10 échecs, **tous en 403**.

`api_key_handlers.rs` comparait `claims.role` à `"SYNDIC"` / `"SUPERADMIN"` en **majuscules**. Le JWT porte `UserRole::to_string()`, qui rend `"syndic"` / `"superadmin"` en **minuscules** : la condition était toujours vraie. Création, listage, révocation et rotation de clé répondaient **403 à tout le monde**, syndic légitime inclus.

Ce bug était identifié dès **avril 2026** dans le WBS (*« compare les rôles en MAJUSCULES alors que le JWT retourne des minuscules → 403 sur tous les endpoints »*) et n'a jamais été corrigé. Le WBS coche par ailleurs WP-A7 / #339 « rotation implémentée, 4-cat RED-first, **aucun 501 ne part en bêta** » : les tests 4 catégories existent bel et bien dans `e2e_api_keys.rs` — ils n'ont simplement jamais été exécutés. L'endpoint ne renvoyait pas 501, il renvoyait 403.

Même bug de casse trouvé dans `ag_session_handlers.rs`. Les deux sont corrigés par des helpers insensibles à la casse.

**C'est l'argument le plus fort de tout cet audit** : les tests débranchés ne coûtaient pas seulement leur maintenance, ils *masquaient* un endpoint mort en production.

### 2. Un drift de contrat introduit par ce chantier même

`e2e_ag_sessions` a échoué sur `body["combined_percentage"].as_f64()` → `None`.

Les commits #661 et « pénalités » affirmaient tous deux **« aucun drift de contrat API »**. C'était **faux**. L'affirmation reposait sur l'absence de ces DTO dans `openapi.json`, pas sur le comportement de sérialisation : `rust_decimal` sérialise un `Decimal` en **chaîne** par défaut, et la feature `serde-with-float` ne change pas ce défaut — elle fournit un module à appliquer explicitement, ce que `dashboard_dto.rs` faisait déjà (*« serialized as JSON number for frontend display »*).

Les conversions faisaient donc passer `combined_percentage` de `60.0` à `"60"`, et de même pour les montants de budget et de relances. **43 champs** ont été annotés `#[serde(with = "rust_decimal::serde::float")]`.

Sans ce test — celui-là même que je décrivais comme « laissé à la CI » alors que la CI ne le joue pas — ce changement silencieux de type serait parti en production.

### Taux constaté — échantillon complet (6 harnais, 97 tests)

| Harnais | Avant | Après correction |
| --- | --- | --- |
| `e2e_buildings` | 7/8 | **8/8** |
| `e2e_api_keys` | 3/13 | **13/13** |
| `e2e_ag_sessions` | 7/8 | **8/8** |
| `e2e_convocations` | 24/24 | 24/24 |
| `e2e_gdpr` | 21/21 | 21/21 |
| `e2e_payments` | 23/23 | 23/23 |
| **Total** | 85/97 | **97/97** |

**Trois défauts au total sur 97 tests**, et un seul relevait d'une dérive de test :

1. le **bug de production** des clés API (10 échecs à lui seul) ;
2. le **drift de contrat** introduit par ce chantier même ;
3. un payload resté sur `organization_id` après la Story 1.2 — et pour lequel `common::create_test_acp` existait déjà, avec une documentation mettant explicitement en garde contre cette erreur.

Autrement dit : ces 488 tests d'intégration sont un **actif dormant en bon état**, pas une dette. Le coût de les brancher tient à la contrainte de compilation, pas à leur réparation.

### Taux constaté

| Harnais | Résultat | Nature |
| --- | --- | --- |
**Note de méthode** : la première passe de mesure a été invalidée par mes propres modifications de code, concurrentes de l'exécution. Mesurer un mur exige un arbre figé.

## Reste ouvert

- Les 488 tests d'intégration API : jamais exécutés, non mesurés ici (échantillonnage à prévoir).
- Projet Playwright `smoke` (98 tests) exclu par le step qui prétend le lancer ; `characterization` (22) limité aux PR vers `main`/`dev`, jamais sur `feature/dev` ; `scenarios` en `continue-on-error`.
- `koprogo-grid` et `load-tests` : hors de tout workflow.
- 3 use cases sans test, 22 handlers sans test d'aucune sorte.
- Plancher de couverture à fixer une fois la pyramide branchée.
