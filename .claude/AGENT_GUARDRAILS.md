# Agent Guardrails — KoproGo

> Documentation des garde-fous IA actifs sur le dépôt. Remplace l'ancien `.claude/hooks.md` (format obsolète).
>
> Pour la stratégie complète et les enseignements : voir issues GitHub [#425](https://github.com/gilmry/koprogo/issues/425), [#426](https://github.com/gilmry/koprogo/issues/426), [#427](https://github.com/gilmry/koprogo/issues/427), [#428](https://github.com/gilmry/koprogo/issues/428).
>
> Pour la méthode complète : voir [`Maury/README.md`](../Maury/README.md) (point d'entrée canonique).

---

## Architecture en 4 couches

| Couche | Mécanisme | Fichiers | Effet |
|---|---|---|---|
| **L1** | Permissions `.claude/settings.json` | `deny`/`ask`/`allow` | Bloque ou demande approbation avant exécution outil |
| **L2** | Hooks Claude Code | `.claude/hooks/*.sh` | Exécute scripts qui peuvent bloquer (exit 2) ou modifier comportement |
| **L3** | Skills + Sous-agents + Slash commands | `.claude/skills/`, `.claude/agents/`, `.claude/commands/` | Workflows packagés que les agents invoquent |
| **L4** | Outillage | `.gitleaks.toml`, `Makefile` (cibles `secret-scan`, `iac-lint`, `claude-check`, `token-budget`), CI | Vérifications réutilisables côté humain ET agent |

---

## L1 — Permissions

`.claude/settings.json` définit trois listes :

### `deny` (jamais autorisé à un agent)
- **Actions destructives prod** : `terraform apply`/`destroy`, `helm install/upgrade/uninstall`, `kubectl apply/delete/exec/patch`, `argocd app sync/delete`, `velero backup delete`, `npm publish`, `cargo publish`, `docker push`.
- **Git destructif** : `git push --force`, `git reset --hard`, `git clean -fd`, `git filter-branch`, `git commit --no-verify`.
- **Système** : `rm -rf /`, `curl|sh`, `wget|sh`.
- **GitHub mutations sensibles** : `gh release create/delete`, `gh repo delete`, `gh secret set/delete`.
- **Édition de fichiers secrets** : `**/.env`, `**/secrets/**`, `**/*.pem`, `**/*.key`, `**/*.tfstate*`, `**/age.key`, `**/.vault_pass`, `**/kubeconfig`, `**/id_rsa`, `**/id_ed25519`.

### `ask` (approbation utilisateur systématique)
- `git push`, `git commit`, `git rebase`, `git merge`, `gh pr create/merge/close`, `gh issue create`.
- `make migrate`, `sqlx migrate run/revert`.
- Édition de `backend/migrations/**`, `infrastructure/**/*.tf`, `infrastructure/**/values.yaml`, `.github/workflows/**`, `CLAUDE.md`, `docker-compose.yml`, `Cargo.toml`, `package.json`, `seed.rs`, `Dockerfile`.

### `allow` (commandes routinières)
Lecture/test : `make` cibles routinières, `cargo` (build/check/test/fmt/clippy/audit), `npm` (run/build/lint/check/test/audit), `terraform plan/validate/fmt`, `helm template/lint`, `kubectl get/describe/logs`, `gh issue/pr list/view`, `git status/diff/log/show`, etc.

---

## L2 — Hooks Claude Code

Configurés dans `.claude/settings.json` sous `hooks.*`. Scripts dans `.claude/hooks/`.

### `PreToolUse Edit|Write|MultiEdit`

#### `pretool-deny-secret-write.sh` (bloquant)
- Refuse écriture vers chemin sensible (env, age, secrets/, pem, key, tfstate, vault_pass, kubeconfig).
- Refuse écriture si contenu match : `AKIA[A-Z0-9]{16}` (AWS), `ghp_/gho_/ghs_` (GitHub PAT), `xox[baprs]-` (Slack), `BEGIN PRIVATE KEY` (PEM), credential pattern hardcodé (password/secret/jwt_secret = "real-looking-value-12+chars").
- **Allowlist** : `*gitleaks.toml`, `*trivyignore`, `.claude/_drafts/`, `.claude/rules/`, `.claude/hooks/`, `.claude/agents/` (configs sécurité contiennent légitimement des patterns).
- Exit 2 si bloqué, message stderr explicite, instruction de demander à l'humain.

#### `pretool-warn-sensitive-edit.sh` (non-bloquant)
- Affiche du contexte stderr pour édition de : migration SQL (rappel "ne pas modifier après application"), `CLAUDE.md` (cible ≤5k tokens), `docker-compose.yml` (vérif secrets), `Dockerfile` (USER non-root), `.github/workflows/` (permissions minimales), `.tf` (sensitive outputs), `values.yaml` (SealedSecrets/digest), `seed.rs` (PII), `Cargo.toml`/`package.json` (audit deps).
- Ne bloque pas — juste rappel. Les `ask` permissions handle l'approbation.

### `PreToolUse Bash`

#### `pretool-deny-prod-action.sh` (bloquant, defense in depth)
- Refuse commandes Bash matchant : `terraform apply/destroy/import/state rm`, `helm install/upgrade/uninstall/rollback`, `kubectl apply/delete/create/edit/patch/exec/cp`, `argocd app sync/delete/create`, `velero backup delete`, `git push --force`, `git reset --hard`, `git clean -fd`, `git commit --no-verify`, `git filter-branch`, `npm/cargo publish`, `docker push`, `gh release create/delete`, `gh repo delete`, `gh secret set/delete`, `rm -rf /`, `rm -rf ~`, `curl|sh`, `wget|sh`.
- **Limitation connue** : sur-détecte sur du texte non-exécutif (commentaires bash, `echo`-strings) qui mentionnent ces patterns. Compromis acceptable pour la sécurité. Pour les tests/dev, écrire les patterns sensibles dans des fichiers (jamais en command-line direct).

### `PostToolUse Edit|Write|MultiEdit`

#### `posttool-format.sh` (best-effort, jamais bloquant)
- Auto-format selon extension : `.rs` → `rustfmt`, `.ts/.svelte/.json/.md/.yaml/.css` → `prettier`, `.tf` → `terraform fmt`, `.sh` → `shfmt`, `.py` → `black`.
- Tous les outils sont best-effort : si absent ou échec, continue silencieusement.

#### `posttool-warn-unwrap.sh` (warning stderr, non-bloquant)
- Compte `.unwrap()` / `.expect(` dans les fichiers `.rs` (hors tests) → warn si > 0.
- Compte `: any` / `as any` dans `.ts`/`.svelte` → warn si > 0.
- Référence #427 (TDD/BDD 4 catégories + frontend TS discipline).

### `UserPromptSubmit`

#### `userprompt-inject-rules.sh`
- Lit `.claude/rules/CRITICAL.md` et le **prepend** au contexte de chaque prompt user.
- Garantit que les règles top-10 soient présentes à chaque tour, sans dépendre de la mémoire de l'agent.

### `Stop`

#### `stop-leak-scan.sh` (bloquant si fuite)
- Exécute `gitleaks protect --staged` + `gitleaks protect` (working tree) avec config `.gitleaks.toml`.
- Si fuite détectée → exit 2, l'agent ne peut pas terminer son tour sans corriger.
- Fallback minimal grep si gitleaks absent.
- Exécute aussi `git diff --check` pour whitespace/conflict markers (warning seulement).

### `SessionStart`

#### `session-start.sh`
- Affiche bannière "guardrails actifs".
- Vérifie déps essentielles (gh, git) et optionnelles (gitleaks, jq, cargo, npm, prettier, rustfmt).
- Warn si branche actuelle ∈ {main, master, production, staging}.

---

## L3 — Skills, sous-agents, slash commands

**À matérialiser** dans S1-S2 du roadmap (cf. issue #428). Cible :

### Skills (`.claude/skills/`)
- `safe-iac-change` : impose plan diff + checklist sécurité avant edit IaC.
- `hexagonal-feature` : génère feature avec `AppError` typé + tests RED-first.
- `human-checkpoint` : matérialise une pause "j'attends ta validation".
- `bdd-e2e-pair` : impose la création BDD + E2E couplée pour toute user-facing behavior.
- `cowork-release-review` : guide pour Claude en mode Cowork-Chrome (cf. #427 partie B).

### Sous-agents (`.claude/agents/`)
- `security-iac-reviewer` : revue sécurité ciblée sur diff IaC.
- `unwrap-fixer` : remplace `unwrap()`/`expect()` par `AppError` typé.
- `hexagonal-purity-checker` : détecte fuites infra (`use sqlx`/`use actix`) dans `domain/`.
- `i18n-coverage-checker` : compare clés FR/NL/EN/DE.
- `tdd-coverage-auditor` : audit matrice 4×N par FR sur PR.
- + 15 personas Maury/SAFe (cf. #428 §6) à matérialiser progressivement.

### Slash commands (`.claude/commands/`)
- `/check-quality` : lint + tests rapides + gitleaks staged + svelte-check.
- `/secret-scan` : gitleaks full-history.
- `/human-review` : checkpoint humain explicite.
- `/safe-pr` : crée PR avec checklist gating.
- `/iac-plan` : `terraform plan` + `tfsec` + `helm template ... | kube-linter`.
- + commandes Maury (`/maury-brief`, `/maury-prd`, ...) à matérialiser.

---

## L4 — Outillage

### `.gitleaks.toml`
Config gitleaks avec allowlists projet :
- Test seeds (alice123, bob123, ... ; koprogo123 dans contexte test).
- Documentation example placeholders (AKIAIOSFODNN7EXAMPLE canonique AWS).
- `.claude/_drafts/`, `.claude/rules/`, `Maury/`, `docs/cowork/`.
- Custom rules : `koprogo-jwt-weak`, `koprogo-helm-default-password`.

### `Makefile`
Cibles guardrails ajoutées :
- `make secret-scan` — gitleaks staged + working tree.
- `make secret-scan-history` — gitleaks tout l'historique.
- `make iac-lint` — terraform fmt + ansible-lint + helm lint + tfsec + kube-linter.
- `make claude-check` — valide settings.json, hooks executables, gitleaks present.
- `make token-budget` — snapshot tokens par artefact.
- `make ci` (étendu) — lint + check-frontend + test + secret-scan.

### CI bloquante (à câbler — issue #425 S3)
- `gitleaks-action` (failure si fuite).
- `tfsec-action` + `checkov-action`.
- `trivy fs` scan supply chain.
- `dependency-review-action` sur PR.

---

## Workflow garde-fous en pratique

```
┌─────────────────────────────────────────────────────────────┐
│  Agent Claude commence un tour (UserPromptSubmit)           │
│  → hook inject CRITICAL.md (top-10 rules) en contexte      │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│  Agent veut Edit/Write/MultiEdit (PreToolUse)               │
│  → deny-secret-write : bloque secret path/content           │
│  → warn-sensitive-edit : affiche contexte (non-bloquant)    │
│  → settings.json `ask` : prompt humain pour fichiers sens.  │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│  Edit accepté → fichier modifié → PostToolUse              │
│  → format auto (rustfmt/prettier/terraform fmt)             │
│  → warn-unwrap pour Rust ; warn-any pour TS/Svelte         │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│  Agent veut Bash (PreToolUse Bash)                          │
│  → deny-prod-action : bloque terraform apply, helm upgrade, │
│      kubectl apply/delete/exec, argocd sync, force-push     │
│  → settings.json `ask` : prompt humain pour git push, gh pr │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│  Agent termine son tour (Stop)                              │
│  → gitleaks staged + working tree                           │
│  → git diff --check whitespace                              │
│  → exit 2 si fuite détectée → agent doit corriger           │
└─────────────────────────────────────────────────────────────┘
```

---

## Désactivation temporaire (debug, urgence)

Pour désactiver un hook ponctuellement :
```bash
# Via env var (si Claude Code respecte)
export CLAUDE_SKIP_HOOKS=1

# Via édition settings.json (commenter la section hook)
# (Préférer : créer une PR de modif, ne jamais désactiver "vite" en prod)
```

**Anti-pattern** : commenter un hook pour passer un commit. Le hook a une raison. Si trop strict, ouvrir une RFC pour le faire évoluer.

---

## Limitations connues

1. **`pretool-deny-prod-action.sh` sur-détecte sur texte non-exécutif** (commentaires bash, echo strings). Trade-off pour la sécurité. Workaround : tests avec patterns sensibles → écrire les payloads dans des fichiers JSON et lire via stdin.
2. **`posttool-format.sh` best-effort** : si rustfmt/prettier absent localement, format silencieux. CI doit catch les fichiers mal formés.
3. **`stop-leak-scan.sh` fallback grep** : si gitleaks absent, détection limitée à AWS keys, GH PAT, PEM. Recommander install gitleaks pour couverture complète.
4. **Hooks Windows Git Bash** : tous les scripts utilisent POSIX bash + jq. Git Bash sur Windows fonctionne. PowerShell pure non supporté (utiliser le `bash` qu'invoque settings.json).

---

## Maintenance

- **Évolution des hooks** : via RFC sous `docs/rfc/NNNN-*.md`, débat en GH Discussion catégorie "Process", merge → ADR.
- **Bumper la version Maury** : si la couche garde-fous évolue significativement, incrémenter `Maury/CHANGELOG.md` (cf. v1.1 ajout des hooks, v1.2 hypothétique pour ajouter @accessibility).
- **Audit des hooks** : tous les 3 mois, rejouer les smoke tests, vérifier nouvelles patterns d'attaque.

---

🤖 Document maintenu en symbiose avec issues #425 / #426 / #427 / #428 et la méthode Maury v1.1.
