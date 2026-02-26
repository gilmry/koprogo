<script lang="ts">
  import { onMount } from "svelte";
  import { noticesApi, type Notice, NoticeStatus } from "../../lib/api/notices";
  import { toast } from "../../stores/toast";
  import NoticeTypeBadge from "./NoticeTypeBadge.svelte";
  import NoticeStatusBadge from "./NoticeStatusBadge.svelte";

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
    try {
      loading = true;
      notice = await noticesApi.getById(noticeId);
    } catch (err: any) {
      toast.error(err.message || "Failed to load notice");
    } finally {
      loading = false;
    }
  }

  async function handleArchive() {
    if (!confirm("Are you sure you want to archive this notice?")) return;

    try {
      archiving = true;
      await noticesApi.archive(noticeId);
      toast.success("Notice archived successfully");
      window.location.href = "/notices";
    } catch (err: any) {
      toast.error(err.message || "Failed to archive notice");
    } finally {
      archiving = false;
    }
  }

  async function handleDelete() {
    if (!confirm("Are you sure you want to delete this notice? This action cannot be undone."))
      return;

    try {
      deleting = true;
      await noticesApi.delete(noticeId);
      toast.success("Notice deleted successfully");
      window.location.href = "/notices";
    } catch (err: any) {
      toast.error(err.message || "Failed to delete notice");
    } finally {
      deleting = false;
    }
  }

  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString("en-US", {
      month: "long",
      day: "numeric",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  $: isAuthor = notice && notice.author_id === currentUserId;
</script>

<div class="bg-white shadow rounded-lg overflow-hidden">
  {#if loading}
    <div class="text-center py-12 text-gray-500">Loading notice...</div>
  {:else if notice}
    <div class="p-6">
      <!-- Header -->
      <div class="flex items-start justify-between mb-4">
        <div class="flex-1">
          <div class="flex items-center gap-2 mb-2">
            <NoticeTypeBadge type={notice.notice_type} />
            <NoticeStatusBadge status={notice.status} />
            {#if notice.is_pinned}
              <span class="text-xs text-gray-500">📌 Pinned</span>
            {/if}
          </div>
          <h1 class="text-2xl font-bold text-gray-900">{notice.title}</h1>
        </div>

        {#if isAuthor}
          <div class="flex gap-2">
            <button
              on:click={handleArchive}
              disabled={archiving || notice.status === "Archived"}
              class="px-3 py-1 text-sm text-gray-700 bg-gray-100 rounded hover:bg-gray-200 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {archiving ? "Archiving..." : "Archive"}
            </button>
            <button
              on:click={handleDelete}
              disabled={deleting}
              class="px-3 py-1 text-sm text-white bg-red-600 rounded hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {deleting ? "Deleting..." : "Delete"}
            </button>
          </div>
        {/if}
      </div>

      <!-- Metadata -->
      <div class="flex flex-wrap items-center gap-4 mb-4 text-sm text-gray-600 border-b border-gray-200 pb-4">
        {#if notice.author_name}
          <span>👤 Posted by {notice.author_name}</span>
        {/if}
        <span>📅 {formatDate(notice.created_at)}</span>
        {#if notice.published_at}
          <span>📤 Published {formatDate(notice.published_at)}</span>
        {/if}
        {#if notice.expires_at}
          <span>⏰ Expires {formatDate(notice.expires_at)}</span>
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
          <h3 class="text-sm font-semibold text-pink-900 mb-1">Event Details</h3>
          <p class="text-sm text-pink-700">📅 {formatDate(notice.event_date)}</p>
          {#if notice.event_location}
            <p class="text-sm text-pink-700">📍 {notice.event_location}</p>
          {/if}
          {#if notice.days_until_event !== undefined && notice.days_until_event !== null}
            <p class="text-sm text-pink-600 mt-1">
              {notice.days_until_event > 0 ? `In ${notice.days_until_event} days` : notice.days_until_event === 0 ? "Today!" : "Past event"}
            </p>
          {/if}
        </div>
      {/if}

      <!-- Contact Info -->
      {#if notice.contact_info}
        <div class="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-4">
          <h3 class="text-sm font-semibold text-blue-900 mb-1">Contact Information</h3>
          <p class="text-sm text-blue-700">{notice.contact_info}</p>
        </div>
      {/if}
    </div>
  {:else}
    <div class="text-center py-12 text-gray-500">Notice not found</div>
  {/if}
</div>
