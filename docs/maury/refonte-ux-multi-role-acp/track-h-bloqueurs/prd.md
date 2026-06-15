---
feature: refonte-ux-multi-role-acp/track-h-bloqueurs
phase: B (Business architecture TOGAF)
status: SIGNED v1.0 par @gilmry 2026-06-15
date: 2026-06-15
authors: [Claude Opus 4.7 (drafting), @gilmry (signature 2026-06-15)]
depends_on: brief.md (SIGNED v1.0 2026-06-15)
---

# PRD Track H — Bloqueurs légaux v0.1.0

> Phase B TOGAF — décrit les Functional Requirements (FR-Hx), goals métier, user journeys, AC business et NFR. Pour l'application architecture, voir `architecture.md`. Pour les stories briefables, voir `stories.md`.

## 1. FR-H1 — `BuildingNotConformantError` typée + mapping 422

### Goal métier

L'API doit refuser tout calcul/mutation opérationnelle sur un building non-conforme avec un **payload exploitable** (delta units, delta quotas, building_id) — pas un message string opaque. L'admin doit pouvoir corriger en connaissance de cause.

### User journey

1. **Admin** crée un building dans `/buildings/new` avec `total_units=182` et `total_tantiemes=10000` (acte de base immeuble de @gilmry) mais ne crée que 181 lots, somme quotas = 9975.0.
2. **Syndic** tente `POST /call-for-funds/send` sur ce building via API directe (ex. script).
3. Backend → use-case → `building.assert_conformant(metrics)?` → `BuildingNotConformantError { building_id: <uuid>, units_delta: 1, quota_delta: 25.0, quota_basis: 10000 }`.
4. AppError::Validation → HTTP 422 + body :
   ```json
   {
     "error": "BUILDING_NOT_CONFORMANT",
     "message": "Building <name> is not conformant — 1 unit missing, quota off by +25.0 / 10000 (acte de base)",
     "details": {
       "building_id": "<uuid>",
       "units_delta": 1,
       "quota_delta": "25.0",
       "quota_basis": 10000
     }
   }
   ```
5. Audit `security_incident` row insérée (type `BUILDING_NOT_CONFORMANT`, user_id syndic, target_building_id).

### AC business

