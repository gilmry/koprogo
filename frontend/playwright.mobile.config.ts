import { defineConfig, devices } from "@playwright/test";

/**
 * Le banc mobile : un navigateur à une largeur de téléphone, sans démo.
 *
 * ── Pourquoi une configuration à part ──────────────────────────────────────
 *
 * `playwright.config.ts` fixe `viewport: 1280×720` sur ses quatre projets et
 * garde `Mobile Chrome` en commentaire. Tout le Track U — barre d'onglets,
 * écran « Plus », tableaux défilants, zones sûres — n'était donc vérifié que
 * par Vitest, c'est-à-dire dans jsdom : **sans mise en page, sans défilement,
 * sans encoche**. jsdom ne peut pas voir une barre d'onglets qui recouvre le
 * dernier élément d'une liste, ni une cible de 36 px. C'est #869.
 *
 * ── Pourquoi le banc ne parle pas à la démo ────────────────────────────────
 *
 * La suite existante vise `koprogo.com` à travers Traefik, ce qui suppose des
 * identifiants, traverse une limite de débit de 5 connexions/minute et un
 * bouncer CrowdSec. Elle est aujourd'hui hors service pour une autre raison :
 * le repli `admin123` ne vaut plus rien sur la démo (#870).
 *
 * Ce banc-ci sert le build statique et **simule tout `/api/v1/`**. Il vérifie
 * la COQUILLE — ce qui se place, ce qui se recouvre, ce qui se mesure — et pas
 * les données. Il n'a donc besoin ni de compte, ni de réseau, ni de la démo :
 * il tourne sur une machine nue, y compris en CI.
 */
export default defineConfig({
  testDir: "./tests/e2e/mobile",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  reporter: [["list"]],

  use: {
    // Pixel 5 : 393 × 851, DPR 2,75. La largeur de téléphone la plus commune,
    // et celle sur laquelle les six tableaux de #866 débordaient.
    ...devices["Pixel 5"],
    baseURL: "http://127.0.0.1:4321",
    trace: "retain-on-failure",

    /**
     * Le service worker de la PWA doit être écarté, sinon rien n'est simulé.
     *
     * `page.route` n'intercepte PAS les requêtes émises depuis un service
     * worker : elles partent réellement, échouent en `net::ERR_FAILED`, et le
     * rafraîchissement silencieux renvoie vers `/login`. Le banc mesurait donc
     * l'écran de connexion en croyant mesurer un tableau de bord — une
     * interception qui ne s'applique pas, exactement ce que
     * `garde-interception-reseau` décrit.
     *
     * Le service worker a sa propre couverture ; il n'a rien à faire dans un
     * test de mise en page.
     */
    serviceWorkers: "block",
  },

  projects: [{ name: "mobile" }],

  /**
   * `astro preview` sert `dist/`, donc le banc éprouve **le build**, pas le
   * serveur de développement. C'est ce qui compte : les classes Tailwind
   * assemblées à l'exécution, les jetons `@theme` et les valeurs arbitraires
   * n'existent qu'après passage du scanner. Deux défauts du Track U étaient
   * exactement de cette nature.
   */
  webServer: {
    // `astro preview` se met en arrière-plan tout seul : son processus de
    // premier plan rend la main aussitôt, et Playwright n'y voit qu'un
    // serveur « sorti trop tôt ». Un serveur de fichiers ordinaire reste au
    // premier plan, ne dépend d'aucun paquet, et sert exactement ce que la
    // production sert — `dist/`, tel quel.
    command:
      "python3 -m http.server 4321 --bind 127.0.0.1 --directory dist 2>/dev/null",
    url: "http://127.0.0.1:4321/login/",
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
