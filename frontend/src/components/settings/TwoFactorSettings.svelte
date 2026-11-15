<script lang="ts">
  // Story #1145 — activer, désactiver et entretenir son second facteur.
  // Le secret et les codes de secours ne vivent que dans ce composant :
  // jamais dans le stockage du navigateur, jamais dans la console.
  import { _ } from "../../lib/i18n";
  import {
    twoFactorApi,
    type TwoFactorSetup,
    type TwoFactorStatus,
  } from "../../lib/api/two_factor";

  let status = $state<TwoFactorStatus | null>(null);
  let setupData = $state<TwoFactorSetup | null>(null);
  let freshCodes = $state<string[]>([]);
  let code = $state("");
  let password = $state("");
  let regenerateCode = $state("");
  let error = $state("");
  let notice = $state("");
  let loading = $state(true);
  let busy = $state(false);

  const loadStatus = async () => {
    loading = true;
    try {
      status = await twoFactorApi.getStatus();
    } catch {
      error = $_("secondFactor.genericError");
    } finally {
      loading = false;
    }
  };

  $effect(() => {
    loadStatus();
  });

  const reset = () => {
    error = "";
    notice = "";
  };

  const startSetup = async () => {
    reset();
    busy = true;
    try {
      setupData = await twoFactorApi.setup();
      code = "";
    } catch {
      error = $_("secondFactor.genericError");
    } finally {
      busy = false;
    }
  };

  const confirmSetup = async (e: Event) => {
    e.preventDefault();
    reset();
    busy = true;
    try {
      await twoFactorApi.enable(code.trim());
      setupData = null;
      code = "";
      notice = $_("secondFactor.enabledMsg");
      await loadStatus();
    } catch {
      error = $_("secondFactor.wrongCode");
      code = "";
    } finally {
      busy = false;
    }
  };

  // Abandon : rien n'est activé. Le secret en attente est remplacé au
  // prochain démarrage (`setup_2fa` efface un secret non activé).
  const cancelSetup = () => {
    setupData = null;
    code = "";
    reset();
  };

  const disable = async (e: Event) => {
    e.preventDefault();
    reset();
    busy = true;
    try {
      await twoFactorApi.disable(password);
      password = "";
      freshCodes = [];
      notice = $_("secondFactor.disabledMsg");
      await loadStatus();
    } catch {
      error = $_("secondFactor.wrongPassword");
      password = "";
    } finally {
      busy = false;
    }
  };

  const regenerate = async (e: Event) => {
    e.preventDefault();
    reset();
    busy = true;
    try {
      const response = await twoFactorApi.regenerateBackupCodes(
        regenerateCode.trim(),
      );
      freshCodes = response.backup_codes;
      regenerateCode = "";
      notice = $_("secondFactor.regeneratedMsg");
      await loadStatus();
    } catch {
      error = $_("secondFactor.wrongCode");
      regenerateCode = "";
    } finally {
      busy = false;
    }
  };
</script>

<div
  class="bg-white rounded-lg shadow-sm border border-gray-200 p-6 space-y-5"
  data-testid="second-factor-settings"
