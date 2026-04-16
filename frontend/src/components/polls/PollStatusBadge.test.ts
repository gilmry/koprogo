import { render, screen } from "@testing-library/svelte";
import { describe, it, expect } from "vitest";
import PollStatusBadge from "./PollStatusBadge.svelte";

vi.mock("../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string) => key);
      return () => {};
    },
  },
}));

describe("PollStatusBadge", () => {
  it("renders Draft status", () => {
    render(PollStatusBadge, { props: { status: "draft" } });
    expect(screen.getByText("polls.status.draft")).toBeInTheDocument();
  });

  it("renders Active status with green styling", () => {
    render(PollStatusBadge, { props: { status: "active" } });
    const badge = screen.getByText("polls.status.active");
    expect(badge.closest("span")).toHaveClass("bg-green-100");
  });

  it("renders Closed status", () => {
    render(PollStatusBadge, { props: { status: "closed" } });
    expect(screen.getByText("polls.status.closed")).toBeInTheDocument();
  });

  it("renders Cancelled status with red styling", () => {
    render(PollStatusBadge, { props: { status: "cancelled" } });
    const badge = screen.getByText("polls.status.cancelled");
    expect(badge.closest("span")).toHaveClass("bg-red-100");
  });
});
