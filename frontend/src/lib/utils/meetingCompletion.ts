// Track H Story H3 — Helpers Meeting completion (Art. 3.87 §3-5 CC).
//
// `isMeetingCompletionError()` détecte un body 422 `MEETING_NOT_COMPLETABLE`.
// `extractMeetingCompletionPayload()` extrait le payload structuré (missing[]).
// `showMeetingCompletionToast()` rend un toast i18n narratif via le store
// `toast`. Mémoire `validate-before-compute` + DoD-H3.
//
// Decimal-as-string : les quotas restent strings, jamais parseFloat
// (mémoire `no-f64-in-money` + ADR-0007).

import { get } from "svelte/store";
import { _ } from "../i18n";
import { toast } from "../../stores/toast";
import type {
  MeetingNotCompletableErrorBody,
  MeetingNotCompletablePayload,
} from "../types/meeting";

/**
 * Type guard — l'erreur reçue est-elle un body 422 `MEETING_NOT_COMPLETABLE` ?
 *
 * Pattern reconnu :
 *   - Objet plain avec `kind === "meeting_not_completable"` ET
 *     `details.code === "MEETING_NOT_COMPLETABLE"`.
 *   - Tolère aussi `Error` wrappers qui exposent `body` ou `response.data`.
 */
export function isMeetingCompletionError(
  err: unknown,
): err is MeetingNotCompletableErrorBody {
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
  if (o.kind !== "meeting_not_completable") return false;
  const details = o.details as Record<string, unknown> | undefined;
  if (!details) return false;
  return details.code === "MEETING_NOT_COMPLETABLE";
}

/**
 * Extrait le payload narratif d'une erreur de meeting completion (le caller
 * doit avoir déjà vérifié via `isMeetingCompletionError()` — ce helper
 * renvoie `null` si la forme ne matche pas).
 */
export function extractMeetingCompletionPayload(
  err: MeetingNotCompletableErrorBody | Record<string, unknown>,
): MeetingNotCompletablePayload | null {
  const candidate =
    (err as { details?: MeetingNotCompletablePayload }).details ??
    (err as { body?: { details?: MeetingNotCompletablePayload } }).body
      ?.details ??
    (
      err as {
        response?: { data?: { details?: MeetingNotCompletablePayload } };
      }
    ).response?.data?.details;
  return candidate && candidate.code === "MEETING_NOT_COMPLETABLE"
    ? candidate
    : null;
}

/**
 * Affiche un toast narratif si l'erreur est un `MEETING_NOT_COMPLETABLE`.
 * Retourne `true` si le toast a été affiché, `false` sinon (caller peut
 * alors fallback sur toast générique).
 *
 * Le store `toast` du projet prend une string — on assemble title + message
 * en un seul libellé i18n (cf. `meeting.complete.toast_*`).
 */
export function showMeetingCompletionToast(err: unknown): boolean {
  if (!isMeetingCompletionError(err)) return false;
  const payload = extractMeetingCompletionPayload(err);
  if (!payload) return false;
  const tt = get(_);
  const n = payload.missing.length;
  const title = tt("meeting.complete.toast_title");
  const message = tt("meeting.complete.toast_message", { values: { n } });
  // duration 8s — narratif, le user doit lire la liste des conditions.
  toast.error(`${title} — ${message}`, 8000);
  return true;
}
