<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from '../../lib/i18n';
  import {
    notificationsApi,
    NotificationType,
    type NotificationPreference,
  } from "../../lib/api/notifications";
  import { toast } from "../../stores/toast";
  import { withErrorHandling } from "../../lib/utils/error.utils";

  export let userId: string;

  let preferences: NotificationPreference[] = [];
  let loading = true;
  let saving = false;

  const notificationTypeLabels: Record<NotificationType, string> = {
    [NotificationType.MeetingReminder]: $_("notifications.type_meeting_reminders"),
    [NotificationType.PaymentDue]: $_("notifications.type_payment_due"),
    [NotificationType.DocumentShared]: $_("notifications.type_document_sharing"),
    [NotificationType.TicketUpdate]: $_("notifications.type_ticket_updates"),
    [NotificationType.TicketAssigned]: $_("notifications.type_ticket_assigned"),
    [NotificationType.TicketResolved]: $_("notifications.type_ticket_resolved"),
    [NotificationType.SystemAlert]: $_("notifications.type_system_alerts"),
    [NotificationType.AccountUpdate]: $_("notifications.type_account_updates"),
    [NotificationType.NewMessage]: $_("notifications.type_new_messages"),
    [NotificationType.ConvocationSent]: $_("notifications.type_convocation_sent"),
    [NotificationType.ResolutionVoting]: $_("notifications.type_resolution_voting"),
    [NotificationType.QuoteReceived]: $_("notifications.type_quote_received"),
    [NotificationType.QuoteAccepted]: $_("notifications.type_quote_accepted"),
    [NotificationType.PaymentSuccess]: $_("notifications.type_payment_success"),
    [NotificationType.PaymentFailed]: $_("notifications.type_payment_failed"),
    [NotificationType.BudgetApproved]: $_("notifications.type_budget_approved"),
    [NotificationType.EtatDateReady]: $_("notifications.type_etat_date_ready"),
    [NotificationType.ExchangeRequested]: $_("notifications.type_exchange_requested"),
    [NotificationType.ExchangeCompleted]: $_("notifications.type_exchange_completed"),
    [NotificationType.AchievementEarned]: $_("notifications.type_achievement_earned"),
    [NotificationType.ChallengeStarted]: $_("notifications.type_challenge_started"),
    [NotificationType.ChallengeCompleted]: $_("notifications.type_challenge_completed"),
  };

  onMount(async () => {
    await loadPreferences();
  });

  async function loadPreferences() {
    loading = true;
    const result = await withErrorHandling({
      action: () => notificationsApi.getPreferences(userId),
      errorMessage: $_("notifications.load_preferences_failed"),
    });
    if (result) preferences = result;
    loading = false;
  }

  async function handleToggle(
    preference: NotificationPreference,
    field: "enabled" | "email_enabled" | "sms_enabled" | "push_enabled",
  ) {
    const updated = await withErrorHandling({
      action: () => notificationsApi.updatePreference(
        userId,
        preference.notification_type,
        { [field]: !(preference as any)[field] },
      ),
      setLoading: (v) => saving = v,
      successMessage: $_("notifications.preference_updated"),
      errorMessage: $_("notifications.update_preference_failed"),
    });
    if (updated) {
      preferences = preferences.map((p) =>
        p.id === preference.id ? updated : p,
      );
    }
  }
</script>

<div class="bg-white shadow rounded-lg" data-testid="notification-preferences">
  <div class="px-6 py-4 border-b border-gray-200">
    <h2 class="text-xl font-semibold text-gray-900">
      {$_("notifications.preferences")}
    </h2>
    <p class="mt-1 text-sm text-gray-600">
      {$_("notifications.preferences_description")}
    </p>
  </div>

  <div class="px-6 py-4">
    {#if loading}
      <div class="text-center py-12 text-gray-500">
        <div
          class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"
        ></div>
        <p class="mt-4">{$_("notifications.loading_preferences")}</p>
      </div>
    {:else if preferences.length === 0}
      <div class="text-center py-12 text-gray-500">
        {$_("notifications.no_preferences_found")}
      </div>
    {:else}
      <div class="space-y-4">
        <!-- Table Header -->
        <div class="grid grid-cols-6 gap-4 pb-2 border-b border-gray-200">
          <div class="col-span-2 text-sm font-medium text-gray-700">
            {$_("notifications.notification_type")}
          </div>
          <div class="text-sm font-medium text-gray-700 text-center">
            {$_("notifications.enabled")}
          </div>
          <div class="text-sm font-medium text-gray-700 text-center">
            {$_("notifications.email")}
          </div>
          <div class="text-sm font-medium text-gray-700 text-center">{$_("notifications.sms")}</div>
          <div class="text-sm font-medium text-gray-700 text-center">{$_("notifications.push")}</div>
        </div>

        <!-- Preference Rows -->
        {#each preferences as preference (preference.id)}
          <div
            class="grid grid-cols-6 gap-4 items-center py-2 border-b border-gray-100"
          >
            <!-- Notification Type Label -->
            <div class="col-span-2 text-sm text-gray-900">
              {notificationTypeLabels[preference.notification_type] ||
                preference.notification_type}
            </div>

            <!-- Master Toggle -->
            <div class="text-center">
              <label class="inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={preference.enabled}
                  on:change={() => handleToggle(preference, "enabled")}
                  disabled={saving}
                  class="sr-only peer"
                />
                <div
                  class="relative w-11 h-6 bg-gray-200 rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-0.5 after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"
                ></div>
              </label>
            </div>

            <!-- Email Toggle -->
            <div class="text-center">
              <label class="inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={preference.email_enabled}
                  on:change={() => handleToggle(preference, "email_enabled")}
                  disabled={!preference.enabled || saving}
                  class="sr-only peer"
                />
                <div
                  class="relative w-11 h-6 bg-gray-200 rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-0.5 after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-green-600 peer-disabled:opacity-50"
                ></div>
              </label>
            </div>

            <!-- SMS Toggle -->
            <div class="text-center">
              <label class="inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={preference.sms_enabled}
                  on:change={() => handleToggle(preference, "sms_enabled")}
                  disabled={!preference.enabled || saving}
                  class="sr-only peer"
                />
                <div
                  class="relative w-11 h-6 bg-gray-200 rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-0.5 after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-green-600 peer-disabled:opacity-50"
                ></div>
              </label>
            </div>

            <!-- Push Toggle -->
            <div class="text-center">
              <label class="inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={preference.push_enabled}
                  on:change={() => handleToggle(preference, "push_enabled")}
                  disabled={!preference.enabled || saving}
                  class="sr-only peer"
                />
                <div
                  class="relative w-11 h-6 bg-gray-200 rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-0.5 after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-green-600 peer-disabled:opacity-50"
                ></div>
              </label>
            </div>
          </div>
        {/each}
      </div>

      <!-- Legend -->
      <div class="mt-6 pt-6 border-t border-gray-200">
        <div class="flex items-start space-x-2 text-sm text-gray-600">
          <svg
            class="h-5 w-5 text-blue-500 flex-shrink-0"
            fill="currentColor"
            viewBox="0 0 20 20"
          >
            <path
              fill-rule="evenodd"
              d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
              clip-rule="evenodd"
            />
          </svg>
          <p>
            {$_("notifications.legend_text")}
          </p>
        </div>
      </div>
    {/if}
  </div>
</div>
