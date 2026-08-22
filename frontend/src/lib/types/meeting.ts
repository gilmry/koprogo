// Track H Story H3 — Types Meeting completion checklist (Art. 3.87 §3-5 CC).
//
// Le BE expose `GET /meetings/{id}/completion-checklist` qui retourne la
// checklist + la liste typée des invariants manquants. Le payload 422 sur
// `POST /meetings/{id}/complete` réutilise exactement le même format
// `missing[]`. Decimal-as-string (mémoire `no-f64-in-money`) — quotas sont
// des strings, jamais parseFloat.

/**
 * Invariant légal manquant pour clôturer une AG.
 *
 * Union discriminée par `type` :
 *  - `ConvocationsNotSent` — Art. 3.87 §3 CC
 *  - `VotesNotClosed` — Art. 3.87 §4 CC (1+ résolution Pending)
 *  - `AttendanceNotRecorded` — Art. 3.87 §5 CC
 *  - `QuorumNotReached` — Art. 3.87 §5 CC (majorité simple > 50%)
 *  - `MinutesDraftMissing` — PV draft pas sauvegardé
 */
export type MissingInvariant =
  | { type: "ConvocationsNotSent" }
  | { type: "VotesNotClosed"; open_resolutions: number }
  | { type: "AttendanceNotRecorded" }
  | {
      type: "QuorumNotReached";
      /** Decimal-as-string. NE PAS parseFloat. */
      attended_quotas: string;
      /** Decimal-as-string. NE PAS parseFloat. */
      total_quotas: string;
    }
  | { type: "MinutesDraftMissing" };

/**
 * Réponse du `GET /meetings/{id}/completion-checklist`.
 *
 * `missing[]` est vide si la réunion peut être clôturée (status code 200
 * dans tous les cas — c'est un état, pas une erreur).
 */
export interface MeetingCompletionChecklistResponse {
  meeting_id: string;
  convocations_sent: boolean;
  open_resolutions: number;
  attendance_recorded: boolean;
  /** Decimal-as-string. */
  attended_quotas: string;
  /** Decimal-as-string. */
  total_quotas: string;
  minutes_draft_exists: boolean;
  missing: MissingInvariant[];
}

/**
 * Payload 422 narratif renvoyé par `POST /meetings/{id}/complete` quand
 * `Meeting::assert_can_complete()` échoue. Consommé par le toast FE.
 *
 * Format BE : `error_response()` injecte `details` dans le body 422.
 */
export interface MeetingNotCompletablePayload {
  code: "MEETING_NOT_COMPLETABLE";
  meeting_id: string;
  missing: MissingInvariant[];
}

/** Forme du body 422 complet (utile pour le typage du toast handler). */
export interface MeetingNotCompletableErrorBody {
  error: string;
  kind: "meeting_not_completable";
  details: MeetingNotCompletablePayload;
}
