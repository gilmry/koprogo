/**
 * Le jeu d'icônes du produit — un seul, et une seule par destination.
 *
 * ── Pourquoi remplacer les émojis ────────────────────────────────────────
 *
 * Les émojis étaient le système d'icônes de toute la navigation. La remise de
 * design en dresse quatre reproches, et les trois premiers sont vérifiables :
 *
 * 1. **Ils se rendent différemment selon le système** — un émoji est une
 *    police, pas un dessin. Le produit n'a donc pas la même allure sur macOS,
 *    Windows et Android, et rien ne le contrôle.
 * 2. **Ils ne s'accordent ni en taille ni en graisse de trait** avec quoi que
 *    ce soit d'autre. Un jeu SVG à `stroke-width` constant, si.
 * 3. **Ils sont annoncés par les lecteurs d'écran.** « bâtiment Immeubles »
 *    plutôt que « Immeubles ». Une icône décorative doit être `aria-hidden`,
 *    ce qu'un caractère ne peut pas être.
 * 4. **Ils entrent en collision.** Mesuré sur `Navigation.svelte` :
 *    42 entrées de menu pour 33 icônes distinctes, dont **six collisions
 *    réelles** entre destinations différentes :
 *
 *        📊  budgets / sondages
 *        📅  assemblées / réservations
 *        📋  états datés / devis
 *        📈  rapports PCMN / supervision
 *        🎫  tickets / mes tickets
 *        👤  copropriétaires / profil
 *
 *    Deux entrées qui portent le même signe ne se distinguent plus que par
 *    leur libellé : l'icône cesse d'informer et devient du bruit.
 *
 * ── La forme, imposée par la remise ──────────────────────────────────────
 *
 * `viewBox` 24×24, `fill="none"`, `stroke="currentColor"`, `stroke-width` 1.7
 * (2.0 à 2.2 pour les chevrons et les icônes d'onglet actif), extrémités et
 * jointures rondes. Rendues à 18–19px en navigation, 15–17px en ligne, 23px
 * dans les barres d'onglets.
 *
 * Chaque icône est décorative : `aria-hidden="true"`, et le nom accessible
 * vit sur le contrôle qui l'entoure. C'est aussi ce qui laisse
 * `getByRole("button", { name })` continuer de fonctionner dans les recettes.
 *
 * Tracés dérivés de Heroicons (MIT), que la remise désigne comme source
 * acceptable et que le dépôt emploie déjà en ligne par endroits.
 */

/** Un tracé, ou plusieurs, d'une icône 24×24. */
export type Icone = string[];

/**
 * Une icône par destination, sans réemploi.
 *
 * Les clés reprennent celles de `navigation.*` dans les fichiers de langue,
 * pour que le rapprochement entre une entrée de menu et son icône se lise
 * sans table de correspondance.
 */
