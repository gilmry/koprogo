import { defineConfig } from "astro/config";
import svelte from "@astrojs/svelte";
import tailwindcss from "@tailwindcss/vite";
import node from "@astrojs/node";
import AstroPWA from "@vite-pwa/astro";

export default defineConfig({
  output: "server",
  adapter: node({
    mode: "standalone",
  }),
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
        theme_color: "#0F766E",
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
