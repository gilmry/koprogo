# Issue #005 - Renforcement de la Sécurité (Rate Limiting, JWT Refresh, CORS)

**Priorité**: 🔴 CRITIQUE
**Estimation**: 10-12 heures
**Labels**: `security`, `backend`, `critical`, `production-ready`

---

## 📋 Description

Renforcer la sécurité de l'application avant la mise en production. Plusieurs vulnérabilités et manques critiques ont été identifiés dans le code audit :

❌ **Problèmes critiques** :
- Pas de rate limiting → Vulnérable aux attaques DDoS/brute-force
- JWT secret hardcodé en fallback (`main.rs:78`)
- CORS ouvert à tous les origins (`*`)
- Pas de refresh token → Sessions non renouvelables
- Logs basiques (env_logger) → Manque de traçabilité
- Pas de détection d'anomalies

---

## 🎯 Objectifs

- [ ] Implémenter rate limiting par IP et par user
- [ ] Sécuriser JWT avec rotation de secrets
- [ ] Ajouter système de refresh tokens
- [ ] Restreindre CORS aux domaines autorisés
- [ ] Migrer vers structured logging (tracing)
- [ ] Ajouter audit logs pour actions sensibles
- [ ] Implémenter 2FA (optionnel mais recommandé)
- [ ] Scanner de vulnérabilités automatique en CI

---

## 📐 Spécifications Techniques

### 1. Rate Limiting

**Stratégie** : Token bucket algorithm

**Limites proposées** :
| Endpoint | Limite | Fenêtre | Action |
|----------|--------|---------|--------|
| `POST /auth/login` | 5 requêtes | 15 min | Block IP |
| `POST /auth/register` | 3 requêtes | 1 heure | Block IP |
| API générale (auth) | 100 req | 1 min | 429 Too Many Requests |
| API générale (no auth) | 20 req | 1 min | 429 Too Many Requests |

**Implementation** : Utiliser `actix-limitation` ou Redis-based rate limiter

---

### 2. JWT Refresh Token

**Flux actuel** :
```
1. User login → JWT access token (expiry: 24h)
2. Token expire → User doit se reconnecter
```

**Nouveau flux** :
```
1. User login → Access token (15min) + Refresh token (7 jours)
2. Access token expire → POST /auth/refresh avec refresh token
3. Retourne nouveau access token
4. Refresh token expire → User doit se reconnecter
```

