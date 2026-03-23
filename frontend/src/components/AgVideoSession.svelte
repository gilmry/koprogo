<script lang="ts">
  import '../lib/i18n';
  import { _ } from 'svelte-i18n';
  import { apiEndpoint } from '../lib/config';
  import { authStore } from '../stores/auth';
  import Button from './ui/Button.svelte';
  import { toast } from '../stores/toast';

  export let meetingId: string;
  export let readOnly = false;

  interface AgSession {
    id: string;
    meeting_id: string;
    platform: string;
    video_url: string;
    host_url?: string;
    status: 'Scheduled' | 'Live' | 'Ended' | 'Cancelled';
    remote_attendees_count: number;
    remote_voting_power: number;
    quorum_remote_contribution: number;
    access_password?: string;
    waiting_room_enabled: boolean;
    recording_enabled: boolean;
    created_at: string;
  }

  let session: AgSession | null = null;
  let loading = false;
  let creating = false;
  let showForm = false;

  let form = {
    platform: 'Jitsi',
    video_url: '',
    host_url: '',
    access_password: '',
    waiting_room_enabled: true,
    recording_enabled: false,
  };

  const platforms = ['Zoom', 'Teams', 'Meet', 'Jitsi', 'Whereby'];

  async function loadSession() {
    loading = true;
    try {
      const token = $authStore.token;
      const res = await fetch(apiEndpoint(`/meetings/${meetingId}/ag-session`), {
        headers: { Authorization: `Bearer ${token}` },
      });
      if (res.ok) {
        session = await res.json();
      } else if (res.status === 404) {
        session = null;
      }
    } catch (e) {
      console.error('Error loading AG session:', e);
    } finally {
      loading = false;
    }
  }

  async function createSession() {
    creating = true;
    try {
      const token = $authStore.token;
      const res = await fetch(apiEndpoint(`/meetings/${meetingId}/ag-session`), {
        method: 'POST',
        headers: {
          Authorization: `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(form),
      });
      if (res.ok) {
        session = await res.json();
        showForm = false;
        toast.show($_('agSession.createSuccess'), 'success');
      } else {
        const err = await res.json();
        toast.show(err.error || $_('agSession.createError'), 'error');
      }
    } catch (e) {
      toast.show($_('agSession.connectionError'), 'error');
    } finally {
      creating = false;
    }
  }

  async function startSession() {
    if (!session) return;
    const token = $authStore.token;
    const res = await fetch(apiEndpoint(`/ag-sessions/${session.id}/start`), {
      method: 'PUT',
      headers: { Authorization: `Bearer ${token}` },
    });
    if (res.ok) {
      session = await res.json();
      toast.show($_('agSession.startSuccess'), 'success');
    }
  }

  async function endSession() {
    if (!session) return;
    const token = $authStore.token;
    const res = await fetch(apiEndpoint(`/ag-sessions/${session.id}/end`), {
      method: 'PUT',
      headers: { Authorization: `Bearer ${token}` },
    });
    if (res.ok) {
      session = await res.json();
      toast.show($_('agSession.endSuccess'), 'success');
    }
  }

  function getStatusBadge(status: string) {
    const map: Record<string, string> = {
      Scheduled: 'bg-blue-100 text-blue-800',
      Live: 'bg-green-100 text-green-800',
      Ended: 'bg-gray-100 text-gray-800',
      Cancelled: 'bg-red-100 text-red-800',
    };
    return map[status] || 'bg-gray-100 text-gray-800';
  }

  $: if (meetingId) loadSession();
</script>

<div class="bg-white rounded-lg border border-gray-200 p-4">
  <div class="flex items-center justify-between mb-4">
    <h3 class="text-sm font-semibold text-gray-900 flex items-center gap-2">
      <svg class="h-4 w-4 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 10l4.553-2.069A1 1 0 0121 8.82v6.36a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z"/>
      </svg>
      {$_('agSession.title')}
    </h3>
    {#if !readOnly && !session && !showForm}
      <Button size="sm" variant="outline" on:click={() => (showForm = true)} data-testid="ag-session-create-btn">
        + {$_('agSession.createButton')}
      </Button>
    {/if}
  </div>

  {#if loading}
    <p class="text-sm text-gray-500">{$_('common.loading')}</p>
  {:else if showForm}
    <form on:submit|preventDefault={createSession} class="space-y-3" data-testid="ag-session-form">
      <div>
        <label class="block text-xs font-medium text-gray-700 mb-1">{$_('agSession.platform')}</label>
        <select bind:value={form.platform} class="w-full rounded border border-gray-300 px-2 py-1 text-sm" data-testid="ag-session-platform-select">
          {#each platforms as p}
            <option value={p}>{p}</option>
          {/each}
        </select>
      </div>
      <div>
        <label class="block text-xs font-medium text-gray-700 mb-1">{$_('agSession.meetingUrl')} *</label>
        <input bind:value={form.video_url} type="url" required class="w-full rounded border border-gray-300 px-2 py-1 text-sm" placeholder="https://meet.jit.si/..." data-testid="ag-session-video-url-input" />
      </div>
      <div>
        <label class="block text-xs font-medium text-gray-700 mb-1">{$_('agSession.hostUrl')}</label>
        <input bind:value={form.host_url} type="url" class="w-full rounded border border-gray-300 px-2 py-1 text-sm" data-testid="ag-session-host-url-input" />
      </div>
      <div>
        <label class="block text-xs font-medium text-gray-700 mb-1">{$_('agSession.password')}</label>
        <input bind:value={form.access_password} type="text" class="w-full rounded border border-gray-300 px-2 py-1 text-sm" data-testid="ag-session-password-input" />
      </div>
      <div class="flex gap-4">
        <label class="flex items-center gap-1 text-xs">
          <input type="checkbox" bind:checked={form.waiting_room_enabled} data-testid="ag-session-waiting-room-checkbox" />
          {$_('agSession.waitingRoom')}
        </label>
        <label class="flex items-center gap-1 text-xs">
          <input type="checkbox" bind:checked={form.recording_enabled} data-testid="ag-session-recording-checkbox" />
          {$_('agSession.recording')}
        </label>
      </div>
      <div class="flex gap-2">
        <Button type="submit" size="sm" loading={creating} data-testid="ag-session-submit-btn">{$_('common.create')}</Button>
        <Button type="button" size="sm" variant="ghost" on:click={() => (showForm = false)} data-testid="ag-session-cancel-btn">{$_('common.cancel')}</Button>
      </div>
    </form>
  {:else if session}
    <div class="space-y-2">
      <div class="flex items-center gap-2">
        <span class="text-sm font-medium">{session.platform}</span>
        <span class="px-1.5 py-0.5 rounded text-xs font-medium {getStatusBadge(session.status)}">
          {session.status}
        </span>
      </div>

      <div class="flex items-center gap-2 text-sm text-gray-600">
        <a href={session.video_url} target="_blank" rel="noopener" class="text-blue-600 hover:underline truncate max-w-xs" data-testid="ag-session-video-link">
          {session.video_url}
        </a>
      </div>

      {#if session.access_password}
        <p class="text-xs text-gray-500">{$_('agSession.password')}: <span class="font-mono">{session.access_password}</span></p>
      {/if}

      <div class="grid grid-cols-3 gap-2 mt-2 text-xs text-gray-600">
        <div class="bg-gray-50 rounded p-1.5 text-center">
          <p class="font-semibold text-gray-900">{session.remote_attendees_count}</p>
          <p>{$_('agSession.remoteAttendees')}</p>
        </div>
        <div class="bg-gray-50 rounded p-1.5 text-center">
          <p class="font-semibold text-gray-900">{(session.remote_voting_power * 100).toFixed(1)}%</p>
          <p>{$_('agSession.votingPower')}</p>
        </div>
        <div class="bg-gray-50 rounded p-1.5 text-center">
          <p class="font-semibold text-gray-900">{(session.quorum_remote_contribution * 100).toFixed(1)}%</p>
          <p>{$_('agSession.quorumContribution')}</p>
        </div>
      </div>

      {#if !readOnly}
        <div class="flex gap-2 mt-2">
          {#if session.status === 'Scheduled'}
            <Button size="sm" variant="primary" on:click={startSession} data-testid="ag-session-start-btn">▶ {$_('agSession.start')}</Button>
          {:else if session.status === 'Live'}
            <Button size="sm" variant="danger" on:click={endSession} data-testid="ag-session-end-btn">⏹ {$_('agSession.end')}</Button>
          {/if}
        </div>
      {/if}
    </div>
  {:else}
    <p class="text-sm text-gray-500 italic">{$_('agSession.noSession')}</p>
  {/if}
</div>
