# WBS — Corrections Bugs UI Review v0.1.0

**Date** : 2026-04-01
**Source** : [HUMAN_REVIEW_REPORT_v0.1.0.md](HUMAN_REVIEW_REPORT_v0.1.0.md)
**Total bugs** : 16 (4 critiques, 4 majeurs, 8 mineurs)
**Estimation globale** : ~5-7 jours dev

---

## Phase 0 — Infrastructure Transversale (prérequis)

> Ces corrections impactent tous les workflows et doivent etre faites en premier.

### T0.1 — Feedback erreurs API (toast sur 4xx/5xx)

| Champ | Valeur |
|-------|--------|
| Bug(s) | Transversal (toutes les erreurs 400 silencieuses) |
| Fichiers | `frontend/src/stores/toast.ts`, `frontend/src/components/ui/ToastContainer.svelte` |
| Probleme | Le systeme de toast existe mais n'est pas branche sur les erreurs API |
| Action | Creer un wrapper API (`fetchWithToast`) ou intercepteur qui affiche automatiquement les erreurs 4xx/5xx via le store toast existant. Brancher sur tous les appels `fetch` des composants |
| Effort | 0.5j |
| Debloque | Tous les bugs "silencieux" (WF1-2, WF7-1, WF8) |

---

## Phase 1 — Bugs Critiques (bloquants release)

> 4 bugs qui empechent l'utilisation des workflows principaux via l'UI.
> **Dependance** : T0.1 (sinon les erreurs restent silencieuses meme apres correction)

### T1.1 — Bouton "Nouvelle reunion" manquant sur /meetings

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF1-1 |
| Fichiers | `frontend/src/pages/meetings.astro` (l.1-28), `frontend/src/components/MeetingList.svelte` (l.110-173) |
| Probleme | La page meetings.astro ne contient qu'un titre et MeetingList, sans bouton de creation |
| Action | Ajouter un bouton "Nouvelle reunion" (visible pour syndic uniquement) qui ouvre un modal de creation d'AG. Creer `MeetingCreateModal.svelte` si inexistant, avec champs : titre, date, type (Ordinary/Extraordinary), building_id |
| Effort | 0.5j |
| Debloque | WF1 complet, puis en cascade WF3-6 |

### T1.2 — POST /convocations omet building_id

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF1-2 |
| Fichiers | `frontend/src/components/convocations/ConvocationPanel.svelte` (l.49-62), `frontend/src/lib/api/convocations.ts` (l.67-73) |
| Probleme | Le body envoye au backend ne contient pas `building_id` → 400 |
| Action | Verifier que `building_id` est bien recupere depuis le contexte (BuildingSelector ou route param) et inclus dans le payload POST. Si le ConvocationPanel recoit building_id en prop, verifier qu'il est passe depuis la page parente |
| Effort | 0.25j |
| Dependance | T1.1 (le meeting doit exister pour creer la convocation) |

### T1.3 — Contrainte DB voting_power <= 1000 vs tantiemes reels

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF2-1 |
| Fichiers backend | `backend/migrations/20251115120000_create_resolutions_and_votes.sql`, `backend/src/domain/entities/vote.rs` |
| Fichiers frontend | `frontend/src/components/resolutions/ResolutionVotePanel.svelte` (l.225, `max="1000"`) |
| Probleme | La contrainte DB et le frontend limitent voting_power a 1000, mais le seed a des lots avec >1000 tantiemes (Emmanuel = 1280) |
| Action | (1) Migration SQL : ALTER CONSTRAINT pour passer la limite a 10000 (ou supprimer la borne haute). (2) Domain entity vote.rs : ajuster la validation. (3) Frontend : modifier `max="1000"` → `max="10000"` dans ResolutionVotePanel. (4) Verifier coherence avec le seed |
| Effort | 0.5j |
| Note | Bug mixte backend+frontend |

### T1.4 — Formulaire ticket envoie body malformate

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF7-1 |
| Fichiers | `frontend/src/components/tickets/TicketCreateModal.svelte` (l.70-90) |
| Probleme | Le body JSON envoye ne correspond pas au DTO attendu par le backend → 400 |
| Action | Comparer le payload envoye (console.log du fetch body) avec `CreateTicketDto` backend. Corriger les champs manquants/malformes. Verifier les types (UUID vs string, enum values). Tester avec les donnees du seed |
| Effort | 0.25j |

---

## Phase 2 — Bugs Majeurs (avant beta)

> 4 bugs qui degradent significativement l'experience.
> **Dependance** : Phase 1 pour pouvoir tester les workflows complets.

