---
feature: refonte-ux-multi-role-acp/phase-b-fe
phase: prd
phase_togaf: B (Business architecture)
agent_bmad: John (Product Manager)
authors: [Claude Opus 4.7 (drafting), @gilmry (signature pending)]
date: 2026-06-09
version: 0.2
status: Draft 0.2 — Maury-grade rewrite (v0.1 jugé insuffisant 2026-06-09)
parent_brief: phase-b-fe/brief.md (v0.2)
issues_source: [#553, Documentation Vivante drift]
changelog:
  - "0.2 (2026-06-09) — Maury-grade rewrite : chaque FR-B-N a Goal métier détaillé + User Journey narratif + flow alternatifs + métriques produit + critères acceptation business. NFR mesurables avec seuils chiffrés. Tableau matriciel FR ↔ Story ↔ Persona ↔ Capacité ↔ Composant ↔ Page Astro."
  - "0.1 (2026-06-09) — Initial (jugé insuffisant)."
---

# PRD — Phase B FE catch-up (refonte UX multi-rôle ACP)

## Méthode Maury — Phase TOGAF B

**GATE de signature humaine** : à signer par @gilmry après brief, avant ouverture Architecture.

---

## 0. Matrice de cohérence FR ↔ Story ↔ Persona ↔ Capacité ↔ Composant ↔ Page

| FR | Story | Persona principale | Capacité CB | Composant clé | Page Astro |
|---|---|---|---|---|---|
| FR-B0 | B0 | (BE wiring) | — préalable — | — | — |
| FR-B1 | B1 | Admin superadmin / Syndic | CB1 | `RoleAssignmentForm`, `RoleAssignmentList` | `/admin/role-assignments` |
| FR-B2 | B2 | Syndic | CB2 | `MagicLinkIssueForm` | `/syndic/magic-links` |
| FR-B3 | B3 | Syndic | CB3 | `MandateIssueForm`, `MandateList`, `ExpirationBadge` | `/syndic/mandates` |
| FR-B4 | B4 | Syndic / Owner board | CB4 | `RoleDelegationForm`, `RoleDelegationList` | `/syndic/role-delegations` |
| FR-B5 | B5 | Owner | CB5 | `TicketCreate` (refacto), `SeveritySelector`, `EvidenceUpload`, `WitnessSelector` | `/tickets/new` |
| FR-B6 | B6 | Syndic / Owner consultant | CB6 | `SyndicResponseForm`, `SyndicResponseList`, `SlaBadge` | `/tickets/[id]` (refacto) |
| FR-B7 | B7 | Syndic / Mandataire | CB7 | `TechnicalSpecCreate`, `TechnicalSpecDetail`, `TechnicalSpecSignatureForm`, `TechnicalSpecVersionTimeline`, `SignatureForm` | `/syndic/technical-specs`, `/syndic/technical-spec` |
| FR-B8 | B8 | Syndic / lecteur reputation | CB8 + CB9 | `ContractorEvaluationForm`, `ContractorReputation`, `ScoreInput` | `/syndic/contractor-evaluations`, `/contractors/[id]/reputation` |
| FR-B9 (gouvernance) | B9 | Maintainer CI | CB10 | (CI workflow) | — |

> Chaque FR ci-dessous est **autoportant pour rédiger une story**. Pour le détail d'implémentation, voir `stories.md` du même dossier.

---

## 1. FR-B0 — Préalable : OpenAPI complete + TS types regen

### 1.1 Goal métier
Sans OpenAPI à jour, le frontend doit caster manuellement les types des nouvelles entités (Mandate, RoleDelegation, SyndicResponse, TechnicalSpec, ContractorEvaluation), ce qui contredit l'objectif "contract anti-drift" de la CI Contract Types Check. FR-B0 garantit que `frontend/src/types/api.d.ts` est généré automatiquement depuis le backend (source de vérité unique).

### 1.2 User Journey
*En tant que Développeur FE Phase B*, *je veux que les types TypeScript des nouvelles entités soient générés automatiquement depuis le backend*, *pour ne pas avoir à dupliquer les définitions et risquer de divergence.*

### 1.3 Critères d'acceptation business
- `make api-docs` regénère un `openapi.json` qui contient bien les paths `/mandates`, `/role-delegations`, `/tickets/{id}/syndic-responses`, `/technical-specs`, `/technical-specs/{id}/signatures`, `/contractor-evaluations`.
- CI Contract Types Check passe vert (job `Contract Types Check (end-to-end anti-drift)` dans `ci.yml`).
- Aucun cast manuel `as unknown as` dans `frontend/src/lib/api/*.ts` Phase B.

### 1.4 NFR liés
- `make api-docs` complète en < 60 s wall-clock sur Docker stable.
- Diff openapi.json post-regen vs commit précédent doit être stable (deterministe).

---

## 2. FR-B1 — UI assignment sous-rôles comptables

### 2.1 Goal métier
Permettre à un Admin superadmin (et à un Syndic pour son organization) d'assigner les nouveaux sous-rôles `accountant.encodeur` / `accountant.emetteur` / `community.moderator` / mandataires (lawyer/notary/amo/architect/bet/warden) sans toucher la DB directement. Concrétise INV-10 BE : la séparation des pouvoirs comptables passe par 2 assignments distincts (encodeur + émetteur), pas un seul rôle composite.

### 2.2 User Journey narratif
*En tant qu'Admin superadmin* (bootstrap initial d'une nouvelle ACP), *je veux pouvoir assigner le sous-rôle `accountant.encodeur` à Marie (la nouvelle stagiaire comptable) tout en gardant Pierre (le comptable senior) sur `accountant.emetteur` pour la séparation des pouvoirs*, *pour respecter INV-10 (un même user ne doit pas pouvoir saisir une facture ET la marquer payée)*.

