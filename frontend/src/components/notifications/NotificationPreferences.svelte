<script lang="ts">
  import { onMount } from "svelte";
  import {
    notificationsApi,
    NotificationType,
    type NotificationPreference,
  } from "../../lib/api/notifications";
  import { addToast } from "../../stores/toast";

  export let userId: string;

  let preferences: NotificationPreference[] = [];
  let loading = true;
  let saving = false;

  const notificationTypeLabels: Record<NotificationType, string> = {
    [NotificationType.MeetingReminder]: "Meeting Reminders",
    [NotificationType.PaymentDue]: "Payment Due Notices",
    [NotificationType.DocumentShared]: "Document Sharing",
    [NotificationType.TicketUpdate]: "Ticket Updates",
    [NotificationType.TicketAssigned]: "Ticket Assignments",
    [NotificationType.TicketResolved]: "Ticket Resolutions",
    [NotificationType.SystemAlert]: "System Alerts",
    [NotificationType.AccountUpdate]: "Account Updates",
    [NotificationType.NewMessage]: "New Messages",
    [NotificationType.ConvocationSent]: "AG Convocations",
    [NotificationType.ResolutionVoting]: "Resolution Voting",
    [NotificationType.QuoteReceived]: "Quote Received",
    [NotificationType.QuoteAccepted]: "Quote Accepted",
    [NotificationType.PaymentSuccess]: "Payment Success",
    [NotificationType.PaymentFailed]: "Payment Failed",
    [NotificationType.BudgetApproved]: "Budget Approved",
    [NotificationType.EtatDateReady]: "État Daté Ready",
    [NotificationType.ExchangeRequested]: "SEL Exchange Requested",
    [NotificationType.ExchangeCompleted]: "SEL Exchange Completed",
    [NotificationType.AchievementEarned]: "Achievement Earned",
    [NotificationType.ChallengeStarted]: "Challenge Started",
    [NotificationType.ChallengeCompleted]: "Challenge Completed",
  };

  onMount(async () => {
    await loadPreferences();
  });

  async function loadPreferences() {
    try {
      loading = true;
      preferences = await notificationsApi.getPreferences(userId);
    } catch (err: any) {
      addToast({
        message: err.message || "Failed to load preferences",
        type: "error",
      });
    } finally {
      loading = false;
    }
  }

  async function handleToggle(
    preference: NotificationPreference,
    field: "enabled" | "email_enabled" | "sms_enabled" | "push_enabled",
  ) {
    try {
      saving = true;
      const updated = await notificationsApi.updatePreference(
        userId,
        preference.notification_type,
        { [field]: !(preference as any)[field] },
      );

      // Update local state
      preferences = preferences.map((p) =>
        p.id === preference.id ? updated : p,
      );

      addToast({
        message: "Preference updated successfully",
        type: "success",
      });
    } catch (err: any) {
      addToast({
        message: err.message || "Failed to update preference",
        type: "error",
      });
    } finally {
      saving = false;
    }
  }
</script>

<div class="bg-white shadow rounded-lg">
  <div class="px-6 py-4 border-b border-gray-200">
    <h2 class="text-xl font-semibold text-gray-900">
      Notification Preferences
    </h2>
    <p class="mt-1 text-sm text-gray-600">
      Choose how you want to receive notifications for each type of event.
    </p>
  </div>

  <div class="px-6 py-4">
    {#if loading}
      <div class="text-center py-12 text-gray-500">
        <div
          class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"
        ></div>
        <p class="mt-4">Loading preferences...</p>
      </div>
    {:else if preferences.length === 0}
      <div class="text-center py-12 text-gray-500">
        No notification preferences found.
      </div>
    {:else}
      <div class="space-y-4">
        <!-- Table Header -->
        <div class="grid grid-cols-6 gap-4 pb-2 border-b border-gray-200">
          <div class="col-span-2 text-sm font-medium text-gray-700">
            Notification Type
          </div>
          <div class="text-sm font-medium text-gray-700 text-center">
            Enabled
          </div>
          <div class="text-sm font-medium text-gray-700 text-center">
            Email
          </div>
          <div class="text-sm font-medium text-gray-700 text-center">SMS</div>
          <div class="text-sm font-medium text-gray-700 text-center">Push</div>
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
            Toggle "Enabled" to turn off all notifications for that type.
            Individual channel toggles (Email, SMS, Push) only work when the
            main toggle is enabled.
          </p>
        </div>
      </div>
    {/if}
  </div>
</div>
