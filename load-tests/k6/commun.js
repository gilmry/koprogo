// Briques communes aux scénarios k6 de KoproGo.
//
// Chaque scénario rend deux choses à la fin du run :
// - un résumé lisible (débit, latences, taux d'échec) ;
// - une ligne `SLUIS_RESUME {...}`, au format que Sluis lit dans la sortie
//   de k6 (requêtes, débit, p50, p99, part des requêtes en échec).
import http from 'k6/http';

/** URL de l'API visée, sans barre finale. */
export const CIBLE = (__ENV.BASE_URL || 'http://localhost:8080').replace(/\/$/, '');

/** Compte de démo semé par `seed.rs` (Résidence Grand Place). */
const EMAIL = __ENV.KOPROGO_EMAIL || 'syndic@grandplace.be';
const MOT_DE_PASSE = __ENV.KOPROGO_PASSWORD || 'syndic123';

const EN_TETES = { 'Content-Type': 'application/json', Accept: 'application/json' };

/** Options partagées : corps de réponse jetés, centiles du résumé. */
export const OPTIONS_COMMUNES = {
  // Comme wrk : on mesure le serveur, pas la mémoire du générateur.
  discardResponseBodies: true,
  summaryTrendStats: ['avg', 'p(50)', 'p(75)', 'p(90)', 'p(95)', 'p(99)', 'p(99.9)', 'max'],
};

/** Envoie une requête, avec le jeton s'il y en a un. */
export function envoyer(methode, chemin, corps, jeton) {
  const en_tetes = Object.assign({}, EN_TETES);
  if (jeton) {
    en_tetes.Authorization = `Bearer ${jeton}`;
  }
  return http.request(methode, `${CIBLE}${chemin}`, corps || null, {
    headers: en_tetes,
    tags: { name: `${methode} ${chemin}` },
  });
}

/** Lecture de préparation (hors mesure utile) : corps conservé. */
export function lire(chemin, jeton) {
  return http.get(`${CIBLE}${chemin}`, {
    headers: { Authorization: `Bearer ${jeton}`, Accept: 'application/json' },
    responseType: 'text',
    tags: { name: `préparation ${chemin}` },
  });
}

/** Ouvre une session ; rend la réponse (corps conservé pour le jeton). */
export function seConnecter() {
  return http.post(
    `${CIBLE}/api/v1/auth/login`,
    JSON.stringify({ email: EMAIL, password: MOT_DE_PASSE }),
    { headers: EN_TETES, responseType: 'text', tags: { name: 'POST /api/v1/auth/login' } },
  );
}

/** Extrait le jeton d'une réponse de connexion, ou `null`. */
export function jetonDe(reponse) {
  if (reponse.status !== 200 || !reponse.body) {
    return null;
  }
  try {
    return JSON.parse(reponse.body).token || null;
  } catch (_) {
    return null;
  }
}

// Jeton de l'utilisateur virtuel courant : chaque VU a son propre contexte
// JavaScript, donc sa propre variable.
let jetonDuVu = null;

/**
 * Jeton du VU, obtenu une seule fois. Tant que la connexion échoue,
 * chaque itération la retente (comme les scripts wrk d'origine) et rend
 * `null` : l'itération ne fait alors que cette tentative.
 */
export function jetonUneFoisParVu() {
  if (!jetonDuVu) {
    jetonDuVu = jetonDe(seConnecter());
  }
  return jetonDuVu;
}

/** Tire une entrée d'une table `[poids, ...]` au prorata des poids. */
export function tirer(table) {
  const total = table.reduce((somme, entree) => somme + entree[0], 0);
  let tirage = Math.random() * total;
  for (const entree of table) {
    tirage -= entree[0];
    if (tirage < 0) {
      return entree;
    }
  }
  return table[table.length - 1];
}

function ms(valeur) {
  return valeur === undefined ? 'n/a' : `${valeur.toFixed(2)}ms`;
}

/** `handleSummary` des scénarios : texte lisible + ligne pour Sluis. */
export function resumer(titre, data) {
  const requetes = data.metrics.http_reqs ? data.metrics.http_reqs.values : {};
  const durees = data.metrics.http_req_duration ? data.metrics.http_req_duration.values : {};
  const echecs = data.metrics.http_req_failed ? data.metrics.http_req_failed.values : {};
  const total = requetes.count || 0;
  const taux = echecs.rate || 0;
  const lignes = [
    '------------------------------',
    titre,
    '------------------------------',
    `  Requêtes : ${total}`,
    `  En échec : ${Math.round(total * taux)} (${(taux * 100).toFixed(2)} %)`,
    `  Débit : ${(requetes.rate || 0).toFixed(2)} req/s`,
    '',
    'Latences :',
  ];
  for (const centile of ['p(50)', 'p(75)', 'p(90)', 'p(95)', 'p(99)', 'p(99.9)']) {
    lignes.push(`  ${centile} : ${ms(durees[centile])}`);
  }
  lignes.push('------------------------------');
  const sluis = {
    requetes: total,
    debit: requetes.rate,
    p50: durees['p(50)'],
    p99: durees['p(99)'],
    // Part des requêtes en échec (statut inattendu ou erreur réseau).
    echecs: echecs.rate,
  };
  return {
    stdout: `${lignes.join('\n')}\nSLUIS_RESUME ${JSON.stringify(sluis)}\n`,
  };
}