### 2.3 Flux alternatifs
- **Flux principal** : Admin login → `/admin/role-assignments` → "Nouvelle assignation" → modal → submit → row visible.
- **Flux alternatif** : Syndic (non-superadmin) accède à `/admin/role-assignments` → page rendue mais sans le bouton "Nouvelle assignation" si Syndic n'a pas le droit cross-org (ou avec filtre auto sur son org).
- **Flux dégradé** : Backend renvoie 422 sur role invalide → message inline rouge sans toaster brut.

### 2.4 Critères d'acceptation business
- Un assignment créé via UI apparaît immédiatement dans la table (rafraîchissement local sans full reload).
- La révocation affiche un modal de confirmation (action destructive — UX-pattern).
- L'expiration affichée respecte le fuseau horaire local du user (`Intl.RelativeTimeFormat`).

### 2.5 NFR liés
- Composant rendu < 500 ms (P95) après navigation.
- axe-core 0 violation.
- Bundle composant + dépendances < 12 KB gzip.

---

## 3. FR-B2 — UI émission MagicLink

### 3.1 Goal métier
Permettre à un Syndic d'émettre un token signé HMAC envoyable par email/SMS à un contractor externe pour qu'il consulte un ticket / quote / invoice / contractor_evaluation **sans créer de compte**. Story 3.2 BE a livré l'endpoint ; Story 3.3 BE a livré la page PWA destinataire ; FR-B2 livre la page émettrice.

### 3.2 User Journey narratif
*En tant que Syndic*, *après qu'un contractor a confirmé sa disponibilité pour réparer une fuite*, *je veux pouvoir lui envoyer un lien magique en 30 secondes (sans qu'il crée un compte)*, *pour gagner du temps et augmenter le taux de réponse contractor.*

### 3.3 Flux alternatifs
- **Flux principal** : Syndic émet → page result affiche URL une seule fois → Syndic copie → coller dans son mail → contractor clique → PWA s'ouvre (Story 3.3).
- **Flux alternatif** : Syndic émet pour `scope_kind=quote` (devis) → contractor doit le soumettre via PWA → workflow identique.
- **Flux dégradé** : Si Syndic ferme la fenêtre sans copier → impossible de retrouver le token (raison de sécurité, single-use). L'écran result l'avertit clairement.

### 3.4 Critères d'acceptation business
- Token affiché dans `<input readonly>` (PAS visible dans `<p>` qui permettrait sélection accidentelle/leak via screenshot).
- Bouton "Copier" utilise l'API Clipboard moderne (`navigator.clipboard.writeText`) avec fallback document.execCommand pour HTTP dev.
- Une fois copié, toast success "URL copiée dans le presse-papier".
- Warning persistant tant que le token est affiché : "Ce lien ne sera plus jamais affiché, copiez-le maintenant".

### 3.5 NFR liés
- Aucun stockage token en `localStorage` / `sessionStorage` (INV-FE5).
- Page reload ne ramène pas le token (state local seul).

---

## 4. FR-B3 — UI émission Mandate

### 4.1 Goal métier
Matérialiser dans l'UI le mandat juridique qu'un Syndic peut émettre vers un Notaire (pour vente d'unit), un Avocat (pour litige), un AMO/Architecte/BET (pour travaux), un Gardien (pour gestion quotidienne d'un bâtiment). Pour chaque mandate, un motif juridique 10-500 chars est obligatoire (traçabilité légale). Validité max 5 ans (INV BE).

