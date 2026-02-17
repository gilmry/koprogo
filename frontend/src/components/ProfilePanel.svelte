<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import { authStore } from '../stores/auth';
  import { UserRole } from '../lib/types';
  import type { GdprExport, GdprCanEraseResponse } from '../lib/types';
  import { toast } from '../stores/toast';

  let editMode = false;
  let saving = false;
  let editEmail = '';
  let editFirstName = '';
  let editLastName = '';

  // GDPR states
  let gdprExporting = false;
  let gdprErasing = false;
  let gdprRestricting = false;
  let gdprMarketingLoading = false;
  let canErase: GdprCanEraseResponse | null = null;

  $: user = $authStore.user;

  onMount(async () => {
    if (user) {
      editEmail = user.email;
      editFirstName = user.first_name;
      editLastName = user.last_name;
    }
  });

  function startEdit() {
    if (user) {
      editEmail = user.email;
      editFirstName = user.first_name;
      editLastName = user.last_name;
      editMode = true;
    }
  }

  function cancelEdit() {
    editMode = false;
  }

  async function saveProfile() {
    if (!editFirstName.trim() || !editLastName.trim() || !editEmail.trim()) {
      toast.error('Tous les champs sont obligatoires');
      return;
    }

    try {
      saving = true;
      await api.put('/gdpr/rectify', {
        email: editEmail !== user?.email ? editEmail : undefined,
        first_name: editFirstName !== user?.first_name ? editFirstName : undefined,
        last_name: editLastName !== user?.last_name ? editLastName : undefined,
      });

      // Update local auth store
      if (user) {
        await authStore.updateUser({
          ...user,
          email: editEmail,
          first_name: editFirstName,
          last_name: editLastName,
        });
      }

      toast.success('Profil mis à jour (Art. 16 RGPD - Droit de rectification)');
      editMode = false;
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de la mise à jour du profil');
    } finally {
      saving = false;
    }
  }

  async function handleGdprExport() {
    try {
      gdprExporting = true;
      const data = await api.get<GdprExport>('/gdpr/export');
      // Download as JSON file
      const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `koprogo-donnees-personnelles-${new Date().toISOString().slice(0, 10)}.json`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
      toast.success('Données exportées (Art. 15 RGPD - Droit d\'accès)');
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de l\'export');
    } finally {
      gdprExporting = false;
    }
  }

  async function handleCheckCanErase() {
    try {
      canErase = await api.get<GdprCanEraseResponse>('/gdpr/can-erase');
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de la vérification');
    }
  }

  async function handleGdprErase() {
    if (!confirm('ATTENTION : Cette action est IRRÉVERSIBLE. Toutes vos données personnelles seront anonymisées. Êtes-vous sûr ?')) return;
    if (!confirm('Dernière confirmation : vos données seront définitivement anonymisées. Continuer ?')) return;

    try {
      gdprErasing = true;
      await api.delete('/gdpr/erase');
      toast.success('Données anonymisées (Art. 17 RGPD - Droit à l\'effacement)');
      await authStore.logout();
      window.location.href = '/login';
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de l\'anonymisation');
    } finally {
      gdprErasing = false;
    }
  }

  async function handleRestrictProcessing() {
    if (!confirm('Restreindre le traitement de vos données ? Certaines fonctionnalités pourraient être limitées.')) return;

    try {
      gdprRestricting = true;
      await api.put('/gdpr/restrict-processing', {});
      toast.success('Traitement restreint (Art. 18 RGPD - Droit à la limitation)');
    } catch (err: any) {
      toast.error(err.message || 'Erreur');
    } finally {
      gdprRestricting = false;
    }
  }

  async function handleMarketingOptOut(optOut: boolean) {
    try {
      gdprMarketingLoading = true;
      await api.put('/gdpr/marketing-preference', { opt_out: optOut });
      toast.success(optOut
        ? 'Désabonnement marketing effectué (Art. 21 RGPD)'
        : 'Préférences marketing mises à jour');
    } catch (err: any) {
      toast.error(err.message || 'Erreur');
    } finally {
      gdprMarketingLoading = false;
    }
  }

  function getRoleLabel(role: UserRole | undefined): string {
    switch (role) {
      case UserRole.SUPERADMIN: return 'Administrateur plateforme';
      case UserRole.SYNDIC: return 'Syndic';
      case UserRole.ACCOUNTANT: return 'Comptable';
      case UserRole.OWNER: return 'Copropriétaire';
      default: return 'Utilisateur';
    }
  }

  function formatDate(dateStr: string | undefined): string {
    if (!dateStr) return '-';
    return new Date(dateStr).toLocaleDateString('fr-BE', {
      day: '2-digit',
      month: 'long',
      year: 'numeric',
    });
  }
</script>

