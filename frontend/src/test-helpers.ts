// Test helpers — contourne le mismatch de types entre Svelte 5 (Component<P, E>
// avec brand `ComponentInternals`) et @testing-library/svelte 5.3.1 (ComponentImport<C>).
//
// Pattern à utiliser dans les *.test.ts au lieu de `import { render } from '@testing-library/svelte'`.
// Re-export complet du module + override `render` accepting any component.

import {
  render as svelteRender,
  type SvelteComponentOptions,
  type RenderResult,
} from "@testing-library/svelte";

// Re-export tout ce qui est typé correctement
export {
  screen,
  fireEvent,
  cleanup,
  act,
  waitFor,
  waitForElementToBeRemoved,
  within,
  findByRole,
  findByText,
  getByRole,
  getByText,
  queryByRole,
  queryByText,
} from "@testing-library/svelte";

/**
 * Typed wrapper around @testing-library/svelte's `render`.
 *
 * Bypasses the `ComponentImport<C>` strictness mismatch between Svelte 5
 * compiled components (`Component<P, E>` with `Brand<"ComponentInternals">`)
 * and the testing-library expected `ComponentType<C>` shape.
 *
 * Functionally identical to `render` from @testing-library/svelte —
 * Svelte 5 components ARE compatible at runtime, only the static type
 * inference fails.
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function render<C extends (...args: any[]) => any>(
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  Component: C | { default: C } | any,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  options?: SvelteComponentOptions<any> | any,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
): RenderResult<any> {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return svelteRender(Component as any, options) as RenderResult<any>;
}
