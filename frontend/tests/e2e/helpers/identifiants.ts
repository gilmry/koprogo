/**
 * Les identifiants du superadministrateur, en UN seul endroit.
 *
 * `admin123` était écrit en dur à quarante-cinq endroits de la suite, alors
 * que ce mot de passe a été tourné le 2026-09-06 (#763). Centraliser laissait
 * un seul endroit à corriger le jour de la bascule.
 *
 * ── Ce fichier avait annoncé le jour, et le jour est venu ────────────────
 *
 * Il disait : « le repli du seed joue tant que la CI ne définit pas
 * `KOPROGO_SUPERADMIN_PASSWORD` — **le jour où elle le fera, toute la suite
 * e2e tombera d'un coup**, et aucun message d'erreur ne parlera du mot de
 * passe. »
 *
 * C'est arrivé le 2026-09-10 contre la démo (#870) : `401 Invalid
 * credentials`, un message qui parle d'identifiants là où le défaut est de
 * configuration. La prévision était juste ; ce qui manquait, c'était de la
 * rendre lisible au moment où elle se réalise.
 *
 * Rétablir le mot de passe ne suffit pas : `seed_superadmin` fait un **upsert
 * à chaque démarrage** depuis l'environnement, donc le prochain déploiement
 * reprend la valeur et le piège se réarme. Le garde ci-dessous est la seule
 * parade qui survit à un redéploiement.
 */

/** La valeur du seed. Elle reste le défaut : la CI amorce sa propre base. */
const REPLI_DU_SEED = "admin123";

export const ADMIN_EMAIL =
  process.env.KOPROGO_SUPERADMIN_EMAIL || "admin@koprogo.com";

export const ADMIN_PASSWORD =
  process.env.KOPROGO_SUPERADMIN_PASSWORD || REPLI_DU_SEED;

/**
 * Un hôte que la suite n'amorce pas elle-même.
 *
 * En CI comme en local, la base est créée par la campagne et `admin123` y est
 * légitime. Un garde qui casserait la CI pour protéger la démo serait pire que
 * le défaut qu'il corrige.
 */
function estDistant(adresse: string | undefined): boolean {
  if (!adresse) return false;
  let hote: string;
  try {
    hote = new URL(adresse).hostname;
  } catch {
    // Une adresse illisible n'est pas notre affaire : ce n'est pas ce garde
    // qui authentifie, et remplacer une panne par une autre n'aide personne.
    return false;
  }
  // `URL` rend `[::1]` sous la forme `[::1]`, crochets compris.
  const locaux = ["localhost", "127.0.0.1", "0.0.0.0", "::1", "[::1]", ""];
  return !locaux.includes(hote);
}

/**
 * S'arrête AVANT la première requête si la campagne vise un hôte distant sans
 * que l'opérateur ait choisi d'identifiants.
 *
 * ── Ce que ce garde surveille VRAIMENT ─────────────────────────────────
 *
 * Sa première version comparait le mot de passe à `admin123`. C'était une
 * erreur de conception, et le 2026-09-12 l'a rendue visible : la démo a reçu
 * `KOPROGO_SUPERADMIN_PASSWORD=admin123` dans son environnement, pour que
 * l'upsert du seed cesse d'effacer la valeur à chaque redémarrage. Le garde
 * aurait alors refusé une campagne parfaitement légitime, parce que la bonne
 * valeur ressemblait à la mauvaise.
 *
 * Le danger n'est pas la VALEUR, c'est de **ne pas avoir choisi**. Un repli
 * silencieux contre un hôte qu'on n'amorce pas rend un `401` qui parle
 * d'identifiants là où le défaut est de configuration — une demi-journée
 * d'enquête sur un défaut produit qui n'existe pas (#870).
 *
 * Le garde regarde donc si la variable est POSÉE, pas ce qu'elle contient.
 * Choisir `admin123` en connaissance de cause est une décision d'exploitation ;
 * elle ne regarde pas ce fichier. Le serveur, lui, la signale déjà :
 *
 * > SÉCURITÉ : le superadmin utilise le mot de passe par défaut, lisible dans
 * > le dépôt public.
 *
 * Le message ne recopie jamais la valeur : une erreur atterrit dans les
 * journaux de CI, qui se conservent.
 */
export function verifieLesIdentifiants(
  adresse: string | undefined = process.env.PLAYWRIGHT_BASE_URL,
  motDePasseChoisi: string | undefined = process.env
    .KOPROGO_SUPERADMIN_PASSWORD,
): void {
  if (!estDistant(adresse)) return;
  if (motDePasseChoisi) return;

  throw new Error(
    `PLAYWRIGHT_BASE_URL désigne « ${adresse} », un hôte que cette suite ` +
      `n'amorce pas, et aucun identifiant n'a été choisi.\n\n` +
      `Cet hôte reçoit son superadministrateur par upsert au démarrage, ` +
      `depuis son propre environnement : le repli du seed n'y vaut que si ` +
      `l'exploitant l'y a lui-même posé. Sinon la connexion rendra ` +
      `« 401 Invalid credentials » — un message qui parle d'identifiants là ` +
      `où le défaut est de configuration.\n\n` +
      `Exportez les deux variables avant de lancer la campagne :\n` +
      `    KOPROGO_SUPERADMIN_EMAIL\n` +
      `    KOPROGO_SUPERADMIN_PASSWORD\n\n` +
      `Contre localhost, rien à faire : la base y est amorcée par la ` +
      `campagne (#870).`,
  );
}

// Les 27 specs qui importent ce module déclenchent la vérification à leur
// chargement, donc avant la première requête. C'est le seul endroit qui les
// couvre toutes sans `globalSetup`.
verifieLesIdentifiants();
