import { defineConfig } from "astro/config";
import svelte from "@astrojs/svelte";
import tailwindcss from "@tailwindcss/vite";
import AstroPWA from "@vite-pwa/astro";

export default defineConfig({
  output: "static",

  /**
   * La barre d'outils de développement est retirée dès qu'on pilote le
   * navigateur.
   *
   * Elle injecte ses propres `<h1>` dans la page — « Audit », « No
   * accessibility or performance issues detected », « Settings ». Le 2026-09-09,
   * `notice-board` a échoué dessus : `locator("h1")` résolvait QUATRE éléments,
   * dont trois de la barre, et Playwright refuse en mode strict. Le titre de
   * l'annonce était pourtant le bon.
   *
   * Elle est surtout VISIBLE : les scénarios `scenarios` sont enregistrés en
   * vidéo comme documentation vivante, et la barre apparaît au bas de chaque
   * image. Filmer le produit pour des copropriétaires en y laissant un outil
   * de développeur, c'est montrer autre chose que le produit.
   *
   * Conditionnée à `PLAYWRIGHT`, pour ne rien retirer au développement
   * quotidien.
   */
  devToolbar: { enabled: !process.env.PLAYWRIGHT },
  integrations: [
    svelte(),
    AstroPWA({
      mode: "development",
      base: "/",
      scope: "/",
      includeAssets: ["favicon.svg"],
      registerType: "autoUpdate",
      manifest: {
        name: "KoproGo - Gestion de Copropriété",
        short_name: "KoproGo",
        description:
          "Application de gestion de copropriété avec synchronisation offline",
        theme_color: "#15803d",
        background_color: "#ffffff",
        display: "standalone",
        orientation: "portrait",
        icons: [
          {
            src: "/pwa-192x192.png",
            sizes: "192x192",
            type: "image/png",
            purpose: "any maskable",
          },
          {
            src: "/pwa-512x512.png",
            sizes: "512x512",
            type: "image/png",
            purpose: "any maskable",
          },
        ],
      },
      workbox: {
        navigateFallback: "/",
        globPatterns: ["**/*.{css,js,html,svg,png,ico,txt}"],
        runtimeCaching: [
          {
            urlPattern: /^http:\/\/127\.0\.0\.1:8080\/api\/.*/i,
            handler: "NetworkFirst",
            options: {
              cacheName: "api-cache",
              expiration: {
                maxEntries: 100,
                maxAgeSeconds: 60 * 60 * 24, // 24 hours
              },
              cacheableResponse: {
                statuses: [0, 200],
              },
            },
          },
        ],
      },
      devOptions: {
        enabled: true,
        navigateFallbackAllowlist: [/^\//],
        suppressWarnings: true,
      },
    }),
  ],
  vite: {
    plugins: [tailwindcss()],
  },
  server: {
    port: 3000,
    host: true,
  },
});
