// Scénario « health » : GET /api/v1/health en boucle. Sert de chauffe avant
// une campagne, et de plancher (le chemin le plus court de l'API).
import { envoyer, OPTIONS_COMMUNES, resumer } from './commun.js';

export const options = Object.assign({}, OPTIONS_COMMUNES);

export default function () {
  envoyer('GET', '/api/v1/health');
}

export function handleSummary(data) {
  return resumer('Santé (GET /health)', data);
}
