# Internationalization (i18n) Guide

Version: 1.0.0 | Languages: FR, NL, EN

## Frontend i18n

### Configuration

```typescript
// frontend/src/lib/i18n.ts
const translations = {
  fr: {
    'auth.login': 'Connexion',
    'buildings.list': 'Liste des immeubles',
    'units.owners': 'Copropriétaires'
  },
  nl: {
    'auth.login': 'Inloggen',
    'buildings.list': 'Lijst van gebouwen',
    'units.owners': 'Mede-eigenaars'
  },
  en: {
    'auth.login': 'Login',
    'buildings.list': 'Buildings list',
    'units.owners': 'Co-owners'
  }
};
```

### Usage in Svelte

```svelte
<script>
  import { t } from '$lib/i18n';
</script>

<button>{$t('auth.login')}</button>
```

## Backend i18n

### API Error Messages

```rust
// Support multiple languages via Accept-Language header
let error_msg = match lang.as_str() {
    "fr" => "Bâtiment non trouvé",
    "nl" => "Gebouw niet gevonden",
    _ => "Building not found"
};
```

## Adding New Translations

1. Add key to `translations` object
2. Translate to FR, NL, EN
3. Use `t('new.key')` in components
4. Test with language selector

## Translation Files

- `frontend/src/lib/translations/fr.json`
- `frontend/src/lib/translations/nl.json`
- `frontend/src/lib/translations/en.json`

---

**Version**: 1.0.0
