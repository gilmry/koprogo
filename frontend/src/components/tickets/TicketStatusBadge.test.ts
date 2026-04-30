import { render, screen } from "@testing-library/svelte";
import { describe, it, expect } from "vitest";
import TicketStatusBadge from "./TicketStatusBadge.svelte";

// Mock svelte-i18n since it needs runtime init
vi.mock("../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string) => key); // Return key as-is for testing
      return () => {};
    },
  },
}));

describe("TicketStatusBadge", () => {
  it("renders Open status with blue styling", () => {
    render(TicketStatusBadge, { props: { status: "Open" } });
    const badge = screen.getByText("tickets.statuses.open");
    expect(badge).toBeInTheDocument();
    expect(badge.className).toContain("bg-blue-100");
  });

  it("renders InProgress status with yellow styling", () => {
    render(TicketStatusBadge, { props: { status: "InProgress" } });
    const badge = screen.getByText("tickets.statuses.inProgress");
    expect(badge.className).toContain("bg-yellow-100");
  });

  it("renders Resolved status with green styling", () => {
    render(TicketStatusBadge, { props: { status: "Resolved" } });
    const badge = screen.getByText("tickets.statuses.resolved");
    expect(badge.className).toContain("bg-green-100");
  });

  it("renders Closed status with gray styling", () => {
    render(TicketStatusBadge, { props: { status: "Closed" } });
    const badge = screen.getByText("tickets.statuses.closed");
    expect(badge.className).toContain("bg-gray-100");
  });

  it("renders Cancelled status with red styling", () => {
    render(TicketStatusBadge, { props: { status: "Cancelled" } });
    const badge = screen.getByText("tickets.statuses.cancelled");
    expect(badge.className).toContain("bg-red-100");
  });

  it("falls back to Open for unknown status", () => {
    render(TicketStatusBadge, { props: { status: "Unknown" as any } });
    const badge = screen.getByText("tickets.statuses.open");
    expect(badge.className).toContain("bg-blue-100");
  });
});
