<script lang="ts">
  // Svelte 5 runes mode — migrated from legacy (STORY-P7-604)
  import { _ } from '../../lib/i18n';
  import {
    resolutionsApi,
    type Resolution,
    MajorityType,
    ResolutionType,
  } from '../../lib/api/resolutions';
  import { toast } from '../../stores/toast';
  import { withErrorHandling } from '../../lib/utils/error.utils';

  let {
    meetingId,
    oncreated,
  }: {
    meetingId: string;
    oncreated?: (resolution: Resolution | null) => void;
  } = $props();

  let title = $state('');
  let description = $state('');
  let resolutionType = $state<string>(ResolutionType.Ordinary);
  let majorityRequired = $state<string>(MajorityType.Absolute);
  let loading = $state(false);

  async function handleSubmit() {
    if (!title.trim()) {
      toast.error($_("resolutions.create.titleRequired"));
      return;
    }

    await withErrorHandling({
      action: () => resolutionsApi.create(meetingId, {
        meeting_id: meetingId,
        title: title.trim(),
        description: description.trim(),
        resolution_type: resolutionType,
        majority_required: majorityRequired as any,
      }),
      setLoading: (v: boolean) => loading = v,
      successMessage: $_("resolutions.create.success"),
      errorMessage: $_("resolutions.create.error"),
      onSuccess: (resolution: Resolution) => {
        oncreated?.(resolution);
        title = '';
        description = '';
        resolutionType = ResolutionType.Ordinary;
        majorityRequired = MajorityType.Absolute;
      },
    });
  }
</script>

<form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }} class="bg-gray-50 border border-gray-200 rounded-lg p-4">
  <h4 class="text-sm font-semibold text-gray-900 mb-3">{$_("resolutions.create.title")}</h4>

  <div class="space-y-3">
    <div>
      <label for="resolution-title" class="block text-xs font-medium text-gray-700 mb-1">
        {$_("common.title")} *
      </label>
      <input
        id="resolution-title"
        type="text"
        bind:value={title}
        placeholder={$_("resolutions.create.titlePlaceholder")}
        required
        class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
        data-testid="resolution-title-input"
      />
    </div>

    <div>
      <label for="resolution-description" class="block text-xs font-medium text-gray-700 mb-1">
        {$_("common.description")}
      </label>
      <textarea
        id="resolution-description"
        bind:value={description}
        rows="2"
        placeholder={$_("resolutions.create.descriptionPlaceholder")}
        class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
        data-testid="resolution-description-textarea"
      ></textarea>
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div>
        <label for="resolution-type" class="block text-xs font-medium text-gray-700 mb-1">
          {$_("common.type")}
        </label>
        <select
          id="resolution-type"
          bind:value={resolutionType}
          class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
          data-testid="resolution-type-select"
        >
          <option value={ResolutionType.Ordinary}>{$_("resolutions.create.typeOrdinary")}</option>
          <option value={ResolutionType.Extraordinary}>{$_("resolutions.create.typeExtraordinary")}</option>
        </select>
      </div>

      <div>
        <label for="resolution-majority" class="block text-xs font-medium text-gray-700 mb-1">
          {$_("resolutions.create.majorityRequired")}
        </label>
        <select
          id="resolution-majority"
          bind:value={majorityRequired}
          class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
          data-testid="resolution-majority-select"
        >
          <option value={MajorityType.Absolute}>{$_("resolutions.create.majorityAbsolute")}</option>
          <option value={MajorityType.TwoThirds}>{$_("resolutions.create.majorityTwoThirds")}</option>
          <option value={MajorityType.FourFifths}>{$_("resolutions.create.majorityFourFifths")}</option>
          <option value={MajorityType.Unanimity}>{$_("resolutions.create.majorityUnanimity")}</option>
        </select>
      </div>
    </div>

    <div class="p-3 bg-yellow-50 border border-yellow-200 rounded-md text-xs text-yellow-800">
      <strong>{$_("resolutions.create.belgianLaw")}:</strong> {$_("resolutions.create.legalText")}
    </div>
  </div>

  <div class="flex justify-end gap-2 mt-4">
    <button
      type="button"
      onclick={() => oncreated?.(null)}
      class="px-4 py-2 text-sm text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
      data-testid="resolution-cancel-btn"
    >
      {$_("common.cancel")}
    </button>
    <button
      type="submit"
      disabled={loading || !title.trim()}
      class="px-4 py-2 text-sm text-white bg-indigo-600 rounded-md hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed"
      data-testid="resolution-submit-btn"
    >
      {loading ? $_("resolutions.create.creating") : $_("resolutions.create.submit")}
    </button>
  </div>
</form>
