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
    // Couverture des tests unitaires de composants et de stores.
    // Ne mesure PAS ce que couvrent les specs Playwright : celles-ci
    // s'exécutent contre un serveur, hors de ce process.
    coverage: {
      provider: "v8",
      reporter: ["text-summary", "html", "lcov"],
      reportsDirectory: "../coverage/frontend",
      include: ["src/**/*.{ts,svelte}"],
      exclude: [
        "src/**/*.test.ts",
        "src/test-setup.ts",
        "src/types/**", // types générés depuis OpenAPI
        "src/env.d.ts",
      ],
      // Pas de `thresholds` tant que le plancher réel n'est pas mesuré :
      // un seuil posé au hasard bloque la CI ou ne sert à rien. À fixer
      // juste sous la valeur constatée, une fois la pyramide complète.
    },
  },
});
