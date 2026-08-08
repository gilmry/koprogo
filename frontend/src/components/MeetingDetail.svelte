<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "../lib/i18n";
  import { api } from "../lib/api";
  import type { Meeting, Building } from "../lib/types";
  import { authStore } from "../stores/auth";
  import Button from "./ui/Button.svelte";
  import MeetingDocuments from "./MeetingDocuments.svelte";
  import ResolutionList from "./resolutions/ResolutionList.svelte";
  import ConvocationPanel from "./convocations/ConvocationPanel.svelte";
  import QuorumPanel from "./meetings/QuorumPanel.svelte";
  import { toast } from "../stores/toast";
  import { formatDateTime } from "../lib/utils/date.utils";
  import {
    withErrorHandling,
    withLoadingState,
  } from "../lib/utils/error.utils";
  // Track H Story H3 — Checklist Art. 3.87 §3-5 CC + toast 422 narratif.
  import MissingInvariantsList from "../lib/components/shared/MissingInvariantsList.svelte";
  import { showMeetingCompletionToast } from "../lib/utils/meetingCompletion";
  import type { MeetingCompletionChecklistResponse } from "../lib/types/meeting";

  let meeting: Meeting | null = null;
  let building: Building | null = null;
  let loading = true;
  let error = "";
  let meetingId: string = "";
  // Track H Story H3 — État courant des invariants Art. 3.87 §3-5 CC.
  let completionChecklist: MeetingCompletionChecklistResponse | null = null;
  $: canCompleteMeeting =
    !!completionChecklist && completionChecklist.missing.length === 0;
  $: missingInvariantsCount = completionChecklist?.missing.length ?? 0;

  // STORY-P7-301: canManage gating — syndic/superadmin only can act.
  $: userRole =
    $authStore.user?.activeRole?.role || $authStore.user?.role || "";
  $: canManage = userRole === "syndic" || userRole === "superadmin";

  onMount(() => {
    const urlParams = new URLSearchParams(window.location.search);
    meetingId = urlParams.get("id") || "";

    if (!meetingId) {
      error = $_("meetings.missing_id");
      loading = false;
      return;
    }

    loadMeeting();
  });

  async function loadMeeting() {
    await withLoadingState({
      action: async () => {
        const m = await api.get<Meeting>(`/meetings/${meetingId}`);
        let b: Building | null = null;
        if (m && m.building_id) {
          try {
            b = await api.get<Building>(`/buildings/${m.building_id}`);
          } catch (e) {
            console.error("Error loading building:", e);
          }
        }
        // Track H Story H3 — Charge la checklist de clôture (best-effort).
        // En cas d'échec (endpoint indisponible / 404), on laisse `null` —
        // le bouton « Clôturer » est désactivé safe par défaut (canCompleteMeeting=false).
        let cl: MeetingCompletionChecklistResponse | null = null;
        if (m) {
          try {
            cl = await api.get<MeetingCompletionChecklistResponse>(
              `/meetings/${meetingId}/completion-checklist`,
            );
          } catch (e) {
            console.warn(
              "Completion checklist unavailable (defaulting to disabled):",
              e,
            );
          }
        }
        return { meeting: m, building: b, checklist: cl };
      },
      setLoading: (v) => (loading = v),
      setError: (v) => (error = v),
      onSuccess: (result) => {
        meeting = result.meeting;
        building = result.building;
        completionChecklist = result.checklist;
      },
      errorMessage: $_("meetings.error_loading_meeting"),
    });
  }

  const handleGoBack = () => {
    window.history.back();
  };

  const handleComplete = async () => {
    if (!meeting) return;

    // Track H Story H3 — Defense-in-depth FE : ne soumet pas si la checklist
    // indique des invariants manquants (le BE refuse de toute façon en 422).
    if (!canCompleteMeeting) {
      showMeetingCompletionToast({
        error: "blocked",
        kind: "meeting_not_completable",
        details: {
          code: "MEETING_NOT_COMPLETABLE",
          meeting_id: meeting.id,
          missing: completionChecklist?.missing ?? [],
        },
      });
      return;
    }

    const attendees = prompt($_("meetings.prompt_attendees"));
    if (!attendees) return;

    const attendeesCount = parseInt(attendees);
    if (isNaN(attendeesCount) || attendeesCount < 0) {
      toast.error($_("meetings.invalid_number"));
      return;
    }

    try {
      await api.post(`/meetings/${meeting!.id}/complete`, {
        attendees_count: attendeesCount,
      });
      toast.success($_("meetings.marked_completed"));
      await loadMeeting();
    } catch (err) {
      // Track H Story H3 — 422 narratif MEETING_NOT_COMPLETABLE → toast i18n
      // + re-fetch checklist pour rafraîchir la liste affichée.
      if (showMeetingCompletionToast(err)) {
        await loadMeeting();
        return;
      }
      toast.error($_("common.error_updating"));
    }
  };

  const handleCancel = async () => {
    if (!meeting) return;

    if (!confirm($_("meetings.confirm_cancel"))) {
      return;
    }

    await withErrorHandling({
      action: () => api.post(`/meetings/${meeting!.id}/cancel`, {}),
      successMessage: $_("meetings.cancelled_success"),
      errorMessage: $_("meetings.error_cancelling"),
      onSuccess: () => {
        loadMeeting();
      },
    });
  };

  const handleReschedule = async () => {
    if (!meeting) return;

    const newDate = prompt($_("meetings.prompt_new_date"));
    if (!newDate) return;

    const date = new Date(newDate);
    if (isNaN(date.getTime())) {
      toast.error($_("meetings.invalid_date_format"));
      return;
    }

    await withErrorHandling({
      action: () =>
        api.post(`/meetings/${meeting!.id}/reschedule`, {
          scheduled_date: date.toISOString(),
        }),
      successMessage: $_("meetings.rescheduled_success"),
      errorMessage: $_("meetings.error_rescheduling"),
      onSuccess: () => {
        loadMeeting();
      },
    });
  };

  function getStatusBadge(status: string): { class: string; label: string } {
    const badges: Record<string, { class: string; label: string }> = {
      Scheduled: {
        class: "bg-blue-100 text-blue-800",
        label: $_("meetings.status_scheduled"),
      },
      Completed: {
        class: "bg-green-100 text-green-800",
        label: $_("meetings.status_completed"),
      },
      Cancelled: {
        class: "bg-red-100 text-red-800",
        label: $_("meetings.status_cancelled"),
      },
    };
    return (
      badges[status] || { class: "bg-gray-100 text-gray-800", label: status }
    );
  }

  function getMeetingTypeLabel(type: string): string {
    const labels: Record<string, string> = {
      ordinary: $_("meetings.type_ordinary"),
      extraordinary: $_("meetings.type_extraordinary"),
    };
    return labels[type?.toLowerCase()] || type;
  }

  function isUpcoming(date: string): boolean {
    return new Date(date) > new Date();
  }
