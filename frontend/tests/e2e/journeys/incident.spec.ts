/**
 * Harnais n°1 — le gate E2E du parcours de l'incident.
 *
 *   - @happy    — les dix étapes aboutissent : un incident est signalé,
 *                 reçu, confié, résolu et clos.
 *   - @edge     — le parcours fait bien agir deux acteurs distincts.
 *   - @negative — le formulaire refuse un incident daté du futur.
 *   - @security — un copropriétaire d'une AUTRE organisation ne voit pas ce
 *                 signalement. Une plainte porte l'adresse de quelqu'un.
 *
 * Les trois derniers rejouent l'amorçage du parcours, jamais ses étapes : ce
 * qu'ils éprouvent n'a pas sa place dans une vidéo de démonstration, et un
 * parcours filmé qui montrerait un refus d'accès n'apprendrait rien à un
 * visiteur.
 */
import { test, expect } from "@playwright/test";
import { Scene } from "./scene";
import { acteursDe } from "./parcours";
import { incident, mondeDuParcours } from "./incident.journey";
import { adminLogin } from "../helpers/auth";
import { API_BASE } from "../helpers/adresses";

const parcours = incident;

test.describe(`Parcours de référence — ${parcours.titre}`, () => {
  test(`@happy ${parcours.slug} aboutit de bout en bout`, async ({ page }) => {
    // Dix étapes, quatre transitions d'état côté serveur, deux connexions.
    // Le plafond par défaut ne couvre pas l'amorçage seul.
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

    // Le parcours a bien produit un dossier identifié. Sans lui, les dix
    // étapes auraient pu s'enchaîner sur des écrans vides.
    expect(
      mondeDuParcours()?.ticketId,
      "Le parcours s'est déroulé sans qu'aucun ticket ne reçoive " +
        "d'identifiant : rien n'a été créé.",
    ).toBeTruthy();
  });

  test(`@edge ${parcours.slug} fait agir la copropriétaire ET le syndic`, async () => {
    // Règle 9 de CRITICAL.md. Un parcours qui perdrait sa bascule d'acteur
    // montrerait une syndic signalant sa propre fuite, puis la traitant : ça
    // ne démontre plus rien du produit.
    expect(
      acteursDe(parcours),
      "Le cycle de l'incident tient à ce que deux personnes distinctes " +
        "interviennent : celle qui constate et celle qui traite.",
    ).toEqual(["copropriétaire", "syndic"]);
  });

  test(`@negative ${parcours.slug} — le formulaire ne laisse pas déposer une plainte datée du futur`, async ({
    page,
  }) => {
    test.setTimeout(180_000);
    const comptes = await parcours.amorcer(page);
    const monde = mondeDuParcours()!;
    const scene = new Scene(page, comptes);
    await scene.devenir("copropriétaire");

    await page.goto(`/tickets/new?buildingId=${monde.buildingId}`);
    await expect(page.getByTestId("ticket-create-form-element")).toBeVisible({
      timeout: 20000,
    });
    await page
      .getByTestId("ticket-create-kind-select")
      .selectOption("complaint");

    // Tout est valide SAUF la date : titre, description et gravité remplis.
    // Sans cela, le bouton serait désactivé pour une autre raison et ce test
    // passerait au vert sans rien éprouver de la règle de date.
    await page.getByTestId("ticket-create-title-input").fill("Date du futur");
    await page
      .getByTestId("ticket-create-description-textarea")
      .fill(
        "Une description suffisamment longue pour dépasser le minimum de " +
          "vingt caractères imposé par le formulaire.",
      );
    await page.getByTestId("ticket-severity-radio-high").click();

    const demain = new Date(Date.now() + 24 * 3600 * 1000)
      .toISOString()
      .slice(0, 10);
    await page.getByTestId("ticket-create-incident-date-input").fill(demain);

    // ── Ce que le produit fait, et que je n'avais pas supposé ────────────
    //
    // J'attendais un envoi refusé avec un message. Le formulaire fait mieux :
    // il DÉSACTIVE le dépôt tant que la date n'est pas valide. Mesuré au
    // navigateur — `<button disabled aria-disabled="true">Déposer la
    // plainte</button>`.
    //
    // On décrit donc ce qui se passe, pas ce qu'on attendait. Une plainte ne
    // peut pas porter sur un incident qui n'a pas encore eu lieu : ce
    // registre peut finir devant un juge de paix, et des dates invérifiables
    // y coûteraient plus cher qu'un champ pénible à remplir.
    await expect(
      page.getByTestId("ticket-create-incident-date-error"),
      "Rien ne dit à la plaignante pourquoi elle ne peut pas déposer.",
    ).toBeVisible({ timeout: 10000 });
    await expect(
      page.getByTestId("ticket-create-submit"),
      "Le dépôt reste possible avec une date de demain.",
    ).toBeDisabled();

    // Et la veille passe : la règle borne le futur, pas le passé.
    const hier = new Date(Date.now() - 24 * 3600 * 1000)
      .toISOString()
      .slice(0, 10);
    await page.getByTestId("ticket-create-incident-date-input").fill(hier);
    await expect(
      page.getByTestId("ticket-create-submit"),
      "Une date valide ne rouvre pas le dépôt : la règle bloque plus que le " +
        "futur.",
    ).toBeEnabled({ timeout: 10000 });
  });

  test(`@security ${parcours.slug} — une autre organisation ne voit pas ce signalement`, async ({
    page,
  }) => {
    test.setTimeout(240_000);

    // Tout se joue au niveau de l'API : ce qu'on éprouve est le
    // cloisonnement du serveur, pas le rendu d'un écran. Passer par le
    // navigateur n'ajouterait qu'une couche de cache entre la question et
    // la réponse.
    const comptes = await parcours.amorcer(page);
    const monde = mondeDuParcours()!;

    const session = await page.request.post(`${API_BASE}/auth/login`, {
      data: {
        email: comptes.copropriétaire!.email,
        password: comptes.copropriétaire!.motDePasse,
      },
    });
    expect(session.ok(), `connexion: HTTP ${session.status()}`).toBeTruthy();
    const jetonCopro = (await session.json()).token;

    const creation = await page.request.post(`${API_BASE}/tickets`, {
      data: {
        building_id: monde.buildingId,
        title: monde.titreDuSignalement,
        description:
          "Une flaque se forme chaque jour sous la trappe technique du hall.",
        category: "Plumbing",
        priority: "High",
      },
      headers: { Authorization: `Bearer ${jetonCopro}` },
    });
    expect(
      creation.ok(),
      `Le signalement n'a pas pu être créé : HTTP ${creation.status()} — ` +
        `${(await creation.text()).slice(0, 200)}`,
    ).toBeTruthy();
    const ticket = await creation.json();
    expect(ticket.id).toBeTruthy();

    // Un copropriétaire d'une organisation SANS aucun lien.
    const horodatage = Date.now();
    const emailEtranger = `incident-etranger-${horodatage}@example.com`;
    const adminToken = await adminLogin(page);
    const orgEtrangere = await page.request.post(`${API_BASE}/organizations`, {
      data: {
        name: `Incident Etrangere ${horodatage}`,
        slug: `incident-etrangere-${horodatage}`,
        contact_email: emailEtranger,
        subscription_plan: "professional",
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    expect(orgEtrangere.ok()).toBeTruthy();
    const org = await orgEtrangere.json();

    await page.context().clearCookies();
    const inscription = await page.request.post(`${API_BASE}/auth/register`, {
      data: {
        email: emailEtranger,
        password: "test123456",
        first_name: "Etrangere",
        last_name: "AuDossier",
        role: "owner",
        organization_id: org.id,
      },
    });
    expect(
      inscription.ok(),
      `inscription: HTTP ${inscription.status()} — ` +
        `${(await inscription.text()).slice(0, 200)}`,
    ).toBeTruthy();
    const jetonEtranger = (await inscription.json()).token;

    // Il demande le signalement en le désignant par son identifiant. C'est
    // le cas réel : un identifiant fuite par un lien recopié, une capture
    // d'écran, un courriel transféré.
    const lecture = await page.request.get(`${API_BASE}/tickets/${ticket.id}`, {
      headers: { Authorization: `Bearer ${jetonEtranger}` },
    });

    expect(
      lecture.status(),
      "Un copropriétaire d'une autre organisation lit un signalement qui ne " +
        "le concerne pas. Une plainte porte une adresse, un étage, et " +
        `souvent un nom : ${(await lecture.text()).slice(0, 200)}`,
    ).toBeGreaterThanOrEqual(400);
  });
});
