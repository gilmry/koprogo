==================================================================================================================================
Issue #548: bug(e2e): WP-D1/FE1 — ripple Playwright (59 specs) après JWT→cookie : auth.ts init-ordering 'Database not initialized'
==================================================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: bug,priority:high bug:majeur
:Assignees: Unassigned
:Created: 2026-05-19
:Updated: 2026-05-19
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/548>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Constat
   
   Après WP-FE1 (#343, JWT localStorage→cookie HttpOnly, PR #543 mergée feature/dev), CI **Playwright E2E = 59 failed / 126 passed** (job 76764373709). Cascade : Login, Meetings, OwnerDashboard, GDPR, Notifications, Tickets… + helper `registerAndLoginAsSyndic`.
   
   ## Cause racine (tranchée)
   
   `frontend/src/stores/auth.ts` — inversion d'ordonnancement init :
   - `init()` (l.194) appelle `refreshAccessToken()` AVANT `localDB.init()` (l.197).
   - `refreshAccessToken()` (l.304-308) sur succès fait `syncService.setToken()` + `localDB.saveUser()` (exigent DB init).
   - DB pas init → `Database not initialized` → catch (l.324) → `clearSession()` → renvoie `false`.
   - `init()` voit `refreshed=false` → `clearSession()` → jamais authentifié après reload malgré cookie valide → app non initialisée → specs auth timeout.
   
   ## Recette (RED-first, pas de fake-fix)
   
   Rouge = 59 specs Playwright CI. La partie sécurité du refresh réussit ; seule la persistance cache échoue prématurément. Fix = écriture cache (`syncService.setToken` + `localDB.saveUser`) **best-effort non-fatale** dans `refreshAccessToken` (auth ≠ cache ; `init()` rattrape `localDB.init()` juste après). Validation = CI Playwright (~50 min, docker-heavy ; pas local — mémoire vélocité).
   
   ## Critères d'acceptation
   - [ ] `refreshAccessToken` : échec cache local non-fatal ; renvoie `true` si access token obtenu
   - [ ] CI Playwright : plus de cascade `Database not initialized` ; specs auth verts (plancher ≈ pré-FE1)
   - [ ] Invariant WP-FE1 intact (aucun token en localStorage)
   - [ ] Tout rouge résiduel tracé
   
   ## Refs
   #343 (WP-FE1) · #541/#543 · WP-D1 (WBS) · feature/dev `9f91bcc`

.. raw:: html

   </div>

