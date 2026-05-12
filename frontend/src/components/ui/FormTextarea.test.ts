import { render, screen, fireEvent } from "../../test-helpers";
import { describe, it, expect } from "vitest";
import FormTextarea from "./FormTextarea.svelte";

describe("FormTextarea", () => {
  it("renders with label", () => {
    render(FormTextarea, {
      props: { id: "test", label: "Description" },
    });
    expect(screen.getByLabelText("Description")).toBeInTheDocument();
  });

  it("shows error message and red border", () => {
    render(FormTextarea, {
      props: { id: "test", label: "Desc", error: "Too short" },
    });
    expect(screen.getByText("Too short")).toBeInTheDocument();
    const textarea = screen.getByRole("textbox");
    expect(textarea.className).toContain("border-red-500");
  });

  it("updates value on input", async () => {
    render(FormTextarea, {
      props: { id: "test", label: "Desc", value: "" },
    });
    const textarea = screen.getByRole("textbox");
    await fireEvent.input(textarea, { target: { value: "hello world" } });
    expect((textarea as HTMLTextAreaElement).value).toBe("hello world");
  });
});
