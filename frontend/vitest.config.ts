import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  // Production mode disables Svelte 5 DEV-only runtime checks like
  // "$state rune outside svelte" that fire when bind:value compiled
  // output is accessed from .test.ts files.
  mode: "production",
  // `mode: "production"` above already disables HMR; @sveltejs/vite-plugin-svelte
  // 7.x dropped the standalone `hot` option this used to be spelled out with.
  plugins: [svelte()],
  resolve: {
    conditions: ["browser"],
  },
  test: {
    globals: true,
    environment: "jsdom",
    include: ["src/**/*.test.ts"],
    setupFiles: ["src/test-setup.ts"],
    alias: {
      $lib: "/src/lib",
    },
  },
});
