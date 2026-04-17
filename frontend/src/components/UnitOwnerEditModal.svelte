<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from '../lib/i18n';
  import { api } from '../lib/api';
  import type { UnitOwner, Owner } from '../lib/types';
  import Button from './ui/Button.svelte';

  let { open = $bindable(false), unitOwner = null, currentTotalPercentage = 0, onupdated = () => {}, onclose = () => {} }: {
    open?: boolean; unitOwner?: (UnitOwner & { owner?: Owner }) | null; currentTotalPercentage?: number; onupdated?: () => void; onclose?: () => void;
  } = $props();

  let ownershipPercentage = $state(0); let isPrimaryContact = $state(false);
  let loading = $state(false); let error = $state('');

  let currentOwnerPercentage = $derived(unitOwner ? unitOwner.ownership_percentage : 0);
  let totalWithoutCurrent = $derived(currentTotalPercentage - currentOwnerPercentage);
  let availablePercentage = $derived(Math.max(0, (1 - totalWithoutCurrent) * 100));
  let wouldExceed = $derived(ownershipPercentage > 0 && ownershipPercentage > availablePercentage + 0.01);

  $effect(() => { if (unitOwner && open) { ownershipPercentage = unitOwner.ownership_percentage * 100; isPrimaryContact = unitOwner.is_primary_contact; error = ''; } });

  function handleClose() { onclose(); }

  async function handleSubmit() {
    error = '';
    if (ownershipPercentage <= 0 || ownershipPercentage > 100) { error = $_('units.percentage_must_be_valid'); return; }
    if (ownershipPercentage > availablePercentage + 0.01) { error = $_('units.quota_would_exceed', { values: { available: availablePercentage.toFixed(2) } }); return; }
    if (!unitOwner) { error = $_('units.no_relationship_selected'); return; }
    try { loading = true; const percentageDecimal = ownershipPercentage / 100; await api.put(`/unit-owners/${unitOwner.id}`, { ownership_percentage: percentageDecimal, is_primary_contact: isPrimaryContact }); onupdated(); open = false; } catch (e) { error = e instanceof Error ? e.message : $_('common.error_updating'); console.error('Error updating unit owner:', e); } finally { loading = false; }
  }
</script>

{#if open}
  <div class="fixed inset-0 z-50 overflow-y-auto"><div class="flex min-h-screen items-center justify-center p-4">
    <div class="fixed inset-0 bg-black bg-opacity-50 transition-opacity" onclick={handleClose}></div>
    <div class="relative bg-white rounded-lg shadow-xl max-w-md w-full p-6 z-10">
      <div class="flex justify-between items-center mb-4"><h2 class="text-xl font-bold text-gray-900">{$_('units.edit_ownership')}</h2><button onclick={handleClose} class="text-gray-400 hover:text-gray-500"><span class="text-2xl">&times;</span></button></div>
      {#if unitOwner?.owner}<div class="mb-4 p-3 bg-gray-50 rounded-lg"><p class="font-semibold text-gray-900">{unitOwner.owner.first_name} {unitOwner.owner.last_name}</p><p class="text-sm text-gray-600">{unitOwner.owner.email}</p></div>{/if}
      {#if error}<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-4">{error}</div>{/if}
      <form onsubmit={(e: Event) => { e.preventDefault(); handleSubmit(); }} class="space-y-4">
        <div><label for="ownershipPercentage" class="block text-sm font-medium text-gray-700 mb-1">{$_('units.ownership_percentage')} *</label>
          <input id="ownershipPercentage" type="number" step="0.01" min="0.01" max={availablePercentage > 0 ? availablePercentage : 100} bind:value={ownershipPercentage} placeholder="Ex: 50.00" required class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500" class:border-red-500={wouldExceed} class:border-gray-300={!wouldExceed} />
          <div class="flex justify-between items-center mt-1"><p class="text-xs text-gray-500">{$_('units.quota_sum_100')}</p><p class="text-xs font-semibold" class:text-green-600={availablePercentage > 0} class:text-red-600={availablePercentage <= 0}>{$_('units.maximum', { values: { pct: availablePercentage.toFixed(2) } })}</p></div>
          {#if wouldExceed}<p class="text-xs text-red-600 mt-1 font-medium">{$_('units.quota_would_exceed_100')}</p>{/if}
        </div>
        <div class="flex items-center"><input id="isPrimaryContact" type="checkbox" bind:checked={isPrimaryContact} class="h-4 w-4 text-primary-600 focus:ring-primary-500 border-gray-300 rounded" /><label for="isPrimaryContact" class="ml-2 block text-sm text-gray-700">{$_('units.primary_contact')}</label></div>
        <p class="text-xs text-gray-500 -mt-2 ml-6">{$_('units.primary_contact_help')}</p>
        <div class="flex gap-2 pt-4"><Button type="submit" variant="primary" disabled={loading || wouldExceed}>{loading ? $_('common.saving') : $_('common.save')}</Button><Button type="button" variant="outline" onclick={handleClose}>{$_('common.cancel')}</Button></div>
      </form>
    </div>
  </div></div>
{/if}
