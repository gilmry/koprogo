/**
 * Le balayage — ouvrir CHAQUE écran, et dire ce qu'on y a vu.
 *
 * ── Pourquoi un balayage, alors qu'il y a déjà sept parcours ─────────────
 *
 * Les sept parcours métier traversent 23 chemins. `src/pages` en déclare 95
 * balayables. Soixante-douze écrans du produit n'avaient donc **jamais été
 * ouverts par un navigateur** dans ce dépôt — y compris les quatre tableaux
 * de bord de rôle, `/buildings`, `/meetings`, `/tickets`, `/documents`.
 *
 * Un parcours métier raconte une histoire : il choisit ses écrans, et c'est
 * sa force. Le balayage fait l'inverse — il n'a aucune histoire, il passe
 * partout. C'est ce qui lui permet de trouver ce que personne ne cherchait.
 *
 * ── Ce qu'il ne fait PAS ──────────────────────────────────────────────────
 *
 * Il ne rend **aucun verdict**. Arbitrage du PO du 2026-09-19 : « décrire, ne
 * rien casser ». Un balayage qui ferait rougir la CI sur 72 écrans jamais
 * éprouvés serait ingérable le premier jour, et on le débrancherait le
 * second. Il décrit, il filme, il produit un tableau.
 *
 * C'est aussi la règle de l'élément 3 du skill `documentation-vivante` :
 * « la preuve de valeur n'est PAS un test : elle ne doit pas faire tomber le
 * build » (#876).
 *
 * ── Comment il tient dans un film regardable ──────────────────────────────
 *
 * Narrer chaque page coûterait ~6 s l'unité, soit ~9 min par rôle. Les
 * routes sont donc GROUPÉES par section : un chapitre par groupe, puis un
 * passage rapide à l'intérieur. Seules les **anomalies** sont narrées
 * individuellement — ce sont elles qu'on veut voir à l'écran, et elles seules
 * méritent un chapitre dans la galerie.
 */
import type { Page } from "@playwright/test";
import type { Acteur, Etape, Parcours } from "./parcours";
import { inventaireDesEcrans } from "./inventaire-pages";
import { adminLogin } from "../helpers/auth";
import { API_BASE } from "../helpers/adresses";
import { ADMIN_EMAIL, ADMIN_PASSWORD } from "../helpers/identifiants";

/** Ce qu'on a observé en ouvrant un écran. Aucun jugement, des faits. */
export interface Observation {
  readonly route: string;
  /** Où l'on a ATTERRI. Différent de `route` ⇒ la route a rebondi. */
  readonly arrivee: string;
  /** Longueur du texte rendu, bannière de narration exclue. */
  readonly taille: number;
  readonly erreursConsole: number;
  /**
   * Le PREMIER message d'erreur, tronqué.
   *
   * Un compteur dit qu'il y a un défaut, jamais lequel : le balayage aurait
   * livré « 14 écrans jettent des erreurs » et laissé rouvrir les 14 à la
   * main. Garder le message coûte une ligne et supprime l'enquête.
   */
  readonly premiereErreur: string | null;
}

/**
 * Sous ce seuil, l'écran n'a rien montré d'exploitable.
 *
 * 200 caractères, c'est moins qu'un en-tête plus un menu. Un écran qui n'en
 * rend pas davantage n'a ni table, ni formulaire, ni message d'état : il est
 * vide au sens où un utilisateur l'entendrait.
 */
const SEUIL_ECRAN_MAIGRE = 200;

/** Le premier segment d'une route, ou « racine ». */
function sectionDe(route: string): string {
  const segments = route.split("/").filter(Boolean);
  return segments.length > 1 ? segments[0] : "racine";
}

/**
 * Au-delà de ce nombre, un groupe se découpe.
 *
 * Mesuré : la section « racine » porte 57 des 95 routes. Un chapitre unique
 * l'aurait couverte, et le film aurait affiché « Section racine » pendant
 * plusieurs minutes — un chapitrage qui ne permet plus de sauter à l'étape
 * ne chapitre plus rien.
 */
const MAX_PAR_CHAPITRE = 10;

