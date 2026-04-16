import { render, screen, fireEvent } from "@testing-library/svelte";
import { describe, it, expect, vi } from "vitest";
import Modal from "./Modal.svelte";

describe("Modal", () => {
  it("does not render when isOpen is false", () => {
    render(Modal, { props: { isOpen: false, title: "Test Modal" } });
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("renders when isOpen is true", () => {
    render(Modal, { props: { isOpen: true, title: "Test Modal" } });
    expect(screen.getByRole("dialog")).toBeInTheDocument();
    expect(screen.getByText("Test Modal")).toBeInTheDocument();
  });

  it("shows close button by default", () => {
    render(Modal, { props: { isOpen: true, title: "Test" } });
    expect(screen.getByLabelText("Close")).toBeInTheDocument();
  });

  it("hides close button when showClose is false", () => {
    render(Modal, {
      props: { isOpen: true, title: "Test", showClose: false },
    });
    expect(screen.queryByLabelText("Close")).not.toBeInTheDocument();
  });

  it("applies correct size class", () => {
    render(Modal, {
      props: { isOpen: true, title: "Test", size: "lg" },
    });
    const dialog = screen.getByRole("dialog");
    expect(dialog.className).toContain("max-w-4xl");
  });

  it("has scrollable content area", () => {
    render(Modal, {
      props: { isOpen: true, title: "Test" },
    });
    const content = document.querySelector(".overflow-y-auto");
    expect(content).toBeInTheDocument();
  });
});
