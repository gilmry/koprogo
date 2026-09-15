import { test, expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { ADMIN_EMAIL, ADMIN_PASSWORD } from "./helpers/identifiants";
import { API_BASE } from "./helpers/adresses";
import { amorce, amorceToleree } from "./helpers/amorcage";
import { selectOptionByNameIfPresent } from "./helpers/name-match";
import { humanLogin, humanClick, waitForSpinner } from "./helpers/video-pace";

/**
 * L'audit d'accessibilité, sur des écrans authentifiés RÉELS (#865).
 *
 * ── Ce que ce fichier ajoute, et à quoi il ne touche pas ───────────────────
 *
 * `tests/e2e/Accessibility.spec.ts` compte dix tests, les dix sur `/login`.
 * `tests/e2e/mobile/accessibilite.spec.ts` a depuis comblé l'essentiel :
 * vingt destinations de barre d'onglets, quatre rôles, zéro violation — mais
 * avec une session PEINTE (`ouvreEnTantQue`, sans réseau ni base), parce
 * qu'au moment où il a été écrit les comptes de la démo ne fonctionnaient pas
 * (#870). Un banc sans réseau ne peut pas ouvrir une modale qui charge des
 * immeubles, ni un formulaire qui les soumet : il n'y a rien à soumettre.
 *
 * La pile de recette tourne depuis (2026-09-12, cf. RELEASE.md), donc les
 * deux familles qui manquaient encore à la DoD — **formulaire** et
 * **modale**, avec de vraies données — et le **tiroir mobile**, dont le
 * piège de focus n'avait jamais été exercé par un test, redeviennent
 * atteignables avec de vrais comptes.
 *
 * Ce fichier n'audite donc PAS à nouveau le tableau de bord et les listes
 * déjà couverts par `mobile/accessibilite.spec.ts` sur les vingt
 * destinations : un test de tableau de bord et un test de liste suffisent
 * ici à satisfaire la DoD (« au moins un écran de chaque famille ») avec de
 * vrais comptes, sans dupliquer un travail déjà fait.
 *
 * ── Le choix de l'écran formulaire/modale ───────────────────────────────────
 *
 * La création de ticket de KoproGo vit DANS une boîte de dialogue
 * (`Modal.svelte`, `role="dialog"`) : un seul écran sert donc les deux
 * familles « formulaire » et « modale » de la DoD. Ce n'est pas un raccourci
 * — c'est la structure réelle du produit, et prétendre le contraire aurait
 * demandé de fabriquer un second écran qui n'existe pas.
 *
 * ── Comptes de recette ───────────────────────────────────────────────────
 *
 * `francois@syndic-leroy.be` (syndic) et `alice@residence-parc.be`
 * (copropriétaire) sont les personas fixes du monde de scénario
 * (`backend/src/infrastructure/database/seed.rs`), déjà utilisés par
 * `scenarios/ticket-lifecycle.scenario.ts`. Le rôle audité correspond
 * toujours à l'écran audité — c'est exactement ce que le critère
 * `@security` de la story exige.
 */

const WCAG_AA_TAGS = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"];

function detailViolations(
  violations: {
    id: string;
    impact?: string | null;
    help: string;
    nodes: { html: string }[];
  }[],
): string {
  return violations
    .map(
      (v) =>
        `  ${v.id} (${v.impact}) × ${v.nodes.length}\n` +
        `    ${v.help}\n` +
        v.nodes
          .slice(0, 3)
          .map((n) => `      ${n.html.slice(0, 110)}`)
          .join("\n"),
    )
    .join("\n");
}

/** Sélectionne « Résidence du Parc » si le sélecteur d'immeuble est affiché. */
async function selectionnerImmeuble(page: Page): Promise<void> {
  await waitForSpinner(page);
  await selectOptionByNameIfPresent(
    page,
    page.getByTestId("building-selector"),
    "Résidence du Parc",
    "AccessibiliteEcransAuthentifies.spec.ts",
  );
  await waitForSpinner(page);
}

test.describe("Audit WCAG AA — écrans authentifiés, comptes de recette réels (#865)", () => {
  test.setTimeout(60_000);

  test.beforeAll(async ({ request }) => {
    const adminResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: ADMIN_EMAIL, password: ADMIN_PASSWORD },
    });
    const admin = await amorce(adminResp, "POST /auth/login (admin)");
    const seedResp = await request.post(`${API_BASE}/seed/scenario/world`, {
      headers: { Authorization: `Bearer ${admin.token}` },
    });
    // Tolérant : le monde de scénario peut déjà exister, semé par un autre
    // fichier de la même campagne (cf. scenarios/ticket-lifecycle.scenario.ts).
    await amorceToleree(seedResp, "POST /seed/scenario/world");
  });

  test.afterAll(async ({ request }) => {
    const adminResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: ADMIN_EMAIL, password: ADMIN_PASSWORD },
    });
    const admin = await amorce(adminResp, "POST /auth/login (admin, cleanup)");
    await request.delete(`${API_BASE}/seed/scenario/world`, {
      headers: { Authorization: `Bearer ${admin.token}` },
    });
  });

  test("@happy tableau de bord syndic — WCAG_AA_TAGS complet, aucune violation", async ({
    page,
  }) => {
    await humanLogin(page, "francois@syndic-leroy.be", "francois123");

    const resultat = await new AxeBuilder({ page })
      .withTags(WCAG_AA_TAGS)
      .analyze();

    expect(
      resultat.violations,
      `/syndic viole ${resultat.violations.length} règle(s) :\n${detailViolations(resultat.violations)}`,
    ).toEqual([]);
  });

  test("@happy liste des tickets (syndic) — WCAG_AA_TAGS complet, aucune violation", async ({
    page,
  }) => {
    await humanLogin(page, "francois@syndic-leroy.be", "francois123");
    await humanClick(page, "nav-link-tickets");
    await waitForSpinner(page);
    await selectionnerImmeuble(page);

    const resultat = await new AxeBuilder({ page })
      .withTags(WCAG_AA_TAGS)
      .analyze();

    expect(
      resultat.violations,
      `/tickets viole ${resultat.violations.length} règle(s) :\n${detailViolations(resultat.violations)}`,
    ).toEqual([]);
  });

  test("@happy formulaire et modale de création de ticket — WCAG_AA_TAGS complet, aucune violation", async ({
    page,
  }) => {
    await humanLogin(page, "francois@syndic-leroy.be", "francois123");
    await humanClick(page, "nav-link-tickets");
    await waitForSpinner(page);
    await selectionnerImmeuble(page);

    await humanClick(page, "tickets-create-btn");
    await expect(page.getByTestId("ticket-create-form")).toBeVisible({
      timeout: 10000,
    });

    const resultat = await new AxeBuilder({ page })
      .withTags(WCAG_AA_TAGS)
      .analyze();

    expect(
      resultat.violations,
      `La modale de création de ticket viole ${resultat.violations.length} règle(s) :\n${detailViolations(resultat.violations)}`,
    ).toEqual([]);
  });

  test("@edge tiroir mobile — piège de focus, ordre de tabulation, fond inerte", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await humanLogin(page, "francois@syndic-leroy.be", "francois123");

    await humanClick(page, "hamburger-button");
    const tiroir = page.getByTestId("mobile-drawer");
    await expect(tiroir).toBeVisible();

    // `inert` rend l'arrière-plan INEXISTANT pour un lecteur d'écran, pas
    // seulement hors d'atteinte au Tab (Navigation.svelte, #831).
    const fondInerte = await page.evaluate(
      () => document.getElementById("app-content")?.hasAttribute("inert") ?? false,
    );
    expect(
      fondInerte,
      "le fond (#app-content) doit porter `inert` tant que le tiroir est ouvert",
    ).toBe(true);

    // L'ouverture déplace le focus sur le bouton de fermeture du tiroir.
    await expect(page.getByTestId("nav-drawer-close-button")).toBeFocused();

    // Tabuler jusqu'au dernier élément focalisable du tiroir doit y RESTER —
    // pas s'échapper derrière l'overlay, dans une page qu'on ne voit pas
    // (c'est le finding #794 que `piegerLeFocus` corrige).
    const nbFocalisables = await tiroir.evaluate((el) =>
      Array.from(
        el.querySelectorAll(
          'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
        ),
      ).filter((n) => (n as HTMLElement).offsetParent !== null).length,
    );
    expect(
      nbFocalisables,
      "le tiroir doit contenir au moins un élément focalisable",
    ).toBeGreaterThan(0);

    for (let i = 0; i < nbFocalisables; i++) {
      await page.keyboard.press("Tab");
    }
    const resteDansLeTiroir = await page.evaluate(
      () => document.activeElement?.closest('[data-testid="mobile-drawer"]') !== null,
    );
    expect(
      resteDansLeTiroir,
      "après avoir tabulé jusqu'au dernier élément du tiroir, le focus doit y rester (piège actif)",
    ).toBe(true);

    // Échap referme le tiroir et rend le focus au bouton hamburger.
    await page.keyboard.press("Escape");
    await expect(tiroir).not.toBeVisible();
    await expect(page.getByTestId("hamburger-button")).toBeFocused();
  });

  test("@edge modale de création de ticket — piège de focus et restitution", async ({
    page,
  }) => {
    await humanLogin(page, "francois@syndic-leroy.be", "francois123");
    await humanClick(page, "nav-link-tickets");
    await waitForSpinner(page);
    await selectionnerImmeuble(page);

    const boutonOuverture = page.getByTestId("tickets-create-btn");
    await humanClick(page, "tickets-create-btn");
    const dialogue = page.getByRole("dialog");
    await expect(dialogue).toBeVisible();

    // Le premier élément focalisable du dialogue est son bouton de
    // fermeture (Modal.svelte : `FocusManager` + `trapFocus`).
    await expect(page.getByTestId("modal-close-button")).toBeFocused();

    // Shift+Tab depuis le premier doit boucler sur le DERNIER (le bouton de
    // soumission), pas s'échapper derrière le fond.
    await page.keyboard.press("Shift+Tab");
    await expect(page.getByTestId("ticket-submit-btn")).toBeFocused();

    // Tab depuis le dernier doit reboucler sur le premier.
    await page.keyboard.press("Tab");
    await expect(page.getByTestId("modal-close-button")).toBeFocused();

    // Échap referme la modale et restitue le focus au bouton qui l'a ouverte.
    await page.keyboard.press("Escape");
    await expect(dialogue).not.toBeVisible();
    await expect(boutonOuverture).toBeFocused();
  });

  test("@negative témoin de rougeur — une zone défilante sans clavier fait échouer l'audit", async ({
    page,
  }) => {
    // Le défaut réel qui a ouvert #865 : une table rendue défilante SANS
    // être atteignable au clavier (ni `tabindex="0"`, ni `role="region"`).
    // Si l'injection suivante ne fait PAS rougir la suite, elle ne regarde
    // rien — c'est le témoin que la story exige explicitement, pas une
    // vérification de principe.
    await humanLogin(page, "francois@syndic-leroy.be", "francois123");

    await page.evaluate(() => {
      const zone = document.createElement("div");
      zone.setAttribute("data-testid", "temoin-defilement-sans-clavier");
      zone.style.overflow = "auto";
      zone.style.width = "150px";
      zone.innerHTML =
        '<table style="width:900px"><tr><td>colonne hors champ, jamais atteignable sans souris</td></tr></table>';
      document.getElementById("app-content")?.appendChild(zone);
    });

    const resultat = await new AxeBuilder({ page })
      .withTags(WCAG_AA_TAGS)
      .analyze();
    const idsTrouves = resultat.violations.map((v) => v.id);

    expect(
      idsTrouves,
      `Une zone défilante sans tabindex a été injectée volontairement ; axe ` +
        `devait rapporter "scrollable-region-focusable" et ne l'a pas fait. ` +
        `Violations trouvées : ${idsTrouves.join(", ") || "(aucune)"} — ` +
        `l'audit ne voit pas ce défaut, donc son zéro ailleurs ne prouve rien.`,
    ).toContain("scrollable-region-focusable");
  });

  test("@security comptes de recette dédiés — la session copropriétaire n'atteint pas le tableau de bord syndic", async ({
    page,
  }) => {
    // La DoD de #865 est explicite : auditer un écran authentifié avec le
    // MAUVAIS rôle n'examinerait pas l'écran qu'on croit. Ce test vérifie
    // que la discipline « bon compte, bon rôle » suivie par les tests
    // ci-dessus n'est pas cosmétique : une session copropriétaire ne PEUT
    // PAS atteindre `/syndic` — `RouteGuard` la renvoie sur son propre
    // tableau de bord (`frontend/src/lib/guards.ts`, `roleGuards["/syndic"]
    // = [SYNDIC]`).
    await humanLogin(page, "alice@residence-parc.be", "alice123");

    await page.goto("/syndic");
    await page.waitForURL(/\/owner(\/|$|\?)/, { timeout: 15000 });

    expect(
      page.url(),
      "une session copropriétaire a atteint /syndic : un audit qui s'y " +
        "connecterait avec ce compte verrait la mauvaise interface, et son " +
        "résultat ne dirait rien de l'écran syndic visé",
    ).not.toContain("/syndic");
  });
});