function grouperParSection(routes: string[]): Map<string, string[]> {
  const parSection = new Map<string, string[]>();
  for (const route of routes) {
    const section = sectionDe(route);
    if (!parSection.has(section)) parSection.set(section, []);
    parSection.get(section)!.push(route);
  }

  // Les gros groupes sont tranchés en morceaux nommés par leur INTERVALLE :
  // « de /accountant à /buildings » se lit sans connaître le découpage, et
  // aucune liste de domaines tenue à la main ne peut dériver.
  const groupes = new Map<string, string[]>();
  for (const [section, liste] of parSection) {
    if (liste.length <= MAX_PAR_CHAPITRE) {
      groupes.set(section, liste);
      continue;
    }
    for (let i = 0; i < liste.length; i += MAX_PAR_CHAPITRE) {
      const tranche = liste.slice(i, i + MAX_PAR_CHAPITRE);
      groupes.set(
        `${section} ${tranche[0]} → ${tranche[tranche.length - 1]}`,
        tranche,
      );
    }
  }
  return groupes;
}

/**
 * Mesure ce que la page rend, bannière de narration DÉDUITE.
 *
 * Sans cette déduction, le balayage mesurerait sa propre narration et
 * conclurait que tous les écrans sont peuplés — un instrument qui se mesure
 * lui-même ne mesure rien.
 */
async function tailleDuContenu(page: Page): Promise<number> {
  return page
    .evaluate(() => {
      const banniere = document.getElementById("vitrine-narration");
      const bruit = banniere?.textContent ?? "";
      const tout = (document.body as HTMLElement)?.innerText ?? "";
      return tout.replace(bruit, "").trim().length;
    })
    .catch(() => 0);
}

function estAnormale(o: Observation): boolean {
  return (
    o.arrivee !== o.route ||
    o.taille < SEUIL_ECRAN_MAIGRE ||
    o.erreursConsole > 0
  );
}

/** La phrase qu'on incruste à l'écran quand un écran sort de l'ordinaire. */
function phraseAnomalie(o: Observation): string {
  if (o.arrivee !== o.route) {
    return `${o.route} ne s'ouvre pas pour ce rôle : on atterrit sur ${o.arrivee}.`;
  }
  if (o.taille < SEUIL_ECRAN_MAIGRE) {
    return `${o.route} s'ouvre mais ne montre presque rien (${o.taille} caractères).`;
  }
  return (
    `${o.route} s'ouvre, mais la console proteste : ` +
    `${o.premiereErreur ?? `${o.erreursConsole} erreur(s)`}`
  );
}

/** Rôle produit ↔ acteur tel qu'un humain le nomme. */
const ROLE_DE: Record<string, string> = {
  syndic: "syndic",
  copropriétaire: "owner",
  comptable: "accountant",
  administrateur: "superadmin",
};

/**
 * Crée une organisation et un compte porteur du rôle demandé.
 *
 * L'administrateur fait exception : il opère AU-DESSUS du multi-tenant, son
 * compte vient de l'environnement et son mot de passe tourne au déploiement
 * (cf. `administration.journey.ts`). Lui en fabriquer un ici filmerait un
 * monde que personne d'autre ne rejoue.
 */
async function amorcerLeCompte(page: Page, acteur: Acteur) {
  if (acteur === "administrateur") {
    await page.context().clearCookies();
    return {
      administrateur: { email: ADMIN_EMAIL, motDePasse: ADMIN_PASSWORD },
    };
  }

  const horodatage = Date.now();
  const motDePasse = "test123456";
  const email = `balayage-${ROLE_DE[acteur]}-${horodatage}@example.com`;
  const jetonAdmin = await adminLogin(page);

  const reponseOrg = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `Balayage ${acteur} ${horodatage}`,
      slug: `balayage-${ROLE_DE[acteur]}-${horodatage}`,
      contact_email: email,
      subscription_plan: "professional",
    },
    headers: { Authorization: `Bearer ${jetonAdmin}` },
  });
  if (!reponseOrg.ok()) {
    throw new Error(
      `amorçage:organisation a répondu ${reponseOrg.status()} — ` +
        `le balayage de « ${acteur} » ne peut pas commencer sans organisation.`,
    );
  }
  const org = (await reponseOrg.json()) as { id: string };

  // Le cookie de l'ADMINISTRATEUR doit partir avant l'enregistrement : depuis
  // #769, le serveur ne pose pas de session quand l'appelant en a déjà une,
  // et le nouveau compte repartirait sans jeton. Même piège que `auth.ts:335`.
  await page.context().clearCookies();

  const reponseCompte = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: motDePasse,
      first_name: "Balayage",
      last_name: acteur,
      role: ROLE_DE[acteur],
      organization_id: org.id,
    },
  });
  if (!reponseCompte.ok()) {
    throw new Error(
      `amorçage:compte a répondu ${reponseCompte.status()} pour le rôle ` +
        `« ${ROLE_DE[acteur]} » — le balayage ne peut pas se connecter.`,
    );
  }

  return { [acteur]: { email, motDePasse } };
}

