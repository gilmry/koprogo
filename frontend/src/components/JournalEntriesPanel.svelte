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
  import JournalEntryList from "./JournalEntryList.svelte";

  let { buildingId = null }: { buildingId?: string | null } = $props();

  // Un compteur plutot qu'un booleen : deux creations successives doivent
  // declencher deux rechargements, ce qu'un drapeau remis a false ne
  // garantirait pas sans effet de bord.
  let reloadToken = $state(0);
</script>

<div class="space-y-8">
  <JournalEntryForm {buildingId} onSuccess={() => (reloadToken += 1)} />
  <JournalEntryList {buildingId} {reloadToken} />
</div>
