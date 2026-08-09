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

### Grep exhaustifs confirmant l'absence d'autres instances des 2 bugs connus (2026-08-09)

- `<Composant ... on:click=/on:submit=/on:change=/on:input=/on:keydown=` sur toute balise capitalisée : **0 résultat**.
- Comparaisons `.status === 'PascalCase'` croisées avec la liste des enums `#[serde(rename_all = "snake_case")]` backend : Resolution/Poll utilisent un pattern sûr (const object généré, valeurs déjà snake_case) ; Budget/AgSession/PaymentReminder/Ticket n'ont **pas** de `rename_all` donc PascalCase est correct côté backend — pas des bugs. Seul `payment.rs` (Transaction/PaymentMethodType) était cassé, cf. ci-dessus.
