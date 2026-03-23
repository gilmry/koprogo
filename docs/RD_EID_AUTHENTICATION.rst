==========================================================
R&D: Authentification forte eID/itsme® pour votes AG
==========================================================

Issues: #223, #48
Status: Research Phase
Phase: Jalon 3 (Différenciation)

.. contents::
   :depth: 2

Overview
========

Belgian property owners can authenticate strongly using:

1. **itsme®**: Belgian mobile ID app (4M+ users, OpenID Connect)
2. **Belgian eID card**: Smart card reader + certificate (legacy)
3. **FAS (Federal Authentication Service)**: CSAM portal (B2B)

This R&D focuses on **itsme® integration** for remote general assembly (AG) voting, where strong authentication is legally mandated (Article 3.87 §2 Belgian Code Civil).

Why Strong Authentication for AG Voting?
=========================================

Belgian law requires **qualified electronic signatures** (QES) for:

- Remote voting at general assemblies (visioconférence)
- Financial decisions (expense approval > EUR 5,000)
- Board member election
- Amendments to building regulations

**Current risk**: JWT session tokens alone don't prove identity to notaries/auditors.

**Solution**: Link votes to government-verified identities via eIDAS-compliant authentication.

itsme® Platform Overview
=========================

**itsme®** is Belgium's national mobile ID:

- **Adoption**: 4M+ active users (~40% of Belgian adult population)
- **Standard**: OpenID Connect (OAuth2.0 variant)
- **Regulation**: eIDAS Qualified Trust Service Provider (QTSP)
- **Backend**: Managed by Belgian Digital agency + telco partners
- **Cost**: Free for nonprofits (ASBL), EUR 500-2K/year for commercial orgs

Key credentials included in itsme®:

- Name + given name
- Email
- Phone number
- NISS (National Identity Number, 11 digits) — special category data
- Certificate (digital signature)
- Age (18+, 65+)

itsme® OpenID Connect Flow
===========================

**Step 1: Redirect to itsme®**

.. code-block:: text

   User clicks "Login with itsme®" on KoproGo
   ↓
   Browser redirects to:
   https://idp.prd.itsme.services/v1/authorization?
     client_id=koprogo_client_id
     &response_type=code
     &scope=openid+profile+email+http://itsme.services/v1/claim/BENationalNumber
     &redirect_uri=https://app.koprogo.be/auth/itsme/callback
     &state=random_value
     &nonce=random_value

**Step 2: User authenticates with itsme®**

- Opens itsme® mobile app
- Approves KoproGo login request
- Optionally reviews scopes (what data will be shared)

**Step 3: Authorization code returned**

.. code-block:: text

   Browser redirected back:
   https://app.koprogo.be/auth/itsme/callback?
     code=auth_code_xyz
     &state=random_value

**Step 4: Exchange code for tokens**

.. code-block:: http

   POST https://idp.prd.itsme.services/v1/token
   Content-Type: application/x-www-form-urlencoded

   code=auth_code_xyz
   &client_id=koprogo_client_id
   &client_secret=secret_xyz
   &redirect_uri=https://app.koprogo.be/auth/itsme/callback
   &grant_type=authorization_code

**Response**:

.. code-block:: json

   {
     "access_token": "eyJhbGciOiJSUzI1NiIs...",
     "token_type": "Bearer",
     "expires_in": 3600,
     "id_token": "eyJhbGciOiJSUzI1NiIs...",
     "refresh_token": "refresh_token_xyz"
   }

**Step 5: Decode ID token (JWT)**

The ``id_token`` JWT contains user identity claims:

.. code-block:: json

   {
     "iss": "https://idp.prd.itsme.services",
     "sub": "unique_subject_id",
     "aud": "koprogo_client_id",
     "iat": 1703001234,
     "exp": 1703004834,
     "given_name": "Jean",
     "family_name": "Dupont",
     "email": "jean.dupont@example.com",
     "phone_number": "+32498765432",
     "http://itsme.services/v1/claim/BENationalNumber": "92.03.14-123.45",
     "http://itsme.services/v1/claim/issuedAt": "2020-01-15",
     "http://itsme.services/v1/claim/gender": "M",
     "http://itsme.services/v1/claim/birthDate": "1992-03-14"
   }

