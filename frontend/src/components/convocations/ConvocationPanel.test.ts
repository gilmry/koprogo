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
const send = vi.fn();
const getEligibleRecipients = vi.fn();

vi.mock("../../lib/api/convocations", async () => {
  const actual = await vi.importActual<
    typeof import("../../lib/api/convocations")
  >("../../lib/api/convocations");
  return {
    ...actual,
    convocationsApi: {
      getByMeetingId: (...args: any[]) => getByMeetingId(...args),
      create: (...args: any[]) => create(...args),
      send: (...args: any[]) => send(...args),
      getEligibleRecipients: (...args: any[]) => getEligibleRecipients(...args),
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

const draftConvocation = {
  ...baseConvocation,
  status: ConvocationStatus.Draft,
  total_recipients: 0,
  opened_count: 0,
  will_attend_count: 0,
};

describe("ConvocationPanel", () => {
  beforeEach(() => {
    getByMeetingId.mockReset();
    create.mockReset();
    send.mockReset();
    getEligibleRecipients.mockReset();
    getEligibleRecipients.mockResolvedValue([]);
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

  // #780 verrou 1 — écran de sélection des destinataires. Avant ces tests,
  // rien n'exerçait le chemin Draft/Scheduled + envoi : « 0 destinataire »
  // était un libellé, jamais un contrôle.

  it("@happy — le syndic choisit les destinataires puis envoie la convocation", async () => {
    getByMeetingId.mockResolvedValue(draftConvocation);
    getEligibleRecipients.mockResolvedValue([
      { owner_id: "o1", full_name: "Alice Dupont", email: "alice@test.be" },
      { owner_id: "o2", full_name: "Bob Peeters", email: "bob@test.be" },
    ]);
    send.mockResolvedValue({
      ...draftConvocation,
      status: ConvocationStatus.Sent,
    });
    mockAuthState.set({
      user: syndicUser,
      isAuthenticated: true,
      isLoading: false,
      token: "t",
    });

    render(ConvocationPanel, { props: { meetingId: "m1" } });

    await screen.findByTestId("convocation-recipient-selector-checkbox-o1");
    const sendBtn = screen.getByTestId("convocation-btn-send");
    expect(sendBtn).not.toBeDisabled();

    // « Envoyer » ouvre une confirmation (#844, ex-`confirm()` natif) ; la
    // requête ne part qu'à sa validation.
    await fireEvent.click(sendBtn);
    await fireEvent.click(await screen.findByTestId("confirm-dialog-confirm"));

    expect(send).toHaveBeenCalledWith("c1", ["o1", "o2"]);
  });

  it("@negative — décocher tous les destinataires désactive l'envoi et avertit", async () => {
    getByMeetingId.mockResolvedValue(draftConvocation);
    getEligibleRecipients.mockResolvedValue([
      { owner_id: "o1", full_name: "Alice Dupont", email: "alice@test.be" },
    ]);
    mockAuthState.set({
      user: syndicUser,
      isAuthenticated: true,
      isLoading: false,
      token: "t",
    });

    render(ConvocationPanel, { props: { meetingId: "m1" } });

    const checkbox = await screen.findByTestId(
      "convocation-recipient-selector-checkbox-o1",
    );
    await fireEvent.click(checkbox); // décoche l'unique destinataire

    expect(
      await screen.findByText(
        "convocations.recipientSelector.noneSelectedWarning",
      ),
    ).toBeInTheDocument();
    expect(screen.getByTestId("convocation-btn-send")).toBeDisabled();
  });

  it("@edge — immeuble sans copropriétaire éligible : l'envoi reste ouvert par défaut", async () => {
    getByMeetingId.mockResolvedValue(draftConvocation);
    getEligibleRecipients.mockResolvedValue([]);
    mockAuthState.set({
      user: syndicUser,
      isAuthenticated: true,
      isLoading: false,
      token: "t",
    });

    render(ConvocationPanel, { props: { meetingId: "m1" } });

    expect(
      await screen.findByTestId("convocation-recipient-selector-empty"),
    ).toBeInTheDocument();
    // Aucune liste à choisir : le champ reste `null` côté client, et c'est
    // le serveur qui refusera explicitement au moment de l'envoi (backend
    // `send_convocation`), pas l'UI qui décide à sa place ici.
    expect(screen.getByTestId("convocation-btn-send")).not.toBeDisabled();
  });
});
