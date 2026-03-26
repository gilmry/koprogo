<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from '../../lib/i18n';
  import { noticesApi, type Notice, NoticeStatus } from "../../lib/api/notices";
  import { toast } from "../../stores/toast";
  import NoticeTypeBadge from "./NoticeTypeBadge.svelte";
  import NoticeStatusBadge from "./NoticeStatusBadge.svelte";
  import { formatDateTime } from "../../lib/utils/date.utils";
  import { withLoadingState, withErrorHandling } from "../../lib/utils/error.utils";

  export let noticeId: string;
  export let currentUserId: string;

  let notice: Notice | null = null;
  let loading = true;
  let deleting = false;
  let archiving = false;

  onMount(async () => {
    await loadNotice();
    // Increment view count
    try {
      await noticesApi.incrementViewCount(noticeId);
    } catch (err) {
      // Silently fail view count increment
      console.error("Failed to increment view count:", err);
    }
  });

  async function loadNotice() {
    await withLoadingState({
      action: () => noticesApi.getById(noticeId),
      setLoading: (v) => loading = v,
      setError: () => {},
      onSuccess: (data) => notice = data,
      errorMessage: "Failed to load notice",
    });
  }

  async function handleArchive() {
    if (!confirm($_("notices.archive_confirmation"))) return;
    await withErrorHandling({
      action: () => noticesApi.archive(noticeId),
      setLoading: (v) => archiving = v,
      successMessage: $_("notices.archived_successfully"),
      errorMessage: $_("notices.archive_failed"),
      onSuccess: () => { window.location.href = "/notices"; },
    });
  }

  async function handleDelete() {
    if (!confirm($_("notices.delete_confirmation"))) return;
    await withErrorHandling({
      action: () => noticesApi.delete(noticeId),
      setLoading: (v) => deleting = v,
      successMessage: $_("notices.deleted_successfully"),
      errorMessage: $_("notices.delete_failed"),
      onSuccess: () => { window.location.href = "/notices"; },
    });
  }

  $: isAuthor = notice && notice.author_id === currentUserId;
</script>

<div class="bg-white shadow rounded-lg overflow-hidden" data-testid="notice-detail">
  {#if loading}
    <div class="text-center py-12 text-gray-500" data-testid="notice-detail-loading">{$_("notices.loading")}</div>
  {:else if notice}
    <div class="p-6">
      <!-- Header -->
      <div class="flex items-start justify-between mb-4">
        <div class="flex-1">
          <div class="flex items-center gap-2 mb-2">
            <NoticeTypeBadge type={notice.notice_type} />
            <NoticeStatusBadge status={notice.status} />
            {#if notice.is_pinned}
              <span class="text-xs text-gray-500">📌 {$_("notices.pinned")}</span>
            {/if}
          </div>
          <h1 class="text-2xl font-bold text-gray-900">{notice.title}</h1>
        </div>

        {#if isAuthor}
          <div class="flex gap-2">
            <button
              on:click={handleArchive}
              disabled={archiving || notice.status === "Archived"}
              data-testid="notice-archive-btn"
              class="px-3 py-1 text-sm text-gray-700 bg-gray-100 rounded hover:bg-gray-200 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {archiving ? $_("notices.archiving") : $_("notices.archive")}
            </button>
            <button
              on:click={handleDelete}
              disabled={deleting}
              data-testid="notice-delete-btn"
              class="px-3 py-1 text-sm text-white bg-red-600 rounded hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {deleting ? $_("notices.deleting") : $_("common.delete")}
            </button>
          </div>
        {/if}
      </div>

      <!-- Metadata -->
      <div class="flex flex-wrap items-center gap-4 mb-4 text-sm text-gray-600 border-b border-gray-200 pb-4">
        {#if notice.author_name}
          <span>👤 {$_("notices.posted_by")} {notice.author_name}</span>
        {/if}
        <span>📅 {formatDateTime(notice.created_at)}</span>
        {#if notice.published_at}
          <span>📤 {$_("notices.published")} {formatDateTime(notice.published_at)}</span>
        {/if}
        {#if notice.expires_at}
          <span>⏰ {$_("notices.expires")} {formatDateTime(notice.expires_at)}</span>
        {/if}
      </div>

      <!-- Category -->
      {#if notice.category}
        <div class="mb-4">
          <span class="inline-block px-3 py-1 text-sm bg-gray-100 text-gray-700 rounded">
            {notice.category}
          </span>
        </div>
      {/if}

      <!-- Content -->
      <div class="prose max-w-none mb-6">
        <p class="whitespace-pre-wrap text-gray-700">{notice.content}</p>
      </div>

      <!-- Event Info -->
      {#if notice.event_date}
        <div class="bg-pink-50 border border-pink-200 rounded-lg p-4 mb-4">
          <h3 class="text-sm font-semibold text-pink-900 mb-1">{$_("notices.event_details")}</h3>
          <p class="text-sm text-pink-700">📅 {formatDateTime(notice.event_date)}</p>
          {#if notice.event_location}
            <p class="text-sm text-pink-700">📍 {notice.event_location}</p>
          {/if}
          {#if notice.days_until_event !== undefined && notice.days_until_event !== null}
            <p class="text-sm text-pink-600 mt-1">
              {notice.days_until_event > 0 ? $_("notices.in_days", { values: { days: notice.days_until_event } }) : notice.days_until_event === 0 ? $_("notices.today") : $_("notices.past_event")}
            </p>
          {/if}
        </div>
      {/if}

      <!-- Contact Info -->
      {#if notice.contact_info}
        <div class="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-4">
          <h3 class="text-sm font-semibold text-blue-900 mb-1">{$_("notices.contact_information")}</h3>
          <p class="text-sm text-blue-700">{notice.contact_info}</p>
        </div>
      {/if}
    </div>
  {:else}
    <div class="text-center py-12 text-gray-500">{$_("notices.not_found")}</div>
  {/if}
</div>