/**
 * Construit le parcours de balayage d'un rôle.
 *
 * Toutes les routes de l'inventaire sont tentées, sans filtrage par
 * `canAccessRoute`. C'est délibéré : cette fonction rend `true` pour toute
 * route absente de son registre (`guards.ts:117`), donc s'y fier reviendrait
 * à filmer ce que le code CROIT autorisé plutôt que ce qui se passe. Le
 * rebond observé est un fait ; la permission déclarée n'est qu'une intention.
 */
export function balayagePour(acteur: Acteur): Parcours {
  const routes = inventaireDesEcrans();
  const groupes = grouperParSection(routes);
  const observations: Observation[] = [];
  let erreursConsole = 0;
  let messages: string[] = [];

  const retenir = (texte: string) => {
    erreursConsole += 1;
    if (messages.length < 3) messages.push(texte.slice(0, 200));
  };

  const etapes: Etape[] = [
    {
      id: "0-connexion",
      acteur,
      description:
        `Balayage systématique du produit vu par le ${acteur} : ` +
        `${routes.length} écrans ouverts un par un. Ce parcours ne raconte ` +
        `rien — il constate.`,
      action: async (scene) => {
        scene.page.on("console", (m) => {
          if (m.type() === "error") retenir(m.text());
        });
        scene.page.on("pageerror", (e) => retenir(String(e)));
        scene.page.on("requestfailed", (r) =>
          retenir(`${r.method()} ${r.url()} — ${r.failure()?.errorText}`),
        );
        await scene.devenir(acteur);
      },
    },
  ];

  for (const [section, routesDeLaSection] of groupes) {
    etapes.push({
      id: `section-${section}`,
      acteur,
      description: `Section « ${section} » — ${routesDeLaSection.length} écran(s).`,
      action: async (scene) => {
        for (const route of routesDeLaSection) {
          const avant = erreursConsole;
          messages = [];
          await scene.survoler(route);
          const arrivee = new URL(scene.page.url()).pathname.replace(/\/$/, "");
          const observation: Observation = {
            route,
            arrivee: arrivee || "/",
            taille: await tailleDuContenu(scene.page),
            erreursConsole: erreursConsole - avant,
            premiereErreur: messages[0] ?? null,
          };
          observations.push(observation);
          // Seules les anomalies deviennent un chapitre : un balayage qui
          // narrerait ses succès noierait ses trouvailles dans du bruit.
          if (estAnormale(observation)) {
            await scene.raconter(phraseAnomalie(observation), acteur);
          }
        }
      },
    });
  }

  etapes.push({
    id: "bilan",
    acteur,
    description: "Bilan du balayage.",
    action: async (scene) => {
      const rebonds = observations.filter((o) => o.arrivee !== o.route);
      const maigres = observations.filter(
        (o) => o.arrivee === o.route && o.taille < SEUIL_ECRAN_MAIGRE,
      );
      const bruyants = observations.filter((o) => o.erreursConsole > 0);

      // Le tableau part sur la sortie standard : il devient consultable dans
      // le journal du job, là où un lecteur ira chercher le détail que la
      // vidéo ne peut pas porter.
      console.log(`\n### Balayage — ${acteur}\n`);
      console.log(
        "| écran | arrivée | caractères | erreurs | première erreur |",
      );
      console.log("|---|---|---:|---:|---|");
      for (const o of observations) {
        const message = (o.premiereErreur ?? "").replace(/\|/g, "¦");
        console.log(
          `| ${o.route} | ${o.arrivee === o.route ? "—" : o.arrivee} | ` +
            `${o.taille} | ${o.erreursConsole} | ${message} |`,
        );
      }

      await scene.raconter(
        `${observations.length} écrans ouverts : ${rebonds.length} rebondissent, ` +
          `${maigres.length} ne montrent presque rien, ${bruyants.length} ` +
          `jettent des erreurs. Le détail est dans le journal.`,
        acteur,
      );
    },
  });

  return {
    slug: `balayage-${ROLE_DE[acteur]}`,
    titre: `Balayage complet — ${acteur}`,
    propos:
      `Tous les écrans du produit, ouverts un par un par un ${acteur}. ` +
      `Ce parcours ne démontre pas une valeur : il établit un état des lieux, ` +
      `et rend visibles les écrans que personne n'avait jamais ouverts.`,
    amorcer: (page) => amorcerLeCompte(page, acteur),
    etapes,
  };
}
