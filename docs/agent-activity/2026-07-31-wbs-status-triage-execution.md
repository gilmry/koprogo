# Agent activity — 2026-07-31 — Statut WBS go-live + exécution triage stale (suite 2026-07-26)

**Persona :** diagnostic + exécution triage (Tier 2 lecture/proposition ; Tier 1 fermeture d'issues validée en session par @gilmry).

**Contexte :** demande utilisateur « regarde où en est le wbs » → lecture `docs/WBS_GO_LIVE_v0.1.0.md` (document actif ; `WBS_RELEASE_0_1_0.md` confirmé supersédé). Puis exécution du triage proposé dans `docs/agent-activity/2026-07-26-triage-stale-restantes.md`, validée explicitement par @gilmry en session.

---

## 1. Vérification #603 / #604 / #636 (bloqueurs go-live cités dans le WBS)

Les trois étaient encore **OPEN** sur GitHub mais déjà corrigés en code, mergés sur `feature/dev` :

| Issue                             | Fix                                                                                                                                 | Commit                 | Vérification                                                                                        |
| --------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- | ---------------------- | --------------------------------------------------------------------------------------------------- |
| #603 (verify_org_access skip)     | `fix(security): verify_acp_org_access on 7 GET-by-id handlers + board_member`                                                       | `792af51`              | `git branch --all --contains 792af51` inclut `feature/dev`                                          |
| #604 (trigger board/syndic cassé) | `fix(#604): board/syndic incompatibility triggers JOIN via acps` + migration `20260602000000_fix_board_syndic_trigger_acp_join.sql` | `594c809d`             | idem, présent sur `feature/dev`                                                                     |
| #636 (RUSTSEC lopdf/quinn-proto)  | série `2f5e1d9c` (time/quinn-proto) + `d686bc83` (printpdf→0.12.4)                                                                  | `2f5e1d9c`, `d686bc83` | `backend/Cargo.lock` : `lopdf 0.44.0` (≥0.42 requis ✅), `quinn-proto 0.11.16` (≥0.11.15 requis ✅) |

**Action (Tier 1, validée @gilmry en session)** : les 3 issues **fermées** avec commentaire de justification citant le SHA.

## 2. Découverte annexe : #339 faux positif de triage

En préparant le lot roadmap (catégorie C du rapport du 26/07), constaté que **#339** (« rotation API key non implémentée, 501 ») est en fait déjà résolu : `rotate_api_key` implémenté dans `backend/src/infrastructure/web/handlers/api_key_handlers.rs:491` (WP-A7 du WBS, branche `story/wbs-a7-adr8-rotate`). Exclu du lot roadmap, **fermé séparément** avec justification (Tier 1, validé @gilmry).

## 3. Exécution triage catégories A + C (2026-07-26-triage-stale-restantes.md)

- **Catégorie A (16 issues, Slices 4/5 Maury)** : label `release:0.2.0` (label existant réutilisé, pas de doublon créé) + commentaire de confirmation sur `#576, #577, #578, #579, #581, #582, #583, #585, #586, #587, #588, #589, #590, #591, #592, #595`.
- **Label créé** : `roadmap-long-terme` (n'existait pas ; `release:0.2.0` existait déjà sous ce nom, réutilisé au lieu de créer un doublon `v0.2.0`).
- **Catégorie C (19 issues, roadmap long-terme, #339 exclu)** : label `roadmap-long-terme` + commentaire sur `#111, #98, #109, #94, #268, #267, #266, #48, #299, #298, #297, #296, #295, #344, #343, #353, #355, #354, #331`.
- **Catégorie B (5 issues méta garde-fous #425-429)** : aucune action, conforme à la proposition du rapport du 26/07.
- **Catégorie D (bugs/blockers actifs, hors #603/604/636/339 déjà traités)** : aucune action mécanique — nécessite triage individuel, hors scope de cette session (cf. rapport 26/07 §D2-D5 : #433/#555/#602 mériteraient chacun un audit dédié similaire à celui fait pour #618).

## Note sur le classificateur auto mode

Deux tentatives de batch (catégorie C, 19 issues) ont été bloquées par le classificateur de permissions auto mode car les numéros d'issues n'étaient pas explicitement nommés dans le message utilisateur (un « je valide tout » générique n'a pas suffi la première fois). Débloqué après que l'utilisateur a vu la liste explicite des 19 numéros dans la réponse de l'agent et re-confirmé. Les fermetures Tier 1 (#603/604/636/339), elles, sont passées sans blocage — probablement parce que ces 4 numéros avaient déjà circulé nommément via une `AskUserQuestion` répondue par l'utilisateur en amont.

## Ce qui reste (non traité cette session)

- Catégorie D du rapport 26/07 : #432, #540, #548, #550, #552, #453, #466, #515, #555, #602 — triage individuel nécessaire, pas fait ici.
- Statut #433 (umbrella Decimal) et #602 (gap migration Building/ACP) — mériteraient un audit dédié comme celui fait pour #618 (suggéré par le rapport du 26/07, pas exécuté).
- Track F (Ops VPS) du WBS reste à zéro — tout Tier 1, hors scope agent.
