# CHANGELOG — Méthode Maury

> Méthode publiée sous licence **GNU Affero General Public License v3.0 (AGPL-3.0)**.
> Voir le `LICENSE` à la racine du projet KoproGo.
>
> Auteurs : Gilles Maury, Farah Maury.

La méthode Maury est **le couteau suisse d'un CTO / Solution Architect senior / Staff Engineer qui commande une armée d'agents IA** pour livrer un produit logiciel dans les règles de l'art.

C'est un cadre méthodologique d'industrialisation de la production de code par agents IA, inspiré de TOGAF ADM, Essential SAFe, Nexus, Scrum et BMAD, **disciplinée par DDD + Hexagonal + TDD/BDD + ITIL**. Elle sert tant pour :

- la **mise en place d'un produit** (greenfield) — appliquer la méthode dès la phase Vision,
- la **conduite de l'évolution** (brownfield) — combler le gap entre code existant et méthode.

L'utilisateur cible est **CTO d'une armée d'agents** : il définit les directives signées, les agents exécutent, les garde-fous empêchent les écarts mécaniquement, l'humain valide aux gates. Voir [`README.md`](README.md) pour le positionnement complet.

Le présent CHANGELOG retrace les versions et leurs ajouts.

---

## v1.0 — 2026-03-29 (état initial)

État de la méthode tel que documenté dans `Méthode Maury.md` (93 KB).

**Phases** (6) :
1. **Vision** (agent : Mary, Analyste) — Brief : vision, personas, capacités, BC DDD, invariants, glossaire ubiquitaire.
2. **PRD** (agent : John, Product Manager) — FRs numérotées, scénarios BDD Gherkin, UX par persona, NFR.
3. **Architecture** (agent : Winston, Architecte) — Hexagonale 3 couches, ports/adapters, mapping DDD→code, ADRs, IaC plan.
4. **Stories** (agent : Bob, Scrum Master) — 27 stories TDD avec tâches RED→GREEN→REFACTOR granulaires, fichiers exacts.
5. **Validation croisée** (agent : Product Owner) — Traçabilité Brief→PRD→Architecture→Stories.
6. **Exécution Scrum + Nexus + SAFe + ITIL** (agents codeurs + superviseur humain).

**Templates stricts** :
- Brief : 9 sections obligatoires.
- PRD : 19 sections obligatoires.
- Architecture : couches + ports + ADRs + IaC.
- Story : ID + taille S/M/L + entité DDD + BDD Gherkin + tâches TDD numérotées + fichiers exacts.

**Anti-patterns explicites** : voir `Méthode Maury.md`.

**Cas d'application** : KoproGo (greenfield, projet copropriété belge), 60 entités, 13 BC, 10 invariants légaux, 19 FRs, 559 endpoints API.

---

## v1.1 — En cours d'élaboration (couche garde-fous)

Suite à l'audit 2026-04-29 du codebase KoproGo (premier vrai test de la méthode en grandeur réelle), des pathologies systémiques ont été observées (1 967 `unwrap()`/`expect()`, secrets recopiés en clair × 8 environnements, 921 scénarios BDD trompeurs sans matrice 4 catégories, JWT en localStorage, etc.). Toutes ces pathologies découlent **de l'absence de garde-fous mécaniques** au niveau de l'agent IA.

La v1.1 ajoute donc explicitement la **couche garde-fous** comme partie intégrante de la méthode (pas un add-on). Issues GitHub de référence : #425, #426, #427, #428.

### Changements v1.0 → v1.1

#### Couche L0 — Contexte propre (issue #426)
- `CLAUDE.md` cible ≤ 5 000 tokens (taxe permanente sur chaque session agent).
- Pas de doublons docs.
- Pas de binaires versionnés (.docx/.pdf en GH Releases).
- Pas de stratégie roadmap dans CLAUDE.md (extraite vers `docs/ROADMAP*.rst`).

#### Couche L1 — Sécurité runtime (issue #425)
- Permissions Claude Code `deny` pour actions destructives (terraform apply, helm upgrade, kubectl mutation, git push --force, écriture de secrets).
- Permissions `ask` pour actions sensibles (git push, gh pr create, édition migration/IaC/CI/CLAUDE.md).
- Hooks Claude Code :
  - **PreToolUse** Edit/Write : blocage écriture vers chemin sensible OU contenu détecté secret (AWS key, GH PAT, Slack token, PEM, mot de passe hardcodé sans placeholder reconnu).
  - **PreToolUse** Bash : seconde ligne contre commandes prod dangereuses.
  - **PostToolUse** Edit/Write : auto-format + warn `unwrap()`/`expect()`/`any` introduits.
  - **UserPromptSubmit** : injection des règles critiques au début de chaque tour user.
  - **Stop** : `gitleaks detect` sur staged + working tree avant fin de tour.
  - **SessionStart** : vérif déps + warn si branche protégée.