**Avantages** :
- Tokens courte durée (limite fenêtre d'attaque)
- Rotation automatique
- Révocation possible (blacklist)

---

### 3. CORS Sécurisé

**Actuel** (main.rs:92) :
```rust
.wrap(Cors::permissive()) // ❌ DANGEREUX
```

**Nouveau** :
```rust
.wrap(
    Cors::default()
        .allowed_origin(&env::var("FRONTEND_URL").unwrap_or("http://localhost:3000"))
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(vec![header::AUTHORIZATION, header::CONTENT_TYPE])
        .max_age(3600)
)
```

---

### 4. Structured Logging

**Actuel** :
```rust
env_logger::init(); // Logs non structurés
```

**Nouveau (tracing)** :
```rust
tracing_subscriber::fmt()
    .with_target(false)
    .with_thread_ids(true)
    .with_level(true)
    .json() // Format JSON pour parsing
    .init();
```

**Événements à logger** :
- Authentification (succès/échec)
- Modifications de données sensibles
- Erreurs 500
- Rate limit triggers
- Token refresh

---

## 🔧 Détails d'Implémentation

### 1. Rate Limiting Middleware

**Fichier** : `backend/src/infrastructure/web/middleware/rate_limiter.rs`

```rust
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use futures_util::future::{ok, Ready};

pub struct RateLimiter {
    requests_per_minute: usize,
    storage: Arc<Mutex<HashMap<String, (usize, Instant)>>>,
}

impl RateLimiter {
    pub fn new(requests_per_minute: usize) -> Self {
        Self {
            requests_per_minute,
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn check_limit(&self, key: &str) -> bool {
        let mut storage = self.storage.lock().unwrap();
        let now = Instant::now();

        if let Some((count, last_reset)) = storage.get_mut(key) {
            if now.duration_since(*last_reset) > Duration::from_secs(60) {
                // Reset compteur après 1 minute
                *count = 1;
                *last_reset = now;
                true
            } else if *count < self.requests_per_minute {
                *count += 1;
                true
            } else {
                false
            }
        } else {
            storage.insert(key.to_string(), (1, now));
            true
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimiterMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimiterMiddleware {
            service,
            limiter: self.clone(),
        })
    }
}

pub struct RateLimiterMiddleware<S> {
    service: S,
    limiter: RateLimiter,
}

impl<S, B> Service<ServiceRequest> for RateLimiterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = /* ... */;

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();

        if !self.limiter.check_limit(&ip) {
            return Box::pin(async move {
                Ok(req.into_response(
                    HttpResponse::TooManyRequests()
                        .json("Rate limit exceeded. Try again later.")
                        .into_body(),
                ))
            });
        }

        Box::pin(self.service.call(req))
    }
}
```

**Usage** (main.rs) :
```rust
use crate::infrastructure::web::middleware::rate_limiter::RateLimiter;

App::new()
    .wrap(RateLimiter::new(100)) // 100 req/min
    .route("/auth/login", web::post().to(login_handler)
        .wrap(RateLimiter::new(5))) // 5 req/min pour login
```

**Alternative** : Utiliser Redis pour rate limiting distribué (multi-instances)

---

### 2. Refresh Token System

**Fichier** : `backend/src/domain/entities/refresh_token.rs`

```rust
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
}

impl RefreshToken {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            token: Uuid::new_v4().to_string(), // Mieux: utiliser crypto random
            expires_at: Utc::now() + Duration::days(7),
            revoked: false,
            created_at: Utc::now(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.revoked && self.expires_at > Utc::now()
    }

    pub fn revoke(&mut self) {
        self.revoked = true;
    }
}
```

**Migration** :
```sql
-- backend/migrations/20250124000000_create_refresh_tokens_table.sql
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    INDEX idx_refresh_tokens_token (token),
    INDEX idx_refresh_tokens_user_id (user_id)
);
```

**Use Case** : `backend/src/application/use_cases/auth_use_cases.rs`

```rust
impl AuthUseCases {
    pub async fn refresh_access_token(
        &self,
        refresh_token: String,
    ) -> Result<(String, String), String> {
        // 1. Vérifier refresh token en DB
        let token_entity = self
            .refresh_token_repo
            .find_by_token(&refresh_token)
            .await
            .map_err(|_| "Invalid refresh token".to_string())?;

        if !token_entity.is_valid() {
            return Err("Refresh token expired or revoked".to_string());
        }

        // 2. Récupérer user
        let user = self.user_repo.find_by_id(token_entity.user_id).await?;

        // 3. Générer nouveau access token
        let new_access_token = self.generate_jwt(&user)?;

        // 4. Optionnel: Rotation du refresh token
        let new_refresh_token = RefreshToken::new(user.id);
        self.refresh_token_repo.create(&new_refresh_token).await?;

        // 5. Révoquer ancien refresh token
        self.refresh_token_repo.revoke(&refresh_token).await?;

        Ok((new_access_token, new_refresh_token.token))
    }
}
```

**Handler** : `backend/src/infrastructure/web/handlers/auth_handlers.rs`

```rust
#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

pub async fn refresh_token(
    use_cases: web::Data<Arc<AuthUseCases>>,
    request: web::Json<RefreshRequest>,
) -> Result<HttpResponse> {
    match use_cases.refresh_access_token(request.refresh_token.clone()).await {
        Ok((access_token, new_refresh_token)) => Ok(HttpResponse::Ok().json(json!({
            "access_token": access_token,
            "refresh_token": new_refresh_token,
            "token_type": "Bearer",
        }))),
        Err(e) => Ok(HttpResponse::Unauthorized().json(e)),
    }
}
```

**Route** :
```rust
.route("/auth/refresh", web::post().to(auth_handlers::refresh_token))
```

---

### 3. CORS Configuration

**Fichier** : `backend/src/infrastructure/web/middleware/cors_config.rs`

```rust
use actix_cors::Cors;
use actix_web::http::header;
use std::env;

pub fn configure_cors() -> Cors {
    let allowed_origins = env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    let origins: Vec<&str> = allowed_origins.split(',').collect();

    let mut cors = Cors::default()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
        ])
        .max_age(3600);

    for origin in origins {
        cors = cors.allowed_origin(origin.trim());
    }

    cors
}
```

**Usage** (main.rs) :
```rust
use crate::infrastructure::web::middleware::cors_config::configure_cors;

App::new()
    .wrap(configure_cors())
```

**Configuration** (.env) :
```
ALLOWED_ORIGINS=http://localhost:3000,https://app.koprogo.com
```

---

### 4. Structured Logging

**Cargo.toml** :
```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
tracing-actix-web = "0.7"
```

**Fichier** : `backend/src/infrastructure/logging/mod.rs`

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}
```

**Usage** (main.rs) :
```rust
mod infrastructure;
use infrastructure::logging::init_logging;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logging();

    tracing::info!("Starting KoproGo API server");

    // ... rest of main
}
```

**Logging dans handlers** :
```rust
use tracing::{info, warn, error};