</script>

<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
  {#if loading}
    <div
      class="flex items-center justify-center min-h-screen"
      data-testid="meeting-detail-loading"
    >
      <div class="text-center">
        <div
          class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"
        ></div>
        <p class="mt-4 text-gray-600">{$_("common.loading")}</p>
      </div>
    </div>
  {:else if error}
    <div
      class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-4"
    >
      {error}
    </div>
    <div class="mt-4">
      <Button variant="outline" onclick={handleGoBack}>
        {$_("common.back")}
      </Button>
    </div>
  {:else if meeting}
    <div class="mb-8">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-4">
          <button
            on:click={handleGoBack}
            class="text-gray-600 hover:text-gray-900"
          >
            {$_("common.back")}
          </button>
          <h1 class="text-3xl font-bold text-gray-900">{meeting.title}</h1>
        </div>
        <div class="flex gap-2 items-start flex-wrap">
          {#if canManage && meeting.status === "Scheduled"}
            <div class="flex flex-col items-start">
              <Button
                variant="primary"
                on:click={handleComplete}
                disabled={!canCompleteMeeting}
                aria-disabled={!canCompleteMeeting}
                aria-describedby={!canCompleteMeeting
                  ? "meeting-complete-disabled-reason"
                  : undefined}
                data-testid="meeting-complete-btn"
              >
                {$_("meetings.mark_completed")}
              </Button>
              {#if !canCompleteMeeting && completionChecklist}
                <p
                  id="meeting-complete-disabled-reason"
                  class="text-xs text-red-700 mt-1"
                  data-testid="meeting-complete-button-disabled-reason"
                >
                  {$_("meeting.complete.disabled_reason", {
                    values: { n: missingInvariantsCount },
                  })}
                </p>
              {/if}
            </div>
            <Button
              variant="outline"
              on:click={handleCancel}
              data-testid="meeting-cancel-btn"
            >
              {$_("common.cancel")}
            </Button>
            <Button
              variant="outline"
              on:click={handleReschedule}
              data-testid="meeting-reschedule-btn"
            >
              {$_("meetings.reschedule")}
            </Button>
          {:else if canManage && meeting.status === "Cancelled"}
            <Button
              variant="primary"
              on:click={handleReschedule}
              data-testid="meeting-reschedule-btn"
            >
              {$_("meetings.reschedule")}
            </Button>
          {/if}
        </div>
      </div>
    </div>

    <div class="bg-white rounded-lg shadow-lg overflow-hidden mb-8">
      <div class="bg-gradient-to-r from-primary-600 to-primary-700 px-6 py-4">
        <div class="flex items-center justify-between">
          <h2 class="text-xl font-semibold text-white">
            {$_("meetings.general_info")}
          </h2>
          <span
            class="px-3 py-1 rounded-full text-sm font-medium {getStatusBadge(
              meeting.status,
            ).class}"
            data-testid="meeting-status-badge"
          >
            {getStatusBadge(meeting.status).label}
          </span>
        </div>
      </div>
      <div class="p-6">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div data-testid="meeting-info-type">
            <h3
              class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2"
            >
              {$_("meetings.meeting_type")}
            </h3>
            <p class="text-lg text-gray-900">
              {getMeetingTypeLabel(meeting.meeting_type)}
            </p>
          </div>

          <div data-testid="meeting-info-date">
            <h3
              class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2"
            >
              {$_("meetings.date_time")}
            </h3>
            <p class="text-lg text-gray-900">
              {formatDateTime(meeting.scheduled_date)}
            </p>
            {#if isUpcoming(meeting.scheduled_date) && meeting.status === "Scheduled"}
              <span
                class="inline-flex items-center px-2 py-1 text-xs font-medium bg-blue-100 text-blue-800 rounded-full mt-1"
              >
                {$_("meetings.upcoming")}
              </span>
            {/if}
          </div>

          <div data-testid="meeting-info-location">
            <h3
              class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2"
            >
              {$_("meetings.location")}
            </h3>
            <p class="text-lg text-gray-900">{meeting.location}</p>
          </div>

          {#if building}
            <div data-testid="meeting-info-building">
              <h3
                class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2"
              >
                {$_("meetings.building")}
              </h3>
              <a
                href="/building-detail?id={building.id}"
                class="text-lg text-primary-600 hover:text-primary-700 hover:underline"
              >
                {building.name}
              </a>
              <p class="text-sm text-gray-600">{building.address}</p>
            </div>
          {/if}

          {#if meeting.attendees_count !== undefined && meeting.attendees_count !== null}
            <div data-testid="meeting-info-attendees">
              <h3
                class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2"
              >
                {$_("meetings.participants")}
              </h3>
              <p class="text-lg text-gray-900">
                {$_("meetings.participants_count", {
                  values: { count: meeting.attendees_count },
                })}
              </p>
            </div>
          {/if}

          {#if meeting.description}
            <div class="md:col-span-2" data-testid="meeting-info-description">
              <h3
                class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2"
              >
                {$_("common.description")}
              </h3>
              <p class="text-gray-900 whitespace-pre-line">
                {meeting.description}
              </p>
            </div>
          {/if}

          {#if meeting.agenda && meeting.agenda.length > 0}
            <div class="md:col-span-2" data-testid="meeting-info-agenda">
              <h3
                class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2"
              >
                {$_("meetings.agenda")}
              </h3>
              <div class="bg-gray-50 rounded-lg p-4">
                <ol class="list-decimal list-inside space-y-2">
                  {#each meeting.agenda as item, index}
                    <li class="text-gray-900">
                      <span class="ml-2">{item}</span>
                    </li>
                  {/each}
                </ol>
              </div>
            </div>
          {/if}
        </div>
      </div>
    </div>

    {#if canManage && meeting.status === "Scheduled" && completionChecklist && completionChecklist.missing.length > 0}
      <!-- Track H Story H3 — Bandeau d'invariants Art. 3.87 §3-5 CC manquants.
           Affiché tant que l'AG n'est pas clôturable. Permet au syndic de
           savoir exactement quoi compléter avant de cliquer « Clôturer ». -->
      <div
        class="bg-red-50 border-l-4 border-red-500 text-red-800 p-4 my-4"
        role="alert"
        aria-live="polite"
        data-testid="meeting-completion-blockers"
      >
        <strong>
          {$_("meeting.complete.blockers_title", {
            values: { n: missingInvariantsCount },
          })}
        </strong>
        <MissingInvariantsList invariants={completionChecklist.missing} />
      </div>
    {/if}

    <div class="mb-8">
      <ConvocationPanel
        {meetingId}
        meetingStatus={meeting.status}
        buildingId={meeting.building_id}
        meetingDate={meeting.scheduled_date}
        meetingType={meeting.meeting_type}
      />
    </div>

    <div class="mb-8">
      <QuorumPanel {meeting} {canManage} />
    </div>

    <div class="mb-8">
      <ResolutionList {meetingId} meetingStatus={meeting.status} />
    </div>

    <div class="mb-8">
      <MeetingDocuments {meetingId} meetingStatus={meeting.status} />
    </div>
  {/if}
</div>
