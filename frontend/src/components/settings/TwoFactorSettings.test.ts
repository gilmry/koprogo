import { render, screen, waitFor, fireEvent } from "../../test-helpers";
import { describe, it, expect, vi, beforeEach } from "vitest";
import TwoFactorSettings from "./TwoFactorSettings.svelte";

vi.mock("../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string) => key);
      return () => {};
    },
  },
}));

const appels = {
  getStatus: vi.fn(),
  setup: vi.fn(),
  enable: vi.fn(),
  disable: vi.fn(),
  regenerateBackupCodes: vi.fn(),
};
vi.mock("../../lib/api/two_factor", () => ({
  twoFactorApi: {
    getStatus: () => appels.getStatus(),
    setup: () => appels.setup(),
    enable: (code: string) => appels.enable(code),
    disable: (motDePasse: string) => appels.disable(motDePasse),
    regenerateBackupCodes: (code: string) => appels.regenerateBackupCodes(code),
  },
}));

const INACTIF = {
  is_enabled: false,
  verified_at: null,
  last_used_at: null,
  backup_codes_remaining: 0,
  backup_codes_low: false,
  needs_reverification: false,
};
const ACTIF = { ...INACTIF, is_enabled: true, backup_codes_remaining: 10 };
const SETUP = {
  secret: "JBSWY3DPEHPK3PXP",
  qr_code_data_url: "data:image/png;base64,AAAA",
  backup_codes: ["ABCD-EFGH", "JKMN-PQRS"],
  issuer: "KoproGo",
  account_name: "alice@example.be",
};

describe("TwoFactorSettings (#1145)", () => {
  beforeEach(() => {
    Object.values(appels).forEach((f) => f.mockReset());
    localStorage.clear();
    sessionStorage.clear();
  });

  it("@happy active le second facteur : QR, codes de secours, puis code", async () => {
    appels.getStatus.mockResolvedValueOnce(INACTIF).mockResolvedValue(ACTIF);
    appels.setup.mockResolvedValue(SETUP);
    appels.enable.mockResolvedValue({ success: true });
    render(TwoFactorSettings);

    await fireEvent.click(await screen.findByTestId("second-factor-start"));
    expect(await screen.findByTestId("second-factor-qr")).toBeTruthy();
    expect(screen.getByTestId("second-factor-secret").textContent).toContain(
      SETUP.secret,
    );
    expect(
      screen.getByTestId("second-factor-backup-codes").textContent,
    ).toContain("ABCD-EFGH");

    await fireEvent.input(screen.getByTestId("second-factor-enable-code"), {
      target: { value: " 123456 " },
    });
    await fireEvent.click(screen.getByTestId("second-factor-enable-submit"));

    await waitFor(() => expect(appels.enable).toHaveBeenCalledWith("123456"));
    expect(
      (await screen.findByTestId("second-factor-status")).textContent,
    ).toContain("secondFactor.statusOn");
    expect(screen.queryByTestId("second-factor-secret")).toBeNull();
  });

  it("@negative un code faux à l'activation laisse le second facteur inactif", async () => {
    appels.getStatus.mockResolvedValue(INACTIF);
    appels.setup.mockResolvedValue(SETUP);
    appels.enable.mockRejectedValue(new Error("400"));
    render(TwoFactorSettings);

    await fireEvent.click(await screen.findByTestId("second-factor-start"));
    await fireEvent.input(
      await screen.findByTestId("second-factor-enable-code"),
      { target: { value: "000000" } },
    );
    await fireEvent.click(screen.getByTestId("second-factor-enable-submit"));

    expect(
      (await screen.findByTestId("second-factor-error")).textContent,
    ).toContain("secondFactor.wrongCode");
    expect(screen.getByTestId("second-factor-setup")).toBeTruthy();
    expect(screen.getByTestId("second-factor-status").textContent).toContain(
      "secondFactor.statusOff",
    );
  });

  it("@negative un mot de passe faux à la désactivation le laisse actif", async () => {
    appels.getStatus.mockResolvedValue(ACTIF);
    appels.disable.mockRejectedValue(new Error("400"));
    render(TwoFactorSettings);

    await fireEvent.input(
      await screen.findByTestId("second-factor-disable-password"),
      { target: { value: "mauvais" } },
    );
    await fireEvent.click(screen.getByTestId("second-factor-disable-submit"));

    expect(
      (await screen.findByTestId("second-factor-error")).textContent,
    ).toContain("secondFactor.wrongPassword");
    expect(screen.getByTestId("second-factor-status").textContent).toContain(
      "secondFactor.statusOn",
    );
  });

  it("@edge abandonner après le QR code ne laisse rien d'activé, et signale les codes rares", async () => {
    appels.getStatus.mockResolvedValue(INACTIF);
    appels.setup.mockResolvedValue(SETUP);
    const { unmount } = render(TwoFactorSettings);

    await fireEvent.click(await screen.findByTestId("second-factor-start"));
    await fireEvent.click(
      await screen.findByTestId("second-factor-setup-cancel"),
    );
    expect(screen.queryByTestId("second-factor-qr")).toBeNull();
    expect(screen.getByTestId("second-factor-start")).toBeTruthy();
    expect(appels.enable).not.toHaveBeenCalled();
    unmount();

    appels.getStatus.mockResolvedValue({
      ...ACTIF,
      backup_codes_remaining: 2,
      backup_codes_low: true,
    });
    render(TwoFactorSettings);
    expect(await screen.findByTestId("second-factor-low-codes")).toBeTruthy();
  });

  it("@security le secret et les codes de secours ne vont jamais dans le stockage du navigateur", async () => {
    appels.getStatus.mockResolvedValue(INACTIF);
    appels.setup.mockResolvedValue(SETUP);
    const journal = vi.spyOn(console, "log");
    render(TwoFactorSettings);

    await fireEvent.click(await screen.findByTestId("second-factor-start"));
    await screen.findByTestId("second-factor-qr");

    const stocke = [localStorage, sessionStorage]
      .flatMap((s) => Object.keys(s).map((k) => s.getItem(k) ?? ""))
      .join(" ");
    expect(stocke).not.toContain(SETUP.secret);
    expect(stocke).not.toContain("ABCD-EFGH");
    const ecrit = journal.mock.calls.flat().map(String).join(" ");
    expect(ecrit).not.toContain(SETUP.secret);
    journal.mockRestore();
  });
});
