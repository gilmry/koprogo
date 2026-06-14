<script lang="ts">
  // Story B7 (Phase B FE) — TechnicalSpecSignatureForm.
  //
  // Form de signature d'une TechnicalSpec en status PendingSignatures.
  // Réutilise l'atomique `SignatureForm` (lib/components/shared) qui porte la
  // checkbox RPGD + le bouton.
  //
  // Particularité : pour les rôles "mandataire" (amo/lawyer/architect — cf.
  // MANDATARY_ROLES), un mandate_id ACTIF est obligatoire. Le composant
  // affiche le rappel "Vous signez en tant que `<role>` via mandat #{id} actif
  // jusqu'au {date}" et empêche la signature si mandate manquant/expiré.
  //
  // INV-FE9 (a11y) :
  //   - Section status="status" pour annoncer le contexte mandate.
  //   - Aria-disabled cohérent (gating via externallyDisabled).
  //
  // data-testid (cf. stories.md §B7) :
  //   tech-spec-sign-mandate-info
  //   tech-spec-sign-submit         (= signature-sign-button du SignatureForm)
  //   tech-spec-sign-no-mandate-warning (si rôle mandataire sans mandate actif)

  import {
    MANDATARY_ROLES,
    type SignatoryRole,
    type SignTechnicalSpecRequest,
    type TechnicalSpecSignatureDto,
  } from "../../api/technical_specs";
  import SignatureForm from "../shared/SignatureForm.svelte";

  // ---------------------------------------------------------------------------
  // Props
  // ---------------------------------------------------------------------------

  let {
    /** UUID de la spec à signer. */
    specId,
    /** Rôle sous lequel on signe. */
    role,
    /** Mandate actif pour le rôle (UUID + date d'expiration ISO 8601).
     *  Obligatoire pour les rôles MANDATARY_ROLES. */
    activeMandate = null as
      | { id: string; validUntil: string }
      | null,
    /** Callback de signature — le parent gère l'appel API + rafraîchissement
     *  de la liste de signatures. */
    onSign,
  }: {
    specId: string;
    role: SignatoryRole;
    activeMandate?: { id: string; validUntil: string } | null;
    onSign: (
      specId: string,
      req: SignTechnicalSpecRequest,
    ) => Promise<TechnicalSpecSignatureDto>;
  } = $props();

  // ---------------------------------------------------------------------------
  // Dérivations
  // ---------------------------------------------------------------------------

  let isMandataryRole = $derived(MANDATARY_ROLES.includes(role));

  /** Mandate actif requis et présent ? */
  let mandateRequiredAndMissing = $derived(
    isMandataryRole && activeMandate === null,
  );

  /** Si rôle mandataire mais pas de mandate → on bloque la signature. */
  let signatureBlocked = $derived(mandateRequiredAndMissing);

  function formatMandateExpiry(iso: string): string {
    try {
      const d = new Date(iso);
      return new Intl.DateTimeFormat("fr-BE", {
        day: "numeric",
        month: "long",
        year: "numeric",
      }).format(d);
    } catch {
      return iso;
    }
  }

  async function handleSign(): Promise<void> {
    const req: SignTechnicalSpecRequest = {
      role,
      mandate_id:
        isMandataryRole && activeMandate ? activeMandate.id : null,
    };
    await onSign(specId, req);
  }
</script>

<section
  class="tech-spec-signature-form rounded-md border border-blue-200 bg-blue-50 p-4"
  aria-labelledby="tech-spec-sign-title"
>
  <h3
    id="tech-spec-sign-title"
    class="mb-2 text-sm font-semibold text-blue-900"
  >
    Signer cette fiche technique
  </h3>

  {#if isMandataryRole && activeMandate}
    <p
      data-testid="tech-spec-sign-mandate-info"
      class="mb-3 text-xs text-blue-800"
      role="status"
    >
      Vous signez en tant que <strong>{role}</strong> via mandat
      <code class="font-mono">#{activeMandate.id.slice(0, 8)}</code>
      actif jusqu'au {formatMandateExpiry(activeMandate.validUntil)}.
    </p>
  {:else if !isMandataryRole}
    <p
      data-testid="tech-spec-sign-mandate-info"
      class="mb-3 text-xs text-blue-800"
      role="status"
    >
      Vous signez en tant que <strong>{role}</strong> (rôle direct — aucun
      mandat requis).
    </p>
  {/if}

  {#if mandateRequiredAndMissing}
    <p
      data-testid="tech-spec-sign-no-mandate-warning"
      class="mb-3 rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-700"
      role="alert"
    >
      Aucun mandat <strong>{role}</strong> actif. Demandez au syndic d'émettre
      un mandat avant de signer.
    </p>
  {/if}

  <SignatureForm
    confirmLabel={`J'ai lu les livrables ci-dessus et j'approuve les engager en tant que ${role}.`}
    signLabel="Signer la fiche technique"
    signingLabel="Signature en cours…"
    onSign={handleSign}
    externallyDisabled={signatureBlocked}
    buttonTestIdOverride="tech-spec-sign-submit"
    checkboxTestIdOverride="tech-spec-sign-confirm-checkbox"
  />
</section>
