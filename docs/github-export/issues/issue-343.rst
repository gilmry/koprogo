=======================================================================================
Issue #343: refactor(frontend): architecture hexagonale light + data-testid + bug fixes
=======================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: enhancement
:Assignees: Unassigned
:Created: 2026-03-26
:Updated: 2026-03-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/343>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Le frontend KoproGo (Astro + Svelte, 178 composants, 22 modules API) a une bonne séparation infrastructure (`lib/api/*.ts`) mais manque d'une couche service/validateur/utilitaires entre les composants et les clients API. Ceci cause une duplication massive :
   
   - **55 composants** redéfinissent `formatDate()` avec des locales inconsistantes
   - **92 composants** ont des blocs `try/catch/toast` identiques (206 occurrences)
   - **4+ composants** dupliquent la logique de validation de formulaires
   - Calculs financiers (TVA belge) copiés-collés entre composants factures
   - Logique machine d'état (actions ticket par statut+rôle) inline
   
   De plus, **158 des 178 composants (89%) n'ont AUCUN `data-testid`**, bloquant l'industrialisation des scénarios Playwright.
   
   ## Objectifs
   
   1. Ajouter les couches `lib/utils/`, `lib/validators/`, `lib/services/`
   2. Injecter `data-testid` sur chaque élément interactif (formulaires, boutons, listes, loading states)
   3. Corriger 4 patterns de bugs récurrents dans chaque composant touché
   4. Impact bundle = zéro (types effacés, fonctions tree-shaken, réduction nette de code)
   
   ## Nouvelles couches
   
   ### `lib/utils/` — Fonctions utilitaires pures
   - `date.utils.ts` — formatDate, isOverdue, todayISO (remplace 55 implémentations dupliquées)
   - `finance.utils.ts` — calculateVAT, formatCurrency, BELGIAN_VAT_RATES
   - `error.utils.ts` — withErrorHandling (remplace 206 blocs try/catch/toast)
   - `filter.utils.ts` — multiFieldSearch, applyFilters
   - `response.utils.ts` — extractArray (normalisation réponses API)
   
   ### `lib/validators/` — Validation centralisée
   - `common.validators.ts` — required, minLength, isEmail, hasErrors
   - `ticket.validators.ts`, `payment.validators.ts`, `ownership.validators.ts`
   
   ### `lib/services/` — Couche application
   - `ticket.service.ts` — getAvailableActions, loadTickets, transitionTicket
   
   ## Bugs corrigés au passage
   
   | Bug | Pattern | Fix |
   |-----|---------|-----|
   | Modal isOpen | `<Modal {open}>` | `<Modal isOpen={open}>` |
   | $$restProps | Composants form sans forward | `{...$$restProps}` sur élément natif |
   | koprogo_user_id | clé localStorage inexistante | Parser depuis `koprogo_user` JSON |
   | Empty UUID | champs UUID optionnels = `""` | `value \|\| undefined` |
   
   ## Scope
   
   - **13 fichiers créés** (utils, validators, services)
   - **~160 fichiers modifiés** (composants + 1 page)
   - **22 fichiers non touchés** (lib/api/*, stores/*, infrastructure)
   - **Impact bundle : NÉGATIF** (bundle plus petit grâce à la déduplication)
   
   🤖 Generated with [Claude Code](https://claude.com/claude-code)

.. raw:: html

   </div>

