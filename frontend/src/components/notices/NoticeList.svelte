<script lang="ts">
  import { onMount } from "svelte";
  import { noticesApi, type Notice, NoticeType, NoticeStatus } from "../../lib/api/notices";
  import { toast } from "../../stores/toast";
  import NoticeTypeBadge from "./NoticeTypeBadge.svelte";
  import NoticeStatusBadge from "./NoticeStatusBadge.svelte";

  export let buildingId: string;
  export let showFilters = true;

  let notices: Notice[] = [];
  let filteredNotices: Notice[] = [];
  let loading = true;
  let searchQuery = "";
  let selectedType: NoticeType | "all" = "all";
  let selectedStatus: NoticeStatus | "active-only" = "active-only";

  onMount(async () => {
    await loadNotices();
  });

  async function loadNotices() {
    try {
      loading = true;
      if (selectedStatus === "active-only") {
        notices = await noticesApi.listActive(buildingId);
      } else {
        notices = await noticesApi.listByBuilding(buildingId);
      }
      applyFilters();
    } catch (err: any) {
      toast.error(err.message || "Failed to load notices");
    } finally {
      loading = false;
    }
  }

  function applyFilters() {
    filteredNotices = notices.filter((notice) => {
      const matchesSearch =
        searchQuery === "" ||
        notice.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        notice.content.toLowerCase().includes(searchQuery.toLowerCase());

      const matchesType = selectedType === "all" || notice.notice_type === selectedType;

      const matchesStatus =
        selectedStatus === "active-only"
          ? notice.status === NoticeStatus.Published
          : notice.status === selectedStatus;

      return matchesSearch && matchesType && matchesStatus;
    });
  }

  $: {
    searchQuery;
    selectedType;
    selectedStatus;
    applyFilters();
  }

  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
    });
  }

  function truncate(text: string, maxLength: number): string {
    if (text.length <= maxLength) return text;
    return text.substring(0, maxLength) + "...";
  }
</script>

<div class="space-y-4">
  {#if showFilters}
    <!-- Filters -->
    <div class="bg-white shadow rounded-lg p-4">
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <!-- Search -->
        <div>
          <label for="search" class="block text-sm font-medium text-gray-700 mb-1">Search</label>
          <input
            type="text"
            id="search"
            bind:value={searchQuery}
            placeholder="Search notices..."
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
          />
        </div>

        <!-- Type Filter -->
        <div>
          <label for="type" class="block text-sm font-medium text-gray-700 mb-1">Type</label>
          <select
            id="type"
            bind:value={selectedType}
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="all">All Types</option>
            {#each Object.values(NoticeType) as type}
              <option value={type}>{type}</option>
            {/each}
          </select>
        </div>

        <!-- Status Filter -->
        <div>
          <label for="status" class="block text-sm font-medium text-gray-700 mb-1">Status</label>
          <select
            id="status"
            bind:value={selectedStatus}
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="active-only">Published Only</option>
            <option value={NoticeStatus.Draft}>Draft</option>
            <option value={NoticeStatus.Published}>Published</option>
            <option value={NoticeStatus.Archived}>Archived</option>
            <option value={NoticeStatus.Expired}>Expired</option>
          </select>
        </div>
      </div>
    </div>
  {/if}

  <!-- Notices List -->
  <div class="bg-white shadow rounded-lg overflow-hidden">
    {#if loading}
      <div class="text-center py-12 text-gray-500">Loading notices...</div>
    {:else if filteredNotices.length === 0}
      <div class="text-center py-12 text-gray-500">
        No notices found. {#if searchQuery || selectedType !== "all"}Try adjusting your filters.{/if}
      </div>
    {:else}
      <ul class="divide-y divide-gray-200">
        {#each filteredNotices as notice}
          <li>
            <a
              href={`/notice-detail?id=${notice.id}`}
              class="block hover:bg-gray-50 transition-colors duration-150 p-4"
            >
              <div class="flex items-start justify-between">
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 mb-1">
                    <NoticeTypeBadge type={notice.notice_type} />
                    <NoticeStatusBadge status={notice.status} />
                  </div>
                  <p class="text-lg font-semibold text-gray-900">{notice.title}</p>
                  <p class="text-sm text-gray-600 mt-1">
                    {truncate(notice.content, 150)}
                  </p>
                  <div class="flex items-center gap-4 mt-2 text-xs text-gray-500">
                    <span>📅 {formatDate(notice.created_at)}</span>
                    {#if notice.author_name}
                      <span>👤 {notice.author_name}</span>
                    {/if}
                    {#if notice.is_pinned}<span>📌 Pinned</span>{/if}
                    {#if notice.expires_at}
                      <span>⏰ Expires {formatDate(notice.expires_at)}</span>
                    {/if}
                  </div>
                </div>
                <div class="ml-4 flex-shrink-0">
                  <svg
                    class="h-5 w-5 text-gray-400"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M9 5l7 7-7 7"
                    />
                  </svg>
                </div>
              </div>
            </a>
          </li>
        {/each}
      </ul>
    {/if}
  </div>

  <!-- Results count -->
  {#if !loading && filteredNotices.length > 0}
    <p class="text-sm text-gray-600 text-center">
      Showing {filteredNotices.length} of {notices.length} notices
    </p>
  {/if}
</div>