GDPR & Data Protection Strategy
================================

**NISS (National Identification Number) is special-category data**:

Under GDPR Article 9(1), processing NISS requires:

- **Legal basis**: Article 6(1)(c) (legal obligation) OR (b) (contract)
- **Purpose limitation**: Can only use NISS for its stated purpose
- **Data minimization**: Don't request NISS unless legally required
- **Retention**: Delete NISS within 30 days of authentication

**KoproGo GDPR Strategy**:

1. **Login**: Request only ``openid``, ``profile``, ``email`` scopes (NO NISS)
2. **AG Voting context**: Request NISS scope ONLY when user initiates a vote
3. **Storage**: Never store NISS; retrieve it fresh for each sensitive action
4. **Audit**: Log: "User X voted using itsme® NISS Y at timestamp Z"
5. **Deletion**: Purge itsme® user identifiers after voting concludes (30 days)

**GDPR Compliance Checklist**:

- ✅ Purpose statement: "Strong authentication for AG voting"
- ✅ Consent flow: User approves scope sharing in itsme® app
- ✅ Data minimization: Only NISS + vote + timestamp
- ✅ Right to object: Users can vote via standard JWT (non-strong)
- ✅ Audit trail: ``audit_log`` table with voting_type = 'itsme_strong'
- ✅ Data deletion: Background job deletes NISS after 30 days
- ✅ Transparency: Privacy policy explains itsme® data handling

Implementation Design
====================

**New tables** (PostgreSQL):

.. code-block:: sql

   -- itsme® user mapping (never stores NISS)
   CREATE TABLE itsme_identities (
       id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       user_id             UUID NOT NULL REFERENCES users(id),
       itsme_subject_id    VARCHAR(100) NOT NULL UNIQUE,  -- From id_token "sub"
       email               VARCHAR(255),
       given_name          VARCHAR(100),
       family_name         VARCHAR(100),
       phone_number        VARCHAR(20),
       gender              VARCHAR(10),
       birth_date          DATE,
       first_authenticated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
       last_authenticated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
       updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
       INDEX idx_itsme_user_id ON user_id
   );

   -- Strong authentication events (audit trail)
   CREATE TABLE strong_auth_events (
       id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       user_id             UUID NOT NULL REFERENCES users(id),
       itsme_subject_id    VARCHAR(100),
       action              VARCHAR(100),  -- vote_cast, expense_approved, etc.
       action_id           UUID,           -- vote_id, expense_id, etc.
       ip_address          INET,
       user_agent          TEXT,
       metadata            JSONB,          -- {niss: "hashed", timestamp: "..."}
       created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
   );

**Rust implementation** (Actix-web + OAuth2):