### 4.2 User Journey narratif
*En tant que Syndic*, *quand un Lot A2 est mis en vente par M. Dupont (Owner)*, *je veux pouvoir mandater Me Notaire X pour signer l'acte au nom de l'ACP*, *avec une validité de 6 mois et un motif clair, pour que Me Notaire puisse accéder aux données ACP nécessaires (étatdaté, charges payées) durant ce délai*.

### 4.3 Flux alternatifs
- **Flux principal** : Syndic crée mandate notary 6 mois → mandate apparaît dans liste avec ExpirationBadge vert "6 mois".
- **Flux alternatif** : Syndic révoque mandate avant terme (e.g. notaire change) → modal confirm → mandate marqué `revoked` → notaire perd accès immédiatement.
- **Flux dégradé** : Backend refuse validité > 5 ans → message inline sans crash.

### 4.4 Critères d'acceptation business
- ExpirationBadge code couleur : vert > 30j, orange ≤ 30j, rouge ≤ 7j, gris = expiré.
- Reason textarea avec compteur live "X/500 (min 10)".
- Liste triable par date d'expiration (le plus urgent en premier) — fonctionnalité bonus.

---

## 5. FR-B4 — UI délégation rôle temporaire

### 5.1 Goal métier
Quand un Syndic part en vacances ou est temporairement indisponible, il doit pouvoir déléguer son rôle pour 1-90 jours à un autre user (typiquement un Owner board_member en confiance). La délégation est **non-transitive** (INV-8 BE) : un user qui a hérité son rôle ne peut PAS le re-déléguer. L'UI doit rendre cette règle visible et infranchissable.

