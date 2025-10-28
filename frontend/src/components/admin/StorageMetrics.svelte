<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { getMetricsUrl } from '../../lib/api';

  interface OperationMetrics {
    provider: string;
    operation: string;
    success: number;
    error: number;
    avgDurationMs: number | null;
  }

  const REFRESH_INTERVAL_MS = 30000;

  let metrics: OperationMetrics[] = [];
  let loading = true;
  let error: string | null = null;
  let lastUpdated: Date | null = null;
  let intervalId: ReturnType<typeof setInterval> | undefined;

  async function fetchMetrics(): Promise<void> {
    try {
      if (!lastUpdated) {
        loading = true;
      }
      error = null;

      const response = await fetch(getMetricsUrl(), {
        headers: {
          Accept: 'text/plain',
        },
      });

      if (!response.ok) {
        throw new Error(`Metrics endpoint returned ${response.status}`);
      }

      const text = await response.text();
      metrics = parseMetrics(text);
      lastUpdated = new Date();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Metrics fetch failed';
      console.error('Failed to load metrics', err);
    } finally {
      loading = false;
    }
  }

  function parseMetrics(scrape: string): OperationMetrics[] {
    interface Aggregate {
      provider: string;
      operation: string;
      success: number;
      error: number;
      durationSum: number;
      durationCount: number;
    }

    const aggregates = new Map<string, Aggregate>();

    const lines = scrape.split('\n');
    for (const rawLine of lines) {
      const line = rawLine.trim();
      if (!line || line.startsWith('#')) continue;

      let match = line.match(/^storage_operation_total\{([^}]*)\}\s+([0-9.]+)$/);
      if (match) {
        const labels = parseLabels(match[1]);
        const value = Number(match[2]);
        const key = `${labels.provider || 'unknown'}|${labels.operation || 'unknown'}`;
        const entry = aggregates.get(key) ?? {
          provider: labels.provider || 'unknown',
          operation: labels.operation || 'unknown',
          success: 0,
          error: 0,
          durationSum: 0,
          durationCount: 0,
        };

        if (labels.result === 'success') {
          entry.success = value;
        } else if (labels.result === 'error') {
          entry.error = value;
        }

        aggregates.set(key, entry);
        continue;
      }

      match = line.match(/^storage_operation_duration_seconds_sum\{([^}]*)\}\s+([0-9.]+)$/);
      if (match) {
        const labels = parseLabels(match[1]);
        const value = Number(match[2]);
        const key = `${labels.provider || 'unknown'}|${labels.operation || 'unknown'}`;
        const entry = aggregates.get(key) ?? {
          provider: labels.provider || 'unknown',
          operation: labels.operation || 'unknown',
          success: 0,
          error: 0,
          durationSum: 0,
          durationCount: 0,
        };

        entry.durationSum = value;
        aggregates.set(key, entry);
        continue;
      }

      match = line.match(/^storage_operation_duration_seconds_count\{([^}]*)\}\s+([0-9.]+)$/);
      if (match) {
        const labels = parseLabels(match[1]);
        const value = Number(match[2]);
        const key = `${labels.provider || 'unknown'}|${labels.operation || 'unknown'}`;
        const entry = aggregates.get(key) ?? {
          provider: labels.provider || 'unknown',
          operation: labels.operation || 'unknown',
          success: 0,
          error: 0,
          durationSum: 0,
          durationCount: 0,
        };

        entry.durationCount = value;
        aggregates.set(key, entry);
        continue;
      }
    }

    return Array.from(aggregates.values())
      .map((entry) => ({
        provider: entry.provider,
        operation: entry.operation,
        success: entry.success,
        error: entry.error,
        avgDurationMs:
          entry.durationCount > 0
            ? (entry.durationSum / entry.durationCount) * 1000
            : null,
      }))
      .sort((a, b) => {
        if (a.provider === b.provider) {
          return a.operation.localeCompare(b.operation);
        }
        return a.provider.localeCompare(b.provider);
      });
  }

  function parseLabels(input: string): Record<string, string> {
    const labels: Record<string, string> = {};
    for (const part of input.split(',')) {
      const [key, rawValue] = part.split('=');
      if (!key || !rawValue) continue;
      labels[key.trim()] = rawValue.replace(/^"|"$/g, '');
    }
    return labels;
  }

  function formatDuration(ms: number | null): string {
    if (ms === null) return '—';
    if (ms >= 1000) {
      return `${(ms / 1000).toFixed(2)} s`;
    }
    return `${ms.toFixed(2)} ms`;
  }

  function formatTimestamp(date: Date | null): string {
    if (!date) return '—';
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  }

  function getStatusClass(errors: number): string {
    return errors > 0 ? 'text-red-600 font-semibold' : 'text-green-600';
  }

  onMount(() => {
    fetchMetrics();
    intervalId = setInterval(fetchMetrics, REFRESH_INTERVAL_MS);
  });

  onDestroy(() => {
    if (intervalId) {
      clearInterval(intervalId);
    }
  });
</script>

<section class="bg-white border border-gray-200 rounded-xl shadow-sm">
  <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4 p-6 border-b border-gray-100">
    <div>
      <h2 class="text-2xl font-semibold text-gray-900">Storage Monitoring</h2>
      <p class="text-sm text-gray-500">Prometheus metrics for document storage providers</p>
    </div>
    <div class="flex items-center gap-3">
      <span class="text-sm text-gray-500">Dernière mise à jour : {formatTimestamp(lastUpdated)}</span>
      <button
        class="px-4 py-2 rounded-lg bg-primary-600 text-white hover:bg-primary-700 transition text-sm font-medium"
        on:click={fetchMetrics}
        disabled={loading}
      >
        {loading ? 'Actualisation…' : 'Rafraîchir'}
      </button>
    </div>
  </div>

  {#if error}
    <div class="px-6 py-4 bg-red-50 text-red-700 border-b border-red-100">
      {error}
    </div>
  {/if}

  <div class="p-6">
    {#if loading && !lastUpdated}
      <p class="text-gray-500">Chargement des métriques…</p>
    {:else if metrics.length === 0}
      <p class="text-gray-500">Aucune métrique disponible pour le moment.</p>
    {:else}
      <div class="overflow-x-auto">
        <table class="min-w-full text-sm">
          <thead>
            <tr class="text-left text-gray-500 uppercase text-xs tracking-wider">
              <th class="px-4 py-2">Provider</th>
              <th class="px-4 py-2">Operation</th>
              <th class="px-4 py-2">Success</th>
              <th class="px-4 py-2">Errors</th>
              <th class="px-4 py-2">Avg Duration</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-gray-100">
            {#each metrics as row}
              <tr class="hover:bg-gray-50">
                <td class="px-4 py-2 font-medium text-gray-900 capitalize">{row.provider}</td>
                <td class="px-4 py-2 text-gray-700">{row.operation.replace(/_/g, ' ')}</td>
                <td class="px-4 py-2 text-gray-700">{row.success.toFixed(0)}</td>
                <td class={`px-4 py-2 ${getStatusClass(row.error)}`}>{row.error.toFixed(0)}</td>
                <td class="px-4 py-2 text-gray-700">{formatDuration(row.avgDurationMs)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
</section>