.. code-block:: rust

   // Cargo.toml
   [dependencies]
   openidconnect = "2.0"
   reqwest = { version = "0.11", features = ["json"] }

   // backend/src/infrastructure/auth/itsme.rs
   use openidconnect::core::CoreClient;
   use openidconnect::{Client, IssuerUrl, ClientId, ClientSecret, RedirectUrl};

   pub struct ItsmeAuthProvider {
       client: CoreClient,
   }

   impl ItsmeAuthProvider {
       pub fn new(client_id: String, client_secret: String) -> Self {
           let issuer_url = IssuerUrl::new(
               "https://idp.prd.itsme.services".to_string()
           ).unwrap();

           let client = CoreClient::from_issuer_url(
               issuer_url,
               ClientId::new(client_id),
               Some(ClientSecret::new(client_secret)),
               RedirectUrl::new("https://app.koprogo.be/auth/itsme/callback".to_string()).unwrap(),
           ).unwrap();

           ItsmeAuthProvider { client }
       }

       pub fn authorization_url(&self) -> (String, String) {
           let (auth_url, csrf_state, nonce) = self.client
               .authorize_url(CoreResponseType::Code)
               .add_scopes(vec![
                   Scope::new("openid".to_string()),
                   Scope::new("profile".to_string()),
                   Scope::new("email".to_string()),
               ])
               .request_nonce(Nonce::new_random)
               .url();

           (auth_url.to_string(), csrf_state.secret().clone())
       }

       pub async fn exchange_code(&self, code: String) -> Result<IdToken, String> {
           let token_response = self.client
               .exchange_code(AuthorizationCode::new(code))
               .request_async(async_http_client)
               .await
               .map_err(|e| e.to_string())?;

           Ok(token_response.id_token().unwrap().clone())
       }
   }

   // backend/src/infrastructure/web/handlers/auth_handlers.rs
   #[get("/auth/itsme/login")]
   pub async fn itsme_login_redirect(
       itsme: web::Data<ItsmeAuthProvider>,
   ) -> impl Responder {
       let (auth_url, csrf_token) = itsme.authorization_url();

       // Store CSRF token in session (Redis or secure cookie)
       HttpResponse::Found()
           .append_header(("Location", auth_url))
           .finish()
   }

   #[get("/auth/itsme/callback")]
   pub async fn itsme_callback(
       query: web::Query<CallbackParams>,
       itsme: web::Data<ItsmeAuthProvider>,
       user_repo: web::Data<UserRepository>,
       itsme_repo: web::Data<ItsmeIdentityRepository>,
   ) -> impl Responder {
       // Validate CSRF token
       let code = &query.code;
       let id_token = itsme.exchange_code(code.to_string()).await?;

       // Extract claims
       let subject_id = id_token.subject().to_string();
       let email = id_token.email().unwrap().to_string();
       let given_name = id_token.given_name().and_then(|n| n.get(None)).map(|n| n.to_string());

       // Find or create user
       let user = user_repo.find_or_create_by_email(&email).await?;

       // Store itsme identity mapping
       itsme_repo.upsert(&user.id, subject_id, email, given_name).await?;

       // Issue JWT session token
       let jwt = issue_jwt(&user)?;

       HttpResponse::Found()
           .append_header(("Location", format!("/dashboard?token={}", jwt)))
           .finish()
   }

**AG Voting with strong auth**:

.. code-block:: rust

   #[post("/resolutions/{id}/vote-strong")]
   pub async fn vote_with_strong_auth(
       user: AuthenticatedUser,
       req: web::Json<VoteRequest>,
       itsme: web::Data<ItsmeAuthProvider>,
       vote_repo: web::Data<VoteRepository>,
       audit_repo: web::Data<AuditLogRepository>,
   ) -> impl Responder {
       // Step 1: Require fresh itsme authentication
       // (In practice, check if last_authenticated_at < 5 minutes ago)
       let itsme_identity = itsme_repo.get_by_user_id(&user.id).await?;
       if itsme_identity.last_authenticated_at < Utc::now() - Duration::minutes(5) {
           return error("Strong authentication expired. Please login with itsme® again");
       }

       // Step 2: Cast vote (linked to itsme user)
       let vote = Vote {
           resolution_id: req.resolution_id,
           owner_id: user.id,
           choice: &req.choice,
           voting_power: req.voting_power,
           strong_auth_method: "itsme".to_string(),
           itsme_subject_id: Some(&itsme_identity.itsme_subject_id),
           created_at: Utc::now(),
       };
       vote_repo.create(&vote).await?;

       // Step 3: Audit log (GDPR Article 30 compliance)
       audit_repo.log(AuditEvent {
           user_id: user.id,
           action: "vote_strong_auth_cast",
           resource: format!("vote:{}", vote.id),
           ip_address: req.ip.parse()?,
           metadata: json!({
               "itsme_subject_id": hash_for_audit(&itsme_identity.itsme_subject_id),
               "resolution_id": req.resolution_id,
               "choice": req.choice,
           }),
       }).await?;

       ok("Vote recorded with strong authentication")
   }

Legal Compliance
================

**Article 3.87 §2 Belgian Code Civil** (visioconférence for AG):

   "La tenue et le vote à distance peuvent être organisés en utilisant
    une solution de vidéoconférence qualifiée. Les votes sont authentifiés
    par signature électronique qualifiée."

**eIDAS Regulation** (EU 2014/910):

- itsme® is registered as a Qualified Trust Service Provider
- Qualified Electronic Signature (QES) = admissible in Belgian courts
- Signatures generated via itsme® are legally binding

**Belgian Electronic Signature Act**:

- Requires two-factor authentication for sensitive transactions
- itsme® includes biometric + PIN (2FA built-in)
- Audit trail mandatory for disputes

