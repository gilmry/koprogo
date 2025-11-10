# ADR 0005: JWT for Authentication

- **Status**: Accepted
- **Date**: 2025-02-10
- **Track**: Software

## Context

KoproGo needs stateless authentication supporting:
1. Multi-role users (Owner, Syndic, BoardMember, Accountant)
2. Role switching without re-login
3. API access for mobile apps (future)
4. Refresh tokens for long sessions

## Decision

We use **JSON Web Tokens (JWT)** with:
- **Access token**: Short-lived (1h), contains user ID + active role
- **Refresh token**: Long-lived (30d), stored in database for revocation
- **HS256 algorithm**: Symmetric signing (sufficient for single-server MVP)
- **Bearer authentication**: `Authorization: Bearer <token>` header

**Token structure**:
```json
{
  "sub": "user-uuid",
  "email": "syndic@example.com",
  "active_role_id": "role-uuid",
  "exp": 1234567890
}
```

## Consequences

**Positive**:
- ✅ **Stateless**: No server-side session storage (scales horizontally)
- ✅ **Multi-role support**: Token carries active role, switch via `/auth/switch-role`
- ✅ **API-friendly**: Standard Bearer auth for mobile/third-party clients
- ✅ **Revocable**: Refresh tokens stored in DB, can be revoked

**Negative**:
- ⚠️ **Cannot revoke access tokens**: Must wait for expiration (1h max)
- ⚠️ **Token size**: Larger than session IDs (~200 bytes vs 16 bytes)

**Security measures**:
- bcrypt password hashing (cost 12)
- HTTPS only in production
- Refresh token rotation on use
- `exp` claim prevents replay attacks

## Alternatives Considered

1. **Session cookies**:
   - ✅ Smaller, revocable immediately
   - ❌ Requires server-side storage (Redis, PostgreSQL)
   - ❌ Less API-friendly for mobile
   - **Verdict**: Rejected for statelessness

2. **OAuth2 (third-party)**:
   - ✅ Delegate auth to Google/Microsoft
   - ❌ Vendor lock-in, privacy concerns
   - **Verdict**: Future consideration, not MVP

## Implementation

**Library**: `jsonwebtoken` crate
**Secret**: `JWT_SECRET` in `.env` (64+ random chars)

**Endpoints**:
- `POST /auth/login` - Returns access + refresh tokens
- `POST /auth/refresh` - Exchange refresh token for new access token
- `POST /auth/switch-role` - Change active role (issues new access token)
- `POST /auth/logout` - Revoke refresh token

## Next Steps

- ✅ Implement JWT middleware (**Done**)
- ⏳ Switch to RS256 (asymmetric) for multi-server Phase 2
- ⏳ Add rate limiting on `/auth/login` to prevent brute force
- ⏳ Consider adding 2FA for board members (high-value approvals)

## References

- JWT spec: https://datatracker.ietf.org/doc/html/rfc7519
- jsonwebtoken crate: https://github.com/Keats/jsonwebtoken
