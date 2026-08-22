# Agent activity — 2026-05-19 — Recalibrage vélocité / effort WBS (Tier-2)

Persona : planning/rétrospective. Trace versionnée (CRITICAL.md #6/#11) du constat d'effort de la session WBS go-live, pour calibrer les estimations futures.

## Mesure (ancrage timestamps git)

- Span session committée : 2026-05-18 18:41 → 2026-05-19 06:39 (~14-16 h calendaire).
- **~7-9 h d'inactivité** : trou nuit 23:22→05:57 (6 h 35, humain absent) + gaps inter-`SessionStart:resume`.
- Bloc actif 1 (~05-18 16:00→23:22, ~7 h) : WP-B3+#526+sécurité — **gonflé ~3-5×** par 3 crashes Docker OOM (restart GUI = Tier-1 humain) + BDD sérialisé mono-binaire (~15-25 min ×N reruns).
- Bloc actif 2 (~05-19, ~30-45 min réels) : #526+A2+A3 cœur+4-cat — **mode docker-light** (3 agents diagnostic //, `cargo check` 1 m 14 s, zéro crash) → débit **~5-10× supérieur**.

## Conclusion (impact estimation)

Le sizing effort-agent (« passes ») reste fiable ; la variable réelle = **multiplicateur wall-clock (±2×)**, dominé par : (a) stabilité Docker (OOM ~3×/session lourde), (b) discipline docker-light vs BDD-lourd local, (c) latence Tier-1 humaine + gaps humain-absent. **Le goulet n'est pas le raisonnement agent.**

Reste WBS chiffré antérieurement : ~13-18 passes-agent → **~3-4 sessions** en mode docker-light discipliné, **~6-8** si docker-heavy/OOM récurrent. Go-live calendaire borné par F1-F4 (provisioning VPS, Tier-1) + G1 (revue/signature) + disponibilité humaine — pas par l'agent.

## Levier méthode (à appliquer désormais)

Diagnostics en sous-agents read-only parallèles (docker-light) + validation locale `cargo check --tests` + **CI comme gate BDD lourd** (pas de full-BDD local sauf spot-check 1 binaire, zéro appel API docker pendant le run). Gravé en mémoire : `feedback_docker-light-velocity-leverage`. Voir aussi `feedback_subagent-worktree-git-salvage` §3, `project_bdd-ci-exit-code-gotcha`.
