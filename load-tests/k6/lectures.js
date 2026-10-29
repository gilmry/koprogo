// Table de lectures pondérées, partagée par « mixed » et « authenticated-mixed ».
// Mêmes poids que les scripts wrk d'origine : santé 5, immeubles 40, lots 25,
// copropriétaires 15, dépenses 10, assemblées 5.
export const LECTURES = [
  [5, 'GET', '/api/v1/health'],
  [40, 'GET', '/api/v1/buildings'],
  [25, 'GET', '/api/v1/units'],
  [15, 'GET', '/api/v1/owners'],
  [10, 'GET', '/api/v1/expenses'],
  [5, 'GET', '/api/v1/meetings'],
];