### T2.1 — Convocations creees non listees dans l'UI

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF1-3 |
| Fichiers | `frontend/src/pages/convocations.astro` (l.25-57), `frontend/src/components/convocations/ConvocationList.svelte` (l.1-157) |
| Probleme | Les convocations creees via API n'apparaissent pas dans la liste UI |
| Action | Verifier (1) l'URL du GET appelee par ConvocationList (building_id requis ?), (2) le mapping de la reponse API vers les props du composant, (3) le rechargement apres creation. Possiblement le BuildingSelector ne transmet pas l'ID au composant liste |
| Effort | 0.5j |
| Dependance | T1.2 |

### T2.2 — NaN% dans les compteurs de vote

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF2-2 |
| Fichiers | `frontend/src/components/resolutions/ResolutionVotePanel.svelte` (l.44-47, l.141, l.151, l.161) |
| Probleme | `getVotePercentage()` retourne `(count / totalVotes) * 100` → NaN quand totalVotes = 0 |
| Action | Ajouter une garde : `if (totalVotes === 0) return 0;` dans `getVotePercentage()`. Verifier aussi que `totalVotes` est bien peuple depuis l'API (pas undefined) |
| Effort | 0.1j |

### T2.3 — SuperAdmin login ne redirige pas vers /admin

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF14-1 |
| Fichiers | `frontend/src/components/LoginForm.svelte` (l.42-49) |
| Probleme | Le mapping role → redirect (l.43-49) mappe SUPERADMIN vers '/admin' mais la redirection ne s'execute pas |
| Action | Debugger le flux post-login : (1) le role est-il bien SUPERADMIN dans la reponse ? (2) la redirection `window.location.href` ou `goto()` est-elle appelee ? (3) y a-t-il un guard qui re-redirige vers /login ? Verifier aussi que la page /admin.astro existe et est accessible |
| Effort | 0.25j |

### T2.4 — Isolation donnees : Alice voit 3 immeubles au lieu de 1

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF14-2 |
| Fichiers | `frontend/src/components/BuildingSelector.svelte` (l.32-57), `frontend/src/components/BuildingList.svelte` (l.34-50) |
| Probleme | BuildingSelector appelle `GET /buildings?per_page=100` sans filtre organisation/role → retourne tous les immeubles |
| Action | (1) Backend : verifier que GET /buildings filtre par organization_id du user authentifie ET par role (owner ne voit que ses immeubles via unit_owners). (2) Frontend : si le backend filtre deja, le bug est backend. Si non, ajouter le filtre cote backend dans `building_use_cases.rs` ou `building_handlers.rs` |
| Effort | 0.5j |
| Note | Bug probablement backend (filtrage insuffisant) |
| Dependance | T2.3 (BUG-WF14-3 counter incoherent se corrigera aussi) |

---

## Phase 3 — Bugs Mineurs (v0.2)

> 8 bugs cosmetiques ou d'i18n. Peuvent etre traites en batch.

### Lot A — i18n (1 jour)

#### T3.1 — Cles ICU non resolues ({count}, {hours})

| Champ | Valeur |
|-------|--------|
| Bugs | BUG-WF1-4, BUG-SEL-1 |
| Fichiers | `frontend/src/locales/fr.json`, `en.json`, `nl.json`, `de.json` |
| Probleme | Les cles ICU MessageFormat (`{count}`, `{hours}`) s'affichent telles quelles au lieu d'etre interpolees |
| Action | (1) Verifier que la lib i18n utilisee supporte ICU MessageFormat (svelte-i18n / paraglide). (2) Si pas d'ICU, convertir en format supporte (`$t('key', { values: { count: n } })`). (3) Verifier chaque composant qui affiche ces cles pour s'assurer que les variables sont passees |
| Effort | 0.5j |

#### T3.2 — Titres GDPR en anglais

| Champ | Valeur |
|-------|--------|
| Bug | BUG-GDPR-1 |
| Fichiers | `frontend/src/pages/admin/gdpr.astro` (l.6, l.10-20), `frontend/src/pages/settings/gdpr.astro` (l.6, l.16) |
| Probleme | Titres hardcodes en anglais ("GDPR" au lieu de "RGPD", sections en anglais) |
| Action | Remplacer les titres hardcodes par des cles i18n. Verifier que les traductions FR existent dans `fr.json` |
| Effort | 0.25j |