### 5.2 User Journey narratif
*En tant que Syndic*, *je pars en vacances 7 jours*, *je veux déléguer mon rôle syndic à Pierre Dupont (board member en qui j'ai confiance) du 2026-08-01 au 2026-08-08*, *pour que les urgences ticketing soient gérées en mon absence*.

### 5.3 Flux alternatifs
- **Flux non-transitivité** : Pierre (qui a reçu la délégation) tente de re-déléguer à son cousin → page `/syndic/role-delegations` affiche banner persistant + bouton "Nouvelle délégation" ABSENT côté DOM.
- **Flux expiration** : J+8, le rôle syndic de Pierre expire automatiquement (cron BE). UI rafraîchie : Pierre voit son menu syndic disparaître.

### 5.4 Critères d'acceptation business
- Banner non-transitivité a `role="alert"` + texte clair : "Vous avez reçu ce rôle par délégation. Vous ne pouvez pas re-déléguer (cf. INV-8 BE)".
- Liste affiche TOUTES les délégations actives concernant le user (qu'il a données ET reçues).

---

## 6. FR-B5 — UI Ticket Complaint enrichi

### 6.1 Goal métier
Permettre à un Owner victime d'un trouble (nuisance, dégât, défaut prestation contractor) de déposer une plainte structurée avec preuves photos/vidéos/PDF et témoins (autres Owners ou board members), pour constituer un dossier exploitable par Syndic et CdC sans re-collecte ultérieure des preuves.

### 6.2 User Journey narratif
*En tant qu'Owner du Lot B3*, *je subis des nuisances sonores nocturnes répétées du Lot B5*, *je veux pouvoir documenter ma plainte avec*: severity=High, incident_date=la nuit dernière, 3 photos+1 vidéo audio comme preuves, 2 témoins (mes voisins du palier qui ont signé une pétition), description circonstanciée. *Le ticket est créé + notification au Syndic + au Conseil de copropriété (CdC) qui voient le dossier complet pour leur instruction.*

### 6.3 Flux alternatifs
- **Flux text-only** : Owner crée Complaint sans preuves → autorisé + badge "Preuves manquantes" non-bloquant.
- **Flux upload size limit** : Owner tente d'uploader fichier > 10 MB → refusé client-side avec message clair.
- **Flux witness self** : Owner tente de se lister comme témoin → impossible (bouton Add disabled sur sa row + helper text).

### 6.4 Critères d'acceptation business
- Drag&drop multi-fichiers avec preview thumbnails.
- Compteur live "X/10 preuves" et "X/10 témoins".
- Tickets Request (kind par défaut) restent fonctionnels sans cassure (rétrocompat 32 tests existants).
- Description rich-text minimal (sauts de ligne préservés, pas de WYSIWYG).

### 6.5 NFR liés
- Upload S3/MinIO via presigned URL (pas via backend Rust direct — décharger le worker).
- Bundle EvidenceUpload + WitnessSelector + SeveritySelector + refacto TicketCreate ≤ 25 KB gzip.

---

## 7. FR-B6 — UI réponse Syndic + SLA badge

### 7.1 Goal métier
Quand un Owner pose un ticket (Request ou Complaint), un SLA backend automatique est calculé selon `severity` (24h Critical / 72h High / 5j Normal / 10j Low — Story 3.7 BE). FR-B6 expose ce SLA dans l'UI : badge couleur live + form de réponse Syndic append-only (pas d'édition après création — INV-23 BE).

### 7.2 User Journey narratif
*En tant que Syndic*, *je commence ma journée*, *je veux voir le dashboard tickets triés par SLA proche*, *click sur le plus urgent (Complaint Critical, SLA dans 3h)*, *je rédige une réponse "Plombier appelé, intervention demain 9h" + action_proposed=`schedule_inspection`*, *submit*. *Le badge SLA passe au vert, l'Owner est notifié*.

### 7.3 Flux alternatifs
- **Flux append-only** : Syndic se rend compte d'une faute de frappe 1h après → AUCUN bouton "Edit" dans l'UI. Il poste une 2e réponse corrective (conversation chronologique).
- **Flux SLA dépassé** : Réponse postée après `sla_due_at` → badge reste rouge "Hors SLA — escalade CdC déjà déclenchée" (la cron BE a fait son boulot).

### 7.4 Critères d'acceptation business
- SLA badge rafraîchi automatiquement toutes les 60 s tant que la page est ouverte (`$effect` setInterval).
- Conversation chronologique inversée : plus récente en haut.

---

## 8. FR-B7 — UI TechnicalSpec versionnée + signatures multi-parties

### 8.1 Goal métier
Avant d'engager un Contractor pour des travaux importants (toiture, façade, ascenseur), l'ACP doit avoir une fiche technique signée multi-parties (Syndic + AMO + éventuellement Architecte). Story 3.8 BE matérialise versionning semver + signatures append-only. FR-B7 livre l'UI complète : créer, soumettre pour signatures, signer, bumper.

### 8.2 User Journey narratif (3 acteurs)
1. *Syndic Maury* crée TechnicalSpec v1.0.0 "Réfection toiture ardoise" avec deliverables (3 lignes), required_signatures=[Syndic, AMO], 2 attachments PDF.
2. *Syndic Maury* clique "Soumettre pour signatures" → status PendingSignatures → AMO notifié.
3. *AMO Dupont* (mandate actif Story 3.4 sur l'ACP) log in → voit dans son dashboard "Specs en attente" → click la spec → SignatureForm → checkbox "J'ai lu et j'approuve" → click "Signer" → backend valide mandate actif → signature enregistrée → status PASSE à Approved (toutes signatures reçues).
4. *Plus tard*, *Syndic Maury* veut modifier le deliverable 2 → "Nouvelle version (minor)" → 1.0.0 → 1.1.0 → édite deliverables → submit → nouvelle spec en Draft, 1.0.0 marquée Superseded, signatures de 1.0.0 conservées (minor bump non-breaking).
5. *Cas alternatif* : Bump major (1.5.7 → 2.0.0) → modal warning "Toutes les signatures seront invalidées" → confirm → re-signature requise.

### 8.3 Critères d'acceptation business
- Semver strict : "v1.0.0" / "1.0.0-rc1" / "1.0" tous rejetés.
- Bump major = re-signature requise (UI warning explicite avant action).
- Timeline versions visible : versions précédentes grisées (Superseded), actuelle en gras (Approved).
- Mandataire AMO/Lawyer/Architect/etc. ne peut signer QUE si un Mandate actif (Story 3.4) le couvre.

---

## 9. FR-B8 — UI ContractorEvaluation + Reputation

### 9.1 Goal métier
Après une prestation de Contractor (fin de travaux), le Syndic doit évaluer le Contractor sur 5 dimensions (quality, timeliness, communication, cost_compliance, overall). **Refus 422 si TechnicalSpec préalable absente ou non-Approved** (INV-21 BE) — pas de Far West évaluatif. Une page reputation agrège les scores moyens pour aider à choisir Contractors futurs.

### 9.2 User Journey narratif
*En tant que Syndic*, *les travaux toiture sont terminés*, *je veux évaluer le Contractor X qui les a réalisés*, *en référençant la TechnicalSpec Approved + en liant les 2 tickets ouverts pendant le chantier*, *pour aider mes confrères Syndics à choisir Contractors fiables*.

### 9.3 Flux alternatifs
- **Flux spec manquante** : Syndic tente d'évaluer un Contractor sans TechnicalSpec → toast erreur "Une fiche technique signée est requise" + redirect sur création TechSpec.
- **Flux self-evaluation** : evaluator = contractor → bouton submit disabled + message "Un contractor ne peut pas s'évaluer".

### 9.4 Critères d'acceptation business
- 5 scores en input radio 1-5 (pas de slider décimal — exactitude métier).
- Comment 10-2000 chars avec compteur.
- Page reputation affiche moyenne des 5 scores + nombre d'évaluations + listing des évaluations passées (anonymisées ou pas selon politique GDPR).

---

## 10. FR-B9 — Documentation Vivante refresh (gouvernance signal)

### 10.1 Goal métier
La Documentation Vivante (videos générées par Playwright `--project=scenarios`) est le canary pour détecter les régressions UX réelles. En Phase A, on a posé `continue-on-error: true` (commit `a698f6d`) pour décorréler la dette FE. FR-B9 retire ce bypass : à partir de là, une vidéo cassée est un signal réel.

### 10.2 Critères d'acceptation business
- `.github/workflows/ci.yml` step "Run Documentation Vivante scenarios" **n'a plus** `continue-on-error: true`.
- CI verte sans bypass sur `cf41ef4 + B0 + ... + B8` mergés.
- `playwright-report/` artifact contient 8 nouvelles vidéos (1 par CB) + les anciennes.

---

## 11. NFR (Non-Functional Requirements) globaux Phase B

### NFR-B1 — Type safety
- `svelte-check 0 erreur, 0 warning` sur l'ensemble du repo `frontend/`.
- Aucun cast `as unknown as` dans le code Phase B (post B0).

### NFR-B2 — Accessibility
- `@axe-core/playwright` violations = **0** sur chaque page Phase B en CI.
- Tap targets ≥ 44 × 44 px (Tailwind `min-h-[44px]`).
- Contraste WCAG AA = 4,5:1 minimum sur texte courant, 3:1 sur grand texte (`grand text` ≥ 18 px ou bold ≥ 14 px).

### NFR-B3 — Performance
- Bundle Phase B cumulé ≤ +50 KB gzip (sur baseline 4,3 MB total / 0,6 MB gzip JS mesurée 2026-06-07).
- Page initial paint (LCP) ≤ 2 s sur réseau 4G simulé.
- Composant interactif TTI ≤ 500 ms après mount.

### NFR-B4 — Security
- INV-FE5 : pas de JWT en localStorage / sessionStorage.
- INV-FE7 : non-transitivité respectée côté DOM (bouton ABSENT, pas juste disabled).
- INV-FE8 : pas de bouton "Edit" sur entités append-only.
- Multi-tenant : aucune information cross-org dans le DOM si user n'a pas le rôle.

### NFR-B5 — Testability
- Vitest 4-cat par composant Svelte.
- Playwright multi-rôle e2e par CB (≥ 2 acteurs distincts par scénario `@happy`).
- `data-testid` sur 100% éléments interactifs (sélecteurs i18n-safe).

### NFR-B6 — Maintainability
- Composants atomiques (`ExpirationBadge`, `SlaBadge`, `SignatureForm`, `ScoreInput`) dans `lib/components/shared/` réutilisables.
- Modules `lib/api/*.ts` typés depuis `api.d.ts` regen (post-B0).
- Stories `stories.md` self-contained briefables par agent.

---

## 12. Dépendances bloquantes (synthèse — détail brief §8)

- **DEP-B1** : Phase A BE complete ✅ (`cf41ef4`).
- **DEP-B2** : Story B0 (utoipa + api.d.ts) PRÉALABLE OBLIGATOIRE à B1-B8.
- **DEP-B3** : Docker Desktop stable (12-16 GB RAM Docker > Settings).
- **DEP-B4** : signatures humaines `@gilmry` brief + PRD + architecture + stories.

## 13. Hors-scope explicite (synthèse — détail brief §7)

- ❌ Pas de redesign UX global.
- ❌ Pas de nouvelles features produit (CB ne sont QUE des exposes BE).
- ❌ Pas d'i18n NL/EN/DE (FR suffit pour bêta privée fermée).
- ❌ Pas de mobile-native.
- ❌ Pas d'e-signature qualifié eIDAS.

## 14. Gate signature

```
SIGNED-BY:  @____________
DATE:       2026-__-__
NEXT-PHASE: Architecture FE (architecture.md) — débloquée par signature PRD
TRACEABILITY: chaque FR-B-N ↔ Story B-N (stories.md) ↔ WP-I-N (WBS Track I)
```
