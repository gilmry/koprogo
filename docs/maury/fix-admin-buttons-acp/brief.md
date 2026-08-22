---
feature: fix-admin-buttons-acp
phase: Brief court (fix mécanique + fix modèle de données, pas de nouvelle capacité)
status: SIGNED v1.0 par @gilmry 2026-08-08
date: 2026-08-08
authors: [Claude Sonnet 5 (drafting)]
related_issues: [697, 698]
parent_maury: none — trouvé en testant interactivement l'admin au navigateur ("on est dans testing pur")
---

# Brief — Admin bloqué : boutons morts (Svelte 5) + immeuble/ACP (modèle de données)

## 1. Vision

**L'admin ne peut aujourd'hui créer ni une organisation, ni un immeuble, ni un utilisateur, ni gérer une dépense** — pas par 403, pas par bug métier, mais parce que les boutons correspondants ne déclenchent strictement rien au clic. Deux causes indépendantes, empilées sur le même parcours :

1. **#697** — `<Button on:click={...}>` (syntaxe Svelte 4) sur `components/ui/Button.svelte`, qui est un composant Svelte 5 runes-mode. `on:click` est une directive de compilation, jamais reçue par `...restProps` : le bouton est mort, silencieusement, sans warning. 13 occurrences, 6 fichiers.
2. **#698** — Même en réparant #697, la création d'immeuble échoue quand même : `BuildingForm.svelte` envoie `organization_id`, un champ que `CreateBuildingDto` n'a plus depuis la migration 040000 (uniquement `acp_id`, requis). Confusion de domaine : **Organisation = cabinet syndic**, **ACP = entité légale copropriété**, parent direct de l'immeuble — ce n'est pas la même chose (clarifié par @gilmry).

Trouvé en testant interactivement (clic réel, pas visite passive) tous les parcours admin au navigateur, sur demande explicite de @gilmry.

## 2. Personas concernés

### 2.1. Superadmin (cible)

- Doit pouvoir créer une organisation, un immeuble (rattaché à la bonne ACP), un utilisateur, et gérer le cycle de vie d'une dépense (marquer payé/en retard, annuler, réactiver) — aujourd'hui **tout ça est bloqué**.

### 2.2. Tout utilisateur cliquant un `<Button>` "retour" (BuildingDetail, MeetingDetail)

- Navigation cassée silencieusement — pas de bug métier mais expérience dégradée.

## 3. Capacité business (CB)

| CB | Description |
|---|---|
| **CB-1** | Les 13 boutons listés dans #697 déclenchent leur action au clic (ouverture de formulaire, navigation, changement de statut de dépense). |
| **CB-2** | `BuildingForm.svelte` propose un sélecteur **ACP** (pas organisation) et envoie `acp_id` au backend — création d'immeuble fonctionnelle de bout en bout. |
| **CB-3** | `AcpList.svelte` propose un dropdown organisation (au lieu d'un champ UUID brut) pour lier une ACP à son cabinet syndic — cohérence UX avec le reste de l'admin. |

## 4. Invariants techniques (INV)

| INV | Énoncé |
|---|---|
| **INV-1** | Remplacement mécanique `on:click={handler}` → `onclick={handler}` sur les 13 usages de `<Button>` recensés dans #697 (grep exhaustif déjà fait, aucun usage caché). Les `<button>` HTML natifs (minuscule) ne sont pas concernés — hors scope. |
| **INV-2** | `BuildingForm.svelte` : le dropdown organisation est **remplacé**, pas complété, par un dropdown ACP (`GET /acps`, affichage nom + adresse). Payload de création/édition envoie `acp_id`, plus `organization_id`. |
| **INV-3** | `AcpList.svelte` : le champ texte brut `bind:value={form.organization_id}` est remplacé par un `<select>` peuplé via `GET /organizations`, même pattern que le dropdown actuel de `BuildingForm.svelte`. |
| **INV-4** | Aucun changement backend — `CreateBuildingDto`/`building_handlers.rs` acceptent déjà `acp_id` ; `GET /acps` et `GET /organizations` existent déjà. Fix 100% frontend. |
| **INV-5** | Tests 4-cat (voir `stories.md`) — Playwright interactif (clic réel), pas de visite passive : c'est précisément le type de bug qu'une visite passive ne détecte jamais. |

## 5. Critères de succès (SCB)

| SCB | Mesure |
|---|---|
| **SCB-1** | Repro live de #697 (clic "Nouvelle organisation", "Nouvel immeuble", "Nouvel utilisateur") → dialog/form s'ouvre. |
| **SCB-2** | Création d'immeuble de bout en bout (choix ACP → submit → 201 → immeuble visible dans la liste, `acp_id` correct en DB). |
| **SCB-3** | Création d'ACP avec liaison organisation via dropdown (plus de copier-coller d'UUID). |
| **SCB-4** | `npm run build` (astro check + build) et `cargo test --lib` restent verts — aucune régression, aucun changement backend. |
| **SCB-5** | Panneau d'action `ExpenseDetail.svelte` (8 boutons) validé au clic : marquer payé, marquer en retard, annuler, dé-payer, réactiver — chacun produit son changement de statut attendu. |

## 6. Hors-scope explicite

- Filtrage serveur des ACP par organisation dans le dropdown (`GET /acps?organization_id=`) — le volume actuel (bêta fermée) ne le justifie pas ; `GET /acps` sans filtre suffit. Optimisation future si le nombre d'ACP grossit.
- Audit exhaustif d'autres usages `on:click` sur des composants Svelte 5 hors de `<Button>` (ex. `<Modal>`, `<Dropdown>` s'ils existent) — hors périmètre de #697, à ouvrir séparément si découvert.
- Les autres bugs UI/UX trouvés pendant l'exploration passive (i18n manquant sur `/documents` et `/syndic/mandates`, toast trompeur sur `owners/me`, accents manquants, pages orphelines hors nav) — non liés à #697/#698, pas encore issués, hors scope de ce brief.

## 7. Risques et mitigations

| Risque | Probabilité | Impact | Mitigation |
|---|---|---|---|
| Autre usage caché de `on:click` sur `<Button>` non détecté par le grep | Faible | Faible | Le grep (`'<Button[^>]*on:click='`) est exhaustif sur `frontend/src` hors tests ; à revalider en fin de story. |
| Remplacer le dropdown organisation par ACP casse un formulaire d'édition qui affichait déjà un `organization_id` existant | Moyenne | Moyen | Story 2 couvre explicitement le cas édition (`@edge` — immeuble existant avec ACP déjà assignée, dropdown pré-rempli). |
| Confusion résiduelle organisation/ACP dans d'autres composants non audités | Faible | Faible | Hors scope explicite §6 — signalé si découvert, pas traité ici. |

## 8. Signature

```
Mary (Brief) : DRAFT — en attente de signature @gilmry
```

→ Stories directement (pas de PRD/Architecture séparés — fix mécanique + swap de champ, aucune nouvelle capacité backend, aucun choix d'architecture engageant).
