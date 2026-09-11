import type { Page } from "@playwright/test";

/**
 * Le socle du banc mobile : une session peinte, et une API qui ne répond rien.
 *
 * ── Pourquoi aucun jeton ───────────────────────────────────────────────────
 *
 * `stores/auth.ts` le dit lui-même (WP-FE1) : « aucun token en localStorage.
 * `koprogo_user` est un cache d'affichage NON sensible ». La coquille — barre
 * latérale, en-tête, barre d'onglets — se peint depuis ce cache, AVANT le
 * rafraîchissement silencieux.
 *
 * Un test de coquille n'a donc besoin d'aucune identité réelle, et c'est une
 * bonne chose : il ne dépend ni d'un compte, ni du cookie HttpOnly, ni de la
 * démo. Il ne peut pas non plus se tromper de sujet — il ne verra jamais une
 * donnée, seulement une mise en page.
 */
export const ROLES = ["syndic", "owner", "accountant", "superadmin"] as const;
export type Role = (typeof ROLES)[number];

/**
 * L'écran d'accueil de chaque rôle — le premier onglet de sa barre.
 *
 * La racine `/` ne convient pas : elle renvoie vers `/login`, qui n'affiche
 * aucune coquille. Un banc lancé dessus mesurerait l'écran de connexion en
 * croyant mesurer un tableau de bord, et rapporterait sereinement que rien ne
 * déborde.
 *
 * La barre oblique finale évite une redirection 301 du serveur de fichiers.
 */
export const ACCUEIL: Record<Role, string> = {
  syndic: "/syndic/",
  owner: "/owner/",
  accountant: "/accountant/",
  superadmin: "/admin/monitoring/",
};

/**
 * Toute requête d'API rend une collection vide.
 *
 * L'interception se déclare par une EXPRESSION RÉGULIÈRE, jamais par un motif
 * en chaîne : Playwright résout un motif sans schéma contre `baseURL`, et une
 * interception qui ne s'applique pas est pire qu'une absente. C'est la règle
 * que `garde-interception-reseau` tient déjà pour le reste de la suite.
 *
 * Le vide est délibéré. Une coquille qui ne tient qu'avec des données pleines
 * est une coquille qui casse le premier jour d'un nouveau cabinet.
 */
function jetonFactice(): string {
  // Un JWT syntaxiquement valide, non signé, à expiration lointaine. Il ne
  // franchit aucun serveur : la seule chose qui le lit est le minuteur de
  // rafraîchissement du store.
  const entete = Buffer.from(
    JSON.stringify({ alg: "none", typ: "JWT" }),
  ).toString("base64url");
  const charge = Buffer.from(
    JSON.stringify({ sub: "banc", exp: 4102444800 }),
  ).toString("base64url");
  return `${entete}.${charge}.banc`;
}

function utilisateurServi(role: Role) {
  return {
    id: "00000000-0000-0000-0000-0000000000aa",
    email: `banc-${role}@example.com`,
    first_name: "Banc",
    last_name: "Mobile",
    role,
    organization_id: "00000000-0000-0000-0000-0000000000bb",
    is_active: true,
    roles: [],
  };
}

async function simuleLApi(page: Page, role: Role): Promise<void> {
  // ── L'ORDRE COMPTE, et à l'envers de l'intuition ────────────────────────
  //
  // Playwright essaie les interceptions dans l'ORDRE INVERSE d'enregistrement :
  // la dernière posée gagne. Le fourre-tout doit donc être posé EN PREMIER,
  // et les cas particuliers ensuite.
  //
  // Écrit dans l'ordre naturel, le fourre-tout répondait `{data: []}` au
  // rafraîchissement, `user` était `undefined`, et la coquille renvoyait vers
  // `/login`. Le banc mesurait alors l'écran de connexion en croyant mesurer
  // un tableau de bord — sans qu'aucune erreur ne le dise.
  await page.route(/\/api\/v1\//, (route) =>
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ data: [], items: [], total: 0, page: 1 }),
    }),
  );

  // Le rafraîchissement silencieux décide si la coquille se peint : sans
  // session confirmée, chaque page renvoie vers `/login`.
  await page.route(/\/api\/v1\/auth\/refresh/, (route) =>
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        token: jetonFactice(),
        user: utilisateurServi(role),
      }),
    }),
  );
}

/** Peint une session du rôle demandé, sans réseau ni identité. */
export async function ouvreEnTantQue(
  page: Page,
  role: Role,
  chemin: string,
): Promise<void> {
  await simuleLApi(page, role);

  await page.addInitScript(
    ([r]) => {
      window.localStorage.setItem(
        "koprogo_user",
        JSON.stringify({
          id: "00000000-0000-0000-0000-0000000000aa",
          email: `banc-${r}@example.com`,
          first_name: "Banc",
          last_name: "Mobile",
          role: r,
          organizationId: "00000000-0000-0000-0000-0000000000bb",
          buildingIds: [],
          roles: [r],
        }),
      );
    },
    [role],
  );

  await page.goto(chemin);
  // La coquille est hydratée par `client:load` : attendre le document ne
  // suffit pas, il faut attendre qu'un morceau de la coquille existe.
  await page.waitForSelector("[data-testid='tabbar']", { timeout: 15_000 });
}

/**
 * La plus petite dimension d'une cible, en pixels CSS.
 *
 * C'est LA mesure que jsdom ne peut pas rendre : sans mise en page, tout
 * `getBoundingClientRect()` y vaut zéro. Un test unitaire ne peut vérifier
 * qu'une chaîne de classes — `h-11 w-11` — et croire sur parole qu'elle
 * produit 44 px. Ici, c'est le navigateur qui répond.
 */
export async function plusPetiteDimension(
  page: Page,
  ancre: string,
): Promise<number> {
  const boite = await page.getByTestId(ancre).boundingBox();
  if (!boite) throw new Error(`Cible sans boîte : ${ancre}`);
  return Math.min(boite.width, boite.height);
}

/**
 * Ajoute une réponse PARTICULIÈRE, après le socle.
 *
 * À appeler APRÈS `ouvreEnTantQue`. Playwright essaie les interceptions dans
 * l'ordre inverse d'enregistrement : posée avant, celle-ci serait recouverte
 * par le fourre-tout du socle et n'aurait aucun effet — silencieusement, ce
 * qui est le piège que `garde-interception-reseau` décrit déjà.
 *
 * C'est arrivé à la première mesure du sélecteur : la liste sortait vide, et
 * le test aurait conclu « aucune ligne à mesurer, donc rien à redire ».
 */
export async function repond(
  page: Page,
  motif: RegExp,
  corps: unknown,
): Promise<void> {
  await page.route(motif, (route) =>
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(corps),
    }),
  );
}

/** Deux immeubles : assez pour mesurer une ligne, sans décrire un jeu de données. */
export const DEUX_IMMEUBLES = {
  data: [
    {
      id: "11111111-1111-1111-1111-111111111111",
      name: "Résidence Les Érables",
      city: "Bruxelles",
    },
    {
      id: "22222222-2222-2222-2222-222222222222",
      name: "Le Clos du Parc",
      city: "Ixelles",
    },
  ],
  total: 2,
  page: 1,
  per_page: 20,
};
