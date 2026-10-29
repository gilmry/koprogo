// Scénario « authenticated-realistic » : 80 % de lectures, 20 % d'écritures,
// avec session. Équivalent de l'ancien lua/authenticated-realistic.lua, mêmes
// poids. Différences voulues :
// - l'immeuble visé est lu sur la cible au démarrage (`setup`), au lieu d'un
//   identifiant figé dans le script, qui ne valait que pour une base donnée ;
// - les dates sont calculées (assemblées dans le futur), pas écrites en dur ;
// - l'organisation n'est plus envoyée : l'API la tire du jeton.
import exec from 'k6/execution';
import {
  envoyer,
  jetonDe,
  jetonUneFoisParVu,
  lire,
  OPTIONS_COMMUNES,
  resumer,
  seConnecter,
  tirer,
} from './commun.js';

export const options = Object.assign({}, OPTIONS_COMMUNES);

const OPERATIONS = [
  // 80 % de lectures
  [5, 'GET', '/api/v1/health'],
  [30, 'GET', '/api/v1/buildings'],
  [20, 'GET', '/api/v1/units'],
  [12, 'GET', '/api/v1/owners'],
  [10, 'GET', '/api/v1/expenses'],
  [3, 'GET', '/api/v1/meetings'],
  // 20 % d'écritures
  [4, 'POST', '/api/v1/expenses', 'depense_entretien'],
  [2, 'POST', '/api/v1/expenses', 'depense_charges'],
  [2, 'POST', '/api/v1/expenses', 'depense_assurance'],
  [3, 'POST', '/api/v1/owners', 'proprietaire_jean'],
  [2, 'POST', '/api/v1/owners', 'proprietaire_marie'],
  [2, 'POST', '/api/v1/meetings', 'assemblee_ordinaire'],
  [2, 'POST', '/api/v1/meetings', 'assemblee_extraordinaire'],
  [2, 'POST', '/api/v1/units', 'lot_appartement'],
  [1, 'POST', '/api/v1/units', 'lot_parking'],
];

export function setup() {
  const jeton = jetonDe(seConnecter());
  if (!jeton) {
    exec.test.abort('connexion impossible : vérifier BASE_URL et le compte de démo');
  }
  const reponse = lire('/api/v1/buildings', jeton);
  let immeubles = [];
  try {
    immeubles = JSON.parse(reponse.body).data || [];
  } catch (_) {
    immeubles = [];
  }
  if (reponse.status !== 200 || immeubles.length === 0) {
    exec.test.abort('aucun immeuble lisible pour ce compte : semer la base de démo');
  }
  return { immeuble: immeubles[0].id };
}

function unique() {
  return `${Date.now()}${Math.floor(Math.random() * 1e8)}`;
}

function auJour(decalageJours) {
  const date = new Date(Date.now() + decalageJours * 86400000);
  return date.toISOString().slice(0, 10) + 'T00:00:00Z';
}

function hasard(min, max) {
  return min + Math.floor(Math.random() * (max - min + 1));
}

function corps(type, immeuble) {
  const id = unique();
  const depense = (categorie, description, montant, fournisseur, prefixe) => ({
    building_id: immeuble,
    category: categorie,
    description,
    amount: montant,
    expense_date: auJour(0),
    supplier: fournisseur,
    invoice_number: `${prefixe}-${id}`,
  });
  const proprietaire = (prenom, nom, telephone, adresse) => ({
    first_name: prenom,
    last_name: nom,
    email: `${prenom.toLowerCase()}.${nom.toLowerCase()}.${id}@example.com`,
    phone: telephone,
    address: adresse,
    city: 'Bruxelles',
    postal_code: '1000',
    country: 'Belgique',
  });
  const assemblee = (genre, titre, description, jours, lieu) => ({
    building_id: immeuble,
    meeting_type: genre,
    title: titre,
    description,
    scheduled_date: auJour(jours).replace('T00:00:00Z', 'T14:00:00Z'),
    location: lieu,
  });
  switch (type) {
    case 'depense_entretien':
      return depense('Maintenance', 'Entretien mensuel', 150.5, 'Maintenance Plus', 'INV');
    case 'depense_charges':
      return depense('Utilities', 'Eau et électricité', 250.0, 'Energy SA', 'ENE');
    case 'depense_assurance':
      return depense('Insurance', 'Assurance copropriété', 500.0, 'Assur Corp', 'ASS');
    case 'proprietaire_jean':
      return proprietaire('Jean', 'Martin', '+32499123456', 'Avenue Louise 123');
    case 'proprietaire_marie':
      return proprietaire('Marie', 'Dubois', '+32477654321', 'Rue Royale 45');
    case 'assemblee_ordinaire':
      return assemblee('Ordinary', 'Assemblée Générale Ordinaire', 'AG annuelle', 60, 'Salle communale');
    case 'assemblee_extraordinaire':
      return assemblee('Extraordinary', 'AG Extraordinaire', 'Travaux urgents', 30, 'Salle polyvalente');
    case 'lot_appartement':
      return {
        building_id: immeuble,
        unit_number: `T${id}`,
        unit_type: 'Apartment',
        floor: hasard(1, 5),
        surface_area: hasard(50, 120) + 0.5,
        quota: hasard(1, 20),
      };
    case 'lot_parking':
      return {
        building_id: immeuble,
        unit_number: `P${id}`,
        unit_type: 'Parking',
        floor: -1,
        surface_area: 12.5,
        quota: 1,
      };
    default:
      return null;
  }
}

export default function (donnees) {
  const jeton = jetonUneFoisParVu();
  if (!jeton) {
    return;
  }
  const [, methode, chemin, type] = tirer(OPERATIONS);
  const charge = type ? JSON.stringify(corps(type, donnees.immeuble)) : null;
  envoyer(methode, chemin, charge, jeton);
}

export function handleSummary(data) {
  return resumer('Charge réaliste, avec session (80 % lectures / 20 % écritures)', data);
}
