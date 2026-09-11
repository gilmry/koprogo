import { UserRole } from "./types";

/**
 * Les onglets du bas, par rôle — ce qu'un pouce atteint sans effort.
 *
 * ── Ce que le tiroir coûtait ─────────────────────────────────────────────
 *
 * Mesuré sur `Navigation.svelte` : **trente-neuf destinations distinctes**
 * derrière un seul tap, dans le coin supérieur gauche — le moins atteignable
 * au pouce sur un téléphone tenu d'une main.
 *
 * Le problème n'est pas seulement l'atteinte. Un tiroir fermé ne donne
 * **aucun sens du lieu** : l'écran ne dit pas où l'on est, ni ce qu'il y a
 * ailleurs. Une barre d'onglets montre les deux en permanence, au prix de
 * cinq entrées.
 *
 * ── Pourquoi cinq, et pourquoi celles-là ─────────────────────────────────
 *
 * Cinq est le maximum tenable à 390 px sans descendre sous la cible de 44 px.
 * Le choix des quatre premières suit une règle simple : ce qu'on ouvre
 * plusieurs fois par semaine. Le reste va dans « Plus », qui n'est pas un
 * tiroir déguisé mais une **destination** — un écran de menu groupé, avec ses
 * compteurs.
 *
 * Les rôles ne partagent pas les mêmes gestes quotidiens :
 *
 * - le **syndic** vit dans ses échéances et ses charges ;
 * - le **copropriétaire** ouvre l'application pour payer et pour son
 *   assemblée, et c'est le seul rôle qui n'ouvrira jamais un bureau ;
 * - le **comptable** est légitimement desktop-first : son mobile est de la
 *   consultation, pas de la saisie. Rapprocher des écritures sur 390 px
 *   serait une promesse malhonnête ;
 * - l'**administrateur** supervise depuis son téléphone et administre depuis
 *   son poste.
 *
 * ── Le contrat de test ───────────────────────────────────────────────────
 *
 * Chaque onglet porte `tabbar-{cle}`. Ces ancrages sont neufs : ils
 * n'entrent en conflit avec aucun `navigation-menu-*` existant, et
 * `Navigation.test.ts` compte les menus métier du syndic — un onglet ne doit
 * donc JAMAIS recevoir un ancrage en `navigation-menu-*`.
 */
export interface OngletMobile {
  /** Clé stable — sert d'ancrage de recette et de clé de langue. */
  cle: string;
  href: string;
  /** Nom d'un tracé de `lib/icones.ts`. */
  icone: string;
  /** Clé i18n du libellé. Court : 10,5px sous une icône de 23px. */
  libelle: string;
}

/** Le dernier onglet, commun à tous : ce que la barre ne montre pas. */
const PLUS: OngletMobile = {
  cle: "plus",
  href: "/menu",
  icone: "dashboard",
  libelle: "navigation.more",
};

const PAR_ROLE: Record<string, OngletMobile[]> = {
  [UserRole.SYNDIC]: [
    {
      cle: "today",
      href: "/syndic",
      icone: "today",
      libelle: "navigation.today",
    },
    // « Mes ACP » envoyait le syndic sur `/admin/acps`, que
    // `canAccessRoute("/admin/acps", SYNDIC)` refuse : `RouteGuard` le
    // renvoyait aussitôt sur `/syndic`. L'onglet peignait donc sa page une
    // fraction de seconde avant de rebondir.
    //
    // C'était deux fois redondant : la table « Mes ACP » vit déjà sur le
    // tableau de bord, c'est-à-dire sur le PREMIER onglet.
    //
    // Vérifier qu'une page existe ne suffit pas — il faut vérifier que le rôle
    // peut l'ouvrir. C'est ce que la correction des neuf destinations avait
    // manqué, et ce que le banc mobile a fini par voir.
    {
      cle: "buildings",
      href: "/buildings",
      icone: "buildings",
      libelle: "navigation.buildings",
    },
    {
      cle: "expenses",
      href: "/expenses",
      icone: "expenses",
      libelle: "navigation.expenses",
    },
    {
      cle: "meetings",
      href: "/meetings",
      icone: "meetings",
      libelle: "navigation.meetings",
    },
    PLUS,
  ],
  [UserRole.OWNER]: [
    {
      cle: "today",
      href: "/owner",
      icone: "today",
      libelle: "navigation.home",
    },
    // Un copropriétaire dit « mes charges », jamais « mes dépenses ».
    {
      cle: "expenses",
      href: "/owner/expenses",
      icone: "expenses",
      libelle: "navigation.myExpenses",
    },
    {
      cle: "meetings",
      href: "/meetings",
      icone: "meetings",
      libelle: "navigation.meetings",
    },
    // « Voisinage » plutôt que « Communauté » : c'est le mot qu'un habitant
    // emploie. Personne n'appelle son palier une communauté.
    {
      cle: "community",
      href: "/exchanges",
      icone: "localExchanges",
      libelle: "navigation.neighbourhood",
    },
    PLUS,
  ],
  [UserRole.ACCOUNTANT]: [
    // Consultation seulement : trésorerie et impayés. La seule écriture
    // tolérée sur mobile est la relance groupée, qui vit dans « Impayés ».
    {
      cle: "today",
      href: "/accountant",
      icone: "today",
      libelle: "navigation.today",
    },
    {
      cle: "treasury",
      href: "/budgets",
      icone: "budgets",
      libelle: "navigation.treasury",
    },
    {
      cle: "arrears",
      href: "/payment-reminders",
      icone: "reminders",
      libelle: "navigation.arrears",
    },
    {
      cle: "reports",
      href: "/reports",
      icone: "reportsPcmn",
      libelle: "navigation.reportsPcmn",
    },
    PLUS,
  ],
  [UserRole.SUPERADMIN]: [
    // Supervision seule. La gestion des organisations, les exports RGPD et
    // les outils de test restent réservés au poste de travail — et l'écran
    // le DIT, plutôt que de laisser découvrir l'absence.
    {
      cle: "health",
      href: "/admin/monitoring",
      icone: "monitoring",
      libelle: "navigation.health",
    },
    {
      cle: "organizations",
      href: "/admin/organizations",
      icone: "organizations",
      libelle: "navigation.organizations",
    },
    {
      cle: "users",
      href: "/admin/users",
      icone: "users",
      libelle: "navigation.users",
    },
    {
      cle: "auditLog",
      href: "/admin/gdpr",
      icone: "auditLog",
      libelle: "navigation.auditLog",
    },
    PLUS,
  ],
};

/**
 * Les onglets d'un rôle, ou une liste vide s'il n'en a pas.
 *
 * Vide et non « les onglets par défaut » : afficher une barre générique à un
 * rôle inconnu proposerait des destinations que `RouteGuard` refuserait en
 * silence — l'utilisateur taperait et reviendrait d'où il vient, sans
 * message. C'est le défaut que `garde-tuiles-interdites` traque déjà sur les
 * tableaux de bord.
 */
export function ongletsPour(role: string | null | undefined): OngletMobile[] {
  if (!role || typeof role !== "string") return [];
  return PAR_ROLE[role] ?? [];
}

/** Tous les rôles qui ont une barre d'onglets. Sert aux gardes et aux tests. */
export function rolesAvecOnglets(): string[] {
  return Object.keys(PAR_ROLE);
}
