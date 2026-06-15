// Track H Story H3 — Vitest 4-cat `meetingCompletion.ts`.
//
// @happy   Body 422 valide → détecté + payload extrait + toast affiché.
// @edge    Wrappers `Error.body` et `Error.response.data` également détectés.
// @security Erreur arbitraire (string / null / forme inconnue) → pas de
//           toast, pas de fuite, return false.
// @negative Payload tronqué → null safe.

import { describe, it, expect, vi, beforeEach } from "vitest";
import type {
  MeetingNotCompletableErrorBody,
  MeetingNotCompletablePayload,
} from "../types/meeting";

// Hoisted mocks — `vi.mock()` est levé avant les imports, donc on déclare
// les vi.fn() via `vi.hoisted()` pour qu'ils existent au moment de l'appel.
const { toastErrorMock } = vi.hoisted(() => ({
  toastErrorMock: vi.fn(),
}));

vi.mock("../i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string, opts?: any) => {
        if (opts?.values) return `${key} ${JSON.stringify(opts.values)}`;
        return key;
      });
      return () => {};
    },
  },
}));

vi.mock("../../stores/toast", () => ({
  toast: {
    error: toastErrorMock,
    warning: vi.fn(),
    success: vi.fn(),
    show: vi.fn(),
  },
}));

import {
  isMeetingCompletionError,
  extractMeetingCompletionPayload,
  showMeetingCompletionToast,
} from "./meetingCompletion";

const SAMPLE_PAYLOAD: MeetingNotCompletablePayload = {
  code: "MEETING_NOT_COMPLETABLE",
  meeting_id: "11111111-2222-3333-4444-555555555555",
  missing: [
    { type: "ConvocationsNotSent" },
    {
      type: "QuorumNotReached",
      attended_quotas: "400",
      total_quotas: "1000",
    },
  ],
};

const SAMPLE_BODY: MeetingNotCompletableErrorBody = {
  error: "La réunion ne peut pas être clôturée",
  kind: "meeting_not_completable",
  details: SAMPLE_PAYLOAD,
};

beforeEach(() => {
  toastErrorMock.mockClear();
});

describe("meetingCompletion — 4-cat (Track H Story H3)", () => {
  // ----------------------------------------------------------------------
  // @happy
  // ----------------------------------------------------------------------

  it("@happy body direct 422 → isMeetingCompletionError true", () => {
    expect(isMeetingCompletionError(SAMPLE_BODY)).toBe(true);
  });

  it("@happy showMeetingCompletionToast appelle toast.error et retourne true", () => {
    const handled = showMeetingCompletionToast(SAMPLE_BODY);
    expect(handled).toBe(true);
    expect(toastErrorMock).toHaveBeenCalledTimes(1);
    const [msg, duration] = toastErrorMock.mock.calls[0];
    expect(typeof msg).toBe("string");
    expect(msg).toMatch(/meeting\.complete\.toast_title/);
    // n=2 interpolé depuis payload.missing.length
    expect(msg).toContain('"n":2');
    expect(duration).toBe(8000);
  });

  it("@happy extractMeetingCompletionPayload renvoie le payload structuré", () => {
    const payload = extractMeetingCompletionPayload(SAMPLE_BODY);
    expect(payload).not.toBeNull();
    expect(payload?.code).toBe("MEETING_NOT_COMPLETABLE");
    expect(payload?.missing.length).toBe(2);
  });

  // ----------------------------------------------------------------------
  // @edge
  // ----------------------------------------------------------------------

  it("@edge wrapper Error.body 422 → détecté", () => {
    const wrapped = { message: "HTTP 422", body: SAMPLE_BODY };
    expect(isMeetingCompletionError(wrapped)).toBe(true);
    const handled = showMeetingCompletionToast(wrapped);
    expect(handled).toBe(true);
  });

  it("@edge wrapper Error.response.data 422 (axios-style) → détecté", () => {
    const wrapped = { message: "HTTP 422", response: { data: SAMPLE_BODY } };
    expect(isMeetingCompletionError(wrapped)).toBe(true);
    const payload = extractMeetingCompletionPayload(wrapped);
    expect(payload?.code).toBe("MEETING_NOT_COMPLETABLE");
  });

  it("@edge missing[]=[] (réunion complétable) → détecté mais missing vide", () => {
    const empty: MeetingNotCompletableErrorBody = {
      ...SAMPLE_BODY,
      details: { ...SAMPLE_PAYLOAD, missing: [] },
    };
    expect(isMeetingCompletionError(empty)).toBe(true);
    const handled = showMeetingCompletionToast(empty);
    expect(handled).toBe(true);
    const [msg] = toastErrorMock.mock.calls[0];
    expect(msg).toContain('"n":0');
  });

  // ----------------------------------------------------------------------
  // @security
  // ----------------------------------------------------------------------

  it("@security erreur arbitraire (Error string) → pas de toast, return false", () => {
    const err = new Error("Network refused");
    expect(isMeetingCompletionError(err)).toBe(false);
    const handled = showMeetingCompletionToast(err);
    expect(handled).toBe(false);
    expect(toastErrorMock).not.toHaveBeenCalled();
  });

  it("@security null / undefined → return false sans crash", () => {
    expect(isMeetingCompletionError(null)).toBe(false);
    expect(isMeetingCompletionError(undefined)).toBe(false);
    expect(showMeetingCompletionToast(null)).toBe(false);
    expect(showMeetingCompletionToast(undefined)).toBe(false);
  });

  it("@security body avec kind différent → ignoré (defense-in-depth)", () => {
    const wrongKind = {
      error: "Other error",
      kind: "validation",
      details: { code: "OTHER" },
    };
    expect(isMeetingCompletionError(wrongKind)).toBe(false);
    expect(showMeetingCompletionToast(wrongKind)).toBe(false);
  });

  // ----------------------------------------------------------------------
  // @negative
  // ----------------------------------------------------------------------

  it("@negative payload sans details.code → null safe", () => {
    const truncated = {
      error: "X",
      kind: "meeting_not_completable",
      details: { meeting_id: "abc" }, // pas de code !
    };
    expect(isMeetingCompletionError(truncated)).toBe(false);
    expect(extractMeetingCompletionPayload(truncated)).toBeNull();
  });

  it("@negative body sans details du tout → null safe", () => {
    const broken = { error: "X", kind: "meeting_not_completable" };
    expect(isMeetingCompletionError(broken)).toBe(false);
    expect(extractMeetingCompletionPayload(broken)).toBeNull();
  });
});
