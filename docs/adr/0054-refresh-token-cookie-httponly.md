# ADR 0054 : le refresh token vit en cookie HttpOnly, pas en Bearer/localStorage

- **Status**: Accepted (livré `feature/dev`, PR #543 WP-FE1 + fix #550)
- **Date**: 2026-05-19
- **Track**: Software / Security
- **Related**: [ADR 0005](0005-jwt-authentication.md) (JWT access/refresh, Bearer
  header) — amendée ici sur le seul point du **transport et du stockage**
  côté client, pas sur le format JWT ni la rotation.
- **Source** : `docs/backend/JWT_REFRESH_TOKENS.md` §"Amendment 2026-05-19"
  porte l'implémentation complète (backend + frontend + tests).

## Contexte

ADR-0005 (2025-02-10) avait retenu Bearer + JWT en acceptant que le client
gère lui-même le stockage des deux tokens. En pratique, le flow d'origine
plaçait **access token ET refresh token en `localStorage`** côté frontend :
un vol de session par XSS y donnait un accès rejouable pendant toute la durée
de vie du refresh token (7 jours), pas seulement 15 minutes.

C'était un bloquant sécurité identifié avant la bêta fermée (WP-FE1, #343).

## Décision

- **Access token** : mémoire JS uniquement (`frontend/src/lib/accessToken.ts`).
  Jamais persisté (ni `localStorage`, ni `sessionStorage`, ni cookie lisible
  par JS).
- **Refresh token** : cookie `HttpOnly; Secure; SameSite=Strict;
  Path=/api/v1/auth; Max-Age=7j`, posé et lu côté backend uniquement
  (`backend/src/infrastructure/web/auth_cookie.rs`). Illisible par
  JavaScript, non rejouable hors du même site.
- Le corps JSON de `/auth/login` et `/auth/refresh` ne porte plus
  `refresh_token` : uniquement `{ token, user }`.
- `POST /auth/logout` révoque tous les refresh tokens de l'utilisateur et
  expire le cookie côté navigateur.
- **Single-flight refresh** (`frontend/src/stores/auth.ts`,
  `inflightRefresh`) : dédoublonne les appels concurrents à
  `/auth/refresh` — sans quoi deux îlots Astro (`RouteGuard` +
  `Navigation`) hydratant en parallèle rejouent le même cookie, la rotation
  backend révoque le premier, le second échoue en 401, et l'utilisateur est
  déconnecté à chaque navigation (bug prod réel, fix #550).

Rotation et révocation instantanée (côté DB) restent inchangées par rapport
à ADR-0005.

## Alternatives considérées

1. **Garder Bearer + localStorage, ajouter une détection XSS** — rejeté :
   traite le symptôme, pas la cause ; une XSS réussie reste exploitable.
2. **Access token également en cookie HttpOnly** — rejeté pour l'instant :
   empêcherait l'API d'être appelée hors navigateur (mobile natif, Phase 2)
   sans double mécanisme. Le compromis retenu isole le seul artefact
   long-lived (refresh) derrière `HttpOnly`, garde l'access token
   Bearer-compatible et court (15 min).

## Conséquences

**Positif** :
- Vol de session par XSS limité à la fenêtre de l'access token (15 min) —
  le refresh token n'est jamais exposé au JS.
- Pas de session rejouable après fermeture du navigateur si le cookie
  expire ou est révoqué.

**Négatif / à surveiller** :
- `SameSite=Strict` suppose le front et l'API sur le même site (Traefik,
  domaine unique, API en `/api/v1`). Une bascule vers des sous-domaines
  séparés (Phase 2) exige une re-décision (`Lax` + `COOKIE_DOMAIN`).
- `COOKIE_SECURE=false` est nécessaire en dev/E2E sur `http://localhost`
  (sinon le navigateur rejette le cookie hors HTTPS) — ne jamais laisser
  cette valeur en configuration de production.

## Tests (4 catégories)

- Backend : `backend/tests/e2e_auth.rs` —
  `fe1_happy_login_sets_httponly_cookie_and_strips_body`,
  `fe1_security_refresh_via_cookie_rotates_and_body_has_no_refresh`,
  `fe1_negative_refresh_without_or_forged_cookie_is_401`,
  `fe1_edge_old_refresh_cookie_revoked_after_rotation`.
- Frontend (Playwright) : `frontend/tests/e2e/smoke/AuthCookie.spec.ts`.
