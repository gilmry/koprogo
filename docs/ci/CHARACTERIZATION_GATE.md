# Characterization E2E Gate (CI)

> **Story Maury Tx.1** — ref [#593](https://github.com/gilmry/koprogo/issues/593)
> Workflow : [`.github/workflows/characterization-gate.yml`](../../.github/workflows/characterization-gate.yml)
> Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md` §8 Story Tx.1](../maury/refonte-ux-multi-role-acp/stories.md)

## Pourquoi

La refonte UX multi-rôle ACP (slices 1-5) refactor en profondeur des écrans front existants. Sans filet, n'importe quel commit peut casser silencieusement un parcours utilisateur que personne ne re-teste manuellement.

La suite **caractérisation** (créée en Story 0.1) capture le comportement actuel des 6 parcours critiques sous forme d'E2E Playwright :

1. Auth flows (login, 2FA, magic link)
2. Building / unit / quotas
3. Tickets workflow (owner → syndic → owner)
4. AG : convocations + résolutions + votes
5. Notifications sync multi-canal
6. Documents / GDPR exports

Ce job CI est un **gate régression** : tant que les specs caractérisation sont VERTES, on a la garantie qu'aucune slice n'a régressé les parcours majeurs. C'est le filet qui rend la refonte "test-driven" (mémoire [[fe-refactor-test-driven]]).

## Quand ça tourne

- **Trigger** : `pull_request` vers `dev` ou `main` uniquement.
- **PAS sur push**. `feature/dev` est volontairement un buffer d'intégration CI-free (mémoire [[gitflow-feature-dev-buffer]]). Le full CI démarre à `dev`.
- **PAS sur PR vers `feature/dev`** : intentionnel — on accepte la friction d'avoir un commit "vert localement" qui prend l'avion vers `dev` et s'y fait gater, plutôt que d'imposer un cycle CI lourd sur chaque incrément WIP.
- `workflow_dispatch` autorisé pour debug manuel.

## Quoi faire si ROUGE

**RÈGLE D'OR** : ne PAS modifier les specs `frontend/tests/e2e/characterization/*.spec.ts` pour les faire passer. Elles capturent le comportement de référence — elles sont la cible, pas la variable.

Procédure :

1. Télécharger les artifacts `characterization-test-results` (screenshots + videos + traces) sur le run failed.
2. Ouvrir `playwright-report/` localement : `npx playwright show-report frontend/playwright-report`.
3. Identifier la régression réelle dans la slice en cours :
   - Une route renvoyait 200, maintenant 401 ?
   - Un sélecteur stable (`data-testid`) a disparu ?
   - Un i18n key cassé fait disparaître un bouton ?
4. **Corriger la slice**, pas la spec.
5. Exception unique : si le test caractérise un bug réel à corriger pendant la slice, **ajuster la spec dans un commit séparé** avec message explicite `test(characterization): update <X> after slice N intentional behavior change` + lien story + approbation humaine en review (AC `@edge` de la story Tx.1).

## Robustesse de l'exit code

Mémoire [[bdd-ci-exit-code-gotcha]] : exit 0 ne prouve pas le succès, mais **un exit non-zero EST un échec dur** — exactement ce qu'on veut ici. Le step `Run characterization E2E suite` n'a aucun `|| true` ni `|| echo`. Une assertion Playwright qui fail propage l'erreur.

## Branch protection

Le job s'appelle :

- Job key (workflow YAML) : `characterization-tests`
- Display name (UI GitHub + branch protection rule) : `Characterization E2E Gate`

À marquer **`required`** par le mainteneur dans **Settings → Branches → Branch protection rules** pour `dev` et `main`. Cette action est Tier 1 (mutation GH config), l'humain la fait manuellement.

## Retrait du gate

Le gate est temporaire — il sert la refonte. Retirer ce job **uniquement quand toutes les conditions sont réunies** :

- [ ] Slices 1-5 de la refonte UX multi-rôle ACP toutes mergées dans `dev`.
- [ ] Recettes de validation Maury signées par @gilmry (cf. `docs/maury/refonte-ux-multi-role-acp/validation.md`).
- [ ] Suite caractérisation devenue redondante avec la suite E2E "post-refonte" (i.e. la couverture E2E nominale couvre désormais ce que la caractérisation couvrait).

À ce moment-là :

1. Supprimer le job + ce fichier doc.
2. Retirer la "required check" de la branch protection.
3. Archiver les specs `frontend/tests/e2e/characterization/` (ne pas supprimer — valeur historique pour audit refonte).

## Coût indicatif

Le job tourne backend (`cargo build` + `cargo run`) + frontend (`astro dev`) + Playwright Chromium. Sur runner `ubuntu-latest` standard, ordre de grandeur : **8-15 min**. Acceptable pour un gate sur PR (≤ 3 PR/jour pendant la refonte).
