// Story B5 (Phase B FE) — Helper d'upload presigned URL (S3 / MinIO).
//
// Architecture (cf. stories.md §B5 anti-pattern "NE PAS uploader à submission
// finale") :
//   1. FE demande une presigned PUT URL au backend (POST /attachments/presign).
//      Backend renvoie { upload_url, public_url, expires_at }.
//   2. FE fait un PUT direct vers `upload_url` avec le blob (streaming —
//      pas via le backend Rust pour décharger CPU).
//   3. FE garde `public_url` à mettre dans le payload final (Ticket
//      evidence_attachments[]).
//
// MIME inspection (cf. stories.md §B5 @security) : redondant côté client
// (filtre `accept` HTML + check sur le file.type) ET côté backend (validation
// du Content-Type sur PUT presigned). On rejette tôt côté FE pour UX.
//
// Limites métier (cf. stories.md §B5 + INV-FE3 numbers) :
//   - MAX_FILES         : 10
//   - MAX_FILE_SIZE     : 10 MB par fichier
//   - ACCEPTED_MIME     : image/*, video/*, application/pdf (whitelisting)
//
// Pattern d'injection : `presignFn` + `putFn` injectables pour faciliter
// les tests Vitest (pas de fetch réel).

import { api } from "../api";

export const EVIDENCE_MAX_FILES = 10;
export const EVIDENCE_MAX_FILE_SIZE_BYTES = 10 * 1024 * 1024; // 10 MB
/** Préfixes MIME acceptés. application/pdf est exact ; image/, video/ sont
 *  préfixes (acceptent image/jpeg, image/png, video/mp4, etc.). */
export const EVIDENCE_ACCEPTED_MIME_PREFIXES = [
  "image/",
  "video/",
  "application/pdf",
] as const;
/** Attribut `accept` HTML pour le file input — pattern stories.md §B5. */
export const EVIDENCE_ACCEPT_ATTR = "image/*,video/*,application/pdf";

export interface PresignResponse {
  /** URL HTTP PUT signée pour upload direct. */
  upload_url: string;
  /** URL publique à enregistrer dans evidence_attachments[]. */
  public_url: string;
  /** ISO 8601 — expire after. */
  expires_at?: string;
}

/** Validation MIME côté client (whitelisting strict). */
export function isAcceptedMime(mime: string): boolean {
  if (!mime) return false;
  return EVIDENCE_ACCEPTED_MIME_PREFIXES.some((prefix) =>
    prefix.endsWith("/") ? mime.startsWith(prefix) : mime === prefix,
  );
}

/** Validation taille fichier. Retourne true si OK (≤ limite). */
export function isAcceptedSize(sizeBytes: number): boolean {
  return sizeBytes > 0 && sizeBytes <= EVIDENCE_MAX_FILE_SIZE_BYTES;
}

export type UploadErrorCode =
  | "max-files"
  | "too-large"
  | "bad-mime"
  | "presign-failed"
  | "upload-failed";

export class UploadError extends Error {
  code: UploadErrorCode;
  constructor(code: UploadErrorCode, message: string) {
    super(message);
    this.code = code;
    this.name = "UploadError";
  }
}

/** Demande une presigned URL au backend. Endpoint conventionnel :
 *  POST /attachments/presign  { filename, mime, size_bytes } → PresignResponse. */
export async function requestPresignedUrl(
  file: File,
): Promise<PresignResponse> {
  return api.post<PresignResponse>("/attachments/presign", {
    filename: file.name,
    mime: file.type,
    size_bytes: file.size,
  });
}

/** PUT direct du blob vers l'URL signée. Retourne `public_url` ou throw. */
export async function putToPresignedUrl(
  uploadUrl: string,
  file: File,
  publicUrl: string,
): Promise<string> {
  const res = await fetch(uploadUrl, {
    method: "PUT",
    body: file,
    headers: { "Content-Type": file.type },
  });
  if (!res.ok) {
    throw new UploadError(
      "upload-failed",
      `S3/MinIO PUT a renvoyé ${res.status}`,
    );
  }
  return publicUrl;
}

/** Orchestration end-to-end :
 *  - check MIME / size côté client ;
 *  - presign + PUT ;
 *  - retourne `public_url` à stocker dans evidence_attachments. */
export async function uploadEvidence(
  file: File,
  options?: {
    presignFn?: (f: File) => Promise<PresignResponse>;
    putFn?: (uploadUrl: string, f: File, publicUrl: string) => Promise<string>;
  },
): Promise<string> {
  if (!isAcceptedMime(file.type)) {
    throw new UploadError(
      "bad-mime",
      `Type non autorisé : ${file.type || "inconnu"}. Acceptés : image, vidéo, PDF.`,
    );
  }
  if (!isAcceptedSize(file.size)) {
    throw new UploadError(
      "too-large",
      `Taille max ${EVIDENCE_MAX_FILE_SIZE_BYTES / 1024 / 1024} MB (vous avez ${(file.size / 1024 / 1024).toFixed(1)} MB).`,
    );
  }
  const presign = options?.presignFn ?? requestPresignedUrl;
  const put = options?.putFn ?? putToPresignedUrl;

  let signed: PresignResponse;
  try {
    signed = await presign(file);
  } catch (err) {
    throw new UploadError(
      "presign-failed",
      err instanceof Error ? err.message : String(err),
    );
  }
  return put(signed.upload_url, file, signed.public_url);
}
