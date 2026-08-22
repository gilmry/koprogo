==================================================================================
Issue #617: Phase C — Stabilisation Documentation Vivante e2e (8 specs Phase B FE)
==================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: enhancement,track:software priority:medium
:Assignees: Unassigned
:Created: 2026-06-15
:Updated: 2026-06-15
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/617>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Phase B FE (Stories B1-B8) a livré les composants Svelte 5 + Vitest tests 4-catégories. Voir `docs/maury/refonte-ux-multi-role-acp/phase-b-fe/`.
   
   Les **specs Playwright multi-rôle** créés en accompagnement (1 spec par story Phase B) ont été draftés mais **pas validés end-to-end** sur CI complète. Le DoD Phase B était : stories.md AC + Vitest tests + Playwright drafts.
   
   ## Constat post-Phase B (CI run [27515439816](https://github.com/gilmry/koprogo/actions/runs/27515439816))
   
   Une fois la dette `continue-on-error` retirée (Story B9, commit 3de6530), CI Pipeline > Playwright E2E Tests > **Run Playwright smoke tests** échoue sur 8/8 specs `refonte-ux/phase-b-fe/*.spec.ts` :
   
   | Spec | Story | Erreur principale |
   |---|---|---|
   | `role-assignment.spec.ts:152` | B1 | `selectOption` timeout — element detached, redirect /login |
   | `magic-link-issue.spec.ts:195` | B2 | `selectOption(seed.contractorUserId)` — options vides |
   | `magic-link-issue.spec.ts:333` | B2 | idem (scope_id absent) |
   | `mandate-issue.spec.ts:195` | B3 | timeout multi-rôle Syndic → Notary login |
   | `role-delegation.spec.ts:142` | B4 | timeout @happy delegation flow |
   | `role-delegation.spec.ts:197` | B4 | timeout @security non-transitivité |
   | `ticket-complaint.spec.ts:186` | B5 | timeout Owner → Syndic dashboard |
   | `syndic-response-sla.spec.ts:213` | B6 | timeout multi-rôle Owner Complaint → Syndic réponse |
   | `technical-spec-flow.spec.ts:229` | B7 | timeout 3 acteurs Syndic+AMO+Owner |
   | `contractor-eval.spec.ts:248` | B8 | timeout multi-Syndic A→B reputation read-only |
   
   **Pattern commun** :
   - `<select>` rendu mais `<option>` **vides** → API liste backend renvoie 0 (seeds non créées pour ce scope user/org)
   - **Cookie session perdu mid-test** → redirect vers `/login?redirect=...` quand on tente l action suivante
   - Tests qui exigent ≥ 2 logins consécutifs (multi-rôle) plus impactés
   
   ## Décision
   
   Décision @gilmry (2026-06-15) : restaurer la décorrélation temporaire en excluant `refonte-ux/phase-b-fe/` du project `chromium` (playwright.config.ts), créer cette Phase C pour stabiliser dans une fenêtre dédiée.
   
   - La CI redevient verte sur feature/dev (les Vitest Phase B passent déjà).
   - Les specs restent dans le repo pour debug local : `npx playwright test tests/e2e/refonte-ux/phase-b-fe/ --project=chromium --grep \"B2\"` etc.
   - Phase A (BE Stories 3.1-3.9) + Phase B (FE Stories B0-B8 + Vitest) shippées sur feature/dev.
   - Phase C ouvre 8 sub-tasks (C1-C8) à debugger une par une.
   
   ## DoD Phase C
   
   Pour chaque story C1-C8 :
   - [ ] Identifier root cause (seed missing? auth cookie SameSite? race condition? data-testid drift?)
   - [ ] Fix root cause (pas le test — règle CRITICAL.md ligne 12)
   - [ ] Re-activer le spec en retirant la ligne testIgnore pour ce fichier
   - [ ] CI Run Playwright smoke tests verte sur ce spec
   - [ ] Conserve 3 retries → zero flake observée sur 3 runs successifs
   - [ ] Mémoire/issue de root cause si pattern reproductible
   
   Finale :
   - [ ] Retirer totalement `/refonte-ux/phase-b-fe//` de testIgnore playwright.config.ts
   - [ ] CI Pipeline verte sur feature/dev avec les 8 specs activés
   - [ ] PR feature/dev → dev mergée
   
   ## Sub-tasks
   
   - [ ] C1 — Stabiliser `role-assignment.spec.ts` (Story B1)
   - [ ] C2 — Stabiliser `magic-link-issue.spec.ts` (Story B2, 2 tests)
   - [ ] C3 — Stabiliser `mandate-issue.spec.ts` (Story B3)
   - [ ] C4 — Stabiliser `role-delegation.spec.ts` (Story B4, 2 tests)
   - [ ] C5 — Stabiliser `ticket-complaint.spec.ts` (Story B5)
   - [ ] C6 — Stabiliser `syndic-response-sla.spec.ts` (Story B6)
   - [ ] C7 — Stabiliser `technical-spec-flow.spec.ts` (Story B7)
   - [ ] C8 — Stabiliser `contractor-eval.spec.ts` (Story B8)
   
   ## Refs
   
   - Phase B brief : `docs/maury/refonte-ux-multi-role-acp/phase-b-fe/brief.md`
   - Mémoire `feedback_maury-fullstack-first.md` — leçon décorrélation FE/BE
   - Mémoire `feedback_multirole-narrative-scenarios.md` — 8/8 scénarios sont multi-rôle
   - Commit retrait gate (Story B9) : 3de6530
   - Commit testIgnore Phase C : à venir

.. raw:: html

   </div>

