<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from "../../lib/i18n";
  import {
    convocationsApi,
    type Convocation,
    type TrackingSummary,
    ConvocationStatus,
    MeetingType,
  } from "../../lib/api/convocations";
  import { authStore } from "../../stores/auth";
  import { UserRole } from "../../lib/types";
  import { formatDateTime, formatDateShort } from "../../lib/utils/date.utils";
  import { withErrorHandling } from "../../lib/utils/error.utils";
  import ConvocationTrackingSummary from "./ConvocationTrackingSummary.svelte";
  import ConvocationRecipientList from "./ConvocationRecipientList.svelte";
  import ConfirmDialog from "../ui/ConfirmDialog.svelte";
  import Modal from "../ui/Modal.svelte";
  import Button from "../ui/Button.svelte";

  let {
    convocation,
  }: {
    convocation: Convocation;
  } = $props();

  let tracking = $state<TrackingSummary | null>(null);
  let showRecipients = $state(false);
  let actionLoading = $state(false);

  /// L'action en attente de confirmation, ou `null`.
  ///
  /// ── Pourquoi ce détour ────────────────────────────────────────────────
  ///
  /// Cet écran passait par quatre `confirm()` et un `prompt()` natifs. Ce
  /// sont des dialogues du NAVIGATEUR : un navigateur piloté les supprime, et
  /// l'action prend alors la forme exacte d'une panne — aucun dialogue,
  /// aucune requête, aucun message.
  ///
  /// C'est ce qui a fait déclarer mort le bouton « Reporter » d'une assemblée
  /// pendant deux recettes, alors que sa source était correcte (#780). Cet
  /// écran-ci est celui du deuxième verrou de la même issue : il ne peut pas
  /// se permettre d'être intestable.
  ///
  /// Ils ne se traduisent pas non plus — « OK » et « Annuler » viennent de la
  /// locale du navigateur — et ils n'offrent aucun piège de focus. Cf. #844.
  type ActionEnAttente = "envoyer" | "annuler" | "rappels" | "supprimer";
  let actionEnAttente = $state<ActionEnAttente | null>(null);

  /// La date de mise en attente d'envoi, saisie dans une modale plutôt que
  /// dans un `prompt()`.
  let modaleProgrammation = $state(false);
  let dateDenvoi = $state("");

  const TITRES: Record<ActionEnAttente, string> = $derived({
    envoyer: $_("convocations.confirms.sendToAll"),
    annuler: $_("convocations.confirms.cancelConvocation"),
    rappels: $_("convocations.confirms.sendReminders"),
    supprimer: $_("convocations.confirms.deleteConvocation"),
  });

  let isAdmin = $derived(
    $authStore.user?.role === UserRole.SYNDIC ||
      $authStore.user?.role === UserRole.SUPERADMIN,
  );

  $effect(() => {
    if (convocation.status === ConvocationStatus.Sent) {
      (async () => {
        try {
          tracking = await convocationsApi.getTrackingSummary(convocation.id);
        } catch {
          // Non-critical
        }
      })();
    }
  });

  function getMeetingTypeLabel(type: MeetingType): string {
    switch (type) {
      case MeetingType.Ordinary:
        return $_("convocations.meetingType.ordinary");
      case MeetingType.Extraordinary:
        return $_("convocations.meetingType.extraordinary");
      case MeetingType.SecondConvocation:
        return $_("convocations.meetingType.secondConvocation");
      default:
        return type;
    }
  }

  function getStatusConfig(status: ConvocationStatus): {
    bg: string;
    text: string;
    label: string;
  } {
    switch (status) {
      case ConvocationStatus.Draft:
        return {
          bg: "bg-gray-100",
          text: "text-gray-700",
          label: $_("convocations.status.draft"),
        };
      case ConvocationStatus.Scheduled:
        return {
          bg: "bg-blue-100",
          text: "text-blue-700",
          label: $_("convocations.status.scheduled"),
        };
      case ConvocationStatus.Sent:
        return {
          bg: "bg-green-100",
          text: "text-green-700",
          label: $_("convocations.status.sent"),
        };
      case ConvocationStatus.Cancelled:
        return {
          bg: "bg-red-100",
          text: "text-red-700",
          label: $_("convocations.status.cancelled"),
        };
      default:
        return { bg: "bg-gray-100", text: "text-gray-700", label: status };
    }
  }

  function getLegalDeadlineDays(type: MeetingType): number {
    switch (type) {
      case MeetingType.Ordinary:
        return 15;
      case MeetingType.Extraordinary:
        return 8;
      case MeetingType.SecondConvocation:
        return 8;
      default:
        return 15;
    }
  }

  async function handleSchedule() {
    modaleProgrammation = true;
  }

  async function confirmerLaProgrammation() {
    const sendDate = dateDenvoi;
    modaleProgrammation = false;
    dateDenvoi = "";
    if (!sendDate) return;
    const result = await withErrorHandling({
      action: () => convocationsApi.schedule(convocation.id, sendDate),
      setLoading: (v: boolean) => (actionLoading = v),
      successMessage: $_("convocations.messages.scheduled"),
      errorMessage: $_("convocations.errors.schedulingFailed"),
    });
    if (result) convocation = result;
  }

  function handleSend() {
    actionEnAttente = "envoyer";
  }

  async function executer_envoyer() {
    actionEnAttente = null;
    const result = await withErrorHandling({
      action: () => convocationsApi.send(convocation.id),
      setLoading: (v: boolean) => (actionLoading = v),
      successMessage: $_("convocations.messages.sent"),
      errorMessage: $_("convocations.errors.sendingFailed"),
    });
    if (result) convocation = result;
  }

  function handleCancel() {
    actionEnAttente = "annuler";
  }

  async function executer_annuler() {
    actionEnAttente = null;
    const result = await withErrorHandling({
      action: () => convocationsApi.cancel(convocation.id),
      setLoading: (v: boolean) => (actionLoading = v),
      successMessage: $_("convocations.messages.cancelled"),
      errorMessage: $_("convocations.errors.cancellationFailed"),
    });
    if (result) convocation = result;
  }

  function handleSendReminders() {
    actionEnAttente = "rappels";
  }

  async function executer_rappels() {
    actionEnAttente = null;
    await withErrorHandling({
      action: () => convocationsApi.sendReminders(convocation.id),
      setLoading: (v: boolean) => (actionLoading = v),
      successMessage: $_("convocations.messages.remindersEntered"),
      errorMessage: $_("convocations.errors.remindersSendingFailed"),
    });
  }

  function handleDelete() {
    actionEnAttente = "supprimer";
  }

  async function executer_supprimer() {
    actionEnAttente = null;
    await withErrorHandling({
      action: () => convocationsApi.delete(convocation.id),
      setLoading: (v: boolean) => (actionLoading = v),
      successMessage: $_("convocations.messages.deleted"),
      errorMessage: $_("convocations.errors.deletionFailed"),
      onSuccess: () => {
        window.location.href = "/convocations";
      },
    });
  }
