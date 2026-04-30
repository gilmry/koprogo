import { render, screen } from "@testing-library/svelte";
import { describe, it, expect, vi } from "vitest";
import TicketAssignModal from "./TicketAssignModal.svelte";

vi.mock("../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string) => key);
      return () => {};
    },
  },
}));

vi.mock("../../stores/toast", () => ({
  toast: { error: vi.fn(), success: vi.fn(), show: vi.fn() },
}));

vi.mock("../../lib/api", () => ({
  api: {
    get: vi.fn().mockResolvedValue([
      {
        id: "user-1",
        first_name: "Marc",
        last_name: "Dubois",
        role: "contractor",
        profession: "Plombier",
      },
      {
        id: "user-2",
        first_name: "Sophie",
        last_name: "Leroux",
        role: "contractor",
        profession: "Électricienne",
      },
    ]),
  },
}));

vi.mock("../../lib/utils/error.utils", () => ({
  withErrorHandling: vi.fn(async ({ action }) => action()),
}));

describe("TicketAssignModal", () => {
  it("does not render when open is false", () => {
    render(TicketAssignModal, {
      props: { open: false, ticketId: "ticket-1" },
    });
    expect(screen.queryByTestId("ticket-assign-form")).not.toBeInTheDocument();
  });

  it("renders form when open", () => {
    render(TicketAssignModal, {
      props: { open: true, ticketId: "ticket-1" },
    });
    expect(screen.getByTestId("ticket-assign-form")).toBeInTheDocument();
  });

  it("shows description text", () => {
    render(TicketAssignModal, {
      props: { open: true, ticketId: "ticket-1" },
    });
    expect(screen.getByTestId("ticket-assign-description")).toBeInTheDocument();
  });

  it("submit button is disabled initially (no selection)", () => {
    render(TicketAssignModal, {
      props: { open: true, ticketId: "ticket-1" },
    });
    const btn = screen.getByTestId("ticket-assign-submit-btn");
    expect(btn).toBeDisabled();
  });

  it("has cancel button", () => {
    render(TicketAssignModal, {
      props: { open: true, ticketId: "ticket-1" },
    });
    expect(screen.getByTestId("ticket-assign-cancel-btn")).toBeInTheDocument();
  });
});
