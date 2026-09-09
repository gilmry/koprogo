/**
 * Les identifiants du superadministrateur, en UN seul endroit.
 *
 * `admin123` était écrit en dur à quarante-cinq endroits de la suite, alors
 * que ce mot de passe a été tourné le 2026-09-06 (#763). Le repli du seed
 * joue tant que la CI ne définit pas `KOPROGO_SUPERADMIN_PASSWORD` — **le
 * jour où elle le fera, toute la suite e2e tombera d'un coup**, et aucun
 * message d'erreur ne parlera du mot de passe.
 *
 * L'issue #832 nommait ce risque et le chiffrait : « un travail de cinq
 * minutes qui évitera une demi-journée d'enquête ». Le voici.
 *
 * La valeur par défaut reste celle du seed, pour ne rien changer aujourd'hui.
 * Le jour de la bascule, une variable d'environnement suffira.
 */
export const ADMIN_EMAIL =
  process.env.KOPROGO_SUPERADMIN_EMAIL || "admin@koprogo.com";

export const ADMIN_PASSWORD =
  process.env.KOPROGO_SUPERADMIN_PASSWORD || "admin123";