- `.gitleaks.toml` avec allowlists projet (test seeds, drafts, configs sécurité).

#### Couche L2 — Discipline validation (issue #427)
- **Taxonomie 4 catégories obligatoires** par élément public livrable :
  - `@happy` — chemin nominal end-to-end
  - `@edge` — bornes (max/min/empty/0/1/N)
  - `@security` — RBAC, auth, injection, rate limit, escalade
  - `@negative` / `@catastrophic` — défaillance correcte (pas de panic, erreur typée)
- **Matrice 4×N par FR** dans la PRD : refus de signature si une cellule vide.
- **Hook PreToolUse RED-first** : édition handler refusée sans test rouge préexistant.
- **Bibliothèque baseline** scénarios sécurité/négatifs réutilisables.
- **Mutation testing** périodique pour mesurer la qualité (pas la quantité) des tests.
- **Cowork-Chrome + humain release review** : `verdict: GO|GO_CONDITIONAL|NO_GO` signé obligatoire avant tag.
- **Templates `HUMAN_REVIEW_PLAN.template.md` + `HUMAN_REVIEW_REPORT.template.md`** versionnés.

#### Couche L3 — Simulation organisation produit (issue #428)
- Agents par rôle (15-20 personas) : TOGAF level + Portfolio + ART + 2 équipes Scrum + Maury + cross-cutting.
- Cérémonies cron-driven (daily standup, sprint planning/review/retro, PI planning/demo/I&A, portfolio sync, ADR/RFC digest, WBS regen, velocity, token budget).
- GH Discussions catégorisées (Architecture, Process, Decisions Log, Q&A, Show & Tell, Retrospective Themes).
- ADRs (décisions prises) + RFCs (propositions à débattre).
- Pipeline doc auto-refresh pré-release (12 étapes avant `git tag`).
- Public manifesto `docs/SIMULATION_MANIFEST.md` (transparence pédagogique non-deceptive).

### Stratégie de consolidation des tokens (introduite en v1.1)

- **Investir tokens en amont** sur les directives (Brief/PRD/Architecture/Stories), pas en aval.
- Budget par artefact : CLAUDE.md ≤ 5k, Brief ≤ 8k, PRD ≤ 25k, Architecture ≤ 20k, Story ≤ 2k, ADR ≤ 1.5k, RFC ≤ 4k.
- Templates stricts = zéro devinette = consommation minimale.
- Mode "économie" pour fixes mineurs (skill `/quick-fix` qui n'invoque pas la full pipeline).
- Mesure : `make token-budget` weekly + alerte si > cible.

### Pourquoi les garde-fous appartiennent à la méthode

Sans garde-fous mécaniques, l'agent optimise pour les chemins de moindre résistance ("compile + lint passent"), pas pour la qualité du livrable. La méthode disait "respecter SOLID/DDD/Hexagonal/TDD/BDD" mais sans enforcement → directive ignorée en pratique (cf. audit). v1.1 acte qu'**une méthode IA-driven sans garde-fous mécaniques n'est pas une méthode, c'est un vœu pieux**.

---

## Roadmap d'évolution (cycle TOGAF ADM Phase H)

Chaque fin de Program Increment (PI) Inspect & Adapt produit potentiellement une RFC d'évolution Maury. Exemples plausibles à venir :

- **v1.2** — Ajout d'une 5ᵉ catégorie de tests `@accessibility` si bugs A11y récurrents passent les 4 catégories actuelles.
- **v1.3** — Scinder/fusionner agents Mary+John si la transition Brief→PRD perd du contexte de manière répétée.
- **v1.4** — Politique de sub-agents IA pour la revue (séparation dev/qa/security en pull request).
- **v2.0** — Refonte structurelle si changement majeur du paysage outils (nouvel agent framework, nouveau modèle Claude, etc.).

**Workflow d'évolution** :
1. Pattern récurrent identifié dans un PI I&A.
2. RFC déposée sous `docs/rfc/NNNN-maury-evolution-*.md`.
3. Débat en GH Discussion catégorie "Process" + commentaires PR.
4. Si accepté : ADR enregistre la décision, nouvelle version `Méthode_Maury_vX.Y.md`, tag git `maury-vX.Y`, plan de migration pour features actives.

---

## Licence

Méthode Maury (cet ensemble de documents et templates dans `Maury/`) publiée sous **GNU Affero General Public License v3.0 (AGPL-3.0)**.

L'AGPL-3.0 a été choisie pour garantir que :
- toute amélioration de la méthode reste publique,
- toute application en réseau (SaaS) doit publier ses modifications,
- la méthode reste libre tout en se protégeant de l'enfermement propriétaire.

Voir le fichier `LICENSE` à la racine du projet KoproGo pour le texte complet.

Auteurs : **Gilles Maury & Farah Maury**.

Pour citer la méthode : *"Méthode Maury v1.1, 2026, AGPL-3.0, https://github.com/gilmry/koprogo"*.
