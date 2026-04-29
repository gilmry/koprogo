# Méthode Maury

> *Le couteau suisse d'un CTO / Solution Architect senior / Staff Engineer qui commande une armée d'agents IA pour livrer un produit logiciel dans les règles de l'art.*

**Licence : [AGPL-3.0](../LICENSE)** — Auteurs : Gilles Maury & Farah Maury.

---

## Pour qui

**Audience cible** : ingénieur·e·s de niveau **senior+ / staff / CTO de petite équipe**, à l'aise avec :
- la pensée architecture (DDD, Hexagonal, SOLID),
- les cadres agiles emboîtés (TOGAF ADM, Essential SAFe, Nexus, Scrum),
- la pratique TDD/BDD rigoureuse,
- l'orchestration d'agents IA (Claude.ai cowork, Claude Code, agents distants).

**Pas pour** :
- débutant·e·s — la méthode présuppose les bases DDD/Hexagonal/TDD/BDD ;
- équipes humaines pures — la méthode est *pour* travailler avec une armée d'agents IA ; sans cette dimension elle perd son intérêt principal ;
- gestion de projet sans discipline IA — préférer Scrum classique.

## Positionnement

L'utilisateur de Maury est **CTO d'une armée d'agents** :
- Il **définit les directives** (Brief / PRD / Architecture / Stories signées).
- Les **agents exécutent** mécaniquement selon ces directives.
- Les **garde-fous** (couche v1.1) empêchent les écarts.
- L'**humain garde la main aux gates** (sign verdicts, code review release).

**Objectif** : permettre à une seule personne expérimentée de commander un produit logiciel complet (60+ entités, 500+ endpoints, production-ready, conformité légale, tests à 4 catégories obligatoires) que ferait normalement une équipe de 8-15 personnes.

## Comment ça marche

```
        Vision idée
             │
             ▼
    ┌─────────────────────┐
    │  Phase 1 — BRIEF    │  agent : Mary (Analyste TOGAF B-D)
    │  Vision, personas,  │  → docs/maury/<feature>/brief.md
    │  capacités, BC DDD  │  GATE : signature humaine (frontmatter)
    └────────┬────────────┘
             │
             ▼
    ┌─────────────────────┐
    │  Phase 2 — PRD      │  agent : John (PM)
    │  FRs numérotées,    │  → docs/maury/<feature>/prd.md
    │  BDD Gherkin 4-cat  │  GATE : matrice 4×N par FR + signature
    └────────┬────────────┘
             │
             ▼
    ┌─────────────────────┐
    │  Phase 3 — ARCHI    │  agent : Winston (Architecte hexagonal)
    │  Ports, adapters,   │  → docs/maury/<feature>/architecture.md
    │  ADRs, IaC          │  GATE : compile-ready + signature
    └────────┬────────────┘
             │
             ▼
    ┌─────────────────────┐
    │  Phase 4 — STORIES  │  agent : Bob (Scrum Master)
    │  S/M/L,             │  → docs/maury/<feature>/stories.md
    │  TDD RED→GREEN→REF  │  GATE : exécutables IA + signature
    └────────┬────────────┘
             │
             ▼
    ┌─────────────────────┐
    │  Phase 5 — VALID    │  agent : Product Owner
    │  Traçabilité        │  → docs/maury/<feature>/validation-report.md
    │  Brief → Code       │  GATE : score fidelity ≥ 80
    └────────┬────────────┘
             │
             ▼
    ┌─────────────────────┐
    │  Phase 6 — EXÉC     │  agents : dev / qa / release-manager
    │  Sprints Scrum +    │  Garde-fous L1-L2-L3 actifs en continu
    │  Cowork release     │  GATE : Cowork report signé GO
    └─────────────────────┘
```

## Documents-clés (ordre de lecture)

