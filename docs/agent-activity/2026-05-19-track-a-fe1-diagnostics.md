# Agent activity — 2026-05-19 — Diagnostics Track A / FE1 (Tier-2)

Persona : architecte/diagnostic. **Vague docker-light** : 3 sous-agents read-only lancés en parallèle (aucun docker/git/cargo/npm/écriture) → propositions structurées relues par l'orchestrateur. Tier-2 loggé ici (CRITICAL.md #11). Implémentation = orchestrateur, en série, post-validation.

## WP-A3 — EXP-006 journal_entry — IMPLÉMENTÉ par l'orchestrateur (pas sous-agent)
Cause : entité domaine `journal_entry.rs` en `Result<_, String>` ; « AppError sur l'entité » (WBS) viole la pureté hexagonale (AppError = application+actix). Précédent codebase = enum domaine pur `vote.rs ProxyValidationError`. **Option A confirmée humain.** Fait : enum `JournalEntryError` (9 variantes) + `Display` + `impl Error` + `From<JournalEntryError> for String` (pont pour use-case String inchangé, hors scope A3) ; entité `Result<_, JournalEntryError>` (new/validate_lines_balance/validate_line/new_debit/new_credit) ; `From<JournalEntryError> for AppError → Validation` (400, plus 500 Internal) ajouté **en fin** de `error.rs` (coordination anti-conflit A3/A4) ; export mod.rs ; 4 tests in-module → `matches!` typé. Reste : feature `journal_entries.feature` 4-cat (0 tag aujourd'hui ; +@edge 0.1+0.2=0.3, +@security cross-org, taguer @happy/@negative existants) — additif RED-first, suite distincte.

## WP-FE1 — JWT hors localStorage (BLOQUANT SÉCURITÉ) — diagnostic (sous-agent)
- **Fichier réel** = `frontend/src/stores/auth.ts` (PAS lib/auth.ts). Tokens (access+refresh) en `localStorage` (`koprogo_token`/`koprogo_refresh_token`) — vol XSS = session longue rejouable.
- Backend : aucun cookie ; `auth_handlers.rs` login/refresh/register/switch_role renvoient refresh dans le body JSON ; `middleware.rs` lit access via header Bearer (inchangé) ; CORS `main.rs:524-531` **sans `.supports_credentials()`**.
- Cible (infra-only, hexagonal-clean — use-case inchangé) : refresh → cookie `HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth` posé par backend ; access en mémoire JS ; silent-refresh au load (`credentials:"include"`). Wrapper réponse HTTP sans `refresh_token` (ne pas modifier `LoginResponse` DTO).
- 4-cat RED-first : @security (cookie HttpOnly, localStorage vide), @negative (cookie absent/forgé/réutilisé→401), @happy (login→reload→silent-refresh), @edge (refresh à la borne d'expiration). Backend e2e/BDD + Playwright `AuthCookie.spec.ts`.
- **Risques** : R1 ripple `helpers/auth.ts injectAuth`+`global-setup.ts` (~30+ specs ⇒ dépendance WP-D1, FE1 doit livrer le nouveau injectAuth cookie). R2 boot dev cassé : `.supports_credentials()` + `CORS_ALLOWED_ORIGINS=*` (dev/integration `.env.example`, `docker-compose.base.yml:98`) incompatibles → `validate_cors_origins` refuse `*` ⇒ corriger ; `COOKIE_SECURE=false` en compose dev (sinon cookie ignoré sur http). R3 `SameSite=Strict` OK si front/api même site (koprogo.be) — confirmer topo prod. R4 `koprogo_user` localStorage (décision UX). R5 ne pas régresser `register` use-case.
- **Hygiène** : fichier parasite à supprimer (humain) : `backend/src/application/dto/auth_dto.rs.tmp.153245.1763509305518`.
- Parallélisable (aucune dép Track A).

## WP-A5 — EXP-007 quote/etat_date — diagnostic (sous-agent)
- **AUCUNE migration SQL** : colonnes déjà `DECIMAL` (`20251115000000_create_etats_dates.sql`, `20251120150000_create_quotes.sql`). Bug = casts artificiels `::FLOAT8` au SELECT (`etat_date_repository_impl.rs`, 7 SELECT × 5 colonnes) alimentant des champs `f64` d'entité. `quote.rs` **déjà propre** (Decimal partout ; `QuoteScore` f32 = non-monétaire, garder).
- À faire : `etat_date.rs` suppr `use f64;` (l.2 parasite) ; 5 champs (`owner_balance/arrears_amount/monthly_provision_amount/total_balance/approved_works_unpaid`) f64→`Decimal` ; **`unit_area` reste f64 (ADR-0009 physique)** ; enum `EtatDateError`+`QuoteError` domaine purs ; DTO `etat_date_dto.rs` 5 champs→Decimal ; repo retirer `::FLOAT8` (sauf `unit_area`/`avg_processing_days`) ; `From<EtatDateError/QuoteError> for AppError` ; use_cases→`AppError` (ports repo restent String, option (a), borné) ; steps `bdd_governance.rs:6079` parser Decimal (sinon ne compile plus).
- 4-cat : `etat_date.feature`/`quotes.feature` existent, **0 tag** ; modèle = `governance_decimal.feature`. @edge 0.1+0.2=0.3 + round-trip NUMERIC ; @negative montant négatif/transition invalide ; @security cross-org.
- Vertical atomique (ADR-0007 §94 interdit migration partielle) : casse cascade DTO→steps/e2e/tests in-module, à migrer dans la même story.

## WP-A4 — EXP-005 charge_distribution — diagnostic (sous-agent)
- `charge_distribution.rs` : **aucun f64** (Decimal partout, ADR-0007 OK). `Result<_,String>` : domaine (3) + port (8) + use-case (5) + repo impl (8) + handler (4 ad-hoc `.json({"error"})`).
- Règle métier : 0≤quota≤1, Σquotités ≤ `QUOTA_SUM_TOLERANCE=dec!(1.0001)` (borne `>` : 1.0001 passe, 1.00011 échoue), total distribué==total à 1ct. **Gap @security : isolation cross-org ABSENTE** (use-case ne compare jamais `organization_id` appelant vs `expense.organization_id`).
- À faire : enum `ChargeDistributionError` domaine pur + `From for AppError` (Validation 400 ; NotFound/Conflict/Forbidden selon cas) ; port/use-case/repo/handler→AppError ; `charge_distribution.feature` (existe, 0 tag) taguer + @edge (borne 1.0001) + @negative (quota>1/négatif) + @security (cross-org — si fix wiring org dépasse A4 : scénario **RED documenté**, pas vert factice).
- **Overlap error.rs A3/A4/A5** : chaque WP ajoute son `impl From<…Error> for AppError` — insérer **en fin** de section From (après `From<sqlx::Error>`), un bloc par WP, un seul WP édite error.rs à la fois. A3 a déjà ouvert la section (pattern réutilisable).

## Suite (décisions humaines)
- Implémenter FE1 (bloquant sécu, parallélisable) / A4 / A5 depuis ces propositions — ordre & feu humain.
- Réconciliation différée : 5 PR #535-539 + #541 + Track A local (stack local croissant).
- Signaler/supprimer le fichier parasite `auth_dto.rs.tmp.*`.
