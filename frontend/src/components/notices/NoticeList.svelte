<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from '../../lib/i18n';
  import { noticesApi, type Notice, NoticeType, NoticeStatus } from "../../lib/api/notices";
  import { toast } from "../../stores/toast";
  import NoticeTypeBadge from "./NoticeTypeBadge.svelte";
  import NoticeStatusBadge from "./NoticeStatusBadge.svelte";
  import { formatDateShort } from "../../lib/utils/date.utils";
  import { withLoadingState } from "../../lib/utils/error.utils";

  let {
    buildingId,
    showFilters = true,
    initialStatus = "active-only",
  }: {
    buildingId: string;
    showFilters?: boolean;
    initialStatus?: NoticeStatus | "active-only";
  } = $props();

  let notices: Notice[] = $state([]);
  let filteredNotices: Notice[] = $state([]);
  let loading = $state(true);
  let searchQuery = $state("");
  let selectedType: NoticeType | "all" = $state("all");
  // Statut affiché à l'ouverture. La page le force à « Draft » juste après une
  // création, sans quoi l'annonce neuve reste invisible : `Notice::new` la crée
  // en BROUILLON (domaine, `notice.rs`) et la vue par défaut ne montre que les
  // publiées. Constaté en recette le 2026-09-06 (RN-6), rapporté comme
  // « aucune annonce créée » alors que le serveur rendait bien 201.
  // svelte-ignore state_referenced_locally
  // Lecture unique au montage, voulue : la page remonte le composant quand elle
  // veut changer de filtre. Le rendre réactif écraserait le choix de
  // l'utilisateur à chaque rendu du parent.
  let selectedStatus: NoticeStatus | "active-only" = $state(initialStatus);

  $effect(() => {
    loadNotices();
  });

  async function loadNotices() {
    await withLoadingState({
      action: () => selectedStatus === "active-only"
        ? noticesApi.listActive(buildingId)
        : noticesApi.listByBuilding(buildingId),
      setLoading: (v: boolean) => loading = v,
      setError: () => {},
      onSuccess: (data) => { notices = data; applyFilters(); },
      errorMessage: $_("notices.load_failed"),
    });
  }

  function applyFilters() {
    filteredNotices = notices.filter((notice) => {
      const matchesSearch =
        searchQuery === "" ||
        notice.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        (notice.content ?? "").toLowerCase().includes(searchQuery.toLowerCase());

      const matchesType = selectedType === "all" || notice.notice_type === selectedType;

      const matchesStatus =
        selectedStatus === "active-only"
          ? notice.status === NoticeStatus.Published
          : notice.status === selectedStatus;

      return matchesSearch && matchesType && matchesStatus;
    });
  }

  $effect(() => {
    searchQuery;
    selectedType;
    selectedStatus;
    applyFilters();
  });

  /// Tronque un texte, en tolérant son ABSENCE.
  ///
  /// Deux points d'entrée servent la même collection d'annonces et ne rendent
  /// PAS la même forme : `/buildings/{id}/notices` renvoie un
  /// `NoticeSummaryDto`, qui n'a pas de `content` ; `/notices/{id}` renvoie un
  /// `NoticeResponseDto`, qui en a un. Le type TypeScript généré déclare
  /// `content: string` — il décrit le second et ment sur le premier.
  ///
  /// Conséquence mesurée le 2026-09-06 : `text.length` sur `undefined` levait
  /// une `TypeError` PENDANT LE RENDU, donc hors du `try` de `withLoadingState`.
  /// Le `finally` qui remet `loading` à faux n'était jamais atteint, et l'écran
  /// restait sur « Chargement des annonces… » indéfiniment. C'est le RN-7 des
  /// recettes 3 et 4, et il rendait la création d'annonce invérifiable : la
  /// seule vue montrant un brouillon était précisément celle qui ne chargeait
  /// jamais.
  ///
  /// Le défaut de fond — deux formes pour une même collection — est suivi dans
  /// l'issue #765, sur l'absence de contrat OpenAPI. Ici on refuse simplement
  /// qu'un champ absent casse la page.
  function truncate(text: string | undefined | null, maxLength: number): string {
    if (!text) return "";
    if (text.length <= maxLength) return text;
    return text.substring(0, maxLength) + "...";
  }
</script>

<div class="space-y-4" data-testid="notice-list">
  {#if showFilters}
    <!-- Filters -->
    <div class="bg-white shadow rounded-lg p-4" data-testid="notice-list-filters">
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <!-- Search -->
        <div>
          <label for="search" class="block text-sm font-medium text-gray-700 mb-1">{$_("common.search")}</label>
          <input
            type="text"
            id="search"
            bind:value={searchQuery}
            placeholder={$_("notices.search_notices")}
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
          />
        </div>

        <!-- Type Filter -->
        <div>
          <label for="type" class="block text-sm font-medium text-gray-700 mb-1">{$_("notices.type")}</label>
          <select
            id="type"
            bind:value={selectedType}
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="all">{$_("notices.all_types")}</option>
            {#each Object.values(NoticeType) as type}
              <option value={type}>{$_(`notices.noticeType.${type}`)}</option>
            {/each}
          </select>
        </div>

        <!-- Status Filter -->
        <div>
          <label for="status" class="block text-sm font-medium text-gray-700 mb-1">{$_("notices.status")}</label>
          <select
            id="status"
            bind:value={selectedStatus}
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="active-only">{$_("notices.published_only")}</option>
            <option value={NoticeStatus.Draft}>{$_("notices.status_draft")}</option>
            <option value={NoticeStatus.Published}>{$_("notices.status_published")}</option>
            <option value={NoticeStatus.Archived}>{$_("notices.status_archived")}</option>
            <option value={NoticeStatus.Expired}>{$_("notices.status_expired")}</option>
          </select>
        </div>
      </div>
    </div>
  {/if}

  <!-- Notices List -->
  <div class="bg-white shadow rounded-lg overflow-hidden">
    {#if loading}
      <div class="text-center py-12 text-gray-500" data-testid="notice-list-loading">{$_("notices.loading")}</div>
    {:else if filteredNotices.length === 0}
      <div class="text-center py-12 text-gray-500">
        {$_("notices.no_notices_found")} {#if searchQuery || selectedType !== "all"}{$_("notices.try_adjusting_filters")}{/if}
      </div>
    {:else}
      <ul class="divide-y divide-gray-200">
        {#each filteredNotices as notice}
          <li data-testid="notice-list-row">
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
                    <span>📅 {formatDateShort(notice.created_at)}</span>
                    {#if notice.author_name}
                      <span>👤 {notice.author_name}</span>
                    {/if}
                    {#if notice.is_pinned}<span>📌 {$_("notices.pinned")}</span>{/if}
                    {#if notice.expires_at}
                      <span>⏰ {$_("notices.expires")} {formatDateShort(notice.expires_at)}</span>
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
      {$_("notices.showing_results", { values: { shown: filteredNotices.length, total: notices.length } })}
    </p>
  {/if}
</div>
