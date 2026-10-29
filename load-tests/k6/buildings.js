// Scénario « buildings » : lecture de la liste des immeubles, sans session.
// Rotation fixe de 7 GET /buildings pour 2 GET /health, comme l'ancien
// lua/buildings.lua. Sans jeton, /buildings répond 401 : ce scénario mesure
// le chemin de refus, et 401 y est donc un statut attendu.
import http from 'k6/http';
import { envoyer, OPTIONS_COMMUNES, resumer } from './commun.js';

export const options = Object.assign({}, OPTIONS_COMMUNES);

http.setResponseCallback(http.expectedStatuses(200, 401));

const CHEMINS = [
  '/api/v1/buildings',
  '/api/v1/buildings',
  '/api/v1/buildings',
  '/api/v1/buildings',
  '/api/v1/buildings',
  '/api/v1/buildings',
  '/api/v1/buildings',
  '/api/v1/health',
  '/api/v1/health',
];

let rang = 0;

export default function () {
  envoyer('GET', CHEMINS[rang]);
  rang = (rang + 1) % CHEMINS.length;
}

export function handleSummary(data) {
  return resumer('Immeubles, sans session (7 GET /buildings : 2 GET /health)', data);
}
