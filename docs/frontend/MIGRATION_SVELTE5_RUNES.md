# Svelte 5 Runes Migration Guide — KoproGo

## Strategy

KoproGo migrates from Svelte 5 legacy mode (`export let` + `$:`) to runes
(`$props()` + `$state()` + `$derived()` + `$effect()`) **incrementally**.

Svelte 5.55 auto-detects runes vs legacy per-component. No global config
needed. A component that uses `$props()` is compiled in runes mode; one
that uses `export let` stays in legacy mode. Both can coexist in the same
project, even as parent-child.

## Migration Pattern (leaf components)

Leaf components (no child Svelte components) are the simplest to migrate:

```svelte
<!-- BEFORE (legacy) -->
<script lang="ts">
  export let status: string;
  $: config = statusMap[status] || defaultConfig;
</script>

<!-- AFTER (runes) -->
<script lang="ts">
  let { status }: { status: string } = $props();
  let config = $derived(statusMap[status] || defaultConfig);
</script>
```

### Checklist per component

1. `export let X` → `let { X }: { X: Type } = $props()`
2. `$: Y = expr` → `let Y = $derived(expr)`
3. `$: { sideEffect(); }` → `$effect(() => { sideEffect(); })`
4. `let Z = initialValue` (mutable) → `let Z = $state(initialValue)`
5. Remove `createEventDispatcher` → use callback props
6. Run `npx vitest run <test-file>` → must pass
7. Commit

## Parent-Child Interop

When a **runes parent** uses a **legacy child** (e.g. `<Modal on:close={...}>`):
- **`on:` directive for component events DOES NOT WORK in runes mode**
- Workaround: pass callback props (`onClose`, `onCreated`) instead
- This requires the child to accept and call the callback prop
- Plan: migrate entire modules at once (all ticket components, then all
  meeting components) to minimize interop issues

When a **legacy parent** uses a **runes child**:
- Works seamlessly. The runes child accepts props normally.
- `bind:value` from legacy parent to runes child works.

## Known Issues

### vitest + `bind:value` on `<select>` (Svelte 5 DEV mode)

Svelte 5's DEV-mode runtime throws `$state rune outside svelte` when a
component using `bind:value` on a `<select>` element is mounted in vitest/jsdom.
This is because the compiled output references `$state` from the Svelte runtime's
index-client.js, and the DEV guard fires in non-.svelte files.

**Workaround**: `mode: "production"` in vitest.config.ts (partially helps) or
write spec-only tests that don't mount the component.

**Tracking**: https://github.com/sveltejs/svelte/issues/12865

## Progress

| Module | Components | Migrated | Tests |
|---|---|---|---|
| Ticket badges | 2 | 2 ✅ | 11 |
| Resolution badges | 1 | 1 ✅ | 3 |
| Poll badges | 1 | 1 ✅ | 4 |
| Ticket forms/modals | 3 | 0 (needs module-wide migration) | 17 |
| Meeting components | 7 | 0 | 5 |
| Total | 14 | 4 | 40 |
