import type { Page } from "@playwright/test";

/**
 * Fait échouer le test sur toute erreur runtime JS/Svelte non catchée
 * (ex: crash Svelte 5 "Cannot do bind:value={undefined} when value has
 * a fallback value" — un composant qui plante au montage ne laisse aucune
 * trace dans le DOM ni dans les logs réseau, seulement dans `pageerror`).
 */
export function failOnPageErrors(page: Page) {
  page.on("pageerror", (err) => {
    throw new Error(`Erreur runtime JS/Svelte non catchée : ${err.message}`);
  });
}
