# Règles critiques KoproGo (injectées à chaque user prompt)

Source de vérité courte. Pour le détail : issues #425, #426, #427, #428, #429 et `.claude/AGENT_GUARDRAILS.md`.

## Top 11 — non négociables

1. **Aucun secret en clair**. Hooks bloquent l'écriture (.env, *.pem, secrets/, kubeconfig, AWS/GH PAT, hardcoded passwords). Cf. #425.
2. **Pas de prod autonome**. `terraform apply`, `helm upgrade`, `kubectl apply/delete/exec`, `argocd sync`, `git push --force`, `gh release create` sont *deny* — l'humain les fait main.
3. **TDD/BDD 4 catégories obligatoires** par élément public livrable : `@happy` + `@edge` + `@security` + `@negative`. Pas de code sans test rouge préalable. Cf. #427.
4. **`Result<E>` typé, pas `Result<_, String>`**. `unwrap()` / `expect()` interdits hors tests — utiliser `?` + `AppError`. Hook PostToolUse warn.
5. **Itération sur les directives, pas sur le code**. Brief → PRD → Architecture → Story signés *avant* d'écrire du code. Tech debt = directive insuffisante. Cf. mémoire `feedback_maury-token-economy.md`.
6. **Tout dans GitHub**. Issues, PRs, comments, GH Discussions, fichiers versionnés. Pas de décision hors trace. Cf. #428 §4.
7. **Templates stricts > free-form**. Story, PRD, ADR, RFC, brief : sections imposées + frontmatter. Moins de devinette = moins de tokens.
8. **Auto-format tout fichier édité** (Rust, TS/Svelte, Terraform, shell, Python). Hook PostToolUse fait le travail.
9. **Multi-rôles e2e**. Scénarios E2E ont les bons acteurs (syndic crée AG, copropriétaire vote, syndic clôture). Pas un seul login pour tout. Cf. mémoire `feedback_multirole-narrative-scenarios.md`.
10. **v0.1.0 n'est pas en prod**. Aucun système live. Findings = observations expérimentation, pas crise. Cf. mémoire `project_koprogo-current-state.md`.
11. **Modèle Tier 1 / Tier 2 d'autorisation**. Tier 1 (mutation prod, création doc publique, envoi externe, fermeture issue) = **humain valide systématiquement** (message reply OR workflow_dispatch GH OR environment approval). Tier 2 (lecture, diagnostic, proposal, retrieval doc, comments) = autorisé mais **logué** dans `docs/agent-activity/YYYY-MM-DD-<persona>.md`. Si tu hésites entre Tier 1 et Tier 2 : choisis Tier 1 (humain). Cf. #429.
12. **Tooling via docker compose**. cargo / sqlx / rustfmt / clippy / npm / astro / svelte-check vivent dans les containers du `docker-compose.yml` de dev, **PAS** dans le shell hôte. Avant de conclure "outil X non dispo", essayer : `docker compose run --rm backend bash -c "SQLX_OFFLINE=true cargo <cmd>"` ou `docker compose exec backend cargo <cmd>` (si compose up). Idem `docker compose run --rm frontend npm run <cmd>`. Cf. mémoire `feedback_use-docker-compose-for-tooling.md`.

## Si tu hésites

- Demander à l'humain plutôt que deviner (Q&A en GH Discussion).
- Refuser de coder si la directive est floue ; remonter à `/maury-prd` ou `/maury-stories`.
- Préférer un commentaire d'issue à un nouveau fichier doc.
- Préférer Edit ciblé à Write rewrite.
- Avant `git push`, `gh pr create`, ou modif `CLAUDE.md` : confirmer.

## Lignes rouges

- Ne jamais `--no-verify` les git hooks.
- Ne jamais `:latest` un image tag (digest only).
- Ne jamais committer `*.docx`, `*.pdf`, `*.tfstate`, `*.env` (gitignore).
- Ne jamais re-créer un `Result<_, String>` quand `AppError` existe.
- Ne jamais fix un BDD/test "comme ça" pour qu'il passe ; comprendre la cause.
