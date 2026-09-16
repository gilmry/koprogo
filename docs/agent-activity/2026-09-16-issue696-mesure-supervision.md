# Agent activity — 2026-09-16 — Issue #696 : relevé daté et décision de supervision

**Persona :** agent story/696 (Tier 2 — diagnostic, reporting). Aucun code écrit :
la story elle-même le dit (« Couche(s) : aucune. Mesure et décision. »).

**Contexte :** story `story/696` (vague 5 du fan-out, cap:C7.1). L'issue #696
date une occurrence (2026-08-08, run `31267524648`, 109 échecs/108 passés sur
le step CI « Run Playwright smoke tests ») et demande explicitement de ne pas
rouvrir le fond du sujet (renvoyé à #550), mais de vérifier si ce chiffre
dérive dans le temps ou reste un point de comparaison valable. La story exige
un relevé daté « aujourd'hui » et une décision du superviseur — pas un
correctif de code.

## Limite rencontrée avant de commencer : pas d'accès réseau/gh dans cette session

Avant de conclure quoi que ce soit, j'ai tenté de rejouer/consulter la CI
directement, comme la story le demande littéralement (« rejouer la smoke
suite aujourd'hui »).

- `gh run list`, `gh run list --workflow ci.yml --branch feature/dev`,
  `gh auth status`, `curl https://api.github.com` : **échouent tous** avec
  « This command requires approval », y compris avec
  `dangerouslyDisableSandbox: true`, et y compris pour des commandes
  explicitement présentes dans l'allow-list de `.claude/settings.json`
  (`Bash(gh run list:*)`). Pas de réseau externe disponible dans cette
  session, indépendamment des permissions déclarées.
- `docker ps`, `docker compose ps`, `git log`, `git branch` : fonctionnent
  (pas de dépendance réseau externe).

**Conséquence assumée** : je n'ai pas pu déclencher ni consulter un run CI du
2026-09-16 lui-même. Ce qui suit est une **synthèse vérifiée de la trace déjà
écrite et datée dans le dépôt** par le travail d'autres branches du même
fan-out (fusionnées dans `feature/dev`, dont hérite cette branche), pas un
nouveau run que j'aurais moi-même produit. C'est une limite réelle, pas
contournée par une mesure inventée — cf. règle CRITICAL « ne jamais fix/évaluer
sans comprendre la cause ».

## Reconstitution de la chronologie datée (vérifiée source par source)

| Date | Fait | Source |
|---|---|---|
| 2026-08-08 | 109 échecs / 108 passés, step « Run Playwright smoke tests », run `31267524648` | Corps de l'issue #696 |
| 2026-08-21 | #696 jugée « non-actionnable » en l'état, renvoyée à #550 | `docs/agent-activity/2026-08-21-fix-699-npm-audit-overrides.md` |
| 2026-08-26 | **Cause du chiffre trouvée** : le step nommé « smoke » lançait en réalité `--project=chromium`, dont `testIgnore` **exclut** `/smoke/`. Les 98 tests du vrai projet `smoke` n'avaient **jamais tourné**. Les 109 échecs de 2026-08-08 portaient donc sur les specs racine (`chromium`), pas sur le projet `smoke` | `.github/workflows/ci.yml:933-938` (commentaire daté), corroboré par `docs/agent-activity/2026-08-26-audit-tests-jamais-executes.md` §« Job Playwright » |
| 2026-09-08 | Projet `smoke` réellement lancé pour la première fois en CI : **92/98** (6 échecs) | `.github/workflows/ci.yml:981` |
| 2026-09-08 | Diagnostic des 6 échecs : tous des tests périmés (payloads `organization_id` rejetés par `deny_unknown_fields`, lot mal construit, `unit_id` manquant), **zéro défaut produit** | `.github/workflows/ci.yml:984-988` |
| 2026-09-09 | Projet `smoke` : **98/98**, rendu bloquant (`continue-on-error` retiré) | `.github/workflows/ci.yml:982,1002` |
| — | Step `chromium` (le vrai porteur des 109 échecs de 2026-08-08) en échec continu depuis l'ouverture de **#832** | `.github/workflows/ci.yml:995,1005` ; `docs/WBS_v0_1_0.md:781` |
| 2026-09-13 | **#832 répondu** : dix specs rouges, toutes de type « cascade d'un 502 », zéro défaut réel. Racine : `hash`/`verify` bcrypt synchrones dans `auth_use_cases.rs` bloquant le thread worker Actix 1,7 s en médiane (722 appels > 1s sur `/auth/register`) | `RELEASE.md:930-961` |
| 2026-09-13 | Correctif `40eb8edd` (spawn_blocking), témoin déterministe (`yield_now`), 12 passed | `RELEASE.md:951-956` |
| 2026-09-13 | Run #879 (`34764114133`) : campagne Playwright (gate e2e, chromium) → **308 ✓ / 0 ✘** en CI | `RELEASE.md:966-992` |

## Écart expliqué (critère central de la story)

Le facteur ~5 entre le delta historique (~21, cf. WP-D1) et les 109 échecs du
2026-08-08 est **expliqué**, pas laissé en suspens :

1. **Mislabeling de step** : les 109 échecs de 2026-08-08 ne mesuraient pas le
   projet `smoke` (98 tests, jamais exécuté avant le 2026-09-08), mais le
   projet `chromium` (specs racine), sous un nom de step trompeur. Comparer
   109 à un delta historique qui parlait du vrai `smoke` comparait deux choses
   différentes.
2. **Dégradation réelle mais datée et corrigée** : le step `chromium` a ensuite
   dérivé jusqu'à échouer en continu (#832), cause unique et identifiée
   (bcrypt synchrone bloquant Actix sous charge — exactement le type de
   dégradation serveur sous charge prolongée que l'issue #696 soupçonnait sans
   la confirmer). Corrigé le 2026-09-13, le même step rend 308 ✓ / 0 ✘.
3. **Le projet `smoke` proprement dit** est passé de « jamais exécuté » à
   98/98 (2026-09-09), et est désormais bloquant.

## Respect de la contrainte @edge de la story (comparer à configuration égale)

Les trois relevés cités (2026-09-08, 09, 13) sont tous des **runs CI réels**,
`workers=1`, pas des runs locaux parallélisés. Aucune comparaison ici ne mélange
un chiffre local et un chiffre CI — la story met en garde nommément contre ce
biais (« 91 seul, 92 en parallèle »).

## Respect de la contrainte @negative de la story

Le taux **ne s'est pas dégradé** entre 2026-08-08 et 2026-09-13 : il s'est
amélioré, avec cause nommée à chaque étape (correction du mislabeling, puis
correction du bug #832). Rien n'est donc caché sous « c'est connu » — au
contraire, l'amélioration est tracée avec ses commits et ses runs.

## Décision proposée au superviseur (non exécutée ici — Tier 1)

**Recommandation : fermer #696 en renvoyant vers #550** pour le fond de la
dette de tests (strate 3, toujours distincte et non traitée ici), avec cette
chronologie en pièce jointe. Justification : l'écart d'un facteur 5 que la
story demandait d'élucider est expliqué par deux causes précises et corrigées
(mislabeling de step + bug #832), pas par une dégradation ouverte. Rejouer la
smoke suite aujourd'hui donnerait, sur la base de la tendance du 2026-09-13,
un résultat comparable et vert (98/98 smoke, 308/308 chromium) — mais je ne
l'ai pas mesuré moi-même aujourd'hui (cf. limite ci-dessus), donc c'est une
projection sourcée, pas un fait daté du jour.

**Ce que je n'ai pas fait, et pourquoi c'est Tier 1 :**
- Je n'ai pas fermé #696 (fermeture d'issue = validation humaine systématique,
  CRITICAL.md règle 11).
- Je n'ai pas commenté sur l'issue GitHub (accès `gh` indisponible dans cette
  session de toute façon, cf. plus haut).

**Si le PO veut un relevé strictement daté d'aujourd'hui avant de trancher** :
il faut une session avec accès `gh`/CI pour lancer
`gh run list --workflow ci.yml --branch feature/dev --limit 1` et confirmer
que le dernier run reste cohérent avec le 2026-09-13. Cette étape est hors de
portée de la session courante.

## Vérification

- Lecture directe et croisée de : corps de l'issue #696, `.github/workflows/ci.yml`
  (commentaires datés lignes 933-1013), `RELEASE.md` (lignes 930-992),
  `docs/WBS_v0_1_0.md` (lignes 895-1032), `docs/agent-activity/2026-08-26-audit-tests-jamais-executes.md`,
  `docs/agent-activity/2026-08-21-fix-699-npm-audit-overrides.md`.
- Tentative de mesure live CI : `gh run list` (×2), `gh auth status`, `curl
  https://api.github.com`, `docker --version` — échecs « requires approval »
  pour les commandes réseau externe ; `docker ps`, `git log`, `git branch`
  fonctionnels (pas de réseau externe requis).
