// Scénario « auth » : connexions répétées, sans jeton réutilisé.
// Mesure le coût d'une ouverture de session (hachage du mot de passe compris).
// Équivalent de l'ancien lua/auth.lua.
import { OPTIONS_COMMUNES, resumer, seConnecter } from './commun.js';

export const options = Object.assign({}, OPTIONS_COMMUNES);

export default function () {
  seConnecter();
}

export function handleSummary(data) {
  return resumer('Connexion (POST /auth/login)', data);
}
