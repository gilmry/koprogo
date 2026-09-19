# Frontend Components Documentation

Version: 1.0.0 | Astro + Svelte

## Architecture

- **Framework**: Astro (SSG + Islands)
- **UI Components**: Svelte 4
- **Styling**: Tailwind CSS 3
- **State**: Svelte stores
- **i18n**: Custom solution (FR, NL, EN)

## Key Components

### Authentication

**LoginForm.svelte**
- Location: `frontend/src/components/auth/LoginForm.svelte`
- Props: None
- Emits: `login` event with credentials
- Uses: `authStore` for JWT management

### Buildings

**BuildingList.svelte**
- Location: `frontend/src/components/buildings/BuildingList.svelte`
- Props: `buildings: Building[]`
- Features: Search, filter, pagination

**BuildingCreateModal.svelte**
- Location: `frontend/src/components/buildings/BuildingCreateModal.svelte`
- Props: `open: boolean`
- Emits: `create`, `close`

### Units

**UnitOwners.svelte**
- Location: `frontend/src/components/UnitOwners.svelte`
- Props: `unitId: string`
- Features: Multi-owner management, quote-part validation

## Stores

### authStore

```typescript
// frontend/src/lib/stores/authStore.ts
export const authStore = writable<AuthState>({
  isAuthenticated: false,
  user: null,
  token: null,
  activeRole: null
});
```

## Styling Conventions

```svelte
<button class="btn btn-primary">
  <!-- Tailwind utilities -->
</button>
```

## Storybook Setup (Planned)

```bash
npm install --save-dev @storybook/svelte
npx storybook init
```

---

**Version**: 1.0.0