export const ICONES: Record<string, Icone> = {
  // ── Repères généraux ──────────────────────────────────────────────────
  today: ["M12 6v6l4 2", "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18z"],
  dashboard: [
    "M4 13h6V4H4v9z",
    "M14 20h6v-9h-6v9z",
    "M4 20h6v-4H4v4z",
    "M14 8h6V4h-6v4z",
  ],

  // ── Gestion ───────────────────────────────────────────────────────────
  acps: ["M3 21h18", "M6 21V7l6-4 6 4v14", "M10 21v-5h4v5"],
  buildings: [
    "M3 21h18",
    "M5 21V5h9v16",
    "M14 21V9h5v12",
    "M8 9h3M8 13h3M8 17h3",
  ],
  units: ["M4 21V8l8-5 8 5v13", "M9 21v-6h6v6", "M15 11h.01"],
  owners: [
    "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2",
    "M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z",
    "M22 21v-2a4 4 0 0 0-3-3.87",
  ],
  callForFunds: [
    "M3 7h18v12a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V7z",
    "M3 7l3-4h12l3 4",
    "M12 11v5M9.5 13h5",
  ],
  contributions: [
    "M12 3v18",
    "M17 7H9.5a2.5 2.5 0 0 0 0 5h5a2.5 2.5 0 0 1 0 5H6",
  ],
  reminders: [
    "M3 6h13v9H3z",
    "M3 7l6.5 4.5L16 7",
    "M17 19h5M19.5 16.5L22 19l-2.5 2.5",
  ],

  // ── Comptabilité ──────────────────────────────────────────────────────
  expenses: ["M3 12h18", "M3 6h18", "M3 18h10", "M17 15l3 3-3 3"],
  invoiceWorkflow: [
    "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z",
    "M14 2v6h6",
    "M9 15l2 2 4-4",
  ],
  budgets: ["M3 20h18", "M6 20v-7", "M11 20V8", "M16 20v-4", "M21 20V4"],
  etatsDates: [
    "M8 2h8a2 2 0 0 1 2 2v16a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2z",
    "M10 7h4M10 11h4M10 15h2",
  ],
  journalEntries: ["M4 5h16v14H4z", "M4 10h16", "M10 10v9", "M4 15h6"],
  reportsPcmn: ["M12 3v9l7 4", "M12 3a9 9 0 1 1-8.7 11.3"],

  // ── Gouvernance ───────────────────────────────────────────────────────
  meetings: ["M3 5h18v16H3z", "M3 10h18", "M8 3v4M16 3v4", "M8 15h2M14 15h2"],
  convocations: ["M3 6h18v12H3z", "M3 7l9 6 9-6"],
  board: ["M12 2l8 4v6c0 5-3.5 8.5-8 10-4.5-1.5-8-5-8-10V6z", "M9 12l2 2 4-4"],
  documents: [
    "M13 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V9z",
    "M13 3v6h6",
  ],

  // ── Interventions ─────────────────────────────────────────────────────
  tickets: [
    "M4 8a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v2a2 2 0 0 0 0 4v2a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-2a2 2 0 0 0 0-4V8z",
    "M14 6v12",
  ],
  myTickets: [
    "M4 8a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v2a2 2 0 0 0 0 4v2a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-2a2 2 0 0 0 0-4V8z",
    "M9 11l2 2 4-4",
  ],
  quotes: ["M5 4h11l3 3v13H5z", "M9 9h6M9 13h6M9 17h3"],
  workReports: ["M4 4h16v16H4z", "M8 9h8M8 13h8M8 17h4", "M9 2v4M15 2v4"],
  inspections: [
    "M9 3h6v3H9z",
    "M15 4h2a2 2 0 0 1 2 2v6",
    "M9 4H7a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2h5",
    "M17 14a3 3 0 1 0 0 6 3 3 0 0 0 0-6z",
    "M21.5 21.5L19.2 19.2",
  ],

  // ── Communauté ────────────────────────────────────────────────────────
  localExchanges: ["M4 8h13l-3-3", "M20 16H7l3 3"],
  polls: ["M9 20V10", "M15 20V4", "M3 20h18", "M21 20V14"],
  notices: ["M4 4h16v12H4z", "M8 20h8", "M12 16v4", "M8 9h8"],
  bookings: [
    "M3 5h18v16H3z",
    "M3 10h18",
    "M8 3v4M16 3v4",
    "M11 14l1.5 1.5L16 12",
  ],
  sharing: [
    "M18 8a3 3 0 1 0 0-6 3 3 0 0 0 0 6z",
    "M6 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z",
    "M18 22a3 3 0 1 0 0-6 3 3 0 0 0 0 6z",
    "M8.6 13.5l6.8 4M15.4 6.5l-6.8 4",
  ],
  skills: [
    "M12 3l2.6 5.5 6 .9-4.3 4.3 1 6.1-5.3-2.9L6.7 19.8l1-6.1L3.4 9.4l6-.9z",
  ],
  energy: ["M13 2L4 14h7l-1 8 9-12h-7z"],
  gamification: [
    "M7 4h10v5a5 5 0 0 1-10 0z",
    "M7 6H4v1a3 3 0 0 0 3 3M17 6h3v1a3 3 0 0 1-3 3",
    "M9 20h6M12 14v6",
  ],

  // ── Administration ────────────────────────────────────────────────────
  organizations: [
    "M3 21h18",
    "M4 21V3h10v18",
    "M14 21V10h6v11",
    "M7 7h4M7 11h4M7 15h4",
  ],
  users: [
    "M17 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2",
    "M9.5 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z",
  ],
  gdpr: [
    "M12 2l8 4v6c0 5-3.5 8.5-8 10-4.5-1.5-8-5-8-10V6z",
    "M12 11v4M12 8h.01",
  ],
  monitoring: ["M3 12h4l2.5-6 4 12L16 12h5"],
  auditLog: ["M5 4h14v16H5z", "M9 8h6M9 12h6M9 16h3", "M17 17l2 2 3-3"],
  subscriptions: ["M3 8h18v11H3z", "M3 12h18", "M7 16h4"],

  // ── Destinations restantes ────────────────────────────────────────────
  //
  // `sel` et `sharing_short` désignent des destinations distinctes de
  // `localExchanges` et `sharing` dans certains menus de rôle. Leur donner le
  // même tracé rouvrirait la collision que cette refonte ferme : chacune a
  // le sien.
  sel: ["M12 3v18", "M8 7l4-4 4 4", "M16 17l-4 4-4-4", "M3 12h18"],
  sharing_short: [
    "M12 3v12",
    "M8 7l4-4 4 4",
    "M4 15v4a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-4",
  ],
  payments: ["M2 7h20v10H2z", "M2 11h20", "M6 15h3"],
  paymentMethods: [
    "M2 7h20v10H2z",
    "M2 11h20",
    "M17 15h3",
    "M6 15h2",
    "M11 15h2",
  ],
  works: [
    "M14.7 6.3a4 4 0 0 1-5.4 5.4L4 17v3h3l5.3-5.3a4 4 0 0 1 5.4-5.4z",
    "M14.7 6.3l3-3 3 3-3 3z",
  ],
  council: [
    "M9 8a3 3 0 1 0 0-6 3 3 0 0 0 0 6z",
    "M17 9a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z",
    "M2 20v-2a5 5 0 0 1 5-5h4a5 5 0 0 1 5 5v2",
    "M18 13a4 4 0 0 1 4 4v3",
  ],
  admin: [
    "M12 2l7 3v6c0 4.5-3 8-7 9-4-1-7-4.5-7-9V5z",
    "M12 11.5a1.8 1.8 0 1 0 0-3.6 1.8 1.8 0 0 0 0 3.6z",
    "M12 11.5V15",
  ],

  // ── Compte et service ─────────────────────────────────────────────────
  profile: ["M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8z", "M5 21a7 7 0 0 1 14 0"],
  settings: [
    "M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7z",
    "M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-2.9 1.2V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-2.9-1.2l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0-1.2-2.9H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.2-2.9l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 2.9-1.2V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 2.9 1.2l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0 1.2 2.9H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z",
  ],
  logout: [
    "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4",
    "M16 17l5-5-5-5",
    "M21 12H9",
  ],
  notificationBell: [
    "M18 8a6 6 0 1 0-12 0c0 7-3 9-3 9h18s-3-2-3-9",
    "M13.7 21a2 2 0 0 1-3.4 0",
  ],
  search: ["M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14z", "M20 20l-4.5-4.5"],

  // ── Actions de ligne ──────────────────────────────────────────────────
  //
  // Elles remplacent ✏️ et 🗑️, dont la zone de tap effective était d'environ
  // 20 px — un bouton sans remplissage autour d'un caractère. La cible
  // minimale est de 44 px.
  edit: ["M4 20h4l10.5-10.5a2.8 2.8 0 1 0-4-4L4 16z", "M13.5 6.5l4 4"],
  trash: [
    "M4 7h16",
    "M10 4h4a1 1 0 0 1 1 1v2H9V5a1 1 0 0 1 1-1z",
    "M6 7l1 13a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-13",
    "M10 11v7M14 11v7",
  ],

  // ── Signes d'état et de direction ─────────────────────────────────────
  chevronRight: ["M9 6l6 6-6 6"],
  chevronDown: ["M6 9l6 6 6-6"],
  legalScale: [
    "M12 3v18",
    "M7 21h10",
    "M5 7h14",
    "M5 7l-2.5 6a3 3 0 0 0 5 0z",
    "M19 7l2.5 6a3 3 0 0 1-5 0z",
  ],
  alert: [
    "M12 9v4",
    "M12 17h.01",
    "M10.3 3.9L1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z",
  ],
  attachment: [
    "M21.4 11.1l-9.2 9.2a5 5 0 0 1-7.1-7.1l9.2-9.2a3.3 3.3 0 1 1 4.7 4.7l-9.2 9.2a1.7 1.7 0 0 1-2.4-2.4l8.5-8.5",
  ],
};

/** Le nom d'une icône connue. */
export type NomDIcone = keyof typeof ICONES;

/**
 * L'icône existe-t-elle ? Utile aux gardes et aux tests, qui vérifient qu'une
 * entrée de menu n'en réclame pas une qui manque — auquel cas elle ne rendrait
 * rien du tout, en silence.
 */
export function iconeConnue(nom: string): boolean {
  return Object.prototype.hasOwnProperty.call(ICONES, nom);
}
