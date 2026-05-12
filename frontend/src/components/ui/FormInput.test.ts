import { render, screen, fireEvent } from "../../test-helpers";
import { describe, it, expect } from "vitest";
import FormInput from "./FormInput.svelte";

describe("FormInput", () => {
  it("renders with label", () => {
    render(FormInput, { props: { id: "test", label: "Name" } });
    expect(screen.getByLabelText("Name")).toBeInTheDocument();
  });

  it("shows required asterisk when required", () => {
    render(FormInput, {
      props: { id: "test", label: "Email", required: true },
    });
    expect(screen.getByText("*")).toBeInTheDocument();
  });

  it("does not show asterisk when not required", () => {
    render(FormInput, { props: { id: "test", label: "Note" } });
    expect(screen.queryByText("*")).not.toBeInTheDocument();
  });

  it("applies error border when error is set", () => {
    render(FormInput, {
      props: { id: "test", label: "X", error: "Required" },
    });
    const input = screen.getByRole("textbox");
    expect(input.className).toContain("border-red-500");
    expect(screen.getByText("Required")).toBeInTheDocument();
  });

  it("sets aria-invalid when error exists", () => {
    render(FormInput, {
      props: { id: "test", label: "X", error: "Bad" },
    });
    expect(screen.getByRole("textbox")).toHaveAttribute("aria-invalid", "true");
  });

  it("shows hint text when no error", () => {
    render(FormInput, {
      props: { id: "test", label: "X", hint: "Helpful" },
    });
    expect(screen.getByText("Helpful")).toBeInTheDocument();
  });

  it("is disabled when disabled prop is set", () => {
    render(FormInput, {
      props: { id: "test", label: "X", disabled: true },
    });
    expect(screen.getByRole("textbox")).toBeDisabled();
  });

  it("updates value on input", async () => {
    render(FormInput, {
      props: { id: "test", label: "X", value: "" },
    });
    const input = screen.getByRole("textbox");
    await fireEvent.input(input, { target: { value: "hello" } });
    expect((input as HTMLInputElement).value).toBe("hello");
  });
});
