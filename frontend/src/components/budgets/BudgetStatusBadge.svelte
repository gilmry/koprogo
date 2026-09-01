<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from '../../lib/i18n';

  let {
    status,
  }: {
    status: string;
  } = $props();

  // L'API renvoie l'enum en PascalCase ('Draft', 'Submitted'), conformement a
  // `docs/api/openapi.json`. Les cles etaient en minuscules : aucune ne
  // correspondait, et le repli `label: s` affichait le statut brut en anglais.
  //
  // La normalisation en minuscules plutot qu'un simple changement de casse des
  // cles : le MEME enum circule en snake_case dans la base et dans le filtre
  // `?status=draft` accepte par l'API. Indexer sur une forme normalisee evite
  // que la prochaine divergence de casse reproduise le meme silence.
  function getBadge(s: string): { class: string; label: string } {
    const badges: Record<string, { class: string; label: string }> = {
      'draft': { class: 'bg-gray-100 text-gray-800', label: $_('budgets.status.draft') },
      'submitted': { class: 'bg-blue-100 text-blue-800', label: $_('budgets.status.submitted') },
      'approved': { class: 'bg-green-100 text-green-800', label: $_('budgets.status.approved') },
      'rejected': { class: 'bg-red-100 text-red-800', label: $_('budgets.status.rejected') },
      'archived': { class: 'bg-yellow-100 text-yellow-800', label: $_('budgets.status.archived') },
    };
    return badges[(s ?? '').toLowerCase()] || { class: 'bg-gray-100 text-gray-800', label: s };
  }

  let badge = $derived(getBadge(status));
</script>

<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {badge.class}">
  {badge.label}
</span>
