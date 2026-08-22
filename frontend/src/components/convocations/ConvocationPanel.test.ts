// WP-D2 — vitest gate composants critiques bêta (auth store déjà couvert par
// stores/auth.test.ts ; ce fichier couvre le composant convocation manquant).
// Pattern : stubber les boundaries (authStore, i18n, convocationsApi,
// withErrorHandling) — la logique de rendu/permission du composant reste réelle.

import { render, screen, fireEvent } from "../../test-helpers";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { writable, type Writable } from "svelte/store";
import ConvocationPanel from "./ConvocationPanel.svelte";
import { ConvocationStatus, MeetingType } from "../../lib/api/convocations";
import { UserRole, type User } from "../../lib/types";

vi.mock("../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string) => key);
      return () => {};
    },
  },
}));

type AuthState = {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  token: string | null;
};
const mockAuthState: Writable<AuthState> = writable({
  user: null,
  isAuthenticated: false,
  isLoading: false,
  token: null,
});

vi.mock("../../stores/auth", () => ({
  authStore: {
    subscribe: (...args: any[]) => mockAuthState.subscribe(...(args as [any])),
  },
}));

const getByMeetingId = vi.fn();
const create = vi.fn();

vi.mock("../../lib/api/convocations", async () => {
  const actual = await vi.importActual<
    typeof import("../../lib/api/convocations")
  >("../../lib/api/convocations");
  return {
    ...actual,
    convocationsApi: {
      getByMeetingId: (...args: any[]) => getByMeetingId(...args),
      create: (...args: any[]) => create(...args),
      send: vi.fn(),
      cancel: vi.fn(),
      sendReminders: vi.fn(),
      delete: vi.fn(),
      // Consommé par ConvocationTrackingSummary (enfant monté quand status=Sent).
      getTrackingSummary: vi.fn().mockResolvedValue({
        total_recipients: 5,
        email_sent: 5,
        email_opened: 2,
        email_failed: 0,
        will_attend: 3,
        will_not_attend: 0,
        attended: 0,
        did_not_attend: 0,
        pending: 2,
        opening_rate: 0.4,
        attendance_rate: 0.6,
      }),
    },
  };
});

vi.mock("../../lib/utils/error.utils", () => ({
  withErrorHandling: vi.fn(async ({ action, onSuccess, setLoading }) => {
    setLoading?.(true);
    const result = await action();
    setLoading?.(false);
    onSuccess?.(result);
    return result;
  }),
  // Consommé par ConvocationTrackingSummary (enfant monté quand status=Sent).
  withLoadingState: vi.fn(
    async ({ action, setLoading, setError, onSuccess }) => {
      try {
        setLoading?.(true);
        setError?.("");
        const result = await action();
        onSuccess?.(result);
      } catch (err: any) {
        setError?.(err?.message ?? "error");
      } finally {
        setLoading?.(false);
      }
    },
  ),
}));

const syndicUser: User = {
  id: "u1",
  email: "syndic@test.be",
  first_name: "Syn",
  last_name: "Dic",
  role: UserRole.SYNDIC,
  organizationId: "org1",
  roles: [],
};

const ownerUser: User = {
  id: "u2",
  email: "owner@test.be",
  first_name: "Own",
  last_name: "Er",
  role: UserRole.OWNER,
  organizationId: "org1",
  roles: [],
};

const baseConvocation = {
  id: "c1",
  meeting_id: "m1",
  building_id: "b1",
  organization_id: "org1",
  meeting_type: MeetingType.Ordinary,
  meeting_date: "2026-09-01T10:00:00Z",
  minimum_send_date: "2026-08-17T10:00:00Z",
  status: ConvocationStatus.Sent,
  language: "fr",
  total_recipients: 5,
  opened_count: 2,
  will_attend_count: 3,
  respects_legal_deadline: true,
  created_at: "2026-08-01T10:00:00Z",
  updated_at: "2026-08-01T10:00:00Z",
};

describe("ConvocationPanel", () => {
  beforeEach(() => {
    getByMeetingId.mockReset();
    create.mockReset();
    mockAuthState.set({
      user: null,
      isAuthenticated: false,
      isLoading: false,
      token: null,
    });
  });

  it("@happy — displays convocation status and recipient counters once loaded", async () => {
    getByMeetingId.mockResolvedValue(baseConvocation);
    mockAuthState.set({
      user: syndicUser,
      isAuthenticated: true,
      isLoading: false,
      token: "t",
    });

    render(ConvocationPanel, { props: { meetingId: "m1" } });

    expect(
      await screen.findByTestId("convocation-field-type"),
    ).toBeInTheDocument();
    expect(
      screen.getByTestId("convocation-field-meeting-date"),
    ).toBeInTheDocument();
    expect(screen.getByText(/5 common\.recipient/)).toBeInTheDocument();
  });

  it("@edge — no convocation yet + syndic on a scheduled meeting sees the create action", async () => {
    getByMeetingId.mockRejectedValue(new Error("404 not found"));
    mockAuthState.set({
      user: syndicUser,
      isAuthenticated: true,
      isLoading: false,
      token: "t",
    });

    render(ConvocationPanel, {
      props: { meetingId: "m1", meetingStatus: "Scheduled" },
    });

    expect(
      await screen.findByTestId("convocation-btn-create"),
    ).toBeInTheDocument();

    create.mockResolvedValue(baseConvocation);
    await fireEvent.click(screen.getByTestId("convocation-btn-create"));
    expect(create).toHaveBeenCalledWith(
      expect.objectContaining({ meeting_id: "m1", building_id: "" }),
    );
  });

  it("@security — an owner never sees the create/send/cancel actions, convocation or not", async () => {
    getByMeetingId.mockRejectedValue(new Error("404 not found"));
    mockAuthState.set({
      user: ownerUser,
      isAuthenticated: true,
      isLoading: false,
      token: "t",
    });

    render(ConvocationPanel, {
      props: { meetingId: "m1", meetingStatus: "Scheduled" },
    });

    await screen.findByText("convocations.noConvocationCreated");
    expect(
      screen.queryByTestId("convocation-btn-create"),
    ).not.toBeInTheDocument();
  });

  it("@negative — a non-404 API error surfaces a visible retry, not a silent failure", async () => {
    getByMeetingId.mockRejectedValue(new Error("500 internal server error"));
    mockAuthState.set({
      user: syndicUser,
      isAuthenticated: true,
      isLoading: false,
      token: "t",
    });

    render(ConvocationPanel, { props: { meetingId: "m1" } });

    expect(
      await screen.findByText("500 internal server error"),
    ).toBeInTheDocument();
    expect(screen.getByText("common.retry")).toBeInTheDocument();
  });
});
