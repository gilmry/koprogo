import { render, screen, fireEvent } from "../../test-helpers";
import { describe, it, expect, vi } from "vitest";
import QuorumPanel from "./QuorumPanel.svelte";

vi.mock("../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string, opts?: any) => {
        if (opts?.values) return `${key} ${JSON.stringify(opts.values)}`;
        return key;
      });
      return () => {};
    },
  },
}));

vi.mock("../../stores/toast", () => ({
  toast: { error: vi.fn(), warning: vi.fn(), success: vi.fn(), show: vi.fn() },
}));

vi.mock("../../lib/api", () => ({
  api: {
    post: vi.fn().mockResolvedValue({
      quorum_reached: true,
      meeting: {
        id: "m1",
        quorum_validated: true,
        present_quotas: 800,
        total_quotas: 1000,
        quorum_percentage: 80.0,
      },
      message: "OK",
    }),
  },
}));

vi.mock("../../lib/utils/error.utils", () => ({
  withErrorHandling: vi.fn(async ({ action, onSuccess }) => {
    const result = await action();
    if (onSuccess) onSuccess(result);
    return result;
  }),
}));

const baseMeeting = {
  id: "m1",
  building_id: "b1",
  title: "AG Test",
  meeting_type: "Ordinary",
  scheduled_date: "2026-06-01T10:00:00Z",
  location: "Salle commune",
  status: "Scheduled",
  quorum_validated: false,
  present_quotas: null,
  total_quotas: 1000,
} as any;

describe("QuorumPanel", () => {
  it("shows validated badge when quorum already validated", () => {
    render(QuorumPanel, {
      props: {
        meeting: {
          ...baseMeeting,
          quorum_validated: true,
          present_quotas: 800,
          quorum_percentage: 80.0,
        },
        canManage: true,
      },
    });
    expect(screen.getByTestId("quorum-validated-badge")).toBeInTheDocument();
  });

  it("shows form when not validated and canManage", () => {
    render(QuorumPanel, {
      props: { meeting: baseMeeting, canManage: true },
    });
    expect(screen.getByTestId("quorum-present-input")).toBeInTheDocument();
    expect(screen.getByTestId("quorum-total-input")).toBeInTheDocument();
    expect(screen.getByTestId("quorum-validate-btn")).toBeInTheDocument();
  });

  it("shows readonly message when not validated and NOT canManage", () => {
    render(QuorumPanel, {
      props: { meeting: baseMeeting, canManage: false },
    });
    expect(screen.queryByTestId("quorum-validate-btn")).not.toBeInTheDocument();
    expect(screen.getByText("meetings.quorum.readonly")).toBeInTheDocument();
  });

  it("pre-fills total quotas from meeting", () => {
    render(QuorumPanel, {
      props: { meeting: baseMeeting, canManage: true },
    });
    const input = screen.getByTestId("quorum-total-input") as HTMLInputElement;
    expect(input.value).toBe("1000");
  });

  it("shows live percentage preview", async () => {
    render(QuorumPanel, {
      props: { meeting: baseMeeting, canManage: true },
    });
    const presentInput = screen.getByTestId(
      "quorum-present-input",
    ) as HTMLInputElement;
    await fireEvent.input(presentInput, { target: { value: "600" } });
    // Check that 60.0% is shown somewhere in the preview
    const preview = screen.getByText(/60\.0%/);
    expect(preview).toBeInTheDocument();
  });
});
