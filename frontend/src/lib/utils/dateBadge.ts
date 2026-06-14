// Story B3 (Phase B FE) — helper d'expiration partagé par
// `ExpirationBadge.svelte` (B3 Mandate + B4 RoleDelegation) et
// `SlaBadge.svelte` (B6, sœur sémantique). Anti-duplication : la logique
// countdown vit ICI, pas dans chaque composant (cf. stories.md §B3 notes).
//
// Niveaux (cohérence palette INV-FE9 — coloring + texte + icône daltonien) :
//   fresh   > 30 jours restants    → vert
//   soon    ≤ 30 jours restants    → orange (warning)
//   urgent  ≤ 7  jours restants    → rouge  (alerte)
//   expired < 0  jours (passé)     → gris   (info post-mortem)
//
// Bornes (cf. AC @edge stories.md §B3) :
//   - `daysRemaining === 0` → "Expire aujourd'hui" (urgent, fenêtre TZ locale)
//   - `daysRemaining === 1` → "Expire demain"
//   - `daysRemaining < 0`  → "Expiré"
//
// Gotcha #1 (timezone) — `valid_until` côté backend est `TIMESTAMPTZ` UTC.
// Le badge affiche le delta en TZ locale user. On utilise la différence
// absolue de millisecondes / (1000*60*60*24), arrondi par `Math.ceil` pour
// qu'un délai de quelques heures soit encore considéré "1 jour restant"
// (vs `Math.floor` qui basculerait prématurément à 0).
//
// Pourquoi pas `Intl.RelativeTimeFormat` directement ici : on garde la
// présentation FR statique (KoproGo est FR-prioritaire) ; cette helper
// reste pur métier. L'i18n NL/EN/DE viendra avec une refacto Story B12+.

/**
 * Niveau d'urgence d'expiration — pilote la couleur + l'icône du badge.
 *
 * Ordre lexicographique = ordre temporel décroissant (fresh → expired).
 */
export type ExpirationLevel = "fresh" | "soon" | "urgent" | "expired";

/**
 * Statut calculé pour un `validUntil` donné.
 *
 * - `daysRemaining` : entier signé (négatif si déjà passé).
 * - `level` : palette + sémantique badge.
 * - `label` : texte FR prêt à afficher (i18n hors scope B3).
 */
export interface ExpirationStatus {
  daysRemaining: number;
  level: ExpirationLevel;
  label: string;
}

/**
 * Computes l'écart en jours entre `validUntil` et `now`, arrondi vers le haut.
 *
 * Exemples (now = 2026-06-10 12:00 UTC) :
 *   validUntil="2026-06-10T23:59:59Z" → 1   (11h restantes → arrondi 1j)
 *   validUntil="2026-06-10T11:00:00Z" → -1  (1h passée → expiré J-1 ceil-style)
 *   validUntil="2026-06-11T12:00:00Z" → 1
 *   validUntil="2027-06-10T12:00:00Z" → 365
 *
 * @internal exposé pour testabilité — préférer `expirationStatus`.
 */
export function daysBetween(validUntil: Date | string, now: Date = new Date()): number {
  const target = typeof validUntil === "string" ? new Date(validUntil) : validUntil;
  const deltaMs = target.getTime() - now.getTime();
  return Math.ceil(deltaMs / (1000 * 60 * 60 * 24));
}

/**
 * Détermine le niveau d'urgence selon le nombre de jours restants.
 *
 * Seuils canoniques (cf. stories.md §B3 + architecture.md §4.3) :
 *   > 30  → fresh
 *   ≤ 30  → soon
 *   ≤ 7   → urgent (priorité haute → on teste AVANT soon)
 *   < 0   → expired
 *
 * On test `expired` en premier (gating négatif), puis `urgent`, puis `soon`.
 */
export function levelFromDays(daysRemaining: number): ExpirationLevel {
  if (daysRemaining < 0) return "expired";
  if (daysRemaining <= 7) return "urgent";
  if (daysRemaining <= 30) return "soon";
  return "fresh";
}

/**
 * Compose le label FR à afficher dans le badge.
 *
 * Garantit pluriels corrects + variantes "aujourd'hui" / "demain" pour les
 * bornes 0/1 (cf. AC @edge stories.md §B3).
 */
export function labelFromDays(daysRemaining: number): string {
  if (daysRemaining < 0) return "Expiré";
  if (daysRemaining === 0) return "Expire aujourd'hui";
  if (daysRemaining === 1) return "Expire demain";
  if (daysRemaining <= 60) return `Expire dans ${daysRemaining} jours`;
  // Au-delà de 60j, on bascule en mois pour lisibilité (12 mois > 365 jours).
  const months = Math.round(daysRemaining / 30);
  return months === 1 ? "Expire dans 1 mois" : `Expire dans ${months} mois`;
}

/**
 * Point d'entrée principal. Retourne le triplet `{ daysRemaining, level, label }`
 * pour un `validUntil` ISO 8601 (TIMESTAMPTZ backend) ou Date.
 *
 * Cf. usage `ExpirationBadge.svelte` (composant atomique B3+B4) et future
 * réutilisation `SlaBadge` (B6) — bien que B6 ajoute une sémantique SLA
 * (texte "Réponse due dans X" au lieu de "Expire dans X").
 *
 * @param validUntil ISO 8601 string (`"2027-06-10T00:00:00Z"`) ou Date.
 * @param now Optionnel — injectable pour tests déterministes.
 */
export function expirationStatus(
  validUntil: Date | string,
  now: Date = new Date(),
): ExpirationStatus {
  const daysRemaining = daysBetween(validUntil, now);
  const level = levelFromDays(daysRemaining);
  const label = labelFromDays(daysRemaining);
  return { daysRemaining, level, label };
}
