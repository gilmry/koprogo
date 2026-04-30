import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  // Production mode disables Svelte 5 DEV-only runtime checks like
  // "$state rune outside svelte" that fire when bind:value compiled
  // output is accessed from .test.ts files.
  mode: "production",
  plugins: [
    svelte({
      hot: false,
    }),
  ],
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
