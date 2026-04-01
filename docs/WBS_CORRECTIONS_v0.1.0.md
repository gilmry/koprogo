# WBS — Corrections Bugs UI Review v0.1.0

**Date** : 2026-04-01
**Source** : [HUMAN_REVIEW_REPORT_v0.1.0.md](HUMAN_REVIEW_REPORT_v0.1.0.md)
**Total bugs** : 16 (4 critiques, 4 majeurs, 8 mineurs)
**Estimation globale** : ~5-7 jours dev

---

## Phase 0 — Infrastructure Transversale (prérequis)

> Ces corrections impactent tous les workflows et doivent être faites en premier.

### T0.1 — Feedback erreurs API (toast sur 4xx/5xx)

| Champ | Valeur |
|-------|--------|
| Bug(s) | Transversal (toutes les erreurs 400 silencieuses) |
| Fichiers | `frontend/src/stores/toast.ts`, `frontend/src/components/ui/ToastContainer.svelte` |
| Problème | Le système de toast existe mais n'est pas branché sur les erreurs API |
| Action | Créer un wrapper API (`fetchWithToast`) ou intercepteur qui affiche automatiquement les erreurs 4xx/5xx via le store toast existant. Brancher sur tous les appels `fetch` des composants |
| Effort | 0.5j |
| Débloque | Tous les bugs "silencieux" (WF1-2, WF7-1, WF8) |

---

## Phase 1 — Bugs Critiques (bloquants release)

> 4 bugs qui empêchent l'utilisation des workflows principaux via l'UI.
> **Dépendance** : T0.1 (sinon les erreurs restent silencieuses même après correction)

### T1.1 — Bouton "Nouvelle réunion" manquant sur /meetings

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF1-1 |
| Fichiers | `frontend/src/pages/meetings.astro` (l.1-28), `frontend/src/components/MeetingList.svelte` (l.110-173) |
| Problème | La page meetings.astro ne contient qu'un titre et MeetingList, sans bouton de création |
| Action | Ajouter un bouton "Nouvelle réunion" (visible pour syndic uniquement) qui ouvre un modal de création d'AG. Créer `MeetingCreateModal.svelte` si inexistant, avec champs : titre, date, type (Ordinary/Extraordinary), building_id |
| Effort | 0.5j |
| Débloque | WF1 complet, puis en cascade WF3-6 |

### T1.2 — POST /convocations omet building_id

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF1-2 |
| Fichiers | `frontend/src/components/convocations/ConvocationPanel.svelte` (l.49-62), `frontend/src/lib/api/convocations.ts` (l.67-73) |
| Problème | Le body envoyé au backend ne contient pas `building_id` → 400 |
| Action | Vérifier que `building_id` est bien récupéré depuis le contexte (BuildingSelector ou route param) et inclus dans le payload POST. Si le ConvocationPanel reçoit building_id en prop, vérifier qu'il est passé depuis la page parente |
| Effort | 0.25j |
| Dépendance | T1.1 (le meeting doit exister pour créer la convocation) |

### T1.3 — Contrainte DB voting_power <= 1000 vs tantièmes réels

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF2-1 |
| Fichiers backend | `backend/migrations/20251115120000_create_resolutions_and_votes.sql`, `backend/src/domain/entities/vote.rs` |
| Fichiers frontend | `frontend/src/components/resolutions/ResolutionVotePanel.svelte` (l.225, `max="1000"`) |
| Problème | La contrainte DB et le frontend limitent voting_power à 1000, mais le seed a des lots avec >1000 tantièmes (Emmanuel = 1280) |
| Action | (1) Migration SQL : ALTER CONSTRAINT pour passer la limite à 10000 (ou supprimer la borne haute). (2) Domain entity vote.rs : ajuster la validation. (3) Frontend : modifier `max="1000"` → `max="10000"` dans ResolutionVotePanel. (4) Vérifier cohérence avec le seed |
| Effort | 0.5j |
| Note | Bug mixte backend+frontend |

### T1.4 — Formulaire ticket envoie body malformaté

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF7-1 |
| Fichiers | `frontend/src/components/tickets/TicketCreateModal.svelte` (l.70-90) |
| Problème | Le body JSON envoyé ne correspond pas au DTO attendu par le backend → 400 |
| Action | Comparer le payload envoyé (console.log du fetch body) avec `CreateTicketDto` backend. Corriger les champs manquants/malformés. Vérifier les types (UUID vs string, enum values). Tester avec les données du seed |
| Effort | 0.25j |

---

## Phase 2 — Bugs Majeurs (avant beta)

> 4 bugs qui dégradent significativement l'expérience.
> **Dépendance** : Phase 1 pour pouvoir tester les workflows complets.