#### T3.3 — Format date income-statement trop strict

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF10-1 |
| Fichiers backend | Handler de `GET /reports/income-statement` |
| Probleme | Exige `2026-01-01T00:00:00Z` au lieu d'accepter `2026-01-01` |
| Action | Ajouter un parsing flexible cote backend : tenter `NaiveDate::parse` en fallback si `DateTime::parse` echoue, puis convertir en debut de journee UTC |
| Effort | 0.25j |
| Note | Bug backend |

### Lot B — UX Polish (0.5 jour)

#### T3.4 — Bouton "Voter" non visible pour copropriétaire

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF2-3 |
| Fichiers | `frontend/src/components/resolutions/ResolutionVotePanel.svelte` (l.176, l.31, l.246-256) |
| Probleme | La condition `canVote` (l.31) empeche l'affichage du bouton pour les owners |
| Action | Debugger `canVote` : verifier qu'il prend en compte le role "owner" et que la resolution est en statut "Pending". Possiblement le role n'est pas correctement lu depuis le JWT/store |
| Effort | 0.25j |

#### T3.5 — Copyright 2025 dans le footer

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF14-4 |
| Fichiers | `frontend/src/layouts/Layout.astro` (l.156) |
| Probleme | `© 2025` hardcode |
| Action | Remplacer par `© ${new Date().getFullYear()}` ou `© 2024-${new Date().getFullYear()}` |
| Effort | 5min |

#### T3.6 — Counter immeubles incoherent

| Champ | Valeur |
|-------|--------|
| Bug | BUG-WF14-3 |
| Dependance | T2.4 (se corrigera probablement avec l'isolation des donnees) |
| Action | Verifier apres T2.4. Si toujours present, synchroniser le counter avec le nombre reel d'immeubles retournes |
| Effort | 0.1j |

### Lot C — Rate Limiting UX (0.25 jour)

#### T3.7 — Message rate limiting sur le login

| Champ | Valeur |
|-------|--------|
| Bug | BUG-RL-1 |
| Fichiers | `frontend/src/components/LoginForm.svelte` (l.19-60, l.52) |
| Probleme | Le login affiche un message generique d'erreur quand le compte est bloque (429 Too Many Requests) |
| Action | Detecter le status 429 dans la reponse et afficher "Trop de tentatives de connexion. Reessayez dans 15 minutes." au lieu de "Identifiants incorrects" |
| Effort | 0.25j |

---

## Graphe de dependances

```
Phase 0 (prealable)
  T0.1 Toasts API
    │
Phase 1 (critique) ────────────────────────────
    │                    │              │       │
  T1.1 Btn reunion    T1.3 DB vote   T1.4    T0.1
    │                  constraint    Ticket
  T1.2 building_id      │            form
    │                    │
Phase 2 (majeur) ────────────────────────────
    │              │          │          │
  T2.1 Liste     T2.2       T2.3      T2.4
  convoc.        NaN%       Redir.    Isolation
                             admin    donnees
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

## Planning suggeere

| Jour | Taches | Effort |
|------|--------|--------|
| J1 matin | T0.1 — Intercepteur toast erreurs API | 0.5j |
| J1 aprem | T1.1 — Bouton "Nouvelle reunion" + modal | 0.5j |
| J2 matin | T1.2 — Fix building_id convocations | 0.25j |
| J2 aprem | T1.3 — Migration + fix contrainte vote | 0.5j |
| J2 fin | T1.4 — Fix formulaire ticket | 0.25j |
| J3 matin | T2.1 — Liste convocations | 0.5j |
| J3 aprem | T2.2 — Fix NaN% + T2.3 — Redirect admin | 0.35j |
| J4 matin | T2.4 — Isolation donnees par role | 0.5j |
| J4 aprem | T3.1 + T3.2 — i18n pass complet | 0.75j |
| J5 matin | T3.3 + T3.4 + T3.5 + T3.6 — UX polish | 0.6j |
| J5 aprem | T3.7 — Rate limiting UX + tests manuels | 0.25j |

**Total** : ~5 jours dev

---

## Criteres de validation

Apres chaque phase, relancer les workflows concernes du [HUMAN_REVIEW_PLAN_v0.1.0.md](HUMAN_REVIEW_PLAN_v0.1.0.md) :

| Phase | Workflows a retester |
|-------|---------------------|
| Phase 1 | WF1 (convocations), WF2 (votes), WF7 (tickets) |
| Phase 2 | WF1-6 (AG complet), WF14 (dashboards), login |
| Phase 3 | WF10 (comptabilite), WF11 (SEL), WF13 (GDPR), i18n global |

**GO beta publique** = Phase 1 + Phase 2 complete, 0 bug critique, 0 bug majeur.
