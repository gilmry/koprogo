<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { _ } from '../../lib/i18n';
  import {
    resolutionsApi,
    type Resolution,
    MajorityType,
  } from '../../lib/api/resolutions';
  import { toast } from '../../stores/toast';

  export let meetingId: string;

  const dispatch = createEventDispatcher<{ created: Resolution }>();

  let title = '';
  let description = '';
  let resolutionType = 'standard';
  let majorityRequired: MajorityType = MajorityType.Simple;
  let loading = false;

  async function handleSubmit() {
    if (!title.trim()) {
      toast.error($_("resolutions.create.titleRequired"));
      return;
    }

    try {
      loading = true;
      const resolution = await resolutionsApi.create(meetingId, {
        meeting_id: meetingId,
        title: title.trim(),
        description: description.trim(),
        resolution_type: resolutionType,
        majority_required: majorityRequired,
      });
      toast.success($_("resolutions.create.success"));
      dispatch('created', resolution);
      // Reset form
      title = '';
      description = '';
      resolutionType = 'standard';
      majorityRequired = MajorityType.Simple;
    } catch (err: any) {
      toast.error(err.message || $_("resolutions.create.error"));
    } finally {
      loading = false;
    }
  }
</script>

<form on:submit|preventDefault={handleSubmit} class="bg-gray-50 border border-gray-200 rounded-lg p-4">
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
        >
          <option value="standard">{$_("resolutions.create.typeStandard")}</option>
          <option value="budget">{$_("resolutions.create.typeBudget")}</option>
          <option value="works">{$_("resolutions.create.typeWorks")}</option>
          <option value="rules">{$_("resolutions.create.typeRules")}</option>
          <option value="election">{$_("resolutions.create.typeElection")}</option>
          <option value="other">{$_("common.other")}</option>
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
        >
          <option value={MajorityType.Simple}>{$_("resolutions.create.majoritySimple")}</option>
          <option value={MajorityType.Absolute}>{$_("resolutions.create.majorityAbsolute")}</option>
          <option value={MajorityType.Qualified}>{$_("resolutions.create.majorityQualified")}</option>
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
      on:click={() => dispatch('created', null)}
      class="px-4 py-2 text-sm text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
    >
      {$_("common.cancel")}
    </button>
    <button
      type="submit"
      disabled={loading || !title.trim()}
      class="px-4 py-2 text-sm text-white bg-indigo-600 rounded-md hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed"
    >
      {loading ? $_("resolutions.create.creating") : $_("resolutions.create.submit")}
    </button>
  </div>
</form>