### T2.1 — Convocations créées non listées dans l'UI

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF1-3 |
| Fichiers | `frontend/src/pages/convocations.astro` (l.25-57), `frontend/src/components/convocations/ConvocationList.svelte` (l.1-157) |
| Problème | Les convocations créées via API n'apparaissent pas dans la liste UI |
| Action | Vérifier (1) l'URL du GET appelée par ConvocationList (building_id requis ?), (2) le mapping de la réponse API vers les props du composant, (3) le rechargement après création. Possiblement le BuildingSelector ne transmet pas l'ID au composant liste |
| Effort | 0.5j |
| Dépendance | T1.2 |

### T2.2 — NaN% dans les compteurs de vote

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF2-2 |
| Fichiers | `frontend/src/components/resolutions/ResolutionVotePanel.svelte` (l.44-47, l.141, l.151, l.161) |
| Problème | `getVotePercentage()` retourne `(count / totalVotes) * 100` → NaN quand totalVotes = 0 |
| Action | Ajouter une garde : `if (totalVotes === 0) return 0;` dans `getVotePercentage()`. Vérifier aussi que `totalVotes` est bien peuplé depuis l'API (pas undefined) |
| Effort | 0.1j |

### T2.3 — SuperAdmin login ne redirige pas vers /admin

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF14-1 |
| Fichiers | `frontend/src/components/LoginForm.svelte` (l.42-49) |
| Problème | Le mapping role → redirect (l.43-49) mappe SUPERADMIN vers '/admin' mais la redirection ne s'exécute pas |
| Action | Debugger le flux post-login : (1) le rôle est-il bien SUPERADMIN dans la réponse ? (2) la redirection `window.location.href` ou `goto()` est-elle appelée ? (3) y a-t-il un guard qui re-redirige vers /login ? Vérifier aussi que la page /admin.astro existe et est accessible |
| Effort | 0.25j |

### T2.4 — Isolation données : Alice voit 3 immeubles au lieu de 1

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF14-2 |
| Fichiers | `frontend/src/components/BuildingSelector.svelte` (l.32-57), `frontend/src/components/BuildingList.svelte` (l.34-50) |
| Problème | BuildingSelector appelle `GET /buildings?per_page=100` sans filtre organisation/rôle → retourne tous les immeubles |
| Action | (1) Backend : vérifier que GET /buildings filtre par organization_id du user authentifié ET par rôle (owner ne voit que ses immeubles via unit_owners). (2) Frontend : si le backend filtre déjà, le bug est backend. Si non, ajouter le filtre côté backend dans `building_use_cases.rs` ou `building_handlers.rs` |
| Effort | 0.5j |
| Note | Bug probablement backend (filtrage insuffisant) |
| Dépendance | T2.3 (BUG-WF14-3 counter incohérent se corrigera aussi) |

---

## Phase 3 — Bugs Mineurs (v0.2)

> 8 bugs cosmétiques ou d'i18n. Peuvent être traités en batch.

### Lot A — i18n (1 jour)

#### T3.1 — Clés ICU non résolues ({count}, {hours})

| Champ | Valeur |
|-------|--------|
| Bugs | BUG-WF1-4, BUG-SEL-1 |
| Fichiers | `frontend/src/locales/fr.json`, `en.json`, `nl.json`, `de.json` |
| Problème | Les clés ICU MessageFormat (`{count}`, `{hours}`) s'affichent telles quelles au lieu d'être interpolées |
| Action | (1) Vérifier que la lib i18n utilisée supporte ICU MessageFormat (svelte-i18n / paraglide). (2) Si pas d'ICU, convertir en format supporté (`$t('key', { values: { count: n } })`). (3) Vérifier chaque composant qui affiche ces clés pour s'assurer que les variables sont passées |
| Effort | 0.5j |

#### T3.2 — Titres GDPR en anglais

| Champ | Valeur |
|-------|--------|
| Bug | BUG-GDPR-1 |
| Fichiers | `frontend/src/pages/admin/gdpr.astro` (l.6, l.10-20), `frontend/src/pages/settings/gdpr.astro` (l.6, l.16) |
| Problème | Titres hardcodés en anglais ("GDPR" au lieu de "RGPD", sections en anglais) |
| Action | Remplacer les titres hardcodés par des clés i18n. Vérifier que les traductions FR existent dans `fr.json` |
| Effort | 0.25j |

#### T3.3 — Format date income-statement trop strict

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF10-1 |
| Fichiers backend | Handler de `GET /reports/income-statement` |
| Problème | Exige `2026-01-01T00:00:00Z` au lieu d'accepter `2026-01-01` |
| Action | Ajouter un parsing flexible côté backend : tenter `NaiveDate::parse` en fallback si `DateTime::parse` échoue, puis convertir en début de journée UTC |
| Effort | 0.25j |
| Note | Bug backend |

