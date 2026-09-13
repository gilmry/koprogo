# Testing Svelte 5 Forms via MCP Automation

## Context

KoproGo runs **Svelte 5.55** in **legacy mode** (`export let` + `$:` reactive
statements). There is no `svelte.config.js` with `runes: true` — all 178
components use classic reactivity.

## The Problem

When using MCP browser automation tools (e.g. Claude Cowork Chrome extension),
programmatic input setters like:

```js
// This does NOT trigger Svelte's bind:value
inputElement.value = "new text";
```

...do **not** trigger Svelte's `bind:value` two-way binding. The DOM input
shows the new value, but the Svelte component state stays empty. Consequently:

- Form validation reads empty fields and blocks submission silently
- The `handleSubmit()` function sends empty/stale values to the backend
- The audit falsely reports "POST 400 — missing field building_id" etc.

## The Fix: Dispatch a Native InputEvent

After setting the value programmatically, dispatch an `InputEvent` to trigger
Svelte's event listener:

```js
// Step 1: Use the native HTMLInputElement setter (bypasses any framework proxy)
const nativeSetter = Object.getOwnPropertyDescriptor(
  window.HTMLInputElement.prototype,
  "value"
).set;
nativeSetter.call(inputElement, "new text");

// Step 2: Dispatch an InputEvent so Svelte picks up the change
inputElement.dispatchEvent(new InputEvent("input", { bubbles: true }));
```

## For Select Elements

```js
const nativeSetter = Object.getOwnPropertyDescriptor(
  window.HTMLSelectElement.prototype,
  "value"
).set;
nativeSetter.call(selectElement, "option_value");
selectElement.dispatchEvent(new Event("change", { bubbles: true }));
```

## Preferred: Use Playwright's Native Methods

Playwright's `page.fill()` and `page.selectOption()` already dispatch the
correct events internally. Use them instead of raw DOM manipulation:

```ts
// Playwright — correct, triggers bind:value
await page.fill('[data-testid="ticket-title-input"]', "Fuite robinet");
await page.selectOption('[data-testid="ticket-priority-select"]', "High");
```

## Why This Matters

Audits v1 through v7 (April 2026) repeatedly flagged forms as "broken"
because the MCP automation tool used `form_input` which does NOT dispatch
InputEvent. The forms work perfectly for real human users typing on a
keyboard. See `docs/cowork/Plan-Maury-2026-04-13.md` STORY-P7-501.

## Future: Runes Migration

When KoproGo migrates to Svelte 5 runes (`$state`, `$props`, `$derived`),
this workaround will no longer be necessary — runes use fine-grained
reactivity that does not depend on DOM events for state synchronization.
See Epic P7-6 in the Maury plan.
