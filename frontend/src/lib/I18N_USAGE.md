# i18n Usage Guide

## Overview

KoproGo supports 4 languages following Belgian requirements:

1. **Nederlands (NL)** - 60% Belgium - Priority #1 🇳🇱
2. **Français (FR)** - 40% Belgium - Priority #2 🇫🇷
3. **Deutsch (DE)** - <1% Belgium - Legally required 🇩🇪
4. **English (EN)** - International competitiveness 🇬🇧

Default language: **Dutch (NL)**

## Setup

### 1. Initialize i18n in your layout

Add to your main layout (e.g., `src/layouts/MainLayout.astro`):

```astro
---
import '../lib/i18n';
---
```

### 2. Use translations in Svelte components

```svelte
<script>
  import { _ } from 'svelte-i18n';
</script>

<h1>{$_('buildings.title')}</h1>
<button>{$_('common.create')}</button>
<p>{$_('navigation.dashboard')}</p>
```

### 3. Use translations with parameters

```svelte
<script>
  import { _ } from 'svelte-i18n';
</script>

<p>{$_('validation.minLength', { values: { count: 5 } })}</p>
<!-- Outputs: "Minimum 5 characters required" (in selected language) -->
```

### 4. Add Language Selector to Navigation

```svelte
<script>
  import LanguageSelector from '../components/LanguageSelector.svelte';
</script>

<nav>
  <!-- Your navigation items -->
  <LanguageSelector />
</nav>
```

## Translation Keys Structure

All translations are organized in `/src/locales/{lang}.json`:

- `common.*` - Common actions (save, cancel, edit, delete, etc.)
- `navigation.*` - Navigation menu items
- `buildings.*` - Building-related translations
- `units.*` - Unit-related translations
- `owners.*` - Owner-related translations
- `expenses.*` - Expense-related translations
- `meetings.*` - Meeting-related translations
- `documents.*` - Document-related translations
- `reports.*` - Report-related translations
- `validation.*` - Validation messages

## Adding New Translations

1. Add the key to all 4 locale files (`nl.json`, `fr.json`, `de.json`, `en.json`)
2. Use the key in your component with `$_('your.new.key')`

Example:

```json
// In nl.json
{
  "myFeature": {
    "title": "Mijn Functie",
    "description": "Dit is een nieuwe functie"
  }
}

// In fr.json
{
  "myFeature": {
    "title": "Ma Fonctionnalité",
    "description": "Ceci est une nouvelle fonctionnalité"
  }
}
```

## Language Persistence

User language preference is automatically saved to `localStorage` and restored on next visit.

## Programmatic Language Change

```svelte
<script>
  import { locale } from 'svelte-i18n';

  function switchToFrench() {
    locale.set('fr');
    localStorage.setItem('preferred-language', 'fr');
  }
</script>
```

## Formatting Dates and Numbers

```svelte
<script>
  import { date, number } from 'svelte-i18n';
</script>

<p>{$date(new Date(), { format: 'short' })}</p>
<p>{$number(1234.56, { style: 'currency', currency: 'EUR' })}</p>
```

## Best Practices

1. **Always use translation keys** - Never hardcode text in components
2. **Keep keys organized** - Use the existing structure (common, navigation, etc.)
3. **Translate everything** - Include error messages, placeholders, tooltips
4. **Test all languages** - Verify translations render correctly in all 4 languages
5. **Use placeholders** - For dynamic content use `{variable}` in translation strings

## Example: Complete Form

```svelte
<script>
  import { _ } from 'svelte-i18n';
  import LanguageSelector from '../components/LanguageSelector.svelte';
</script>

<div class="p-4">
  <!-- Language Selector in Header -->
  <div class="flex justify-end mb-4">
    <LanguageSelector />
  </div>

  <!-- Form -->
  <form>
    <h2>{$_('buildings.create')}</h2>

    <label>
      {$_('buildings.name')}
      <input type="text" placeholder={$_('buildings.name')} required />
    </label>

    <label>
      {$_('buildings.address')}
      <input type="text" placeholder={$_('buildings.address')} required />
    </label>

    <label>
      {$_('buildings.city')}
      <input type="text" placeholder={$_('buildings.city')} required />
    </label>

    <button type="submit">{$_('common.save')}</button>
    <button type="button">{$_('common.cancel')}</button>
  </form>
</div>
```

## API Integration

When making API calls, you can include the `Accept-Language` header:

```typescript
import { locale } from "svelte-i18n";
import { get } from "svelte/store";

async function fetchData() {
  const currentLocale = get(locale);

  const response = await fetch("/api/v1/buildings", {
    headers: {
      "Accept-Language": currentLocale || "nl",
      "Content-Type": "application/json",
    },
  });

  return response.json();
}
```

The backend will use this to return error messages in the correct language.

## PCN Reports - Multilingual

PCN reports now include labels in all 4 languages:

- Account codes remain the same (610, 611, 612, etc.)
- Labels are provided in NL, FR, DE, EN
- Excel exports show all 4 language columns
- JSON API responses include all translations

Example JSON response:

```json
{
  "account_code": "611",
  "account_label_nl": "Onderhoud en kleine herstellingen",
  "account_label_fr": "Entretien et petites réparations",
  "account_label_de": "Wartung und kleine Reparaturen",
  "account_label_en": "Maintenance and minor repairs",
  "total_amount": 1500.0,
  "entry_count": 5
}
```
