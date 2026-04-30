---
persona: code-reviewer
created: 2026-04-29
last_updated: 2026-04-29
---

# Mémoire persistante — `code-reviewer`

## Findings cross-cutting de l'audit 2026-04-29

### 🔴 Critical (correctness + sécurité)

- **JWT en `localStorage`** (`frontend/src/stores/auth.ts:16-37`) → XSS-exploitable.
- **Helm values avec `koprogo123`** dupliqué × 8 envs (`infrastructure/_shared/helm/koprogo/values.yaml:111-114` + 7 autres).
- **f64 en code monétaire/comptable** (référence `rust-expert`).
- **CORS `Origin: "*"`** (`infrastructure/_shared/helm/koprogo/values.yaml:21`) alors que CLAUDE.md affirme "no wildcards".

### 🟠 Important (maintainability)

- **Composants Svelte > 500 LOC** : `InvoiceWorkflow.svelte` 887, `AdminGdprPanel.svelte` 660, `InvoiceForm.svelte` 633, `UserForm.svelte` 591.
- **18 modals dupliqués** : pattern à abstraire en `<EntityCrudModal>`.
- **88 pages Astro toutes en `client:load`** : SSG/SSR contournés.
- **16 `fetch()` directs** dans composants (vs `lib/api.ts`).
- **i18n strings hardcodées** : 23+ détectées par scan (vs 4 locales requises FR/NL/EN/DE).
- **Tests frontend = 13/181 ≈ 7 %** : couverture critique manquante (Invoice, GDPR, UserForm 0 tests).
- **921 BDD scénarios sans matrice 4×N par FR** : couverture trompeuse, manque `@security`/`@negative`.

## Conventions acceptées (à enforce)

- **SOLID** : Single Responsibility en priorité (composant > 500 LOC = bell qui sonne).
- **DRY** : pattern dupliqué ≥ 3 occurrences = candidate à abstraction.
- **KISS** : préférer code lisible à code clever.
- **Test 4 catégories** : par élément public livrable (cf. CRITICAL.md règle #3).
- **i18n** : toute string user-facing a sa clé dans 4 locales (vérifier `frontend/src/locales/{fr,nl,en,de}.json`).
- **A11y** : composants Svelte interactifs respectent WCAG 2.1 AA (focus management, aria-*, keyboard nav).
- **ADR obligatoire** sur PRs touchant `domain/`, `infrastructure/`, ou introduisant nouvelle dépendance/pattern.

## Mapping topic → persona expert (à tagger en review)

| Topic | Expert |
|---|---|
| Idiomes Rust spécifiques | `rust-expert` |
| Frontend Astro/Svelte/a11y | `astro-svelte-expert` |
| Sécurité (XSS, RBAC, CSP, secrets) | `security-officer` |
| CI/CD / GitOps | `devops-engineer` |
| IaC / Terraform / Ansible | `platform-engineer` |
| Tests insuffisants par catégorie | `qa-team-X` selon scope |
| UX / accessibility | `ux-designer` |
| Méthode / process | `scrum-master-X` ou RFC |
| Architecture entreprise / cross-cutting | `togaf-chief-architect` |

## Patterns récurrents à signaler dès qu'observés

- Test scenario `@happy` uniquement → tag `qa-X` + référence #427 §A.3 matrice 4×N.
- Mismatch domain/DTO/migration types → tag `rust-expert` ou stack expert pertinent.
- God component (> 500 LOC) → propose découpage par responsabilité.
- ADR manquant pour décision archi → request RFC.

## Lessons learned

- L'audit a montré que des disciplines déclarées dans CLAUDE.md (4-cat tests, no `:latest`, no wildcards) n'étaient PAS respectées en pratique. **Les déclarations sans enforcement mécanique se dégradent.** Le rôle de `code-reviewer` est précisément de chasser ces écarts entre directive et pratique.

## Liens

- [`.claude/agents/code-reviewer.md`](code-reviewer.md)
- Issues : [#425](https://github.com/gilmry/koprogo/issues/425), [#427](https://github.com/gilmry/koprogo/issues/427), [#428](https://github.com/gilmry/koprogo/issues/428)
- Memory : `feedback_tdd-bdd-four-categories.md`, `project_no-f64-in-money.md`
