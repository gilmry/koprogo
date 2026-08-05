# Agent activity — 2026-07-26 — Proposition de triage des issues stale restantes

**Persona :** triage read-only (Tier 2 — proposition uniquement. Aucun label/milestone/fermeture appliqué dans ce document. Toute action listée reste à valider par l'humain — CRITICAL.md #11).

**Branche :** suite de `docs/agent-activity/2026-07-26-sync-audit-feature-dev.md` — les 23 issues §2.1 de ce rapport ont été fermées le même jour ; ce document couvre les **61 issues ouvertes restantes** sans update ≥60 jours (sur les 72 initialement détectées).

**Scope :** demande explicite de l'utilisateur — « tu fais le tri et tu me proposes un plan » pour ces 61 issues. Ce document propose une catégorisation + une action par groupe ; rien n'est exécuté.

---

## Résumé

| Catégorie                                                                | Nombre | Action proposée                                                            |
| ------------------------------------------------------------------------ | ------ | -------------------------------------------------------------------------- |
| A — Slices 4/5 Maury, non démarrées (cohérent avec report v0.2.0 du WBS) | 16     | Label `v0.2.0` + commentaire de confirmation, garder ouvertes              |
| B — Méta garde-fous (#425-#429)                                          | 5      | Ne rien faire — umbrellas volontairement permanentes                       |
| C — Roadmap long-terme non priorisé                                      | 20     | Garder ouvertes, label `roadmap-long-terme` pour sortir du bruit de triage |
| D — Bugs/blockers actifs                                                 | 20     | Triage individuel — détail ci-dessous                                      |

---

## A — Slices 4/5 Maury sans commit (16 issues)

Toutes labellisées `maury` + `track-h-conformite`, toutes datées 66 jours (créées en bloc le 2026-05-20 avec le reste du pipeline #556). Le WBS (`docs/WBS_GO_LIVE_v0.1.0.md`, ligne ~201) déclare déjà explicitement ces slices en **report v0.2.0** : _« WP-H4/H5/H6 = extension produit à arbitrer »_. Le fait qu'aucun commit ne les implémente est donc cohérent avec la décision produit existante, pas un oubli.

- **Slice 4 (gouvernance hybride/eIDAS)** : #576, #577, #578, #579, #581, #582, #583 (7 — #580 et #584 déjà fermées §2.1)
- **Slice 5 (modularité)** : #585, #586, #587, #588, #589, #590, #591, #592 (8)
- **Tx.3** : #595 (log `docs/agent-activity/` — ironique, cette issue documente la convention que ce rapport applique)

**Proposition** : ajouter un label `v0.2.0` (à créer s'il n'existe pas) + un commentaire bref _« Confirmé différé v0.2.0 par décision WBS 2026-06-15 (D6 @gilmry) — cf. WBS_GO_LIVE §Track H »_ sur les 16, pour qu'elles cessent d'apparaître comme « oubliées » dans un futur triage. Pas de fermeture (le travail reste réel, juste pas priorisé v0.1.0).

## B — Méta garde-fous (5 issues)

#425, #426, #427, #428, #429 — les 5 issues fondatrices citées dans `CLAUDE.md`/`.claude/rules/CRITICAL.md` comme source des règles top-11. Par construction, ce sont des **umbrellas de gouvernance vivantes**, pas des tickets de travail ponctuel — leur staleness (87 jours) ne signale pas un abandon.

**Proposition** : ne rien faire. Si l'utilisateur veut un signal de vie régulier dessus, envisager un commentaire trimestriel plutôt qu'un traitement de staleness générique.

## C — Roadmap long-terme non priorisé (20 issues)

Tickets `priority:low`/`priority:medium` sur des capacités futures non engagées (mobile natif, IA, IoT, API v2, Tauri desktop/mobile, K3s/GitOps infra, RFC recherche) :

#111, #98, #109, #94, #268, #267, #266, #48, #299, #298, #297, #296, #295, #339, #344, #343, #353, #355, #354, #331

**Proposition** : garder ouvertes (ce sont de vrais sujets roadmap, pas des bugs oubliés), mais ajouter un label `roadmap-long-terme` (à créer) pour que les futurs triages de staleness les excluent automatiquement — actuellement ils polluent chaque audit de fraîcheur alors qu'ils n'ont pas vocation à bouger avant v0.2.0+.

Cas particulier à noter : **#48** (« itsme/eID ») est référencée par la Story 4.2 (#577, catégorie A ci-dessus, « closes #48 ») — sa fermeture dépendra de la Slice 4, pas d'un traitement isolé.

## D — Bugs/blockers actifs (20 issues) — triage individuel

Contrairement aux catégories A-C, ces issues portent un vrai risque si elles restent ignorées. Proposition par sous-groupe :

### D1 — Sécurité / conformité (action recommandée : prioriser maintenant)

- **#432** — 14 vulnérabilités Dependabot sur `main` (5 high). `main` étant très en retard sur `feature/dev` (cf. audit précédent), vérifier si ces vulns sont déjà résolues côté `feature/dev` avant de retraiter côté `main`.
- **#603** — `[SECURITY]` régression `verify_org_access` skip sur 7 handlers. Cité comme bloqueur go-live dans le WBS — à ne pas laisser traîner.
- **#604** — `[BLOCKER]` trigger référençant une colonne supprimée. Idem, cité bloqueur go-live WBS.

### D2 — Migration Decimal (rattachées à l'umbrella #433, elle-même stale)

- **#433** (umbrella), #443, #439, #438, #455, #525, #534, #521 — cluster cohérent autour de la migration `f64` → `Decimal`. Certaines stories du pipeline #556 (ex. Story 1.4/#561, déjà fermée) couvrent une partie de ce périmètre. **Proposition** : vérifier si #433 peut être requalifiée "en grande partie faite" (comme pour #618) plutôt que fermée en bloc — nécessite un audit dédié similaire à celui fait pour #618, hors scope de ce document.

### D3 — Tests / CI

- **#540** (inventaire BDD pré-existants), **#548** et **#550** (Playwright), **#552** (400 sur endpoints building) — tous `bug:majeur`. #550 est explicitement référencée par la Story Tx.2 (#594, fermée §2.1) comme partiellement traitée (« strate 1-2 faites, strate 3 différée » selon le WBS ligne 96). **Proposition** : rouvrir la discussion sur #550 spécifiquement pour confirmer si la strate 3 reste un gap réel ou si elle a été couverte depuis.

### D4 — Infra / GitOps (non bloquant v0.1.0 mais vieillissant)

- **#453**, **#466**, **#515** — pipeline TLS, RFC GitOps multi-env, ArgoCD fresh-cluster. Cohérent avec Track F (Ops VPS) identifié comme "non démarré" dans l'audit WBS précédent. **Proposition** : regrouper sous un label `track-f-ops-vps` existant ou à créer, traiter ensemble quand Track F démarre plutôt qu'individuellement.

### D5 — Dette technique généraliste

- **#555** (EPIC `Result<_, String>` → `AppError`, 1263 violations) — trop gros pour un triage ponctuel, nécessite son propre plan de découpage en stories (comme #433 ou #556).
- **#602** (gap migration Building/ACP) — probablement partiellement couvert par les stories Slice 1 fermées §2.1 (#558-#561). À vérifier si le gap persiste.

---

## Ce qui n'est PAS proposé

- Aucune fermeture dans ce document (contrairement à §2.1 du rapport précédent qui avait des preuves de commit) — ces 61 issues n'ont pas de commit d'implémentation identifié, fermer serait spéculatif.
- Aucun label créé/appliqué automatiquement — `v0.2.0` et `roadmap-long-terme` sont proposés, pas créés.
- Pas de nouvel audit détaillé pour #433/#555/#602 (mérite chacun son propre passage, comme celui fait pour #618) — signalé comme travail de suivi, pas fait ici.

## Prochaine étape suggérée

Si validé, l'exécution de ce triage (labels + commentaires) est un lot Tier-2 mécanique d'une dizaine de minutes — à confirmer avant de le lancer.
