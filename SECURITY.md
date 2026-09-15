# Security Policy

## Supported Versions

We release patches for security vulnerabilities for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Known Security Advisories

The following security advisories are known and have been assessed for risk:

### RUSTSEC-2023-0071: rsa 0.9.8 - Marvin Attack

**Status**: Accepted Risk (Low Impact)
**Severity**: 5.9 (Medium)
**Date**: 2023-11-22
**Affected**: `rsa 0.9.8` via `sqlx-mysql`
**Last-Reviewed**: 2026-09-13 (#432 triage — still accepted, no sqlx-mysql upstream fix yet, mysql feature still disabled)

**Assessment**:
- This vulnerability is in the `rsa` crate used by `sqlx-mysql`
- KoproGo uses **PostgreSQL only** (not MySQL), so this dependency path is not active in production
- The `mysql` feature is NOT enabled in our sqlx configuration
- **Impact**: None in production environment
- **Mitigation**: Monitor for updates to sqlx that will resolve this transitive dependency

**Why Accepted**:
- No fixed upgrade available yet
- Not used in our production code path
- Waiting for upstream fix in sqlx

### RUSTSEC-2025-0111: tokio-tar 0.3.1 - PAX Headers Parsing

**Status**: Accepted Risk (Test Only)
**Severity**: Not Critical
**Date**: 2025-10-21
**Affected**: `tokio-tar 0.3.1` via `testcontainers`
**Last-Reviewed**: 2026-09-13 (#432 triage — still test-only, no testcontainers upstream fix yet)

**Assessment**:
- This vulnerability is in `tokio-tar` used by `testcontainers`
- `testcontainers` is used **only in test/development environment**
- Never included in production builds
- **Impact**: Test environment only
- **Mitigation**: Monitor for updates to testcontainers

**Why Accepted**:
- No fixed upgrade available yet
- Only affects test infrastructure, not production code
- Test containers run in isolated Docker environments
- Waiting for upstream fix in testcontainers

### RUSTSEC-2026-0049 / 0098 / 0099 / 0104 / 0258: AWS SDK legacy TLS chain (rustls-webpki 0.101.7, h2 0.3.27)

**Status**: Accepted Risk (Upstream Blocked)
**Severity**: Medium
**Date**: 2026-03-21 (0049/0098/0099/0104), 2026-08 (0258)
**Affected**: `rustls-webpki 0.101.7` and `h2 0.3.27` via `rustls@0.21` → `hyper-rustls@0.24` → `hyper@0.14.32` → `aws-smithy-http-client@1.4.0` → `aws-config` / `aws-sdk-s3`
**Last-Reviewed**: 2026-09-13 (#432 triage — same accepted family, no upstream release yet; see `backend/.cargo/audit.toml` for the authoritative per-ID list)

**Assessment**:
- The fixed versions (`rustls-webpki >= 0.103.x`, actix-web's own `h2 >= 0.4.16`) are already present elsewhere in the dependency tree
- The vulnerable versions are pulled in exclusively by the AWS SDK's legacy `hyper-rustls@0.24` / `rustls@0.21` / `hyper@0.14` chain, which has no newer `aws-smithy-http-client` release to migrate to
- `RUSTSEC-2026-0049/0098/0099` — CRL/URI/wildcard certificate-name-constraint matching in `rustls-webpki 0.101.7`; AWS's own SDK usage relies on OCSP stapling, not CRL, and does not hit the wildcard/URI constraint code paths this advisory describes
- `RUSTSEC-2026-0104` — reachable panic in CRL parsing (same webpki version, same non-CRL usage)
- `RUSTSEC-2026-0258` — unbounded empty DATA frames (DoS) in `h2 0.3.27`; this is the AWS SDK's internal `h2` instance, distinct from the `h2 >= 0.4.16` actix-web itself already uses for the public API
- **Impact**: Limited to AWS S3 backup upload/restore paths only (not user-facing API)

**Why Accepted**:
- Blocked by upstream AWS SDK Rust — cannot be resolved without an `aws-smithy-http-client` release that drops the legacy `hyper-rustls@0.24` chain
- The attack surface is restricted to S3 backup operations, not the main API
- Monitored via `cargo audit` in CI

**Mitigation**:
- Will be resolved automatically once AWS SDK migrates to `hyper-rustls@0.26` / `rustls@0.22` / `h2@0.4.x`
- Track upstream: https://github.com/awslabs/aws-sdk-rust

---

### CVE-2026-32766 / RUSTSEC-2026-0112 / 0113 / 0145 / RUSTSEC-2025-0134: testcontainers/bollard dev-dependency chain (test-only)

**Status**: Accepted Risk (Dev-Only, Mitigated)
**Severity**: Low
**Date**: 2026-03-21 (CVE-2026-32766), 2026-06+ (0112/0113/0145/0134)
**Affected**: `astral-tokio-tar 0.5.6`/`0.6.1` and `rustls-pemfile 2.2.0`, both via `testcontainers@0.27.x` / `bollard` (dev-dependencies only)
**Last-Reviewed**: 2026-09-13 (#432 triage — same test-only family, no `testcontainers` release beyond 0.27.x yet)

**Assessment**:
- `testcontainers@0.27.3` is the latest available version and still depends on the vulnerable `astral-tokio-tar` / `rustls-pemfile` versions
- These crates are used **exclusively in test builds** — never compiled into production binaries
- `RUSTSEC-2026-0112` (PAX header desynchronization) and `RUSTSEC-2026-0113` (`unpack_in` chmod via symlinks) affect `astral-tokio-tar 0.6.1`, the same crate/path as CVE-2026-32766 (0.5.6); `RUSTSEC-2026-0145` is the same advisory re-published against the newer version
- `RUSTSEC-2025-0134` (`rustls-pemfile 2.2.0` unmaintained) is pulled by `bollard`, testcontainers' Docker client — no direct usage in koprogo code
- The fixed version (`astral-tokio-tar >= 0.6.0`) is explicitly declared as a direct dependency in `Cargo.toml` to ensure 0.6.0 is present alongside the vulnerable transitive version
- **Impact**: None in production; test environment only, running in isolated Docker containers

**Why Accepted**:
- No fixed upgrade available for testcontainers/bollard
- Zero production exposure (dev-dependency, `#[cfg(test)]`)
- Mitigated by explicit `astral-tokio-tar = "0.6"` override where applicable

**Mitigation**:
- Will be resolved once `testcontainers`/`bollard` update their dependencies
- Track upstream: https://github.com/testcontainers/testcontainers-rs

---

### RUSTSEC-2026-0002: lru 0.12.5 - IterMut violates Stacked Borrows (unsound)

**Status**: Accepted Risk (Transitive, No Direct Usage)
**Severity**: Low
**Date**: 2026-04
**Affected**: `lru 0.12.5` via `aws-sdk-s3` (connection pooling, internal)
**Last-Reviewed**: 2026-09-13 (#432 triage)

**Assessment**:
- `aws-sdk-s3` (latest) still depends on `lru 0.12.5`; no `0.13.x` available through that path
- koprogo never calls `lru` directly — used internally by `aws-sdk-s3` for connection pooling
- **Impact**: Theoretical unsoundness (Stacked Borrows violation in `IterMut`); no known exploit reachable through `aws-sdk-s3`'s usage pattern

**Why Accepted**:
- No newer `lru` reachable without an `aws-sdk-s3` release
- Soundness issue, not a known practical exploit path

**Mitigation**:
- Track `aws-sdk-s3` for an `lru 0.13+` bump

---

### RUSTSEC-2026-0187: lopdf 0.31.0 - stack overflow via deeply nested PDF objects

**Status**: Accepted Risk (Attack Surface Does Not Apply)
**Severity**: Medium
**Date**: 2026-05
**Affected**: `lopdf 0.31.0` via `printpdf 0.7` (direct dependency, pinned major)
**Last-Reviewed**: 2026-09-13 (#432 triage)

**Assessment**:
- Fixing requires `printpdf >= 0.12`, which pulls `lopdf >= 0.42` — a breaking API migration across 5 major `printpdf` versions, touching every PDF exporter in `backend/src/domain/services/` (annual report, meeting minutes, PCN, owner statement, ownership contract, convocation, work quote)
- koprogo only **generates** PDFs via `printpdf` — no code path parses untrusted PDF input (no `lopdf::Document::load` call anywhere in `backend/src`)
- **Impact**: None — the deeply-nested-input parser attack this advisory describes is not reachable; koprogo never parses PDF input

**Why Accepted**:
- Attack surface (untrusted PDF parsing) does not exist in koprogo's usage of the crate
- The fix requires a breaking, cross-cutting `printpdf` major migration disproportionate to a non-reachable risk

**Mitigation**:
- Revisit if a PDF upload/parsing feature is ever added (would change the reachability assessment above)

---

### RUSTSEC-2026-0235: rkyv 0.7.46 - insufficient archive validation (OOB reads)

**Status**: Accepted Risk (Transitive, No Direct Usage)
**Severity**: Medium
**Date**: 2026-06
**Affected**: `rkyv 0.7.46` via `rust_decimal`'s optional `rkyv` feature
**Last-Reviewed**: 2026-09-13 (#432 triage)

**Assessment**:
- Pulled by `rust_decimal`'s optional `rkyv` feature; koprogo never calls `rkyv` directly or deserializes untrusted `rkyv`-format bytes
- **Impact**: None — no reachable code path deserializes attacker-controlled `rkyv` archives

**Why Accepted**:
- No direct usage; feature is transitively enabled, not exercised on untrusted input

**Mitigation**:
- Monitor `rust_decimal` for an `rkyv 0.8+` upgrade

---

### GitHub Dependabot Alerts — h3 (npm) — SSE/Path Traversal

**Status**: Accepted Risk (Transitive, No Production Exposure)
**Severity**: Medium/High
**Date**: 2026-03-21
**Affected**: `h3` (transitive npm dependency in frontend build tooling)
**Last-Reviewed**: 2026-09-13 (#432 triage — same conclusion, static-site frontend, no Node runtime in prod)

**Assessment**:
- `npm audit` reports **0 vulnerabilities** in the frontend production dependency tree
- The Dependabot alerts target `h3` versions in the lockfile that are pulled transitively by build/dev tooling (not runtime code shipped to users)
- `h3` is a Node.js HTTP framework; KoproGo's frontend is a **static site** (Astro SSG) — no Node.js server runs in production
- **Impact**: None in production; the frontend is served as static files

**Why Accepted**:
- `npm audit` confirms clean production tree
- No server-side Node.js runtime in production frontend
- Transitively fixed via package updates

---

### RESOLVED — npm `devalue` (HIGH, DoS sparse-array) + `svelte` ×4 (MEDIUM, XSS/ReDoS) — closed 2026-05-17

**Status**: Resolved (no longer accepted risk — kept here as closure record, cf. #432)
**Severity**: 1 High + 4 Medium at time of alert
**Date reported**: ~2026-04-30 (GitHub Dependabot on `feature/dev` push)
**Date resolved**: 2026-05-17
**Affected**: `devalue < 5.8.1` (DoS via crafted sparse array), `svelte < 5.55.7` (XSS ×3, ReDoS) — both direct frontend dependencies

**Resolution**: `package.json` ranges (`svelte ^5.55.5`, `devalue >=5.6.4`) already permitted the patched versions, so this was a lockfile-only bump: `npm update svelte devalue` → `svelte 5.55.7`, `devalue 5.8.1`. Verified via `npm audit --omit=dev` (0 vulnerabilities) and `npm run build` (green, no regression). Full detail: `docs/agent-activity/2026-05-17-wbs-b2.md`.

**2026-09-13 re-verification (#432 triage)**: `origin/main`'s `frontend/package-lock.json` resolves `svelte` to `5.57.0` and `devalue` to `5.8.1` — both at or beyond the patched versions above. The fix landed on `main`, not only on `feature/dev` (this had been an open question in the 2026-07-26 triage log). Confirmed by static diff, not a live `npm audit` re-run (no network/npm-registry access in this session).

---

### RESOLVED — npm `@babel/plugin-transform-modules-systemjs` 7.12.0–7.29.0 (arbitrary code on malicious input)

**Status**: Resolved by ordinary dependency drift (was: accepted risk, dev-only, as of 2026-05-17)
**Severity**: High (transitive **devDependency** — Babel transform, build-time only, never shipped to the browser)
**Date accepted**: 2026-05-17 (`npm audit fix` non-force did not resolve it; `--force` would have forced an unrelated breaking major in the build chain, declined per CRITICAL.md)
**Date resolved**: 2026-09-13 (observed during #432 re-triage; resolved by an intervening, unrelated dependency update — not a dedicated action)

**2026-09-13 re-verification (#432 triage)**: `origin/main`'s `frontend/package-lock.json` resolves this package to `7.29.8`, outside the vulnerable `7.12.0–7.29.0` range. No dedicated bump was needed; normal Babel-chain updates carried it past the fix threshold. Confirmed by static diff, not a live `npm audit` re-run.

---

### Monitoring

We actively monitor these advisories and will update dependencies as soon as fixes become available. You can check the current status:

```bash
cd backend && cargo audit
```

To verify these advisories are ignored in CI:

```bash
cat backend/.cargo/audit.toml
```

**Review cadence**: every accepted residual above is re-reviewed at least once per release cut (`release/vX.Y.Z`) and whenever `cargo audit` / `npm audit` output changes in CI — never left to go stale silently. See issue #432 (dependency vulnerability triage) and `docs/agent-activity/2026-09-13-security-officer.md` for the audit trail behind the current `Last-Reviewed` dates. `backend/.cargo/audit.toml` carries the matching `# Reviewed:` comment per entry as the CI-enforced source of truth.

## Reporting a Vulnerability

The KoproGo team takes security bugs seriously. We appreciate your efforts to responsibly disclose your findings, and will make every effort to acknowledge your contributions.

### How to Report a Security Vulnerability?

**Please do NOT report security vulnerabilities through public GitHub issues.**

Instead, please report security vulnerabilities by email to:

**abuse@koprogo.com**

You should receive a response within 48 hours. If for some reason you do not, please follow up via email to ensure we received your original message.

### What to Include in Your Report

Please include the following information in your report:

- Type of issue (e.g. buffer overflow, SQL injection, cross-site scripting, etc.)
- Full paths of source file(s) related to the manifestation of the issue
- The location of the affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the issue, including how an attacker might exploit the issue

This information will help us triage your report more quickly.

### Disclosure Policy

When the security team receives a security bug report, they will:

1. Confirm the problem and determine the affected versions
2. Audit code to find any potential similar problems
3. Prepare fixes for all releases still under maintenance
4. Release new security fix versions as soon as possible

### Comments on this Policy

If you have suggestions on how this process could be improved, please submit a pull request or email abuse@koprogo.com.

## Security Best Practices for Contributors

### Authentication & Authorization

- **Never commit credentials** to the repository (API keys, passwords, tokens, etc.)
- Use environment variables for sensitive configuration
- Implement proper JWT validation and refresh token rotation
- Follow the principle of least privilege for user roles

### Data Protection

- **GDPR Compliance**: All personal data must be handled according to GDPR
- Encrypt sensitive data at rest (passwords, personal information)
- Use HTTPS for all external communications
- Implement proper data retention and deletion policies

### Input Validation

- **Validate all inputs** at domain layer (entity constructors)
- Sanitize user inputs to prevent injection attacks
- Use prepared statements for all database queries (sqlx)
- Implement rate limiting on API endpoints

### Dependencies

- Regularly run `make audit` to check for vulnerable dependencies
- Keep all dependencies up-to-date
- Review security advisories for Rust crates and npm packages
- Use `cargo audit` in CI/CD pipeline

### Code Review

- All code changes must go through pull request review
- Security-sensitive changes require review by maintainers
- Run `make ci` before pushing (lint + test + audit)
- Use static analysis tools (clippy with `-D warnings`)

### Infrastructure Security

- Keep Docker images up-to-date
- Use minimal base images (Alpine, distroless)
- Run containers with least privileges (non-root user)
- Implement proper logging and monitoring

## Secure Development Workflow

1. **Before Development**
   - Review security requirements
   - Check for existing security patterns in codebase

2. **During Development**
   - Follow hexagonal architecture principles (isolation)
   - Validate inputs at domain boundaries
   - Write tests for security-critical code
   - Never disable security checks

3. **Before Commit**
   - Run `make pre-commit` (format + lint)
   - Check for hardcoded secrets (`git diff`)
   - Review changes for security implications

4. **Before Push**
   - Run `make ci` (lint + test + audit)
   - Ensure all tests pass
   - Review audit report

5. **Pull Request**
   - Describe security implications if any
   - Tag security-sensitive PRs
   - Wait for maintainer review

## Common Vulnerabilities to Avoid

### SQL Injection

❌ **Bad** (vulnerable):
```rust
let query = format!("SELECT * FROM users WHERE email = '{}'", email);
```

✅ **Good** (safe):
```rust
sqlx::query!("SELECT * FROM users WHERE email = $1", email)
```

### Authentication Bypass

❌ **Bad** (no validation):
```rust
pub fn mark_as_paid(&mut self) {
    self.status = PaymentStatus::Paid;
}
```

✅ **Good** (with validation):
```rust
pub fn mark_as_paid(&mut self, user: &User) -> Result<(), String> {
    if !user.can_manage_payments() {
        return Err("Unauthorized".to_string());
    }
    self.status = PaymentStatus::Paid;
    Ok(())
}
```

### Sensitive Data Exposure

❌ **Bad** (exposes password hash):
```rust
#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String, // ❌ Exposed!
}
```

✅ **Good** (no sensitive data):
```rust
#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    // password_hash excluded
}
```

### Path Traversal

❌ **Bad** (vulnerable):
```rust
let path = format!("uploads/{}", filename);
```

✅ **Good** (validated):
```rust
let filename = Path::new(&filename)
    .file_name()
    .ok_or("Invalid filename")?;
let path = PathBuf::from("uploads").join(filename);
```

## Security Testing

### Unit Tests

Test security-critical logic:

```rust
#[test]
fn test_payment_amount_must_be_positive() {
    let result = Payment::new(Uuid::new_v4(), Decimal::from(-100), ...);
    assert!(result.is_err());
}

#[test]
fn test_unauthorized_user_cannot_mark_paid() {
    let mut payment = create_test_payment();
    let unauthorized_user = create_test_user_without_permissions();

    let result = payment.mark_as_paid(&unauthorized_user);
    assert!(result.is_err());
}
```

### Integration Tests

Test authentication and authorization:

```rust
#[tokio::test]
async fn test_cannot_access_other_org_data() {
    // Create two organizations
    let org1 = create_test_org("org1").await;
    let org2 = create_test_org("org2").await;

    // User from org1 tries to access org2 data
    let org1_user_token = login_as_org_user(&org1).await;
    let response = get_buildings_with_token(&org1_user_token, org2.id).await;

    assert_eq!(response.status(), 403); // Forbidden
}
```

## Security Checklist

Use this checklist for security-sensitive features:

- [ ] Input validation implemented at domain layer
- [ ] Authorization checks implemented for all endpoints
- [ ] Sensitive data excluded from API responses
- [ ] SQL injection prevention (using sqlx prepared statements)
- [ ] XSS prevention (proper encoding in frontend)
- [ ] CSRF tokens implemented for state-changing operations
- [ ] Rate limiting implemented for public endpoints
- [ ] Logging doesn't include sensitive data
- [ ] Error messages don't leak system information
- [ ] Dependencies audited (`make audit` passes)
- [ ] Security tests written and passing
- [ ] Code reviewed by maintainer

## Security Contacts

- **Security Email**: abuse@koprogo.com
- **Project Maintainer**: gilles maury
- **Response Time**: Within 48 hours

## Hall of Fame

We would like to thank the following researchers for responsibly disclosing security vulnerabilities:

- [Your name could be here!]

---

**Last updated**: 2026-09-13 (#432 triage — full residual list synced with `backend/.cargo/audit.toml`, per-entry `Last-Reviewed` dates added, npm `devalue`/`svelte`/babel closures recorded)

For questions about this policy, contact abuse@koprogo.com.
