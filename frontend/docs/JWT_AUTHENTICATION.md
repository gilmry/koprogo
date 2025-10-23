# Frontend JWT Authentication Guide

**Date**: 2025-10-23
**Issue**: #020 - Multi-Tenancy Security (Frontend)

## Overview

The frontend now uses JWT-only authentication for all API requests. The `organization_id` is managed securely through JWT tokens and cannot be specified in request bodies.

## Key Changes

### ✅ What's Already Configured

1. **API Client (`src/lib/api.ts`)**
   - Automatically includes JWT token in `Authorization: Bearer <token>` header
   - Token stored in `localStorage` as `auth_token`
   - All requests automatically authenticated

2. **Login Flow (`src/components/LoginForm.svelte`)**
   - Receives JWT token from `/api/v1/auth/login` endpoint
   - Stores token in localStorage
   - Extracts `organization_id` from user response (read-only)

3. **Auth Store**
   - Manages user session
   - Stores organization_id from JWT claims
   - Automatically refreshes tokens

### ❌ What NOT to Do

**DO NOT** send `organization_id` in request bodies for create operations:

```typescript
// ❌ BAD - Don't do this anymore!
await api.post('/buildings', {
  organization_id: user.organizationId, // REMOVED!
  name: 'My Building',
  address: '123 Main St',
  // ...
});

// ✅ GOOD - Organization comes from JWT automatically
await api.post('/buildings', {
  name: 'My Building',
  address: '123 Main St',
  // ... (no organization_id needed)
});
```

### Updated API Endpoints

All create endpoints now extract `organization_id` from JWT token:

| Endpoint | Method | Organization Source |
|----------|--------|-------------------|
| `/buildings` | POST | JWT token only |
| `/units` | POST | JWT token only |
| `/owners` | POST | JWT token only |
| `/expenses` | POST | JWT token only |
| `/meetings` | POST | JWT token only |
| `/documents` | POST | JWT token only (not in multipart form) |

## Frontend Implementation Example

### Creating a Building

```typescript
// src/components/BuildingForm.svelte
<script lang="ts">
  import { api } from '../lib/api';

  async function createBuilding(formData: any) {
    try {
      // No need to add organization_id - it comes from JWT!
      const building = await api.post('/buildings', {
        name: formData.name,
        address: formData.address,
        city: formData.city,
        postal_code: formData.postalCode,
        country: formData.country,
        total_units: formData.totalUnits,
        construction_year: formData.constructionYear,
      });

      console.log('Building created:', building);
    } catch (error) {
      console.error('Error creating building:', error);
    }
  }
</script>
```

### Document Upload (Multipart)

```typescript
// Uploading a document (multipart form)
async function uploadDocument(file: File, buildingId: string, title: string) {
  const formData = new FormData();

  // ❌ DON'T include organization_id anymore
  // formData.append('organization_id', user.organizationId);

  // ✅ Only include these fields
  formData.append('file', file);
  formData.append('building_id', buildingId);
  formData.append('document_type', 'invoice');
  formData.append('title', title);
  formData.append('uploaded_by', user.id);

  const response = await fetch(`${API_BASE_URL}/documents`, {
    method: 'POST',
    headers: {
      // JWT token is added automatically by getHeaders()
      'Authorization': `Bearer ${localStorage.getItem('auth_token')}`,
    },
    body: formData,
  });

  return response.json();
}
```

## Security Benefits

### Server-Side Validation
- Backend validates JWT signature
- Extracts `organization_id` from cryptographic claims
- Ignores any `organization_id` sent in request body

### Attack Prevention
```typescript
// Even if a malicious user tries this:
await api.post('/buildings', {
  organization_id: 'other-org-uuid', // ← This is IGNORED!
  name: 'Evil Building',
});

// The backend will:
// 1. Validate JWT token
// 2. Extract organization_id from token claims
// 3. Override request body with JWT value
// 4. Create building in user's actual organization only
```

## Testing

### Manual Testing with Browser DevTools

1. **Login**
   ```javascript
   // Check token is stored
   localStorage.getItem('auth_token')
   ```

2. **Check Request Headers**
   - Open Network tab in DevTools
   - Make a POST request
   - Verify `Authorization: Bearer <token>` header is present

3. **Attempt to Forge organization_id**
   ```javascript
   // Try to create building in different org (should fail)
   await api.post('/buildings', {
     organization_id: 'fake-uuid', // Will be ignored
     name: 'Test Building',
   });
   // Backend uses JWT organization_id, not the fake one!
   ```

### Automated Testing

```typescript
// tests/security.spec.ts
import { test, expect } from '@playwright/test';

test('Cannot create building in another organization', async ({ page }) => {
  // Login as user A
  await page.goto('/login');
  await page.fill('input[type="email"]', 'userA@example.com');
  await page.fill('input[type="password"]', 'password123');
  await page.click('button[type="submit"]');

  // Try to create building with different org_id in payload
  const response = await page.request.post('/api/v1/buildings', {
    headers: {
      'Authorization': `Bearer ${await page.evaluate(() => localStorage.getItem('auth_token'))}`,
    },
    data: {
      organization_id: 'other-org-uuid', // Attempt to forge
      name: 'Evil Building',
      address: '123 Hack St',
      city: 'Brussels',
      postal_code: '1000',
      country: 'Belgium',
      total_units: 10,
    },
  });

  expect(response.ok()).toBeTruthy();

  // Verify building was created in user A's org, not the forged one
  const building = await response.json();
  const userOrg = await page.evaluate(() =>
    JSON.parse(localStorage.getItem('user')).organizationId
  );

  expect(building.organization_id).toBe(userOrg);
  expect(building.organization_id).not.toBe('other-org-uuid');
});
```

## Migration Checklist

If you have existing frontend code that sends `organization_id`:

- [ ] Remove `organization_id` from all POST request bodies
- [ ] Remove `organization_id` from multipart form uploads
- [ ] Ensure JWT token is always sent in Authorization header
- [ ] Update TypeScript interfaces to remove organization_id from create DTOs
- [ ] Test that all create operations still work
- [ ] Verify organization isolation in browser DevTools

## Troubleshooting

### Error: "Missing authorization header"
**Solution**: Ensure token is stored in localStorage and api.ts is being used

### Error: "User does not belong to an organization"
**Solution**: User needs to be assigned to an organization during registration

### Error: 401 Unauthorized
**Solutions**:
- Check token hasn't expired (15 minute lifetime)
- Use refresh token to get new access token
- Re-login if refresh token also expired

### Organization mismatch in responses
**Not possible anymore!** Backend enforces organization from JWT token only.

## References

- Backend documentation: `backend/docs/JWT_SECURITY.md`
- Auth middleware: `backend/src/infrastructure/web/middleware.rs`
- API client: `frontend/src/lib/api.ts`
- Auth store: `frontend/src/stores/auth.ts`

---

**Status**: ✅ Frontend already JWT-ready
**Backend Status**: ✅ All endpoints secured
**Security Level**: 🔒 Cryptographically enforced multi-tenancy
