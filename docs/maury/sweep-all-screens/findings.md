---
feature: sweep-all-screens
status: EN COURS
date: 2026-08-09
authors: [Claude Sonnet 5 (drafting)]
parent_brief: brief.md
---

# Findings — sweep-all-screens

Journal continu des trouvailles pendant l'audit systématique, par rôle. Chaque entrée : constat, preuve, action (fixé / documenté / hors-scope).

## Admin

### ✅ FIXÉ — `ExpenseDetail.svelte` : `getPaymentStatusBadge`/`getPaymentMethodLabel` en PascalCase

- **Constat** : comparaient `payment.status`/`payment.method` (type `TransactionStatus`/`PaymentMethodType`, backend `#[serde(rename_all = "snake_case")]`, `payment.rs:7-30`) à des clés PascalCase (`'Succeeded'`, `'SepaDebit'`...). La section "Paiements" de la fiche dépense affichait du texte brut non traduit au lieu du badge.
- **Trouvé par** : grep systématique `#[serde(rename_all = "snake_case")]` sur toutes les entités backend + cross-check frontend, après avoir découvert le même pattern 2× (Story 1 `payment_status`).
- **Fix** : clés `Record` passées en snake_case (`'succeeded'`, `'sepa_debit'`, etc.), même correction que Story 1.
- **Vérifié** : `npm run build` 0 erreur.

### 📋 DOCUMENTÉ (pas fixé) — `InspectionStatus` frontend diverge du backend

- **Constat** : `frontend/src/lib/api/inspections.ts:32-37` définit `InspectionStatus = {Pending, Completed, Failed, PassedWithRemarks}` (valeurs `pending/completed/failed/passed_with_remarks`), alors que le backend `InspectionStatus` (`technical_inspection.rs:72-79`) a `{Scheduled, InProgress, Completed, Failed, Overdue, Cancelled}` (`scheduled/in_progress/completed/failed/overdue/cancelled`) — vocabulaire complètement différent, pas juste une casse.
- **Déjà connu** : commentaire explicite dans le code, `STORY-P7-704`, "Enums kept hand-written because backend schemas diverge". Pas une régression fraîche.
- **Impact réel non mesuré** : pas de endpoint `GET /inspections` plat pour vérifier rapidement (scope building/organization uniquement) ; nécessiterait un fixture building pour tester en direct.
- **Action** : hors-scope de ce sweep (réconciliation = son propre chantier, cf. brief §4). Signalé pour arbitrage @gilmry : soit aligner le frontend sur le vrai contrat backend, soit clarifier pourquoi le hand-written diverge (peut-être un DTO de réponse différent de l'entité domaine, à vérifier).

## Syndic

Visite des 36 écrans syndic (console + requêtes réseau) : **33/36 propres**. 2 écarts investigués :

### ✅ Non-bug (investigué, confirmé bénin) — `getMyOwner()` 404 sur `/exchanges` et `/notices`

- `exchanges.astro`/`notices.astro` appellent `getMyOwner()` (`GET /owners/me`) pour résoudre l'ownerId d'un syndic qui n'a pas de fiche Owner — 404 normal, catché silencieusement (`if (owner) {...}`, pas de toast). Le "console.error" observé est juste le logging réseau automatique de Chrome, pas une erreur applicative. Corrige/précise une trouvaille de l'exploration passive du début de session ("toast trompeur owners/me") — après investigation, il n'y a **pas** de toast, juste un log console silencieux.

### ✅ FIXÉ — Toast "Accès refusé" superflu sur `/syndic/role-delegations`

- **Constat** : `RoleDelegationsPage.svelte:82` appelle `GET /organizations?per_page=1000` (superadmin-only) pour enrichir l'affichage des noms d'organisation dans les délégations — 403 pour un syndic. Le `.catch()` local empêche le crash mais **`api.get()` déclenche un toast automatique avant** que le catch ne s'exécute (toute erreur HTTP sauf `silent: true`, `lib/api.ts:141-143`).
- **Fix** : ajout de `{ silent: true }`, même pattern déjà établi dans `tryGetOrganizationName()` (`lib/api/organizations.ts:50`) pour exactement ce cas (appel optionnel d'enrichissement, 403 attendu pour non-superadmin).
- **Vérifié** : `npm run build` 0 erreur. Les autres appels à `/organizations?per_page=1000` du repo sont tous dans des pages superadmin-only (`OrganizationList`, `AcpList`, `UserForm` via `UserListAdmin`, `RoleAssignmentForm`) — jamais atteints par un rôle non-superadmin, pas de 403 possible en pratique, pas besoin du même fix.

### Grep exhaustifs confirmant l'absence d'autres instances des 2 bugs connus (2026-08-09)

- `<Composant ... on:click=/on:submit=/on:change=/on:input=/on:keydown=` sur toute balise capitalisée : **0 résultat**.
- Comparaisons `.status === 'PascalCase'` croisées avec la liste des enums `#[serde(rename_all = "snake_case")]` backend : Resolution/Poll utilisent un pattern sûr (const object généré, valeurs déjà snake_case) ; Budget/AgSession/PaymentReminder/Ticket n'ont **pas** de `rename_all` donc PascalCase est correct côté backend — pas des bugs. Seul `payment.rs` (Transaction/PaymentMethodType) était cassé, cf. ci-dessus.
