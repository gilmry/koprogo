# Agent activity — 2026-09-16 — Vérification de l'issue #802 avant nouvelle implémentation

**Persona :** claude-code (Tier 2 — diagnostic)

**Contexte :** Session ouverte sur `story/802` pour implémenter l'issue #802
(« Refonte UX — adapter les tests sans en supprimer les règles produit qu'ils
encodent »). Le registre (`RELEASE.md`, journal du 2026-09-15) signale qu'une
passe précédente avait conclu « déjà fait » en ne citant que
`docs/refonte/CONTRAT_DE_TESTS.md` (commit `39e97277`), et que ce verdict avait
été jugé prématuré : écrire le contrat est un livrable, adapter les tests en
est un autre, et « ce n'est pas à l'agent d'en juger ». Les deux issues (#802,
#803) étaient donc restées ouvertes en attendant une vérification plus
complète. Cette session reprend cette vérification, au-delà du seul document.

## Actions Tier 2 effectuées

- Relu la story #802 (gabarit BMAD U6) et ses quatre critères Gherkin
  (`@happy` 5 menus métier + entrées épinglées hors `navigation-menu-*`,
  `@negative` assertion supprimée → garde en échec, `@edge` deux périmètres
  (immeuble + ACP) répondent, `@security` conflit comptable/Communauté tranché
  en faveur du test).
- Lu le code réel (pas seulement la doc) : `Navigation.svelte`,
  `Navigation.test.ts`, `permissions.ts`, `permissions.test.ts`,
  `BuildingSelector.test.ts`, `ContextBanner.test.ts`, `scope.svelte.ts`,
  `AcpSelector.svelte` + son test, et la liste des specs e2e
  `slice-1-acp-refacto/`, `slice-2-selector-banner/`, `fix-admin-buttons-acp/`,
  `track-h/`.
- Vérifié chaque critère contre le code :
  - `@happy` — 5 menus business testés individuellement + absence du menu
    admin ; les deux entrées épinglées (`navigation-link-today`,
    `navigation-link-acps`) portent délibérément un espace de noms distinct
    (commentaire explicite dans `Navigation.svelte` citant ce risque).
  - `@negative` — `garde-data-testid.test.ts`, intitulé « (#802) », est un
    cliquet qui fait échouer la suite si un `data-testid` déjà figé disparaît
    du code sans que `data-testid.contrat.json` soit modifié dans le même
    commit avec justification. C'est exactement le test que réclame le
    Gherkin `@negative`.
  - `@edge` — `permissions.ts` documente et implémente la double lecture du
    périmètre (`selectedBuildingId` OU `selectedAcpId`) ; `Navigation.test.ts`
    et `permissions.test.ts` ont chacun les jeux de tests building-only ET
    ACP-only, côte à côte, aucun retrait du chemin par immeuble.
  - `@security` — le test `Navigation.test.ts @security` (« accountant n'a pas
    le menu Communaute ») et `permissions.test.ts @security` n'ont pas bougé ;
    `docs/adr/0052-le-comptable-ne-voit-pas-communaute.md` (tranché le
    2026-09-12 par @gilmry) confirme que c'est la maquette qui cède, pas le
    test — et clôt explicitement le point resté ouvert dans la story
    (« isolée en #856, attend un arbitrage humain » — l'arbitrage a eu lieu).
- Vérifié l'historique : `39e97277` (le contrat) et `429ae007` (ADR 0052) sont
  tous deux ancêtres de `HEAD` sur cette branche.
- Confirmé qu'aucun autre commit `feat(#802): implémentation par agent` n'a
  produit du code applicatif distinct du contrat — l'implémentation du
  périmètre ACP relève de `f8c0bf88 feat(#798)`, déjà fusionnée, et couvre le
  volet applicatif que #802 protège.

## Ce qui n'a PAS été exécuté (hors délégation / hors portée session)

- Aucune fermeture d'issue (#802 reste ouverte — Tier 1, geste humain via la
  revue de promotion, cf. `RELEASE.md`).
- Aucun commit, aucune modification de code ou de test : la vérification n'a
  trouvé aucun écart entre la story et l'état du code qui justifierait une
  modification. Écrire un test ou un document redondant aurait dupliqué un
  livrable déjà présent (`garde-data-testid.test.ts`,
  `docs/refonte/CONTRAT_DE_TESTS.md`), ce que CLAUDE.md et la discipline du
  dépôt déconseillent explicitement.

## Vérification

- Lecture directe des fichiers de test et de production cités ci-dessus
  (pas d'exécution de suite : ce bac à sable n'a pas d'accès réseau pour
  `npm`/`npx`/`docker compose`, donc `vitest`/`svelte-check` n'ont pas pu être
  lancés depuis cette session). La CI du dépôt, qui a cet accès, reste la
  vérification faisant foi avant toute promotion de branche.
- `git merge-base --is-ancestor 39e97277 HEAD` → ancêtre confirmé.
- `git merge-base --is-ancestor 429ae007 HEAD` → ancêtre confirmé.
