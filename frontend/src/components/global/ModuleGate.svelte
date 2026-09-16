<script lang="ts">
  // Story 5.2 — ModuleGate (garde d'affichage par module ACP).
  //
  // ADR-0015 (modularité par capacité) ; deps Story 5.1 (#585, backend).
  //
  // `<ModuleGate module="community">…</ModuleGate>` ne rend son contenu que
  // si le module est activé pour l'ACP couramment sélectionnée
  // (`scope.selectedAcpId`, cf. `stores/scope.svelte.ts`). Se resynchronise
  // automatiquement à chaque bascule d'ACP en cours de session (AC @edge).
  //
  // ── Ce que ce composant n'est PAS (AC @security) ─────────────────────────
  //
  // Masquer un menu n'a jamais cloisonné quoi que ce soit. La décision
  // d'activation vient TOUJOURS d'un aller-retour serveur
  // (`listEnabledModules`, cf. `lib/api/modules.ts`) — jamais d'une règle
  // câblée côté client — et tout échec de cet aller-retour masque le
  // contenu (fail-closed, cf. `enabled_modules.svelte.ts`). La garde réelle
  // reste le middleware backend `ModuleGuard` (Story 5.1) : ce composant est
  // de la défense en profondeur purement cosmétique, pas un contrôle
  // d'accès. Cf. #585.
  //
  // ── Module inconnu (AC @negative) ────────────────────────────────────────
  //
  // Un nom de module hors `MODULE_NAMES` est une erreur de programmation
  // (typo dans un composant appelant), pas un état utilisateur. Elle échoue
  // visiblement côté développeur (`console.error`) et silencieusement côté
  // utilisateur (fragment vide, jamais de message affiché).
  //
  // data-testid (contrat stable, cf. issue #586) :
  //   module-gate-{{module}} — présent SSI le contenu est rendu.

  import type { Snippet } from "svelte";
  import { scope } from "../../stores/scope.svelte";
  import {
    isAcpModulesLoaded,
    isModuleEnabled,
    loadEnabledModulesForAcp,
  } from "../../stores/enabled_modules.svelte";
  import { isKnownModule, UnknownModuleError } from "../../lib/api/modules";

  let { module, children }: { module: string; children?: Snippet } =
    $props();

  /** ACP en cours de fetch — évite un re-fetch concurrent pour le même id. */
  let inFlightAcpId: string | null = null;

  // Resync sur bascule ACP : `scope.selectedAcpId` change (clic utilisateur
  // sur `AcpSelector`, ou tout autre mutateur du store `scope`) → on
  // recharge les modules de la nouvelle ACP si on ne les a pas déjà.
  $effect(() => {
    const acpId = scope.selectedAcpId;
    if (!acpId) return;
    if (isAcpModulesLoaded(acpId)) return;
    if (inFlightAcpId === acpId) return;

    inFlightAcpId = acpId;
    void loadEnabledModulesForAcp(acpId).finally(() => {
      if (inFlightAcpId === acpId) inFlightAcpId = null;
    });
  });

  // Échec visible côté développeur uniquement — jamais affiché à l'écran.
  $effect(() => {
    if (!isKnownModule(module)) {
      console.error(new UnknownModuleError(module));
    }
  });

  let isEnabled = $derived(
    isKnownModule(module) && isModuleEnabled(scope.selectedAcpId, module),
  );
</script>

{#if isEnabled}
  <div data-testid={`module-gate-${module}`}>
    {@render children?.()}
  </div>
{/if}