pub async fn login(
    use_cases: web::Data<Arc<AuthUseCases>>,
    request: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    info!(email = %request.email, "Login attempt");

    match use_cases.login(request.into_inner()).await {
        Ok(response) => {
            info!(user_id = %response.user.id, "Login successful");
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            warn!(email = %request.email, error = %e, "Login failed");
            Ok(HttpResponse::Unauthorized().json(e))
        }
    }
}
```

---

### 5. Audit Logs

**Migration** :
```sql
-- backend/migrations/20250124000001_create_audit_logs_table.sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID,
    ip_address INET,
    user_agent TEXT,
    details JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    INDEX idx_audit_logs_user_id (user_id),
    INDEX idx_audit_logs_created_at (created_at),
    INDEX idx_audit_logs_action (action)
);
```

**Entité** : `backend/src/domain/entities/audit_log.rs`

```rust
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde_json::Value;

pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<Value>,
    pub created_at: DateTime<Utc>,
}

impl AuditLog {
    pub fn new(
        user_id: Option<Uuid>,
        action: impl Into<String>,
        resource_type: impl Into<String>,
        resource_id: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            action: action.into(),
            resource_type: resource_type.into(),
            resource_id,
            ip_address: None,
            user_agent: None,
            details: None,
            created_at: Utc::now(),
        }
    }
}
```

**Usage** :
```rust
// Après suppression d'un building
audit_log_repo.create(&AuditLog::new(
    Some(current_user_id),
    "DELETE",
    "Building",
    Some(building_id),
)).await?;
```

---

### 6. JWT Secret Rotation

**Problème actuel** (main.rs:78) :
```rust
let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
// ❌ Fallback dangereux
```

**Solution** :
```rust
let jwt_secret = env::var("JWT_SECRET")
    .expect("JWT_SECRET must be set in environment variables");

// Validation
if jwt_secret.len() < 32 {
    panic!("JWT_SECRET must be at least 32 characters long");
}
```

**.env.example** :
```
# Générer avec: openssl rand -base64 32
JWT_SECRET=your-super-secret-key-min-32-chars-long
```

**Rotation** : Implémenter support de multiples secrets (ancien + nouveau) pendant période de transition.

---

### 7. 2FA (Two-Factor Authentication) - Optionnel

**Fichier** : `backend/src/application/use_cases/two_factor_use_cases.rs`

```rust
use totp_rs::{Algorithm, Secret, TOTP};

pub struct TwoFactorUseCases;

impl TwoFactorUseCases {
    pub fn generate_secret() -> String {
        Secret::generate_secret().to_string()
    }

    pub fn generate_qr_code(user_email: &str, secret: &str) -> Result<String, String> {
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(secret.to_string()).to_bytes().unwrap(),
            Some("KoproGo".to_string()),
            user_email.to_string(),
        )
        .map_err(|e| e.to_string())?;