Implementation Timeline
=======================

**Phase 1: Setup (1 week)**

- Request itsme® developer account (free for ASBL, approval 1-2 weeks)
- Configure itsme® sandbox credentials
- Set up test environment

**Phase 2: Backend Integration (2 weeks)**

- Implement itsme® OAuth2 flow
- Create ``itsme_identities`` table
- Add strong auth event logging
- Write unit tests (100% coverage of auth logic)

**Phase 3: Voting Integration (2 weeks)**

- Add ``strong_auth_method`` field to votes table
- Implement ``vote_with_strong_auth()`` endpoint
- Add UI button "Vote with itsme®" (distinct from standard JWT)
- Integration tests with itsme® sandbox

**Phase 4: Documentation & Training (1 week)**

- Privacy policy updates (itsme® data handling)
- User guides (how to use itsme® for voting)
- Board training materials
- Notary/auditor documentation

**Phase 5: Pilot Program (4 weeks)**

- Beta with 3-5 buildings (real itsme® production)
- Gather feedback on UX
- Monitor authentication failures
- Audit security logs

**Total**: 10 weeks (~2.5 developer-months)

Fallback for Non-users
======================

Not all owners have itsme® (older population, some immigrants). Voting options:

1. **Standard JWT** (non-strong): Owner can vote via email link + password
2. **2FA TOTP** (medium strength): Already implemented (Issue #78)
3. **itsme® strong** (high strength): Owners with itsme®
4. **In-person voting** (highest strength): Physical attendance

Board decisions on "strong authentication required?" on a per-meeting basis.

Risk Mitigation
===============

**Risk**: itsme® server down during vote

**Mitigation**:
- Implement fallback to JWT voting
- Log failed itsme® authentications for retry
- Extend voting deadline if service outage > 1 hour

**Risk**: User loses access to itsme® (phone stolen, app deleted)

**Mitigation**:
- Allow email-verified voting as fallback
- Document recovery process (syndic override with audit trail)
- Implement backup codes (similar to 2FA backup codes)

**Risk**: NISS data exposure in logs/backups

**Mitigation**:
- Hash NISS in audit logs (never store plaintext)
- Encrypt NISS in memory (libsodium)
- Implement encryption-at-rest for PostgreSQL backups
- Daily automated NISS deletion (cron job, 30-day retention)

Security Hardening
===================

1. **TLS 1.3 only** for itsme® communication
2. **PKCE (Proof Key for Code Exchange)** for OAuth2 (prevents code interception)
3. **CSRF tokens** for every itsme® redirect
4. **ID token signature validation** (verify itsme® public keys)
5. **Nonce validation** (prevent replay attacks)
6. **IP allowlisting** for itsme® callback (optional, improves security)

Estimated Costs
===============

- **itsme® developer account**: Free for ASBL, EUR 500-2K/year commercial
- **itsme® license**: Included in account (no per-auth cost)
- **Development**: ~2.5 developer-months
- **Infrastructure**: No additional costs (uses existing PostgreSQL)
- **Monitoring**: Standard Prometheus metrics + alerting

Roadmap Integration
====================

- **Jalon 1** (Security): Already included 2FA TOTP
- **Jalon 3** (Différenciation): **itsme® strong auth** ← This R&D
- **Jalon 4** (Automation): Automated vote counting with itsme® signatures
- **Jalon 5** (Mobile): itsme® deep-linking for mobile app

Related Issues
==============

- **#223**: itsme® integration design (this R&D)
- **#48**: AG voting security (parent issue)
- **#78**: 2FA TOTP (already implemented)
- **#88**: Convocations system (depends on voting method)
- **#104**: Court-admissible audit trail (audit_log usage)

References
==========

- `itsme® Developer Portal <https://developer.itsme.be/>`_
- `eIDAS Regulation <https://digital-strategy.ec.europa.eu/en/policies/eidas-regulation>`_
- `OpenID Connect Core 1.0 <https://openid.net/specs/openid-connect-core-1_0.html>`_
- `Belgian eID Interoperability Guide <https://www.eid.belgium.be/>`_
- `Article 3.87 Belgian Code Civil (copropriété) <https://www.justetmieux.be/>`_