### Lot B — UX Polish (0.5 jour)

#### T3.4 — Bouton "Voter" non visible pour copropriétaire

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF2-3 |
| Fichiers | `frontend/src/components/resolutions/ResolutionVotePanel.svelte` (l.176, l.31, l.246-256) |
| Problème | La condition `canVote` (l.31) empêche l'affichage du bouton pour les owners |
| Action | Debugger `canVote` : vérifier qu'il prend en compte le rôle "owner" et que la résolution est en statut "Pending". Possiblement le rôle n'est pas correctement lu depuis le JWT/store |
| Effort | 0.25j |

#### T3.5 — Copyright 2025 dans le footer

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF14-4 |
| Fichiers | `frontend/src/layouts/Layout.astro` (l.156) |
| Problème | `© 2025` hardcodé |
| Action | Remplacer par `© ${new Date().getFullYear()}` ou `© 2024-${new Date().getFullYear()}` |
| Effort | 5min |

#### T3.6 — Counter immeubles incohérent

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF14-3 |
| Dépendance | T2.4 (se corrigera probablement avec l'isolation des données) |
| Action | Vérifier après T2.4. Si toujours présent, synchroniser le counter avec le nombre réel d'immeubles retournés |
| Effort | 0.1j |

### Lot C — Rate Limiting UX (0.25 jour)

#### T3.7 — Message rate limiting sur le login

| Champ | Valeur |
|-------|--------|
| Bug | BUG-RL-1 |
| Fichiers | `frontend/src/components/LoginForm.svelte` (l.19-60, l.52) |
| Problème | Le login affiche un message générique d'erreur quand le compte est bloqué (429 Too Many Requests) |
| Action | Détecter le status 429 dans la réponse et afficher "Trop de tentatives de connexion. Réessayez dans 15 minutes." au lieu de "Identifiants incorrects" |
| Effort | 0.25j |

---

## Graphe de dépendances

```
Phase 0 (préalable)
  T0.1 Toasts API
    │
Phase 1 (critique) ────────────────────────────
    │                    │              │       │
  T1.1 Btn réunion    T1.3 DB vote   T1.4    T0.1
    │                  constraint    Ticket
  T1.2 building_id      │            form
    │                    │
Phase 2 (majeur) ────────────────────────────
    │              │          │          │
  T2.1 Liste     T2.2       T2.3      T2.4
  convoc.        NaN%       Redir.    Isolation
                             admin    données
                                        │
Phase 3 (mineur) ────────────────────────│──
    │         │        │        │        │
  T3.1-3.2  T3.3    T3.4    T3.5     T3.6
  i18n      Date    Vote    Footer   Counter
            fmt     btn              (auto?)
              │
            T3.7
            Rate limit msg
```

---

## Planning suggéré

| Jour | Tâches | Effort |
|------|--------|--------|
| J1 matin | T0.1 — Intercepteur toast erreurs API | 0.5j |
| J1 après-midi | T1.1 — Bouton "Nouvelle réunion" + modal | 0.5j |
| J2 matin | T1.2 — Fix building_id convocations | 0.25j |
| J2 après-midi | T1.3 — Migration + fix contrainte vote | 0.5j |
| J2 fin | T1.4 — Fix formulaire ticket | 0.25j |
| J3 matin | T2.1 — Liste convocations | 0.5j |
| J3 après-midi | T2.2 — Fix NaN% + T2.3 — Redirect admin | 0.35j |
| J4 matin | T2.4 — Isolation données par rôle | 0.5j |
| J4 après-midi | T3.1 + T3.2 — i18n pass complet | 0.75j |
| J5 matin | T3.3 + T3.4 + T3.5 + T3.6 — UX polish | 0.6j |
| J5 après-midi | T3.7 — Rate limiting UX + tests manuels | 0.25j |

**Total** : ~5 jours dev

---

## Critères de validation

Après chaque phase, relancer les workflows concernés du [HUMAN_REVIEW_PLAN_v0.1.0.md](HUMAN_REVIEW_PLAN_v0.1.0.md) :

| Phase | Workflows à retester |
|-------|---------------------|
| Phase 1 | WF1 (convocations), WF2 (votes), WF7 (tickets) |
| Phase 2 | WF1-6 (AG complet), WF14 (dashboards), login |
| Phase 3 | WF10 (comptabilité), WF11 (SEL), WF13 (GDPR), i18n global |

**GO beta publique** = Phase 1 + Phase 2 complète, 0 bug critique, 0 bug majeur.
