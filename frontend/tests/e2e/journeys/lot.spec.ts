/**
 * Harnais n°1 — le gate E2E de l'acte de base.
 *
 *   - @happy    — les cinq étapes aboutissent : le compteur passe de rouge à
 *                 vert, et le lot reçoit son propriétaire.
 *   - @negative — le verrou de conformité mord vraiment. Avant l'encodage du
 *                 troisième lot, une dépense est refusée ; après, elle passe.
 *
 * ── Pourquoi le @negative vit ici et pas dans le parcours filmé ───────────
 *
 * Le 422 est ce que le parcours EXPLIQUE, mais le montrer serait filmer un
 * écran d'erreur — et une vitrine qui s'ouvre sur un refus n'apprend rien à
 * un visiteur. Le gate, lui, doit le prouver : sans cette vérification, le
 * compteur vert ne serait qu'une couleur.
 */
import { test, expect } from "@playwright/test";
import { Scene } from "./scene";
import { lot, mondeDuParcours } from "./lot.journey";
import { API_BASE } from "../helpers/adresses";

const parcours = lot;

test.describe(`Parcours de référence — ${parcours.titre}`, () => {
  test(`@happy ${parcours.slug} aboutit de bout en bout`, async ({ page }) => {
    test.setTimeout(300_000);
    const comptes = await test.step("amorçage", () => parcours.amorcer(page));
    const scene = new Scene(page, comptes);

    for (const etape of parcours.etapes) {
      await test.step(`${etape.acteur} · ${etape.id}`, async () => {
        await etape.action(scene);
        if (etape.assertion) await etape.assertion(page);
      });
    }

    expect(scene.narration.length).toBeGreaterThan(0);
  });

  test(`@negative ${parcours.slug} — le verrou de conformité refuse avant, accepte après`, async ({
    page,
  }) => {
    test.setTimeout(240_000);
    const comptes = await parcours.amorcer(page);
    const monde = mondeDuParcours()!;

    const session = await page.request.post(`${API_BASE}/auth/login`, {
      data: {
        email: comptes.syndic!.email,
        password: comptes.syndic!.motDePasse,
      },
    });
    expect(session.ok(), `connexion: HTTP ${session.status()}`).toBeTruthy();
    const jeton = (await session.json()).token;

    const uneDepense = () =>
      page.request.post(`${API_BASE}/expenses`, {
        data: {
          building_id: monde.buildingId,
          category: "Maintenance",
          description: "Entretien des communs",
          amount: 250,
          expense_date: new Date().toISOString(),
        },
        headers: { Authorization: `Bearer ${jeton}` },
      });

    // ── Avant : 800/1000, le verrou tient ────────────────────────────────
    //
    // C'est exactement le message que reçoit un syndic qui n'a pas fini
    // d'encoder son acte de base, et qu'il ne sait pas relier à un lot
    // manquant. Le compteur du parcours filmé est la réponse à ce refus.
    const avant = await uneDepense();
    expect(
      avant.status(),
      "Une dépense passe sur un immeuble non conforme : le verrou de " +
        "l'ADR-0010 ne tient pas, et le compteur du parcours filmé ne " +
        "prévient de rien.",
    ).toBe(422);
    expect(await avant.text()).toContain("CONFORMANT");

    // ── On encode le lot manquant, par l'API ─────────────────────────────
    const acps = await page.request.get(`${API_BASE}/acps`, {
      headers: { Authorization: `Bearer ${jeton}` },
    });
    expect(acps.ok()).toBeTruthy();
    const listeAcps = await acps.json();
    const acpId = (Array.isArray(listeAcps) ? listeAcps : listeAcps.data)[0].id;

    const troisieme = await page.request.post(`${API_BASE}/units`, {
      data: {
        acp_id: acpId,
        building_id: monde.buildingId,
        unit_number: "3A",
        floor: 2,
        surface_area: 78,
        unit_type: "Apartment",
        quota: 200,
      },
      headers: { Authorization: `Bearer ${jeton}` },
    });
    expect(
      troisieme.ok(),
      `création du lot manquant : HTTP ${troisieme.status()} — ` +
        `${(await troisieme.text()).slice(0, 200)}`,
    ).toBeTruthy();

    // ── Après : 1000/1000, la même dépense passe ─────────────────────────
    const apres = await uneDepense();
    expect(
      apres.ok(),
      "La même dépense est encore refusée alors que l'immeuble est " +
        `conforme : HTTP ${apres.status()} — ` +
        `${(await apres.text()).slice(0, 200)}. Le compteur vert du ` +
        "parcours filmé promettrait alors quelque chose de faux.",
    ).toBeTruthy();
  });
});
