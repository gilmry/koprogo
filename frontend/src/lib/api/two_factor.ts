// Story #1145 — client API du second facteur, pour l'écran des réglages.
//
// Endpoints backend (`two_factor_handlers.rs`, scope `/2fa`) :
//   - GET  /2fa/status                  → getStatus()
//   - POST /2fa/setup                   → setup()      secret, QR, 10 codes
//   - POST /2fa/enable                  → enable(code)
//   - POST /2fa/disable                 → disable(motDePasse)
//   - POST /2fa/regenerate-backup-codes → regenerateBackupCodes(code)
//
// Types tirés de `api.d.ts` (routes annotées utoipa depuis #1145).
//
// IMPORTANT — le secret et les codes de secours ne s'affichent qu'une fois.
// Ils ne vont JAMAIS en `localStorage` / `sessionStorage` : uniquement dans un
// `$state` du composant, qui disparaît avec lui (même règle qu'INV-FE5).
//
// `silent: true` partout : l'écran affiche lui-même ses erreurs, dans la
// langue de l'interface, sans relayer le texte anglais du serveur.

import { api } from "../api";
import type { components } from "../../types/api";

type Schemas = components["schemas"];
export type TwoFactorStatus = Schemas["TwoFactorStatusDto"];
export type TwoFactorSetup = Schemas["Setup2FAResponseDto"];
type BackupCodes = Schemas["RegenerateBackupCodesResponseDto"];

export const twoFactorApi = {
  getStatus(): Promise<TwoFactorStatus> {
    return api.get("/2fa/status", { silent: true });
  },

  setup(): Promise<TwoFactorSetup> {
    return api.post("/2fa/setup", undefined, { silent: true });
  },

  enable(totpCode: string): Promise<unknown> {
    return api.post("/2fa/enable", { totp_code: totpCode }, { silent: true });
  },

  disable(currentPassword: string): Promise<unknown> {
    return api.post(
      "/2fa/disable",
      { current_password: currentPassword },
      { silent: true },
    );
  },

  regenerateBackupCodes(totpCode: string): Promise<BackupCodes> {
    return api.post(
      "/2fa/regenerate-backup-codes",
      { totp_code: totpCode },
      { silent: true },
    );
  },
};