>
  <div class="flex items-start justify-between gap-4">
    <div>
      <h2 class="text-lg font-semibold text-gray-900">
        {$_("auth.twoFactor.title")}
      </h2>
      <p class="text-sm text-gray-600 mt-1">{$_("secondFactor.intro")}</p>
    </div>
    {#if status}
      <span
        class="shrink-0 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {status.is_enabled
          ? 'bg-green-100 text-green-800'
          : 'bg-gray-100 text-gray-700'}"
        data-testid="second-factor-status"
      >
        {status.is_enabled
          ? $_("secondFactor.statusOn")
          : $_("secondFactor.statusOff")}
      </span>
    {/if}
  </div>

  {#if error}
    <div
      class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg text-sm"
      role="alert"
      data-testid="second-factor-error"
    >
      {error}
    </div>
  {/if}
  {#if notice}
    <div
      class="bg-green-50 border border-green-200 text-green-800 px-4 py-3 rounded-lg text-sm"
      role="status"
      data-testid="second-factor-notice"
    >
      {notice}
    </div>
  {/if}

  {#if loading}
    <p class="text-sm text-gray-500">{$_("secondFactor.loading")}</p>
  {:else if setupData}
    <!-- Activation : QR, clé manuelle, codes de secours, puis code. -->
    <div class="space-y-4" data-testid="second-factor-setup">
      <p class="text-sm text-gray-700">{$_("secondFactor.scanQr")}</p>
      <img
        src={setupData.qr_code_data_url}
        alt={$_("secondFactor.qrAlt")}
        class="w-48 h-48 border border-gray-200 rounded"
        data-testid="second-factor-qr"
      />
      <div>
        <p class="text-sm text-gray-700">{$_("secondFactor.manualKey")}</p>
        <code
          class="block mt-1 px-3 py-2 bg-gray-50 border border-gray-200 rounded text-sm break-all"
          data-testid="second-factor-secret">{setupData.secret}</code
        >
      </div>
      <div>
        <h3 class="text-sm font-semibold text-gray-900">
          {$_("auth.twoFactor.backupCodes")}
        </h3>
        <p class="text-sm text-amber-700 mt-1">
          {$_("secondFactor.backupCodesWarning")}
        </p>
        <ul
          class="mt-2 grid grid-cols-2 gap-2 font-mono text-sm"
          data-testid="second-factor-backup-codes"
        >
          {#each setupData.backup_codes as backupCode}
            <li class="px-2 py-1 bg-gray-50 border border-gray-200 rounded">
              {backupCode}
            </li>
          {/each}
        </ul>
      </div>
      <form
        onsubmit={confirmSetup}
        class="space-y-3"
        data-testid="second-factor-enable-form"
      >
        <label
          for="second-factor-enable-code"
          class="block text-sm font-medium text-gray-700"
        >
          {$_("auth.twoFactor.code")}
        </label>
        <input
          id="second-factor-enable-code"
          type="text"
          bind:value={code}
          required
          inputmode="numeric"
          autocomplete="one-time-code"
          class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 tracking-widest"
          data-testid="second-factor-enable-code"
        />
        <div class="flex flex-wrap gap-3">
          <button
            type="submit"
            disabled={busy}
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
            data-testid="second-factor-enable-submit"
          >
            {$_("auth.twoFactor.enable")}
          </button>
          <button
            type="button"
            onclick={cancelSetup}
            class="px-4 py-2 text-gray-700 border border-gray-300 rounded-lg hover:bg-gray-50"
            data-testid="second-factor-setup-cancel"
          >
            {$_("secondFactor.cancel")}
          </button>
        </div>
      </form>
    </div>
  {:else if status && !status.is_enabled}
    <button
      type="button"
      onclick={startSetup}
      disabled={busy}
      class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
      data-testid="second-factor-start"
    >
      {$_("auth.twoFactor.setup")}
    </button>
  {:else if status}
    <div class="space-y-5">
      <p class="text-sm text-gray-700" data-testid="second-factor-remaining">
        {$_("secondFactor.remaining", {
          values: { count: status.backup_codes_remaining },
        })}
      </p>
      {#if status.backup_codes_low}
        <p
          class="text-sm text-amber-700"
          role="status"
          data-testid="second-factor-low-codes"
        >
          {$_("secondFactor.lowCodes")}
        </p>
      {/if}

      {#if freshCodes.length > 0}
        <div>
          <p class="text-sm text-amber-700">
            {$_("secondFactor.backupCodesWarning")}
          </p>
          <ul
            class="mt-2 grid grid-cols-2 gap-2 font-mono text-sm"
            data-testid="second-factor-fresh-codes"
          >
            {#each freshCodes as backupCode}
              <li class="px-2 py-1 bg-gray-50 border border-gray-200 rounded">
                {backupCode}
              </li>
            {/each}
          </ul>
        </div>
      {/if}

      <form
        onsubmit={regenerate}
        class="space-y-2"
        data-testid="second-factor-regenerate-form"
      >
        <label
          for="second-factor-regenerate-code"
          class="block text-sm font-medium text-gray-700"
        >
          {$_("secondFactor.regenerateHelp")}
        </label>
        <div class="flex flex-wrap gap-3">
          <input
            id="second-factor-regenerate-code"
            type="text"
            bind:value={regenerateCode}
            required
            inputmode="numeric"
            autocomplete="one-time-code"
            class="flex-1 min-w-0 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            data-testid="second-factor-regenerate-code"
          />
          <button
            type="submit"
            disabled={busy}
            class="px-4 py-2 text-primary-700 border border-primary-300 rounded-lg hover:bg-primary-50 disabled:opacity-50"
            data-testid="second-factor-regenerate-submit"
          >
            {$_("secondFactor.regenerate")}
          </button>
        </div>
      </form>

      <form
        onsubmit={disable}
        class="space-y-2 pt-4 border-t border-gray-200"
        data-testid="second-factor-disable-form"
      >
        <label
          for="second-factor-disable-password"
          class="block text-sm font-medium text-gray-700"
        >
          {$_("secondFactor.passwordLabel")}
        </label>
        <div class="flex flex-wrap gap-3">
          <input
            id="second-factor-disable-password"
            type="password"
            bind:value={password}
            required
            autocomplete="current-password"
            class="flex-1 min-w-0 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            data-testid="second-factor-disable-password"
          />
          <button
            type="submit"
            disabled={busy}
            class="px-4 py-2 text-red-700 border border-red-300 rounded-lg hover:bg-red-50 disabled:opacity-50"
            data-testid="second-factor-disable-submit"
          >
            {$_("auth.twoFactor.disable")}
          </button>
        </div>
      </form>
    </div>
  {/if}
</div>