{#if !user}
  <div class="p-8 text-center">
    <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-amber-600"></div>
    <p class="mt-2 text-sm text-gray-500">Chargement du profil...</p>
  </div>
{:else}
  <div class="space-y-6">
    <!-- Personal Information -->
    <div class="bg-white rounded-lg shadow-lg overflow-hidden">
      <div class="bg-gradient-to-r from-amber-600 to-amber-700 px-6 py-4">
        <div class="flex items-center justify-between">
          <h2 class="text-xl font-semibold text-white">Informations personnelles</h2>
          {#if !editMode}
            <button on:click={startEdit}
              class="px-3 py-1.5 bg-white/20 text-white rounded-lg text-sm font-medium hover:bg-white/30 transition-colors">
              Modifier
            </button>
          {/if}
        </div>
      </div>

      <div class="p-6">
        {#if editMode}
          <div class="space-y-4">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label for="firstName" class="block text-sm font-medium text-gray-700 mb-1">Prénom</label>
                <input id="firstName" type="text" bind:value={editFirstName}
                  class="w-full rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500" />
              </div>
              <div>
                <label for="lastName" class="block text-sm font-medium text-gray-700 mb-1">Nom</label>
                <input id="lastName" type="text" bind:value={editLastName}
                  class="w-full rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500" />
              </div>
              <div class="md:col-span-2">
                <label for="email" class="block text-sm font-medium text-gray-700 mb-1">Email</label>
                <input id="email" type="email" bind:value={editEmail}
                  class="w-full rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500" />
              </div>
            </div>
            <div class="flex gap-2">
              <button on:click={saveProfile} disabled={saving}
                class="px-4 py-2 bg-amber-600 text-white rounded-lg text-sm font-medium hover:bg-amber-700 disabled:opacity-50 transition-colors">
                {saving ? 'Enregistrement...' : 'Enregistrer'}
              </button>
              <button on:click={cancelEdit}
                class="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-200 transition-colors">
                Annuler
              </button>
            </div>
            <p class="text-xs text-gray-400">
              La modification utilise le droit de rectification (Art. 16 RGPD).
            </p>
          </div>
        {:else}
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <p class="text-xs text-gray-500 uppercase tracking-wider mb-1">Prénom</p>
              <p class="text-lg text-gray-900">{user.first_name || '-'}</p>
            </div>
            <div>
              <p class="text-xs text-gray-500 uppercase tracking-wider mb-1">Nom</p>
              <p class="text-lg text-gray-900">{user.last_name || '-'}</p>
            </div>
            <div>
              <p class="text-xs text-gray-500 uppercase tracking-wider mb-1">Email</p>
              <p class="text-lg text-gray-900">{user.email}</p>
            </div>
            <div>
              <p class="text-xs text-gray-500 uppercase tracking-wider mb-1">Membre depuis</p>
              <p class="text-lg text-gray-900">{formatDate(user.created_at)}</p>
            </div>
          </div>
        {/if}
      </div>
    </div>

    <!-- Role Information -->
    <div class="bg-white rounded-lg shadow-lg overflow-hidden">
      <div class="px-6 py-4 border-b border-gray-200">
        <h2 class="text-lg font-semibold text-gray-900">Rôles et accès</h2>
      </div>
      <div class="p-6">
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-xs text-gray-500 uppercase tracking-wider mb-1">Rôle actif</p>
              <p class="text-lg text-gray-900">{getRoleLabel(user.role)}</p>
            </div>
            <span class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-amber-100 text-amber-800">
              {getRoleLabel(user.role)}
            </span>
          </div>

          {#if user.roles && user.roles.length > 1}
            <div class="mt-4">
              <p class="text-xs text-gray-500 uppercase tracking-wider mb-2">Tous les rôles</p>
              <div class="space-y-2">
                {#each user.roles as role}
                  <div class="flex items-center justify-between p-2 rounded-md {role.id === user.activeRole?.id ? 'bg-amber-50 border border-amber-200' : 'bg-gray-50'}">
                    <span class="text-sm text-gray-900">{getRoleLabel(role.role)}</span>
                    <div class="flex items-center gap-2">
                      {#if role.organizationId}
                        <span class="text-xs text-gray-500">{role.organizationId.slice(0, 8)}...</span>
                      {/if}
                      {#if role.isPrimary}
                        <span class="text-xs text-amber-600 font-medium">Principal</span>
                      {/if}
                      {#if role.id === user.activeRole?.id}
                        <span class="text-xs text-green-600 font-medium">Actif</span>
                      {/if}
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          {#if user.buildingIds && user.buildingIds.length > 0}
            <div class="mt-4">
              <p class="text-xs text-gray-500 uppercase tracking-wider mb-1">Immeubles associés</p>
              <p class="text-sm text-gray-700">{user.buildingIds.length} immeuble{user.buildingIds.length > 1 ? 's' : ''}</p>
            </div>
          {/if}
        </div>
      </div>
    </div>

    <!-- GDPR Section -->
    <div class="bg-white rounded-lg shadow-lg overflow-hidden border-l-4 border-blue-500">
      <div class="px-6 py-4 border-b border-gray-200">
        <div class="flex items-center">
          <svg class="w-5 h-5 text-blue-600 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/>
          </svg>
          <h2 class="text-lg font-semibold text-gray-900">RGPD - Mes données personnelles</h2>
        </div>
        <p class="mt-1 text-sm text-gray-500">
          Conformément au Règlement Général sur la Protection des Données.
        </p>
      </div>

      <div class="p-6 space-y-4">
        <!-- Art. 15 - Right of Access -->
        <div class="flex items-start justify-between p-4 bg-blue-50 rounded-lg">
          <div class="flex-1">
            <h3 class="text-sm font-medium text-gray-900">Droit d'accès (Art. 15)</h3>
            <p class="text-xs text-gray-600 mt-1">Télécharger une copie complète de toutes vos données personnelles.</p>
          </div>
          <button on:click={handleGdprExport} disabled={gdprExporting}
            class="ml-4 px-3 py-1.5 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 disabled:opacity-50 transition-colors whitespace-nowrap">
            {gdprExporting ? 'Export...' : 'Exporter mes données'}
          </button>
        </div>

        <!-- Art. 18 - Right to Restriction -->
        <div class="flex items-start justify-between p-4 bg-yellow-50 rounded-lg">
          <div class="flex-1">
            <h3 class="text-sm font-medium text-gray-900">Droit à la limitation (Art. 18)</h3>
            <p class="text-xs text-gray-600 mt-1">Restreindre le traitement de vos données. Certaines fonctionnalités seront limitées.</p>
          </div>
          <button on:click={handleRestrictProcessing} disabled={gdprRestricting}
            class="ml-4 px-3 py-1.5 bg-yellow-600 text-white rounded-lg text-sm font-medium hover:bg-yellow-700 disabled:opacity-50 transition-colors whitespace-nowrap">
            {gdprRestricting ? 'En cours...' : 'Restreindre'}
          </button>
        </div>

        <!-- Art. 21 - Right to Object (Marketing) -->
        <div class="flex items-start justify-between p-4 bg-purple-50 rounded-lg">
          <div class="flex-1">
            <h3 class="text-sm font-medium text-gray-900">Marketing (Art. 21)</h3>
            <p class="text-xs text-gray-600 mt-1">Gérer vos préférences de communication marketing.</p>
          </div>
          <div class="ml-4 flex gap-2">
            <button on:click={() => handleMarketingOptOut(true)} disabled={gdprMarketingLoading}
              class="px-3 py-1.5 bg-gray-100 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-200 disabled:opacity-50 transition-colors whitespace-nowrap">
              Se désabonner
            </button>
            <button on:click={() => handleMarketingOptOut(false)} disabled={gdprMarketingLoading}
              class="px-3 py-1.5 bg-purple-600 text-white rounded-lg text-sm font-medium hover:bg-purple-700 disabled:opacity-50 transition-colors whitespace-nowrap">
              S'abonner
            </button>
          </div>
        </div>

        <!-- Art. 17 - Right to Erasure -->
        <div class="flex items-start justify-between p-4 bg-red-50 rounded-lg border border-red-200">
          <div class="flex-1">
            <h3 class="text-sm font-medium text-red-900">Droit à l'effacement (Art. 17)</h3>
            <p class="text-xs text-red-700 mt-1">Demander l'anonymisation définitive de toutes vos données. Cette action est <strong>irréversible</strong>.</p>
            {#if canErase}
              {#if canErase.can_erase}
                <p class="text-xs text-green-700 mt-2 font-medium">Vos données peuvent être effacées.</p>
              {:else}
                <p class="text-xs text-red-700 mt-2 font-medium">Effacement impossible : {canErase.legal_holds} obligation(s) légale(s) en cours.</p>
              {/if}
            {/if}
          </div>
          <div class="ml-4 flex flex-col gap-2">
            <button on:click={handleCheckCanErase}
              class="px-3 py-1.5 bg-gray-100 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-200 transition-colors whitespace-nowrap">
              Vérifier éligibilité
            </button>
            <button on:click={handleGdprErase} disabled={gdprErasing || (canErase !== null && !canErase.can_erase)}
              class="px-3 py-1.5 bg-red-600 text-white rounded-lg text-sm font-medium hover:bg-red-700 disabled:opacity-50 transition-colors whitespace-nowrap">
              {gdprErasing ? 'Anonymisation...' : 'Effacer mes données'}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}
