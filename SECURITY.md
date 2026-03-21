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

### RUSTSEC-2026-0049: rustls-webpki 0.101.7 - Certificate Revocation Enforcement

**Status**: Accepted Risk (Upstream Blocked)
**Severity**: Medium
**Date**: 2026-03-21
**Affected**: `rustls-webpki 0.101.7` via `rustls@0.21` → `hyper-rustls@0.24` → `aws-smithy-http-client@1.1.12` → `aws-config@1.8.15`

**Assessment**:
- The fixed version (`rustls-webpki >= 0.103.10`) is already present in the dependency tree via `rustls@0.22`
- The vulnerable `0.101.7` version is pulled in exclusively by the AWS SDK (`aws-config`, `aws-sdk-s3`) which has not yet migrated from `hyper-rustls@0.24` / `rustls@0.21` to the newer TLS stack
- `aws-config@1.8.15` is the latest available version — no upstream fix exists yet
- **Impact**: Limited to AWS S3 backup upload paths only (not user-facing API)

**Why Accepted**:
- Blocked by upstream AWS SDK Rust — cannot be resolved without an AWS SDK release
- The attack surface is restricted to S3 backup operations, not the main API
- Monitored via `cargo audit` in CI

**Mitigation**:
- Will be resolved automatically once AWS SDK migrates to `hyper-rustls@0.26` / `rustls@0.22`
- Track upstream: https://github.com/awslabs/aws-sdk-rust

---

### CVE-2026-32766: astral-tokio-tar 0.5.6 - Insufficient PAX Extension Validation

**Status**: Accepted Risk (Dev-Only, Mitigated)
**Severity**: Low
**Date**: 2026-03-21
**Affected**: `astral-tokio-tar 0.5.6` via `testcontainers@0.27.1` (dev-dependency only)

**Assessment**:
- `testcontainers@0.27.1` is the latest available version and still depends on `astral-tokio-tar@0.5.6`
- This crate is used **exclusively in test builds** — it is never compiled into production binaries
- The fixed version (`astral-tokio-tar >= 0.6.0`) is explicitly declared as a direct dependency in `Cargo.toml` to ensure 0.6.0 is present alongside 0.5.6
- **Impact**: None in production; test environment only, running in isolated Docker containers

**Why Accepted**:
- No fixed upgrade available for testcontainers
- Zero production exposure (dev-dependency, `#[cfg(test)]`)
- Mitigated by explicit `astral-tokio-tar = "0.6"` override

**Mitigation**:
- Will be resolved once `testcontainers` updates its dependency
- Track upstream: https://github.com/testcontainers/testcontainers-rs

---

### GitHub Dependabot Alerts — h3 (npm) — SSE/Path Traversal

**Status**: Accepted Risk (Transitive, No Production Exposure)
**Severity**: Medium/High
**Date**: 2026-03-21
**Affected**: `h3` (transitive npm dependency in frontend build tooling)

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

### Monitoring

We actively monitor these advisories and will update dependencies as soon as fixes become available. You can check the current status:

```bash
cd backend && cargo audit
```

To verify these advisories are ignored in CI:

```bash
cat backend/.cargo/audit.toml
```

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

**Last updated**: 2026-03-21

For questions about this policy, contact abuse@koprogo.com.
