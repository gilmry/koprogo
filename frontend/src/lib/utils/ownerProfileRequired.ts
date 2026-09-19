// Issue #781 (RN-11, recette 4 du 2026-09-06) — Helper refus "fiche de
// copropriétaire requise".
//
// `isOwnerProfileRequiredError()` détecte un body 403 `kind:
// "owner_profile_required"`, renvoyé par `create_skill` /
// `create_shared_object` / `create_booking` quand `resolve_owner()` échoue
// (cf. `REFUS_RESERVE_AUX_COPROPRIETAIRES` côté backend). Le refus lui-même
// est légitime — offrir une compétence, prêter un objet ou réserver engage
// une personne nommée — mais le message backend est en français fixe : ce
// `kind` stable permet au frontend de le traduire dans les quatre locales
// sans dépendre du libellé (#555, #762 : la vraie correction reste de typer
// `AppError` de bout en bout).
//
// `showOwnerProfileRequiredToast()` rend un toast traduit. Retourne `false`
// si l'erreur n'est pas celle-ci, pour laisser l'appelant fallback sur son
// message générique existant (même pattern que `conformity.ts` /
// `meetingCompletion.ts`).

import { get } from "svelte/store";
import { _ } from "../i18n";
import { toast } from "../../stores/toast";

/**
 * Type guard — l'erreur reçue est-elle le refus 403
 * `kind: "owner_profile_required"` ?
 *
 * Pattern reconnu, identique à `conformity.ts` / `meetingCompletion.ts` :
 *   - Objet plain avec `kind === "owner_profile_required"`.
 *   - Tolère aussi les wrappers `Error.body` / `Error.response.data` — c'est
 *     la forme réelle d'une `ApiError` levée par `apiFetch` (régression
 *     #782 : une exception nue ne portait jamais le `kind`).
 */
export function isOwnerProfileRequiredError(err: unknown): boolean {
  if (!err || typeof err !== "object") return false;
  const direct = err as Record<string, unknown>;
  if (looksLikeBody(direct)) return true;
  const wrapped =
    (direct.body as Record<string, unknown> | undefined) ??
    ((direct.response as Record<string, unknown> | undefined)?.data as
      Record<string, unknown> | undefined);
  return !!wrapped && looksLikeBody(wrapped);
}

function looksLikeBody(o: Record<string, unknown>): boolean {
  return o.kind === "owner_profile_required";
}

/**
 * Affiche un toast traduit si l'erreur est le refus
 * `owner_profile_required`. Retourne `true` si le toast a été affiché,
 * `false` sinon (caller peut alors fallback sur son message générique).
 */
export function showOwnerProfileRequiredToast(err: unknown): boolean {
  if (!isOwnerProfileRequiredError(err)) return false;
  const tt = get(_);
  const title = tt("skills.createModal.ownerProfileRequiredTitle");
  const message = tt("skills.createModal.ownerProfileRequiredMessage");
  toast.error(`${title} — ${message}`, 8000);
  return true;
}
