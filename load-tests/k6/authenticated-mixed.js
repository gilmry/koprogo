// Scénario « authenticated-mixed » : lectures pondérées, avec session.
// Équivalent de l'ancien lua/authenticated-mixed.lua : chaque utilisateur
// virtuel se connecte une fois, puis réutilise son jeton.
import { envoyer, jetonUneFoisParVu, OPTIONS_COMMUNES, resumer, tirer } from './commun.js';
import { LECTURES } from './lectures.js';

export const options = Object.assign({}, OPTIONS_COMMUNES);

export default function () {
  const jeton = jetonUneFoisParVu();
  if (!jeton) {
    return;
  }
  const [, methode, chemin] = tirer(LECTURES);
  envoyer(methode, chemin, null, jeton);
}

export function handleSummary(data) {
  return resumer('Lectures mixtes, avec session', data);
}
