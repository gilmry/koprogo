import { render, screen } from "../../test-helpers";
import { describe, it, expect } from "vitest";
import TicketPriorityBadge from "./TicketPriorityBadge.svelte";

vi.mock("../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string) => key);
      return () => {};
    },
  },
}));

describe("TicketPriorityBadge", () => {
  it("renders Low priority with gray styling", () => {
    render(TicketPriorityBadge, { props: { priority: "Low" } });
    const badge = screen.getByText("tickets.priorities.low");
    expect(badge.closest("span")).toHaveClass("bg-gray-100");
  });

  it("renders Medium priority with blue styling", () => {
    render(TicketPriorityBadge, { props: { priority: "Medium" } });
    const badge = screen.getByText("tickets.priorities.medium");
    expect(badge.closest("span")).toHaveClass("bg-blue-100");
  });

  it("renders High priority with yellow styling", () => {
    render(TicketPriorityBadge, { props: { priority: "High" } });
    const badge = screen.getByText("tickets.priorities.high");
    expect(badge.closest("span")).toHaveClass("bg-yellow-100");
  });

  it("renders Critical priority with red styling", () => {
    render(TicketPriorityBadge, { props: { priority: "Critical" } });
    const badge = screen.getByText("tickets.priorities.critical");
    expect(badge.closest("span")).toHaveClass("bg-red-100");
  });

  it("does NOT have Urgent option (removed per audit v4)", () => {
    // Verify at the type level that Urgent cannot be passed
    // This test documents the intentional removal
    render(TicketPriorityBadge, { props: { priority: "Urgent" as any } });
    // Falls back to Medium (default)
    const badge = screen.getByText("tickets.priorities.medium");
    expect(badge).toBeInTheDocument();
  });
});
