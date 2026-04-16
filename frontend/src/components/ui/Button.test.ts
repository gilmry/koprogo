import { render, screen } from "@testing-library/svelte";
import { describe, it, expect } from "vitest";
import Button from "./Button.svelte";

describe("Button", () => {
  it("renders with default props", () => {
    render(Button, { props: {} });
    const btn = screen.getByRole("button");
    expect(btn).toBeInTheDocument();
    expect(btn).not.toBeDisabled();
  });

  it("renders as disabled when loading", () => {
    render(Button, { props: { loading: true } });
    const btn = screen.getByRole("button");
    expect(btn).toBeDisabled();
  });

  it("renders as disabled when disabled prop is true", () => {
    render(Button, { props: { disabled: true } });
    const btn = screen.getByRole("button");
    expect(btn).toBeDisabled();
  });

  it("applies variant classes", () => {
    render(Button, { props: { variant: "danger" } });
    const btn = screen.getByRole("button");
    expect(btn.className).toContain("bg-red-600");
  });

  it("applies size classes", () => {
    render(Button, { props: { size: "lg" } });
    const btn = screen.getByRole("button");
    expect(btn.className).toContain("px-6");
  });

  it("shows spinner when loading", () => {
    render(Button, { props: { loading: true } });
    const svg = document.querySelector("svg.animate-spin");
    expect(svg).toBeInTheDocument();
  });

  it("uses correct button type", () => {
    render(Button, { props: { type: "submit" } });
    const btn = screen.getByRole("button");
    expect(btn).toHaveAttribute("type", "submit");
  });
});