- AC-H1.1 — `Building::assert_conformant(metrics)` retourne `Ok(())` si conforme.
- AC-H1.2 — Sinon retourne `Err(BuildingNotConformantError { building_id, units_delta: i32, quota_delta: Decimal, quota_basis: i32 })`.
- AC-H1.3 — `From<BuildingNotConformantError> for AppError` mappe vers `AppError::Validation` HTTP 422 avec body structuré incluant `quota_basis`.
- AC-H1.4 — `From<BuildingNotConformantError> for String` aussi disponible pour legacy use-cases `Result<_, String>` (pas réécriture forcée).
- AC-H1.5 — `units_delta` = `self.total_units - metrics.units_count` (positif si manque, négatif si surplus).
- AC-H1.6 — `quota_delta` = `Decimal::from(self.total_tantiemes) - metrics.quota_sum` (positif si manque, négatif si surplus). **Référence = total_tantiemes du building, PAS constante 1000.**
- AC-H1.7 — `quota_basis = self.total_tantiemes` (lu sur le building, valeur acte de base — peut être 1000, 10000, ou autre).
- AC-H1.8 — Bug fix domain : `building.rs:219` et `:226` doivent utiliser `self.total_tantiemes` (méthode d'instance) au lieu de la constante `CONFORMANT_QUOTA_TOTAL`. Variant statique `compute_is_conformant` doit recevoir `total_tantiemes` en param.
- AC-H1.9 — La constante `CONFORMANT_QUOTA_TOTAL = dec!(1000)` est **dépréciée** (suppression — pas conservée). Tests qui s'y référaient mis à jour.
- AC-H1.10 — Tests 4-cat `building::tests::assert_conformant` couvrent (a) building 1000 millièmes conforme, (b) building 10000 conforme, (c) building 1000 drift quota=999.9 → delta 0.1, (d) building 10000 drift quota=9999.9 → delta 0.1, (e) tampering metrics, (f) negative empty.

### NFR

- Performance : assert_conformant() pur Rust, < 1µs (calcul Decimal en mémoire).
- Audit log async (non-bloquant pour la réponse 422).

---

## 2. FR-H2 — `validate-before-compute` sur 4 use-cases calcul

### Goal métier

Aucun calcul opérationnel (charges, appels de fonds, répartition tantièmes, états de date) ne peut s'exécuter sur un building non-conforme. **Pas de chiffre erroné silencieux** : soit conforme → calcul, soit erreur 422 explicite.

### User journey

#### Cas 1 — Syndic tente create_expense sur building non-conforme

1. Syndic ouvre `/expenses/new?building_id=X` (X = building drift quotas).
2. FE banner conformity actif → bouton « Créer dépense » `disabled` + tooltip.
3. Syndic force submit via API → backend → use-case → `building.assert_conformant()?` → 422.
4. FE catch → toast 422 narratif + redirect `/buildings/X`.

#### Cas 2 — Syndic envoie call_for_funds

Idem cas 1 mais sur `POST /call-for-funds/send`.

#### Cas 3 — Génération charge_distribution

Idem cas 1 mais sur `POST /charge-distribution/compute`.

#### Cas 4 — Génération etat_date

Idem cas 1 mais sur `POST /etat-date/generate`.

### AC business

- AC-H2.1 — `expense_use_cases::create_expense(building_id, ...)` appelle `building.assert_conformant(metrics)?` **avant** insert. Test 4-cat `@security` : impossible de bypass. Validation utilise `building.total_tantiemes` (acte de base), pas constante.
- AC-H2.2 — `call_for_funds_use_cases::send_call_for_funds(call_id)` charge le building, appelle `assert_conformant()?` avant calcul des contributions.
- AC-H2.3 — `charge_distribution_use_case::compute_distribution(building_id)` appelle `assert_conformant()?` avant compute.
- AC-H2.4 — `etat_date_use_cases::generate_etat_date(building_id, ...)` appelle `assert_conformant()?` avant génération.
- AC-H2.5 — Chaque use-case retourne `BuildingNotConformantError` (`From<>` vers `AppError` ou `String` selon signature legacy — pas de réécriture massive).
- AC-H2.6 — Audit `security_incident` insert pour chaque tentative bypass (pre-existing trait pour security_incident_repository réutilisé).
- AC-H2.7 — Feature BDD `validate_before_compute.feature` avec 4 use-cases × 4-cat = 16 scénarios RED-first.

### NFR

- Performance : `assert_conformant()` ajoute ≤ 1ms latency (1 query SQL pour BuildingMetrics + Decimal pur).
- Compat : Préserver signatures `Result<_, String>` legacy via `From<BuildingNotConformantError> for String` (mémoire `validate-before-compute`).

---

## 3. FR-H3 — `Meeting::assert_can_complete()` Art. 3.87 §3-5 CC

### Goal métier

Une AG ne peut être marquée `Completed` (= légalement clôturée, opposable aux copropriétaires) que si **tous** les invariants Art. 3.87 §3-5 CC sont satisfaits. Sinon erreur 422 typée + liste précise des invariants manquants.

### User journey

1. **Syndic** ouvre `/meetings/<id>` après l'AG.
2. Clique « Clôturer la réunion » (bouton existant).
3. Backend → use-case `complete_meeting(id, attendees_count)` → `meeting.assert_can_complete(checklist)?`.
4. Checklist :
   - **a)** Convocations envoyées ≥ 15j avant date (Art. 3.87 §3 CC).
   - **b)** Toutes résolutions ont un résultat clôturé (Art. 3.87 §4 — votes).
   - **c)** Présences enregistrées (Art. 3.87 §5 — feuille de présence).
   - **d)** Quorum atteint (déjà vérifié dans complete() actuel, conservé).
   - **e)** PV draft existe (minutes signables a posteriori — v0.1.0 garde 1 signature pas 2 eIDAS).
5. Si KO → `MeetingNotCompletableError { meeting_id, missing: [ConvocationsNotSent, VotesNotClosed] }` → 422.
6. FE catch → toast 422 narratif (« 2 conditions manquantes : convocations non envoyées, votes pas tous clôturés ») + boutons « Envoyer convocations » et « Clôturer votes » mis en avant.

### AC business

- AC-H3.1 — `Meeting::assert_can_complete(&self, checklist: MeetingCompletionChecklist) -> Result<(), MeetingNotCompletableError>`.
- AC-H3.2 — `MeetingCompletionChecklist` struct : `convocations_sent: bool`, `all_resolutions_closed: bool`, `attendance_recorded: bool`, `attendees_count: i32`, `total_quotas: Decimal`, `minutes_draft_exists: bool`.
- AC-H3.3 — Enum `MissingInvariant`: `ConvocationsNotSent`, `VotesNotClosed`, `AttendanceNotRecorded`, `QuorumNotReached { attended, total }`, `MinutesDraftMissing`.
- AC-H3.4 — `MeetingNotCompletableError { meeting_id: Uuid, missing: Vec<MissingInvariant> }`.
- AC-H3.5 — Use case `complete_meeting(id, ...)` charge meeting + checklist + appelle `assert_can_complete()?` **avant** transition statut.
- AC-H3.6 — `From<MeetingNotCompletableError> for AppError` mappe vers `AppError::Validation` 422 avec details.
- AC-H3.7 — Audit `security_incident` insert pour tentative clôture invalide.
- AC-H3.8 — Tests 4-cat `meeting::tests::assert_can_complete` (happy : tous OK / edge : quorum borne / security : tampering invariants / negative : empty checklist).
- AC-H3.9 — Feature BDD `meeting_complete.feature` 4-cat + 5 invariants individuels.

