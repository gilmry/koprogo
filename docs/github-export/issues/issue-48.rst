=========================================================================
Issue #48: feat: Implement strong authentication for voting (itsme, eID)
=========================================================================

:State: **OPEN**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: phase:k8s,track:software priority:low,proptech:blockchain
:Assignees: Unassigned
:Created: 2025-10-27
:Updated: 2025-11-13
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/48>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   **Belgian legal requirement:**
   General assembly votes require **positive identification** of voters to ensure:
   - Only legitimate co-owners vote
   - One person = one owner (no impersonation)
   - Non-repudiation (votes cannot be contested)
   - Audit trail for legal disputes
   
   **Current authentication:** ⚠️ Basic JWT (email + password)
   - Sufficient for read operations
   - **Insufficient for legally binding votes** (AG resolutions)
   - No identity verification
   - Risk of fraud/impersonation
   
   ## Legal Framework (Belgium)
   
   **Code civil (Copropriété):**
   - AG votes are legally binding
   - Requires proof of identity for contested votes
   - Minutes (procès-verbaux) must be legally defensible
   
   **eIDAS Regulation (EU):**
   - Electronic identification for cross-border services
   - Levels: Low, Substantial, **High** (required for voting)
   
   **Acceptable Belgian authentication methods:**
   1. **itsme®** - Most popular (5M+ users in Belgium)
   2. **Belgian eID** (electronic identity card + card reader)
   3. **Qualified electronic signature** (eIDAS compliant)
   
   ## Objective
   
   Implement **strong authentication** specifically for voting operations:
   - itsme® integration (primary)
   - Belgian eID support (secondary)
   - Step-up authentication (elevate from JWT to strong auth when voting)
   - Audit trail for authenticated votes
   
   ## Proposed Architecture
   
   ### Two-Tier Authentication
   
   **Tier 1 - Standard (JWT):**
   - Email + password
   - Used for: browsing, viewing data, non-critical actions
   - Current implementation ✅
   
   **Tier 2 - Strong Authentication:**
   - itsme® or eID
   - **Required for:** casting votes, signing documents
   - New implementation ❌
   
   ### Step-Up Authentication Flow
   
   ```
   User logged in (JWT) → Navigates to voting page
                       ↓
                 Clicks "Vote" button
                       ↓
            System detects: requires strong auth
                       ↓
         Redirect to itsme® authentication
                       ↓
            User authenticates via itsme® app
                       ↓
         itsme® returns identity + verification level
                       ↓
         System validates identity matches owner record
                       ↓
            Vote is cast with strong auth token
                       ↓
         Audit log records: owner_id, itsme® transaction_id
   ```
   
   ## itsme® Integration
   
   ### 1. itsme® Overview
   
   **What is itsme®?**
   - Belgian/EU digital identity app
   - 5+ million active users in Belgium
   - Used by banks, government, utilities
   - Mobile app-based (QR code or deeplink)
   - eIDAS "High" level assurance
   
   **Authentication flow:**
   1. User scans QR code or clicks deeplink
   2. Opens itsme® app on phone
   3. Authenticates with PIN/biometric
   4. itsme® returns verified identity (name, national register number)
   
   ### 2. itsme® OpenID Connect (OIDC) Integration
   
   **Provider:** itsme® acts as OpenID Connect Identity Provider
   
   **Registration:**
   - Create account at https://portal.itsme.be/
   - Obtain Client ID & Client Secret
   - Configure redirect URIs
   
   **Scopes requested:**
   ```
   openid profile email
   service:koprogo_voting  (custom service code)
   ```
   
   **Claims returned:**
   ```json
   {
     "sub": "itsme_user_id",
     "name": "John Doe",
     "given_name": "John",
     "family_name": "Doe",
     "birthdate": "1980-01-01",
     "national_register_number": "80010112345",  // Belgian NISS
     "email": "john@example.com",
     "phone_number": "+32470123456",
     "verified": true,
     "ial": "http://itsme.services/IAL/HIGH"  // Identity Assurance Level
   }
   ```
   
   ### 3. Backend Implementation
   
   **New entity:** `StrongAuthSession`
   
   ```rust
   // backend/src/domain/entities/strong_auth_session.rs
   
   pub struct StrongAuthSession {
       pub id: Uuid,
       pub user_id: Uuid,
       pub owner_id: Uuid,
       pub auth_provider: AuthProvider,  // Itsme, Eid
       pub provider_transaction_id: String,  // itsme transaction ID
       pub national_register_number: String,  // NISS (encrypted)
       pub identity_assurance_level: String,  // "HIGH"
       pub authenticated_at: DateTime<Utc>,
       pub expires_at: DateTime<Utc>,  // 15 minutes
       pub used_for: String,  // "vote:resolution_id"
   }
   
   pub enum AuthProvider {
       Itsme,
       BelgianEid,
   }
   ```
   
   **Database table:**
   ```sql
   CREATE TABLE strong_auth_sessions (
       id UUID PRIMARY KEY,
       user_id UUID NOT NULL REFERENCES users(id),
       owner_id UUID NOT NULL REFERENCES owners(id),
       auth_provider VARCHAR(50) NOT NULL,
       provider_transaction_id VARCHAR(255) NOT NULL,
       national_register_number_encrypted TEXT NOT NULL,  -- Encrypted NISS
       identity_assurance_level VARCHAR(50) NOT NULL,
       authenticated_at TIMESTAMP NOT NULL,
       expires_at TIMESTAMP NOT NULL,
       used_for VARCHAR(255) NOT NULL,
       created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
   );
   
   CREATE INDEX idx_strong_auth_user ON strong_auth_sessions(user_id);
   CREATE INDEX idx_strong_auth_expires ON strong_auth_sessions(expires_at);
   ```
   
   **itsme® OIDC flow:**
   
   ```rust
   // backend/src/infrastructure/auth/itsme_provider.rs
   
   use openidconnect::{
       AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret,
       CsrfToken, IssuerUrl, Nonce, RedirectUrl, Scope,
   };
   
   pub struct ItsmeProvider {
       client: CoreClient,
   }
   
   impl ItsmeProvider {
       pub fn new() -> Self {
           let issuer = IssuerUrl::new("https://idp.e2e.itsme.services/v2".to_string()).unwrap();
           let client_id = ClientId::new(env::var("ITSME_CLIENT_ID").unwrap());
           let client_secret = ClientSecret::new(env::var("ITSME_CLIENT_SECRET").unwrap());
           let redirect_url = RedirectUrl::new(env::var("ITSME_REDIRECT_URI").unwrap()).unwrap();
           
           let client = CoreClient::new(client_id, Some(client_secret), issuer, redirect_url);
           
           Self { client }
       }
       
       // Generate authorization URL (redirect user to itsme®)
       pub fn get_authorization_url(&self, state: String) -> (String, CsrfToken, Nonce) {
           let (auth_url, csrf_token, nonce) = self.client
               .authorize_url(
                   AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                   CsrfToken::new,
                   Nonce::new,
               )
               .add_scope(Scope::new("openid".to_string()))
               .add_scope(Scope::new("profile".to_string()))
               .add_scope(Scope::new("service:koprogo_voting".to_string()))
               .set_state(CsrfToken::new(state))
               .url();
           
           (auth_url.to_string(), csrf_token, nonce)
       }
       
       // Exchange authorization code for tokens
       pub async fn exchange_code(&self, code: AuthorizationCode) -> Result<TokenResponse, String> {
           self.client
               .exchange_code(code)
               .request_async(async_http_client)
               .await
               .map_err(|e| e.to_string())
       }
       
       // Verify ID token and extract claims
       pub async fn verify_identity(&self, id_token: &str) -> Result<ItsmeIdentity, String> {
           // Verify JWT signature, expiration, issuer
           // Extract claims
           // Validate IAL = HIGH
           
           Ok(ItsmeIdentity {
               sub: "...".to_string(),
               name: "John Doe".to_string(),
               national_register_number: "80010112345".to_string(),
               identity_assurance_level: "HIGH".to_string(),
           })
       }
   }
   ```
   
   ### 4. API Endpoints
   
   **Initiate strong authentication:**
   ```
   GET /api/v1/auth/strong/initiate?purpose=vote&resolution_id=<uuid>
   → Returns: { "auth_url": "https://idp.itsme.services/...", "state": "csrf_token" }
   ```
   
   **Callback from itsme®:**
   ```
   GET /api/v1/auth/strong/callback?code=<auth_code>&state=<csrf_token>
   → Validates code, creates StrongAuthSession, returns strong_auth_token
   ```
   
   **Verify strong auth (middleware):**
   ```rust
   #[derive(Debug)]
   pub struct StrongAuth {
       pub session: StrongAuthSession,
   }
   
   impl FromRequest for StrongAuth {
       async fn from_request(req: &HttpRequest, _: &mut Payload) -> Result<Self, Error> {
           let strong_auth_token = req
               .headers()
               .get("X-Strong-Auth-Token")
               .and_then(|h| h.to_str().ok())
               .ok_or(ErrorUnauthorized("Strong auth required"))?;
           
           // Validate token, check expiration
           let session = validate_strong_auth_token(strong_auth_token).await?;
           
           Ok(StrongAuth { session })
       }
   }
   ```
   
   **Protected vote endpoint:**
   ```rust
   #[post("/resolutions/{id}/vote")]
   async fn cast_vote(
       resolution_id: web::Path<Uuid>,
       vote_data: web::Json<VoteRequest>,
       _strong_auth: StrongAuth,  // Requires strong auth
   ) -> Result<HttpResponse, Error> {
       // Cast vote with strong auth session logged
       Ok(HttpResponse::Ok().json(vote))
   }
   ```
   
   ### 5. Frontend Integration
   
   **Strong auth button:**
   
   ```svelte
   <script lang="ts">
     export let resolutionId: string;
     
     async function initiateStrongAuth() {
       const response = await fetch(`/api/v1/auth/strong/initiate?purpose=vote&resolution_id=${resolutionId}`);
       const { auth_url, state } = await response.json();
       
       // Store state in sessionStorage
       sessionStorage.setItem('strong_auth_state', state);
       
       // Redirect to itsme®
       window.location.href = auth_url;
     }
     
     async function handleCallback() {
       const urlParams = new URLSearchParams(window.location.search);
       const code = urlParams.get('code');
       const state = urlParams.get('state');
       
       // Validate state (CSRF protection)
       if (state !== sessionStorage.getItem('strong_auth_state')) {
         throw new Error('Invalid state');
       }
       
       // Exchange code for strong auth token
       const response = await fetch(`/api/v1/auth/strong/callback?code=${code}&state=${state}`);
       const { strong_auth_token } = await response.json();
       
       // Store token (short-lived, 15 min)
       sessionStorage.setItem('strong_auth_token', strong_auth_token);
       
       // Redirect back to voting page
       window.location.href = '/meetings/' + resolutionId + '/vote';
     }
   </script>
   
   <button on:click={initiateStrongAuth} class="itsme-button">
     <img src="/itsme-logo.svg" alt="itsme" />
     Authentifier avec itsme®
   </button>
   ```
   
   **Vote with strong auth:**
   ```svelte
   async function castVote(resolutionId: string, choice: VoteChoice) {
     const strongAuthToken = sessionStorage.getItem('strong_auth_token');
     
     const response = await fetch(`/api/v1/resolutions/${resolutionId}/vote`, {
       method: 'POST',
       headers: {
         'Authorization': `Bearer ${localStorage.getItem('token')}`,  // Regular JWT
         'X-Strong-Auth-Token': strongAuthToken,  // Strong auth token
         'Content-Type': 'application/json',
       },
       body: JSON.stringify({ vote_choice: choice }),
     });
     
     if (response.ok) {
       showToast('Vote enregistré avec authentification forte');
     } else {
       showToast('Authentification forte expirée, veuillez vous réauthentifier');
     }
   }
   ```
   
   ## Belgian eID Support (Secondary)
   
   **Alternative for users without itsme®:**
   
   **Flow:**
   1. User connects eID card reader to computer
   2. Backend calls Belgian eID middleware (via PKCS#11 or Web eID)
   3. User enters PIN on card reader
   4. Backend validates certificate chain
   5. Extract national register number from eID certificate
   
   **Implementation complexity:** Higher (requires middleware installation)
   
   **Recommendation:** Phase 2 (after itsme® working)
   
   ## Security Considerations
   
   1. **National Register Number encryption:**
      - Encrypt NISS at rest (AES-256)
      - Never log NISS in plain text
      - Access only for audit/dispute resolution
   
   2. **Token expiration:**
      - Strong auth token valid **15 minutes only**
      - Cannot be reused for different resolutions
      - Revoked after vote cast
   
   3. **CSRF protection:**
      - State parameter validated
      - Nonce validated in ID token
   
   4. **Audit trail:**
      - Log all strong auth attempts (success/failure)
      - Link votes to itsme® transaction IDs
      - Immutable audit log
   
   5. **Privacy (GDPR):**
      - Minimal data collection (only required claims)
      - Delete strong auth sessions after 90 days (audit retention)
      - User consent for itsme® authentication
   
   ## Testing
   
   - [ ] itsme® sandbox environment working
   - [ ] Authorization flow completes successfully
   - [ ] ID token validated correctly
   - [ ] Identity matches owner record (NISS comparison)
   - [ ] Strong auth token expires after 15 min
   - [ ] Vote endpoint rejects without strong auth
   - [ ] Vote endpoint accepts with valid strong auth
   - [ ] Audit log records itsme® transaction ID
   
   ## Acceptance Criteria
   
   - [ ] itsme® OIDC integration complete
   - [ ] StrongAuthSession entity implemented
   - [ ] Database schema with encrypted NISS storage
   - [ ] API endpoints (initiate, callback) functional
   - [ ] Strong auth middleware working
   - [ ] Vote endpoint protected by strong auth
   - [ ] Frontend itsme® button + callback handling
   - [ ] Audit trail complete
   - [ ] GDPR compliant (minimal data, consent)
   - [ ] itsme® logo usage approved (branding guidelines)
   - [ ] Legal review passed (if applicable)
   
   ## Dependencies
   
   **Backend:**
   ```toml
   [dependencies]
   openidconnect = "3.5"
   jsonwebtoken = "9.2"
   ring = "0.17"  # For encryption
   ```
   
   **Environment variables:**
   ```bash
   ITSME_CLIENT_ID=<client_id>
   ITSME_CLIENT_SECRET=<client_secret>
   ITSME_REDIRECT_URI=https://koprogo.com/auth/itsme/callback
   ITSME_ISSUER=https://idp.e2e.itsme.services/v2  # Sandbox
   # ITSME_ISSUER=https://idp.prd.itsme.services/v2  # Production
   
   NISS_ENCRYPTION_KEY=<32-byte-key>  # For NISS encryption
   ```
   
   ## itsme® Registration Process
   
   1. Create account: https://portal.itsme.be/
   2. Submit service request (KoproGo voting)
   3. Provide legal entity info (company registration)
   4. Sign itsme® Terms of Service
   5. Integration review (sandbox testing)
   6. Production approval (~2-4 weeks)
   
   **Cost:** Free for basic usage, paid plans for high volume
   
   ## Effort Estimate
   
   **Large** (5-7 days)
   - Day 1-2: itsme® registration + sandbox setup
   - Day 3-4: Backend OIDC integration + StrongAuthSession
   - Day 5: Frontend integration + UX
   - Day 6: Testing + audit trail
   - Day 7: Security review + documentation
   
   ## Related
   
   - **Blocks:** Issue #46 (Meeting voting system) - Strong auth required before votes
   - Supports: Legal compliance (Belgian copropriété law)
   - Enhances: Security posture
   
   ## Future Enhancements (Post-MVP)
   
   - Belgian eID support (card reader)
   - Qualified electronic signature (eIDAS)
   - French FranceConnect integration (if expanding to France)
   - Mobile biometric authentication (fallback)
   
   ## References
   
   - itsme® Developer Portal: https://brand.belgianmobileid.be/
   - itsme® Technical Documentation: https://belgianmobileid.github.io/doc/
   - OpenID Connect: https://openid.net/connect/
   - eIDAS Regulation: https://ec.europa.eu/digital-building-blocks/sites/display/DIGITAL/eIDAS
   - Belgian eID: https://eid.belgium.be/

.. raw:: html

   </div>

