import { render, screen, fireEvent } from "../../test-helpers";
import { describe, it, expect, vi } from "vitest";
import { tick } from "svelte";
import Modal from "./Modal.svelte";
import ModalFocusTrapHarness from "./__tests__/ModalFocusTrapHarness.svelte";

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

  describe("focus management (STORY-P7-803)", () => {
    it("moves focus to the first focusable child on open", async () => {
      render(ModalFocusTrapHarness, { props: { isOpen: true } });
      await tick();
      // Wait one more microtask for the $effect to run
      await Promise.resolve();

      // The first focusable element inside the dialog is the Close button
      // (comes before the harness's inner buttons in DOM order).
      const closeBtn = screen.getByLabelText("Close");
      expect(document.activeElement).toBe(closeBtn);
    });

    it("triggers onclose when Escape is pressed", async () => {
      const onclose = vi.fn();
      render(Modal, { props: { isOpen: true, title: "Test", onclose } });
      await tick();

      await fireEvent.keyDown(document, { key: "Escape" });
      expect(onclose).toHaveBeenCalledTimes(1);
    });

    it("wraps focus from last to first with Tab", async () => {
      render(ModalFocusTrapHarness, { props: { isOpen: true } });
      await tick();
      await Promise.resolve();

      const dialog = screen.getByRole("dialog");
      const focusables: HTMLElement[] = Array.from(
        dialog.querySelectorAll<HTMLElement>(
          'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
        ),
      );
      const first: HTMLElement = focusables[0];
      const last: HTMLElement = focusables[focusables.length - 1];

      last.focus();
      expect(document.activeElement).toBe(last);

      // Tab from last should wrap to first
      await fireEvent.keyDown(last, { key: "Tab" });
      expect(document.activeElement).toBe(first);
    });

    it("wraps focus from first to last with Shift+Tab", async () => {
      render(ModalFocusTrapHarness, { props: { isOpen: true } });
      await tick();
      await Promise.resolve();

      const dialog = screen.getByRole("dialog");
      const focusables: HTMLElement[] = Array.from(
        dialog.querySelectorAll<HTMLElement>(
          'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
        ),
      );
      const first: HTMLElement = focusables[0];
      const last: HTMLElement = focusables[focusables.length - 1];

      first.focus();
      expect(document.activeElement).toBe(first);

      // Shift+Tab from first should wrap to last
      await fireEvent.keyDown(first, { key: "Tab", shiftKey: true });
      expect(document.activeElement).toBe(last);
    });

    it("restores focus to the originally focused element on close", async () => {
      // Create and focus a trigger button in the document first
      const trigger = document.createElement("button");
      trigger.textContent = "Open";
      trigger.setAttribute("data-testid", "trigger");
      document.body.appendChild(trigger);
      trigger.focus();
      expect(document.activeElement).toBe(trigger);

      const { rerender } = render(ModalFocusTrapHarness, {
        props: { isOpen: true },
      });
      await tick();
      await Promise.resolve();

      // Focus has been moved into the modal
      expect(document.activeElement).not.toBe(trigger);

      // Close the modal
      await rerender({ isOpen: false });
      await tick();
      await Promise.resolve();

      // Focus is restored to the original trigger
      expect(document.activeElement).toBe(trigger);

      document.body.removeChild(trigger);
    });
  });
});
