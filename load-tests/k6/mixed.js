// Scénario « mixed » : lectures pondérées, sans session.
// Équivalent de l'ancien lua/mixed.lua. Sans jeton, toutes les routes
// sauf /health répondent 401 : ce scénario mesure le chemin de refus, et
// 401 y est donc un statut attendu. Pour la charge réelle, voir
// authenticated-mixed.js.
import http from 'k6/http';
import { envoyer, OPTIONS_COMMUNES, resumer, tirer } from './commun.js';
import { LECTURES } from './lectures.js';

export const options = Object.assign({}, OPTIONS_COMMUNES);

http.setResponseCallback(http.expectedStatuses(200, 401));

export default function () {
  const [, methode, chemin] = tirer(LECTURES);
  envoyer(methode, chemin);
}

export function handleSummary(data) {
  return resumer('Lectures mixtes, sans session', data);
}