        totp.get_qr_base64().map_err(|e| e.to_string())
    }

    pub fn verify_code(secret: &str, code: &str) -> bool {
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(secret.to_string()).to_bytes().unwrap(),
            None,
            "user".to_string(),
        )
        .unwrap();

        totp.check_current(code).unwrap_or(false)
    }
}
```

**Cargo.toml** :
```toml
totp-rs = "5.5"
```

---

## ✅ Critères d'Acceptation

### Rate Limiting
- [ ] Login limité à 5 tentatives / 15min par IP
- [ ] API limitée à 100 req/min pour users authentifiés
- [ ] Retourne 429 avec header `Retry-After`
- [ ] Bypass rate limit pour tests (feature flag)

### JWT Refresh
- [ ] Access token expire après 15min
- [ ] Refresh token expire après 7 jours
- [ ] Endpoint `/auth/refresh` fonctionne
- [ ] Ancien refresh token révoqué après utilisation
- [ ] Refresh tokens stockés en DB avec index

### CORS
- [ ] CORS restreint aux origins configurées
- [ ] Variable d'environnement `ALLOWED_ORIGINS`
- [ ] Erreur 403 si origin non autorisé
- [ ] Préflight requests (OPTIONS) supportées

### Logging
- [ ] Logs en format JSON structuré
- [ ] Champs user_id, action, timestamp présents
- [ ] Logs de sécurité (login, échecs auth)
- [ ] Rotation des logs (logrotate)

### Audit
- [ ] Actions critiques loggées (DELETE, UPDATE users)
- [ ] Audit logs queryables par user_id
- [ ] Rétention 1 an minimum

### JWT Secret
- [ ] Pas de fallback hardcodé
- [ ] Validation longueur minimum 32 chars
- [ ] Application crash si JWT_SECRET manquant

---

## 🧪 Plan de Tests

```rust
#[actix_rt::test]
async fn test_rate_limit_login() {
    // Faire 6 tentatives login
    // Vérifier 5 premières OK
    // Vérifier 6ème retourne 429
}

#[actix_rt::test]
async fn test_refresh_token_flow() {
    // Login → Recevoir access + refresh
    // Attendre expiration access (mock time)
    // POST /auth/refresh
    // Vérifier nouveau access token valide
}

#[actix_rt::test]
async fn test_refresh_token_revoked() {
    // Utiliser même refresh token 2 fois
    // Vérifier 2ème appel échoue
}

#[actix_rt::test]
async fn test_cors_allowed_origin() {
    // Requête avec Origin: http://localhost:3000
    // Vérifier header Access-Control-Allow-Origin présent
}

#[actix_rt::test]
async fn test_cors_blocked_origin() {
    // Requête avec Origin: http://evil.com
    // Vérifier rejet
}

#[actix_rt::test]
async fn test_audit_log_created() {
    // Supprimer un building
    // Vérifier audit_log créé avec action=DELETE
}
```

---

## 🚀 Checklist de Développement

- [ ] 1. Créer middleware rate_limiter.rs
- [ ] 2. Créer entité RefreshToken + migration
- [ ] 3. Implémenter refresh_token use case
- [ ] 4. Ajouter endpoint POST /auth/refresh
- [ ] 5. Créer cors_config.rs
- [ ] 6. Migrer vers tracing (remplacer env_logger)
- [ ] 7. Créer entité AuditLog + migration
- [ ] 8. Ajouter audit logging dans handlers critiques
- [ ] 9. Sécuriser JWT_SECRET (validation startup)
- [ ] 10. Mettre à jour .env.example avec nouvelles vars
- [ ] 11. Tests E2E
- [ ] 12. Documentation sécurité
- [ ] 13. Commit : `feat: implement security hardening (rate limiting, JWT refresh, CORS, audit logs)`

---

## 🔐 Variables d'Environnement

**Ajouter dans `.env`** :
```bash
# JWT Configuration
JWT_SECRET=<générer avec: openssl rand -base64 64>
JWT_ACCESS_EXPIRATION=900      # 15 minutes en secondes
JWT_REFRESH_EXPIRATION=604800  # 7 jours en secondes

# CORS
ALLOWED_ORIGINS=http://localhost:3000,https://app.koprogo.com

# Rate Limiting
RATE_LIMIT_REQUESTS_PER_MIN=100
RATE_LIMIT_LOGIN_REQUESTS=5

# Logging
RUST_LOG=info,koprogo=debug,sqlx=warn
LOG_FORMAT=json
```

---

## 📚 Ressources

- [OWASP API Security Top 10](https://owasp.org/www-project-api-security/)
- [actix-limitation](https://docs.rs/actix-limitation/)
- [tracing documentation](https://docs.rs/tracing/)
- [JWT Best Practices](https://datatracker.ietf.org/doc/html/rfc8725)

---

**Créé le** : 2025-10-23
**Milestone** : v1.0 - Production Ready
**Bloque** : Déploiement production
