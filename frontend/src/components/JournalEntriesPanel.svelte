<script lang="ts">
  // Svelte 5 runes mode
  //
  // Assemble la saisie et la consultation du grand livre.
  //
  // POURQUOI UN COMPOSANT D'ASSEMBLAGE
  //   La page est en Astro : ses props d'ilot doivent etre serialisables, donc
  //   `onSuccess` (une fonction) ne peut pas y etre passe. Sans ce niveau
  //   intermediaire, la liste ne saurait pas qu'une ecriture vient d'etre
  //   creee, et il faudrait recharger la page a la main.

  import JournalEntryForm from "./JournalEntryForm.svelte";
  import { scope } from "../stores/scope.svelte";
  import { _ } from "../lib/i18n";
  import JournalEntryList from "./JournalEntryList.svelte";

  let { buildingId = null }: { buildingId?: string | null } = $props();

  /// L'immeuble effectif : celui reçu en propriété, ou celui du périmètre.
  ///
  /// ── Ce qui ne marchait pas ───────────────────────────────────────────────
  ///
  /// `journal-entries.astro` passait `buildingId={null}` **en dur**. Chaque
  /// écriture manuelle partait donc sans immeuble, et le serveur la refusait :
  ///
  ///     Impossible de déterminer l'ACP : une écriture manuelle doit désigner
  ///     un immeuble
  ///
  /// Le refus est juste — une pièce comptable qui ne désigne pas sa
  /// copropriété n'est imputable à personne, c'est le même invariant que #770.
  /// Mais l'écran ne donnait aucun moyen de le satisfaire : **le formulaire
  /// existait, le bouton existait, et la création était impossible pour tout
  /// le monde.**
  ///
  /// Constaté par `AccountantJournalEntriesJourney.spec.ts`, dont l'échec
  /// passait pour un défaut de fixture (#832).
  ///
  /// L'immeuble vient donc du périmètre sélectionné, comme partout ailleurs.
  let immeubleEffectif = $derived(buildingId ?? scope.selectedBuildingId);

  // Un compteur plutot qu'un booleen : deux creations successives doivent
  // declencher deux rechargements, ce qu'un drapeau remis a false ne
  // garantirait pas sans effet de bord.
  let reloadToken = $state(0);
</script>

<div class="space-y-8">
  {#if !immeubleEffectif}
    <!-- Dire le refus AVANT la soumission, pas après.
         Le serveur refuse à raison une écriture sans immeuble ; ce qui ne va
         pas, c'est de laisser remplir un formulaire entier pour l'apprendre
         au dernier clic. -->
    <div
      class="rounded-md border border-amber-200 bg-amber-50 p-4"
      data-testid="journal-entries-no-building"
      role="status"
    >
      <p class="text-sm font-semibold text-amber-900">
        {$_("journal.selectBuildingTitle")}
      </p>
      <p class="mt-1 text-sm text-amber-800">
        {$_("journal.selectBuildingBody")}
      </p>
    </div>
  {:else}
    <JournalEntryForm
      buildingId={immeubleEffectif}
      onSuccess={() => (reloadToken += 1)}
    />
    <JournalEntryList buildingId={immeubleEffectif} {reloadToken} />
  {/if}
</div>
