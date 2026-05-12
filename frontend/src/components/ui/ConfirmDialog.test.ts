import { render, screen } from "../../test-helpers";
import { describe, it, expect } from "vitest";
import ConfirmDialog from "./ConfirmDialog.svelte";

vi.mock("../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string) => key);
      return () => {};
    },
  },
}));

describe("ConfirmDialog", () => {
  it("does not render when isOpen is false", () => {
    render(ConfirmDialog, {
      props: { isOpen: false, title: "Confirm", message: "Sure?" },
    });
    expect(screen.queryByText("Confirm")).not.toBeInTheDocument();
  });

  it("renders title and message when open", () => {
    render(ConfirmDialog, {
      props: { isOpen: true, title: "Delete?", message: "This is permanent." },
    });
    expect(screen.getByText("Delete?")).toBeInTheDocument();
    expect(screen.getByText("This is permanent.")).toBeInTheDocument();
  });

  it("has confirm and cancel buttons", () => {
    render(ConfirmDialog, {
      props: { isOpen: true, title: "X", message: "Y" },
    });
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThanOrEqual(2);
  });
});
