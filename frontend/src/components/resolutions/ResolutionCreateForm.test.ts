import { render, screen, fireEvent } from "@testing-library/svelte";
import { describe, it, expect, vi, beforeEach } from "vitest";
import ResolutionCreateForm from "./ResolutionCreateForm.svelte";

// Mock i18n
vi.mock("../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string) => key);
      return () => {};
    },
  },
}));

// Mock toast
vi.mock("../../stores/toast", () => ({
  toast: { error: vi.fn(), success: vi.fn(), show: vi.fn() },
}));

// Mock API
vi.mock("../../lib/api/resolutions", async () => {
  const actual = await vi.importActual("../../lib/api/resolutions");
  return {
    ...actual,
    resolutionsApi: {
      create: vi.fn().mockResolvedValue({ id: "test-id", title: "Test" }),
    },
  };
});

// Mock error utils
vi.mock("../../lib/utils/error.utils", () => ({
  withErrorHandling: vi.fn(async ({ action, onSuccess }) => {
    const result = await action();
    if (onSuccess) onSuccess(result);
    return result;
  }),
}));

describe("ResolutionCreateForm", () => {
  it("renders with title and description inputs", () => {
    render(ResolutionCreateForm, { props: { meetingId: "meeting-123" } });
    expect(screen.getByTestId("resolution-title-input")).toBeInTheDocument();
    expect(
      screen.getByTestId("resolution-description-textarea"),
    ).toBeInTheDocument();
  });

  it("has resolution type dropdown with 2 options (Ordinary/Extraordinary)", () => {
    render(ResolutionCreateForm, { props: { meetingId: "meeting-123" } });
    const select = screen.getByTestId(
      "resolution-type-select",
    ) as HTMLSelectElement;
    const options = Array.from(select.options);
    expect(options).toHaveLength(2);
    // Values should be snake_case to match backend serde
    expect(options.map((o) => o.value)).toEqual([
      "ordinary",
      "extraordinary",
    ]);
  });

  it("has majority dropdown with 4 options (absolute/two_thirds/four_fifths/unanimity)", () => {
    render(ResolutionCreateForm, { props: { meetingId: "meeting-123" } });
    const select = screen.getByTestId(
      "resolution-majority-select",
    ) as HTMLSelectElement;
    const options = Array.from(select.options);
    expect(options).toHaveLength(4);
    expect(options.map((o) => o.value)).toEqual([
      "absolute",
      "two_thirds",
      "four_fifths",
      "unanimity",
    ]);
  });

  it("submit button is disabled when title is empty", () => {
    render(ResolutionCreateForm, { props: { meetingId: "meeting-123" } });
    const btn = screen.getByTestId("resolution-submit-btn");
    expect(btn).toBeDisabled();
  });

  it("submit button is enabled when title is filled", async () => {
    render(ResolutionCreateForm, { props: { meetingId: "meeting-123" } });
    const input = screen.getByTestId(
      "resolution-title-input",
    ) as HTMLInputElement;
    await fireEvent.input(input, { target: { value: "Test Resolution" } });
    const btn = screen.getByTestId("resolution-submit-btn");
    expect(btn).not.toBeDisabled();
  });

  it("has cancel button", () => {
    render(ResolutionCreateForm, { props: { meetingId: "meeting-123" } });
    expect(screen.getByTestId("resolution-cancel-btn")).toBeInTheDocument();
  });

  it("shows Belgian law notice", () => {
    render(ResolutionCreateForm, { props: { meetingId: "meeting-123" } });
    // belgianLaw is in a <strong> with legalText in the same div
    const notice = document.querySelector(".bg-yellow-50");
    expect(notice).toBeInTheDocument();
  });
});