### NFR

- Performance : assert_can_complete() pur Rust, < 1µs (struct check).
- Charge checklist : 1 query agrégée (convocations.count, resolutions WHERE status, attendances.count, minutes.exists). ≤ 5ms.

---

## 4. FR-B4 — Regression spec « Modifier immeuble »

### Goal métier

Confirmer par test Playwright que le bouton « Modifier » sur la fiche immeuble admin fonctionne. Le code est déjà OK (cf. cartographie 2026-06-15), mais l'issue #553 Bug 1 a été observée en live — un spec de non-régression évite le retour silencieux du bug.

### User journey

1. Admin login → `/buildings` → clique sur une row → `/building-detail?id=X`.
2. Clique bouton « Modifier » → modal `BuildingEditModal` apparaît avec champs pré-remplis.
3. Modifie `name` (ex. ajoute suffix `-edited`).
4. Clique « Enregistrer » → toast success + modal ferme + fiche rechargée avec new name.

### AC business

- AC-B4.1 — Spec `building-edit-modal.spec.ts` (1 fichier, project chromium).
- AC-B4.2 — `@happy` : admin login → fiche → Modifier → submit modifié → vérifier name nouveau visible.
- AC-B4.3 — `@negative` : admin login → fiche → Modifier → vide name → submit → modal reste ouvert + erreur visible.
- AC-B4.4 — data-testid utilisés : `building-edit-submit`, `building-form-name-input`, `building-form-save-submit`, `building-detail-name`.
- AC-B4.5 — Spec se range dans `frontend/tests/e2e/refonte-ux/track-h/building-edit-modal.spec.ts` (nouveau dossier).
- AC-B4.6 — Inclus dans le project chromium (PAS testIgnore).

### NFR

- Spec rapide (< 30s).
- Réutilise admin login helper existant.

---

## 5. Matrice de traçabilité

| FR | CB-Hx | INV-Hx | SCB | Stories | Files BE | Files FE |
|---|---|---|---|---|---|---|
| FR-H1 | CB-H1, H3 | INV-H1, H2, H3, H6 | SCB-H1 | H1 | building.rs (fix constante→`self.total_tantiemes`), application/error.rs, security_incident_repository.rs | ConformityBanner.svelte (nouveau, atomique, affiche `quota_basis`) |
| FR-H2 | CB-H4, H6, H8 | INV-H4, H6 | SCB-H2 | H2 | expense_use_cases.rs, call_for_funds_use_cases.rs, charge_distribution_use_case.rs, etat_date_use_cases.rs, tests/features/validate_before_compute.feature | ConformityBanner sur ExpenseList, CallForFundsList, ChargeDistributionPanel, EtatDatePage ; toast 422 dans handlers FE |
| FR-H3 | CB-H5, H6, H7 | INV-H5, H6 | SCB-H3, H5 | H3 | meeting.rs, meeting_use_cases.rs, ports/meeting_completion_checker.rs (nouveau), tests/features/meeting_complete.feature | MeetingDetail.svelte (bouton Clôturer gaté + toast 422), MissingInvariantsList.svelte (atomique) |
| FR-B4 | CB-H9 | INV-H8 (par cohérence — bouton non disabled) | SCB-H6 | B4 | — (déjà OK) | frontend/tests/e2e/refonte-ux/track-h/building-edit-modal.spec.ts |

## 6. NFR transverses

- **NFR-1 (sécurité)** : impossibilité de bypass par requête API directe (test `@security` couvre login authentifié + payload conforme apparent + entité réelle non-conforme).
- **NFR-2 (a11y WCAG 2.1 AA)** : `ConformityBanner` avec `role="alert"`, contraste ≥ 4.5:1, texte explicite (pas que rouge), icône `⚠️` + texte « Immeuble non conforme » (mémoire a11y-wcag-aa-baseline).
- **NFR-3 (audit)** : chaque tentative bypass → row `security_incident` (type + user_id + target_id + timestamp + raison structurée).
- **NFR-4 (i18n)** : tous messages d'erreur passent par `useTranslations()` FR/NL/EN/DE.
- **NFR-5 (Result<E> typé)** : aucun nouvel `unwrap()`, `expect()` ou `Result<_, String>` introduit hors compat legacy explicitement justifiée.
- **NFR-6 (no f64)** : tous deltas / sums en `Decimal` (cf. mémoire no-f64-in-money).

## 7. Signature

```
Mary (Brief)         : SIGNED v1.0 par @gilmry 2026-06-15
John (PRD)           : SIGNED v1.0 par @gilmry 2026-06-15
```

→ Architecture débloquée (`architecture.md`).