</script>

<div class="space-y-6" data-testid="convocation-detail">
  <div class="bg-white shadow-md rounded-lg p-6">
    <div class="flex items-start justify-between">
      <div>
        <div class="flex items-center gap-3 mb-2">
          <h2 class="text-2xl font-bold text-gray-900">
            {$_("convocations.title")}
          </h2>
          <span
            class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {getStatusConfig(
              convocation.status,
            ).bg} {getStatusConfig(convocation.status).text}"
            data-testid="convocation-detail-status"
          >
            {getStatusConfig(convocation.status).label}
          </span>
          <span
            class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-indigo-100 text-indigo-700"
          >
            {getMeetingTypeLabel(convocation.meeting_type)}
          </span>
        </div>
        <p class="text-sm text-gray-500">
          {$_("common.language")}: {convocation.language.toUpperCase()} - {$_(
            "common.createdOn",
          )}
          {formatDateTime(convocation.created_at)}
        </p>
      </div>

      <div class="text-right" data-testid="convocation-detail-legal-deadline">
        {#if convocation.respects_legal_deadline}
          <span
            class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-green-100 text-green-700"
          >
            {$_("convocations.legalDeadlineRespected", {
              values: { days: getLegalDeadlineDays(convocation.meeting_type) },
            })}
          </span>
        {:else}
          <span
            class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-red-100 text-red-700"
          >
            {$_("convocations.legalDeadlineNotRespected")}
          </span>
        {/if}
      </div>
    </div>

    <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mt-6">
      <div class="p-3 bg-blue-50 rounded-lg">
        <div class="text-xs text-blue-600 font-medium">
          {$_("convocations.meetingDate")}
        </div>
        <div class="text-sm text-blue-900 font-medium">
          {formatDateShort(convocation.meeting_date)}
        </div>
      </div>
      <div class="p-3 bg-amber-50 rounded-lg">
        <div class="text-xs text-amber-600 font-medium">
          {$_("convocations.minimumSendDate")}
        </div>
        <div class="text-sm text-amber-900 font-medium">
          {formatDateShort(convocation.minimum_send_date)}
        </div>
      </div>
      <div
        class="p-3 bg-green-50 rounded-lg"
        data-testid="convocation-detail-recipients-summary"
      >
        <div class="text-xs text-green-600 font-medium">
          {$_("convocations.recipients")}
        </div>
        <div class="text-sm text-green-900 font-bold">
          {convocation.total_recipients}
        </div>
      </div>
      <div class="p-3 bg-purple-50 rounded-lg">
        <div class="text-xs text-purple-600 font-medium">
          {$_("convocations.openings")}
        </div>
        <div class="text-sm text-purple-900 font-bold">
          {convocation.opened_count}/{convocation.total_recipients}
        </div>
      </div>
    </div>
  </div>

  {#if convocation.status === ConvocationStatus.Sent && tracking}
    <div class="bg-white shadow-md rounded-lg p-6">
      <h3 class="text-lg font-medium text-gray-900 mb-4">
        {$_("convocations.trackingTitle")}
      </h3>
      <ConvocationTrackingSummary convocationId={convocation.id} />
    </div>
  {/if}

  {#if convocation.status === ConvocationStatus.Sent}
    <div class="bg-white shadow-md rounded-lg">
      <div
        class="px-6 py-4 border-b border-gray-200 flex items-center justify-between"
      >
        <h3 class="text-lg font-medium text-gray-900">
          {$_("convocations.recipients")}
        </h3>
        <button
          onclick={() => (showRecipients = !showRecipients)}
          data-testid="convocation-detail-btn-toggle-recipients"
          class="text-sm text-blue-600 hover:text-blue-800"
        >
          {showRecipients ? $_("common.hide") : $_("common.show")}
          {$_("common.list")}
        </button>
      </div>
      {#if showRecipients}
        <div class="p-6">
          <ConvocationRecipientList convocationId={convocation.id} />
        </div>
      {/if}
    </div>
  {/if}

  {#if isAdmin}
    <div class="bg-white shadow-md rounded-lg p-6">
      <h3 class="text-lg font-medium text-gray-900 mb-4">
        {$_("common.actions")}
      </h3>
      <div class="flex flex-wrap gap-3">
        {#if convocation.status === ConvocationStatus.Draft}
          <button
            onclick={handleSchedule}
            disabled={actionLoading}
            data-testid="convocation-detail-btn-schedule"
            class="px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 disabled:opacity-50"
          >
            {$_("convocations.actions.scheduleSend")}
          </button>
          <button
            onclick={handleSend}
            disabled={actionLoading}
            data-testid="convocation-detail-btn-send"
            class="px-4 py-2 bg-green-600 text-white text-sm font-medium rounded-md hover:bg-green-700 disabled:opacity-50"
          >
            {$_("convocations.actions.sendNow")}
          </button>
          <button
            onclick={handleDelete}
            disabled={actionLoading}
            data-testid="convocation-detail-btn-delete"
            class="px-4 py-2 bg-red-100 text-red-700 text-sm font-medium rounded-md hover:bg-red-200 disabled:opacity-50"
          >
            {$_("common.delete")}
          </button>
        {/if}

        {#if convocation.status === ConvocationStatus.Scheduled}
          <button
            onclick={handleSend}
            disabled={actionLoading}
            data-testid="convocation-detail-btn-send"
            class="px-4 py-2 bg-green-600 text-white text-sm font-medium rounded-md hover:bg-green-700 disabled:opacity-50"
          >
            {$_("convocations.actions.sendNow")}
          </button>
          <button
            onclick={handleCancel}
            disabled={actionLoading}
            data-testid="convocation-detail-btn-cancel"
            class="px-4 py-2 bg-red-100 text-red-700 text-sm font-medium rounded-md hover:bg-red-200 disabled:opacity-50"
          >
            {$_("common.cancel")}
          </button>
        {/if}

        {#if convocation.status === ConvocationStatus.Sent}
          <button
            onclick={handleSendReminders}
            disabled={actionLoading}
            data-testid="convocation-detail-btn-send-reminders"
            class="px-4 py-2 bg-amber-600 text-white text-sm font-medium rounded-md hover:bg-amber-700 disabled:opacity-50"
          >
            {$_("convocations.actions.sendReminders")}
          </button>
        {/if}
      </div>
    </div>
  {/if}
</div>

<!-- Les dialogues qui remplacent quatre `confirm()` et un `prompt()`.
     Un navigateur piloté supprime les dialogues natifs : l'action prend alors
     la forme exacte d'une panne. Ceux-ci sont dans la page, donc cliquables,
     traduits, et dotés d'un piège de focus. Cf. #844, #780. -->
<ConfirmDialog
  isOpen={actionEnAttente !== null}
  title={$_("common.confirm")}
  message={actionEnAttente ? TITRES[actionEnAttente] : ""}
  variant={actionEnAttente === "supprimer" || actionEnAttente === "annuler"
    ? "danger"
    : "primary"}
  loading={actionLoading}
  onconfirm={() => {
    if (actionEnAttente === "envoyer") executer_envoyer();
    else if (actionEnAttente === "annuler") executer_annuler();
    else if (actionEnAttente === "rappels") executer_rappels();
    else if (actionEnAttente === "supprimer") executer_supprimer();
  }}
  oncancel={() => (actionEnAttente = null)}
/>

<Modal
  isOpen={modaleProgrammation}
  title={$_("convocations.prompts.scheduledSendDate")}
  size="sm"
  onclose={() => {
    modaleProgrammation = false;
    dateDenvoi = "";
  }}
>
  <label
    class="block text-sm font-medium text-gray-700"
    for="convocation-schedule-date"
  >
    {$_("convocations.prompts.scheduledSendDate")}
  </label>
  <input
    id="convocation-schedule-date"
    type="datetime-local"
    bind:value={dateDenvoi}
    data-testid="convocation-schedule-date-input"
    class="mt-1 block w-full rounded-md border-gray-300 shadow-sm sm:text-sm"
  />

  {#snippet footer()}
    <div class="flex justify-end space-x-3">
      <Button
        variant="outline"
        onclick={() => {
          modaleProgrammation = false;
          dateDenvoi = "";
        }}
        data-testid="convocation-schedule-cancel"
      >
        {$_("common.cancel")}
      </Button>
      <Button
        variant="primary"
        onclick={confirmerLaProgrammation}
        disabled={dateDenvoi.length === 0 || actionLoading}
        data-testid="convocation-schedule-submit"
      >
        {$_("common.confirm")}
      </Button>
    </div>
  {/snippet}
</Modal>
