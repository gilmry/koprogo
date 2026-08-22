<script lang="ts">
  // Story B5 (Phase B FE) — EvidenceUpload (drag&drop, multi-fichiers).
  //
  // Réutilisé par :
  //   - TicketCreate.svelte (section conditionnelle kind=Complaint).
  //
  // Contrat métier (cf. stories.md §B5 + INV-FE3 numbers) :
  //   - Max 10 fichiers (anti-DoS + cohérence INV-FE3).
  //   - Max 10 MB par fichier.
  //   - MIME whitelisting (image/*, video/*, application/pdf) — client +
  //     backend (gotcha #1 §B5).
  //   - Upload en streaming dès drop (anti-pattern §B5 : NE PAS attendre
  //     submit final — latence UX).
  //
  // a11y (WCAG 2.1 AA — memory `a11y-wcag-aa-baseline`, checklist §B5) :
  //   - Dropzone a `role="button"` + `aria-label` descriptif + supporte
  //     navigation clavier (Enter / Space active le file input).
  //   - Preview thumbnails ont `alt` descriptif (nom du fichier).
  //   - `<progress>` (HTML natif) ou attribute aria-valuenow pour progression.
  //   - Zone d'erreur aria-live="polite" pour annoncer rejets.
  //
  // Gotcha #3 stories.md §B5 : nettoyer les `URL.createObjectURL` blobs au
  // unmount via `$effect` cleanup (sinon memory leak).
  //
  // data-testid (cf. stories.md §B5 + mission) :
  //   ticket-evidence-upload          (dropzone, role=button)
  //   ticket-evidence-file-input      (input hidden)
  //   ticket-evidence-preview-{i}     (thumbnail)
  //   ticket-evidence-remove-{i}      (bouton ×)
  //   ticket-evidence-count           (compteur "3/10")
  //   ticket-evidence-error           (zone erreur upload)

  import {
    EVIDENCE_MAX_FILES,
    EVIDENCE_MAX_FILE_SIZE_BYTES,
    EVIDENCE_ACCEPT_ATTR,
    isAcceptedMime,
    isAcceptedSize,
    uploadEvidence,
    UploadError,
  } from "../../utils/fileUpload";

  interface EvidenceItem {
    /** Stable id pour le keying Svelte each. */
    localId: string;
    /** Nom de fichier (pour alt et accessibilité). */
    filename: string;
    /** MIME pour décider thumbnail (image) vs icône (pdf/video). */
    mime: string;
    /** Bytes (affichage taille humaine). */
    sizeBytes: number;
    /** ObjectURL pour preview thumbnail — à révoquer au unmount. */
    previewUrl: string | null;
    /** State upload — pending (en cours), done (public_url disponible),
     *  error (échec — peut être retry). */
    status: "pending" | "done" | "error";
    /** Public URL retournée par le backend après PUT presigned. */
    publicUrl: string | null;
    /** Message d'erreur si status=error. */
    errorMsg?: string;
  }

  let {
    /** Liste des public_urls déjà uploadées — propagée vers parent. */
    value = $bindable<string[]>([]),
    /** Permet injection de l'orchestrateur upload pour tests. */
    onUpload = uploadEvidence as (f: File) => Promise<string>,
    /** Callback informatif (parent peut tracer / toaster). */
    onError = undefined as ((err: UploadError) => void) | undefined,
  }: {
    value?: string[];
    onUpload?: (f: File) => Promise<string>;
    onError?: (err: UploadError) => void;
  } = $props();

  // ---------------------------------------------------------------------------
  // State
  // ---------------------------------------------------------------------------

  let items = $state<EvidenceItem[]>([]);
  let dragOver = $state<boolean>(false);
  /** Dernière erreur de validation client (pré-upload). */
  let lastError = $state<string>("");
  /** Référence au file input — déclenché par click sur dropzone. */
  let fileInputEl: HTMLInputElement | null = $state(null);

  // ---------------------------------------------------------------------------
  // Derivations
  // ---------------------------------------------------------------------------

  let count = $derived(items.length);
  let countLabel = $derived(`${count}/${EVIDENCE_MAX_FILES}`);
  let atMax = $derived(count >= EVIDENCE_MAX_FILES);

  // ---------------------------------------------------------------------------
  // Effects
  // ---------------------------------------------------------------------------

  // Cleanup global au unmount (gotcha #3 stories.md §B5).
  $effect(() => {
    return () => {
      for (const it of items) {
        if (it.previewUrl) {
          try {
            URL.revokeObjectURL(it.previewUrl);
          } catch {
            /* ignore */
          }
        }
      }
    };
  });

  // Synchronise `value` (parent bindable) avec les items en status=done.
  $effect(() => {
    const urls = items
      .filter((it) => it.status === "done" && it.publicUrl !== null)
      .map((it) => it.publicUrl as string);
    // Comparaison shallow pour éviter une boucle d'update.
    const sameLength = urls.length === value.length;
    const allEqual =
      sameLength && urls.every((u, i) => u === value[i]);
    if (!allEqual) {
      value = urls;
    }
  });

  // ---------------------------------------------------------------------------
  // Handlers
  // ---------------------------------------------------------------------------

  function makeId(): string {
    return `ev-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
  }

  function makePreview(file: File): string | null {
    if (file.type.startsWith("image/")) {
      try {
        return URL.createObjectURL(file);
      } catch {
        return null;
      }
    }
    return null;
  }

  async function handleFiles(filesIn: FileList | File[]): Promise<void> {
    lastError = "";
    const files = Array.from(filesIn);

    for (const file of files) {
      // 1. Cap quantitatif AVANT validation MIME pour message clair.
      if (items.length >= EVIDENCE_MAX_FILES) {
        lastError = `Maximum ${EVIDENCE_MAX_FILES} preuves.`;
        if (onError)
          onError(new UploadError("max-files", lastError));
        break;
      }
      // 2. MIME whitelisting.
      if (!isAcceptedMime(file.type)) {
        lastError = `Type non autorisé : ${file.type || "inconnu"}. Acceptés : image, vidéo, PDF.`;
        if (onError)
          onError(new UploadError("bad-mime", lastError));
        continue;
      }
      // 3. Taille.
      if (!isAcceptedSize(file.size)) {
        const mb = (EVIDENCE_MAX_FILE_SIZE_BYTES / 1024 / 1024).toFixed(0);
        lastError = `Taille max ${mb} MB (vous avez ${(file.size / 1024 / 1024).toFixed(1)} MB).`;
        if (onError)
          onError(new UploadError("too-large", lastError));
        continue;
      }
      // 4. OK — ajoute l'item en pending + lance l'upload.
      const item: EvidenceItem = {
        localId: makeId(),
        filename: file.name,
        mime: file.type,
        sizeBytes: file.size,
        previewUrl: makePreview(file),
        status: "pending",
        publicUrl: null,
      };
      items = [...items, item];

      // Upload async — on ne bloque pas la boucle (parallèle multi-fichier).
      void uploadOne(item, file);
    }
  }

  async function uploadOne(item: EvidenceItem, file: File): Promise<void> {
    try {
      const publicUrl = await onUpload(file);
      items = items.map((it) =>
        it.localId === item.localId
          ? { ...it, status: "done", publicUrl }
          : it,
      );
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      items = items.map((it) =>
        it.localId === item.localId
          ? { ...it, status: "error", errorMsg: msg }
          : it,
      );
      if (onError && err instanceof UploadError) {
        onError(err);
      }
    }
  }

  function removeItem(localId: string): void {
    const target = items.find((it) => it.localId === localId);
    if (target?.previewUrl) {
      try {
        URL.revokeObjectURL(target.previewUrl);
      } catch {
        /* ignore */
      }
    }
    items = items.filter((it) => it.localId !== localId);
  }

  function onDropZoneKey(ev: KeyboardEvent): void {
    if (ev.key === "Enter" || ev.key === " ") {
      ev.preventDefault();
      fileInputEl?.click();
    }
  }

  function onDropZoneClick(): void {
    fileInputEl?.click();
  }

  function onDragOver(ev: DragEvent): void {
    ev.preventDefault();
    dragOver = true;
  }
  function onDragLeave(ev: DragEvent): void {
    ev.preventDefault();
    dragOver = false;
  }
  function onDrop(ev: DragEvent): void {
    ev.preventDefault();
    dragOver = false;
    const files = ev.dataTransfer?.files;
    if (files && files.length > 0) {
      void handleFiles(files);
    }
  }
  function onFileChange(ev: Event): void {
    const target = ev.target as HTMLInputElement;
    if (target.files && target.files.length > 0) {
      void handleFiles(target.files);
      // Reset pour permettre re-upload du même nom.
      target.value = "";
    }
  }
</script>

<section class="ticket-evidence-upload flex flex-col gap-2">
  <div class="flex items-center justify-between">
    <label
      id="ticket-evidence-label"
      for="ticket-evidence-file-input"
      class="block text-sm font-medium text-gray-700"
    >
      Preuves (photos, vidéos, PDF)
    </label>
    <span
      data-testid="ticket-evidence-count"
      class="text-xs text-gray-500"
      aria-live="polite"
    >
      {countLabel}
    </span>
  </div>

  <!-- Dropzone (role=button + a11y clavier) -->
  <div
    data-testid="ticket-evidence-upload"
    role="button"
    tabindex={atMax ? -1 : 0}
    aria-disabled={atMax}
    aria-labelledby="ticket-evidence-label"
    aria-label={atMax
      ? "Maximum de preuves atteint"
      : "Glissez-déposez des preuves ou cliquez pour sélectionner"}
    onclick={onDropZoneClick}
    onkeydown={onDropZoneKey}
    ondragover={onDragOver}
    ondragleave={onDragLeave}
    ondrop={onDrop}
    class={`flex min-h-[120px] cursor-pointer flex-col items-center justify-center rounded border-2 border-dashed p-4 text-center text-sm transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 ${
      dragOver
        ? "border-blue-500 bg-blue-50"
        : atMax
          ? "cursor-not-allowed border-gray-200 bg-gray-50 text-gray-400"
          : "border-gray-300 bg-white text-gray-600 hover:border-blue-400"
    }`}
  >
    <svg
      aria-hidden="true"
      width="32"
      height="32"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
    >
      <path d="M12 5 V19 M5 12 L12 5 L19 12" />
    </svg>
    <p class="mt-2">
      {atMax
        ? `Maximum ${EVIDENCE_MAX_FILES} preuves atteint`
        : "Glissez-déposez ou cliquez pour ajouter (image, vidéo, PDF — 10 MB max)"}
    </p>
  </div>

  <!-- File input (caché — déclenché par dropzone) -->
  <input
    bind:this={fileInputEl}
    id="ticket-evidence-file-input"
    data-testid="ticket-evidence-file-input"
    type="file"
    multiple
    accept={EVIDENCE_ACCEPT_ATTR}
    onchange={onFileChange}
    disabled={atMax}
    class="sr-only"
  />

  <!-- Zone d'erreur (validation client) -->
  {#if lastError}
    <p
      data-testid="ticket-evidence-error"
      class="rounded border border-red-300 bg-red-50 p-2 text-xs text-red-700"
      role="alert"
      aria-live="polite"
    >
      {lastError}
    </p>
  {/if}

  <!-- Liste preview -->
  {#if items.length > 0}
    <ul class="grid grid-cols-2 gap-2 sm:grid-cols-3">
      {#each items as it, i (it.localId)}
        <li
          class={`relative flex flex-col gap-1 rounded border p-2 text-xs ${
            it.status === "error"
              ? "border-red-300 bg-red-50"
              : it.status === "pending"
                ? "border-blue-300 bg-blue-50"
                : "border-green-300 bg-green-50"
          }`}
        >
          {#if it.previewUrl}
            <img
              data-testid={`ticket-evidence-preview-${i}`}
              src={it.previewUrl}
              alt={`Preuve : ${it.filename}`}
              class="h-20 w-full rounded object-cover"
            />
          {:else}
            <div
              data-testid={`ticket-evidence-preview-${i}`}
              class="flex h-20 w-full items-center justify-center rounded bg-white text-gray-500"
              role="img"
              aria-label={`Preuve : ${it.filename}`}
            >
              {it.mime === "application/pdf" ? "PDF" : "VID"}
            </div>
          {/if}
          <span class="truncate" title={it.filename}>{it.filename}</span>
          {#if it.status === "pending"}
            <progress
              aria-label={`Upload de ${it.filename}`}
              class="h-1 w-full"
            ></progress>
          {/if}
          {#if it.status === "error"}
            <span class="text-red-700" role="alert">{it.errorMsg}</span>
          {/if}
          <button
            type="button"
            data-testid={`ticket-evidence-remove-${i}`}
            onclick={() => removeItem(it.localId)}
            aria-label={`Retirer ${it.filename}`}
            class="absolute right-1 top-1 flex h-6 w-6 items-center justify-center rounded-full bg-white text-gray-600 shadow-sm hover:bg-red-100 hover:text-red-700 focus-visible:outline-2"
          >
            ×
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</section>
