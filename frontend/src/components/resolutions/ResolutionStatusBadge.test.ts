import { render, screen } from "@testing-library/svelte";
import { describe, it, expect } from "vitest";
import ResolutionStatusBadge from "./ResolutionStatusBadge.svelte";

vi.mock("../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string) => key);
      return () => {};
    },
  },
}));

describe("ResolutionStatusBadge", () => {
  it("renders pending status", () => {
    render(ResolutionStatusBadge, { props: { status: "pending" } });
    const el = document.querySelector("span");
    expect(el).toBeInTheDocument();
  });

  it("renders adopted status with green styling", () => {
    render(ResolutionStatusBadge, { props: { status: "adopted" } });
    const el = document.querySelector("span");
    expect(el?.className).toContain("green");
  });

  it("renders rejected status with red styling", () => {
    render(ResolutionStatusBadge, { props: { status: "rejected" } });
    const el = document.querySelector("span");
    expect(el?.className).toContain("red");
  });
});
