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

// =============================================================================
// SLA helpers — Story B6 (cf. stories.md §B6 + architecture.md §4.3)
// =============================================================================
//
// Sémantique distincte de l'expiration mandate :
//   - on calcule un % de temps restant relatif à la fenêtre SLA totale
//     (now → dueAt) au lieu d'un nombre de jours absolu.
//   - on tient compte d'une éventuelle `respondedAt` (ticket déjà répondu) →
//     le badge fige son état (`met` ou `breached`) et indique l'écart vs
//     dueAt avec un signe (T-3h = répondu 3h avant l'échéance, T+1h = en
//     dépassement).
//
// Niveaux (cf. mission §SlaBadge) :
//   met     → réponse postée AVANT due_at (respondedAt ≤ dueAt) → vert
//   breached→ pas de réponse + dueAt passé (now > dueAt) → rouge
//   urgent  → pas encore répondu, ≤ 25% temps restant → rouge
//   warning → pas encore répondu, ≤ 50% temps restant → orange
//   fresh   → pas encore répondu, > 50% temps restant → vert

/** Niveau SLA pilote la couleur + l'icône du badge SLA. */
export type SlaLevel = "met" | "fresh" | "warning" | "urgent" | "breached";

/** Statut SLA calculé — réutilisé par `SlaBadge.svelte`. */
export interface SlaStatus {
  /** Niveau visuel (couleur/icône). */
  level: SlaLevel;
  /** Texte FR prêt à afficher. */
  label: string;
  /**
   * Fraction de temps restant ∈ [0, 1].
   * - `respondedAt` fourni → 0 si on dépasse l'échéance, sinon ratio.
   * - Pas de réponse → max(0, (dueAt - now) / (dueAt - createdAt)).
   * Sert au tooltip / data-attr de debug.
   */
  remainingRatio: number;
  /**
   * Différence signée (en heures, arrondie) entre `respondedAt` et `dueAt`.
   * - Négatif : répondu AVANT l'échéance (ex: -3 → "T-3h ✓").
   * - Positif : répondu APRÈS l'échéance (ex: +1 → "T+1h ⚠").
   * - `null` si pas encore de réponse.
   */
  responseDeltaHours: number | null;
}

/**
 * Formate l'écart en heures pour un libellé court "T-Nh" / "T+Nh".
 * @internal exposé pour tests.
 */
export function formatResponseDelta(deltaHours: number): string {
  const absH = Math.abs(deltaHours);
  const sign = deltaHours <= 0 ? "T-" : "T+";
  if (absH < 1) return `${sign}<1h`;
  return `${sign}${absH}h`;
}

/**
 * Compose le statut SLA d'un ticket — pur, déterministe, testable.
 *
 * @param dueAt        ISO 8601 — `sla_due_at` côté backend.
 * @param respondedAt  ISO 8601 ou `null` — `first_response_at` (timestamp
 *                     de la 1re SyndicResponse). `null` = pas répondu.
 * @param createdAt    ISO 8601 — `created_at` du ticket (pour calculer le
 *                     ratio de progression). Si omis, on prend (dueAt - 24h).
 * @param now          Optionnel — injection pour tests déterministes.
 */
export function slaStatus(
  dueAt: Date | string,
  respondedAt: Date | string | null,
  createdAt?: Date | string,
  now: Date = new Date(),
): SlaStatus {
  const dueDate = typeof dueAt === "string" ? new Date(dueAt) : dueAt;
  const createdDate =
    createdAt !== undefined
      ? typeof createdAt === "string"
        ? new Date(createdAt)
        : createdAt
      : new Date(dueDate.getTime() - 24 * 60 * 60 * 1000);

  // ─── Cas 1 : déjà répondu ─────────────────────────────────────────────────
  if (respondedAt !== null) {
    const respondedDate =
      typeof respondedAt === "string" ? new Date(respondedAt) : respondedAt;
    const deltaMs = respondedDate.getTime() - dueDate.getTime();
    // Round to nearest hour (negative if before due).
    const deltaHours = Math.round(deltaMs / (1000 * 60 * 60));
    const respondedBeforeDue = respondedDate.getTime() <= dueDate.getTime();
    const totalWindowMs = dueDate.getTime() - createdDate.getTime();
    const elapsedMs = respondedDate.getTime() - createdDate.getTime();
    const remainingRatio =
      totalWindowMs > 0
        ? Math.max(0, Math.min(1, 1 - elapsedMs / totalWindowMs))
        : 0;

    if (respondedBeforeDue) {
      return {
        level: "met",
        label: `Réponse postée à ${formatResponseDelta(deltaHours)} ✓`,
        remainingRatio,
        responseDeltaHours: deltaHours,
      };
    }
    // Réponse hors SLA — on garde le niveau "breached" pour rappeler le miss.
    return {
      level: "breached",
      label: `Hors SLA — réponse à ${formatResponseDelta(deltaHours)}`,
      remainingRatio: 0,
      responseDeltaHours: deltaHours,
    };
  }

  // ─── Cas 2 : pas encore répondu ──────────────────────────────────────────
  const totalWindowMs = dueDate.getTime() - createdDate.getTime();
  const remainingMs = dueDate.getTime() - now.getTime();
  const remainingRatio =
    totalWindowMs > 0
      ? Math.max(0, Math.min(1, remainingMs / totalWindowMs))
      : 0;

  // dueAt déjà passé → breached.
  if (remainingMs <= 0) {
    const overdueHours = Math.round(-remainingMs / (1000 * 60 * 60));
    return {
      level: "breached",
      label: `Hors SLA — échéance T+${overdueHours}h`,
      remainingRatio: 0,
      responseDeltaHours: null,
    };
  }

  const remainingHours = Math.max(1, Math.round(remainingMs / (1000 * 60 * 60)));
  const remainingLabel =
    remainingHours < 24
      ? `${remainingHours}h`
      : `${Math.round(remainingHours / 24)}j`;

  // 25% / 50% buckets — INV: urgent (≤ 25%) testé AVANT warning (≤ 50%).
  if (remainingRatio <= 0.25) {
    return {
      level: "urgent",
      label: `Échéance dans ${remainingLabel} ⚠`,
      remainingRatio,
      responseDeltaHours: null,
    };
  }
  if (remainingRatio <= 0.5) {
    return {
      level: "warning",
      label: `Échéance dans ${remainingLabel}`,
      remainingRatio,
      responseDeltaHours: null,
    };
  }
  return {
    level: "fresh",
    label: `Sous SLA (${remainingLabel})`,
    remainingRatio,
    responseDeltaHours: null,
  };
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
