import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte({ hot: false })],
  resolve: {
    // Force browser exports so Svelte 5 uses mount() instead of SSR
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