| # | Document | Quoi |
|---|---|---|
| 1 | **[`Méthode Maury.md`](Méthode%20Maury.md)** | Le manuel complet (93 KB). Phases, agents, gates, templates, anti-patterns. |
| 2 | **[`CHANGELOG.md`](CHANGELOG.md)** | Versions et évolutions. Lire pour savoir où en est la méthode. |
| 3 | [`product-brief.md`](product-brief.md), [`PRD.md`](PRD.md), [`architecture.md`](architecture.md), [`epics-and-stories.md`](epics-and-stories.md) | Application réelle au projet KoproGo (cas d'étude). |
| 4 | [`validation-report.md`](validation-report.md) | Audit traçabilité Brief→Code (cas d'étude KoproGo). |
| 5 | [`analyse-bmad-vs-codebase.md`](analyse-bmad-vs-codebase.md) | Comparaison méthode théorique vs implémentation réelle (lessons learned). |

## Garde-fous (intégrés depuis v1.1)

La v1.1 intègre **mécaniquement** les recettes d'industrialisation IA (cf. issues GitHub [#425](https://github.com/gilmry/koprogo/issues/425) / [#426](https://github.com/gilmry/koprogo/issues/426) / [#427](https://github.com/gilmry/koprogo/issues/427) / [#428](https://github.com/gilmry/koprogo/issues/428) du dépôt KoproGo) :

- **L0 — Contexte propre** : CLAUDE.md trim ≤ 5k tokens, pas de doublons docs, pas de binaires versionnés.
- **L1 — Sécurité runtime** : permissions Claude Code `deny`/`ask`, hooks (PreToolUse secret-write/prod-action, PostToolUse format/warn-unwrap, UserPromptSubmit rules-inject, Stop gitleaks, SessionStart deps-check), `.gitleaks.toml`.
- **L2 — Discipline validation** : taxonomie tests 4 catégories obligatoires (`@happy` + `@edge` + `@security` + `@negative`), RED-first hook, matrice 4×N par FR dans la PRD, Cowork-Chrome + humain release report signé GO.
- **L3 — Simulation organisation produit** : 15-20 agents personas (TOGAF/SAFe/Nexus/Scrum/Maury/cross-cutting), cérémonies cron-driven, ADRs (décisions) + RFCs (propositions), GH Discussions catégorisées, doc auto-refresh pre-release.

**Sans ces garde-fous, la méthode reste une intention. Avec, elle devient un système de production reproductible.**

## Cas d'usage

| Type | Approche | Exemple |
|---|---|---|
| **Greenfield** | Appliquer la méthode dès la phase Vision (Brief). | Nouveau produit, équipe = 1 humain + agents. |
| **Brownfield** | Audit gap → priorisation remédiation → intégration incrémentale. | KoproGo (le projet hôte de cette méthode). |

KoproGo a appliqué la méthode v1.0 tardivement (audit codebase ≠ rigoureux). L'audit du 2026-04-29 a identifié les écarts ; la v1.1 ajoute les garde-fous mécaniques pour empêcher les régressions.

## Évolution (cycle TOGAF ADM Phase H)

La méthode évolue elle-même :
1. Pattern récurrent identifié dans un PI Inspect & Adapt → **RFC** sous `docs/rfc/NNNN-maury-evolution-*.md`.
2. Débat en GH Discussion catégorie "Process".
3. Si accepté → **ADR** + nouvelle version `Méthode_Maury_vX.Y.md` + tag git `maury-vX.Y` + plan de migration pour features actives.

Voir [`CHANGELOG.md`](CHANGELOG.md) pour la roadmap d'évolution (v1.2, v1.3, v2.0).

## Citer la méthode

> *"Méthode Maury v1.1, 2026, AGPL-3.0, https://github.com/gilmry/koprogo"*

---

**Maury n'est pas une méthode parmi d'autres. C'est l'unité de base d'un CTO qui industrialise sa production de code via agents IA, en visant les règles de l'art.**
