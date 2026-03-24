<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { _ } from "svelte-i18n";
  import {
    pollsApi,
    type CreatePollDto,
    type CreatePollOptionDto,
    PollType,
  } from "../../lib/api/polls";
  import BuildingSelector from "../BuildingSelector.svelte";

  const dispatch = createEventDispatcher();

  let selectedBuildingId = "";

  let formData: CreatePollDto = {
    building_id: "",
    poll_type: PollType.YesNo,
    title: "",
    description: "",
    ends_at: "",
    is_anonymous: false,
    allow_multiple_votes: false,
    options: [
      { option_text: "Oui", display_order: 1 },
      { option_text: "Non", display_order: 2 },
    ],
  };

  $: formData.building_id = selectedBuildingId;

  // UI-only fields (not sent to backend)
  let startsAt = new Date().toISOString().split("T")[0];

  let loading = false;
  let error = "";
  let success = false;

  // For multiple choice options
  let newOptionText = "";

  let endsAtDate = "";

  function setDefaultEndDate() {
    if (startsAt) {
      const startDate = new Date(startsAt);
      startDate.setDate(startDate.getDate() + 7); // 7 days by default
      endsAtDate = startDate.toISOString().split("T")[0];
    }
  }

  function onPollTypeChange() {
    formData.options = [];
    if (formData.poll_type === PollType.YesNo) {
      // Auto-create Yes/No options
      formData.options = [
        { option_text: "Oui", display_order: 1 },
        { option_text: "Non", display_order: 2 },
      ];
    }
  }

  function addOption() {
    if (!newOptionText.trim()) return;
    formData.options = [
      ...formData.options,
      {
        option_text: newOptionText.trim(),
        display_order: formData.options.length + 1,
      },
    ];
    newOptionText = "";
  }

  function removeOption(index: number) {
    formData.options = formData.options.filter((_, i) => i !== index);
    // Reorder
    formData.options = formData.options.map((opt, i) => ({
      ...opt,
      display_order: i + 1,
    }));
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    loading = true;
    error = "";
    success = false;

    try {
      // Validate
      if (!formData.building_id) {
        throw new Error($_("polls.createForm.errors.selectBuilding"));
      }
      if (!formData.title.trim()) {
        throw new Error($_("polls.createForm.errors.titleRequired"));
      }
      if (!endsAtDate) {
        throw new Error($_("polls.createForm.errors.endDateRequired"));
      }
      if (endsAtDate <= startsAt) {
        throw new Error($_("polls.createForm.errors.endDateAfterStart"));
      }
      // Convert date to ISO 8601 for backend
      formData.ends_at = new Date(endsAtDate + "T23:59:59Z").toISOString();
      if (
        (formData.poll_type === PollType.YesNo ||
          formData.poll_type === PollType.MultipleChoice) &&
        formData.options.length === 0
      ) {
        throw new Error($_("polls.createForm.errors.optionRequired"));
      }

      const poll = await pollsApi.create(formData);
      success = true;
      dispatch("created", poll);

      // Reset form after 2 seconds
      setTimeout(() => {
        window.location.href = `/polls/detail?id=${poll.id}`;
      }, 1500);
    } catch (err: any) {
      error = err.message || $_("polls.createForm.errors.creationFailed");
      console.error("Failed to create poll:", err);
    } finally {
      loading = false;
    }
  }

  // Initialize default end date
  setDefaultEndDate();
</script>

<div class="bg-white shadow-md rounded-lg p-6">
  <h3 class="text-lg font-medium text-gray-900 mb-4">
    ➕ {$_("polls.createForm.title")}
  </h3>

  <p class="text-sm text-gray-600 mb-6">
    {$_("polls.createForm.formDescription")}
    <strong>{$_("polls.createForm.legalReference")}</strong>.
  </p>

  {#if success}
    <div class="mb-4 p-4 bg-green-50 border border-green-200 rounded-md">
      <p class="text-sm text-green-800">
        ✅ {$_("polls.createForm.successMessage")}
      </p>
    </div>
  {/if}

  {#if error}
    <div class="mb-4 p-4 bg-red-50 border border-red-200 rounded-md">
      <p class="text-sm text-red-800">❌ {error}</p>
    </div>
  {/if}

  <form on:submit={handleSubmit} class="space-y-6" data-testid="create-poll-form">
    <!-- Building Selector -->
    <BuildingSelector bind:selectedBuildingId label={$_("polls.createForm.buildingLabel")} />

    <!-- Poll Type -->
    <div>
      <label for="poll_type" class="block text-sm font-medium text-gray-700">
        {$_("polls.createForm.pollType")} <span class="text-red-500">*</span>
      </label>
      <select
        id="poll_type"
        bind:value={formData.poll_type}
        on:change={onPollTypeChange}
        required
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
        data-testid="create-poll-type-select"
      >
        <option value={PollType.YesNo}>👍👎 {$_("polls.createForm.typeYesNo")}</option>
        <option value={PollType.MultipleChoice}>☑️ {$_("polls.createForm.typeMultiple")}</option>
        <option value={PollType.Rating}>⭐ {$_("polls.createForm.typeRating")}</option>
        <option value={PollType.OpenEnded}>💬 {$_("polls.createForm.typeOpenEnded")}</option>
      </select>
    </div>

    <!-- Question -->
    <div>
      <label for="question" class="block text-sm font-medium text-gray-700">
        {$_("polls.createForm.question")} <span class="text-red-500">*</span>
      </label>
      <input
        type="text"
        id="question"
        bind:value={formData.title}
        required
        placeholder={$_("polls.createForm.questionPlaceholder")}
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
        data-testid="create-poll-question-input"
      />
    </div>

    <!-- Description -->
    <div>
      <label for="description" class="block text-sm font-medium text-gray-700">
        {$_("polls.createForm.description")} ({$_("common.optional")})
      </label>
      <textarea
        id="description"
        bind:value={formData.description}
        rows="3"
        placeholder={$_("polls.createForm.descriptionPlaceholder")}
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
        data-testid="create-poll-description-input"
      ></textarea>
    </div>

    <!-- Options (for YesNo and MultipleChoice) -->
    {#if formData.poll_type === PollType.YesNo}
      <div class="p-4 bg-blue-50 border border-blue-200 rounded-md">
        <p class="text-sm text-blue-800">
          ℹ️ {$_("polls.createForm.yesNoInfo")}
        </p>
      </div>
    {:else if formData.poll_type === PollType.MultipleChoice}
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-2">
          {$_("polls.createForm.options")} <span class="text-red-500">*</span>
        </label>
        <div class="space-y-2 mb-3">
          {#each formData.options as option, index}
            <div class="flex items-center space-x-2">
              <span class="text-sm text-gray-500">{index + 1}.</span>
              <input
                type="text"
                value={option.option_text}
                readonly
                class="flex-1 rounded-md border-gray-300 bg-gray-50"
              />
              <button
                type="button"
                on:click={() => removeOption(index)}
                class="text-red-600 hover:text-red-800"
                title={$_("common.delete")}
              >
                🗑️
              </button>
            </div>
          {/each}
        </div>
        <div class="flex items-center space-x-2">
          <input
            type="text"
            bind:value={newOptionText}
            placeholder={$_("polls.createForm.newOptionPlaceholder")}
            class="flex-1 rounded-md border-gray-300"
            data-testid="create-poll-new-option-input"
            on:keypress={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                addOption();
              }
            }}
          />
          <button
            type="button"
            on:click={addOption}
            class="px-4 py-2 bg-indigo-100 text-indigo-700 rounded-md hover:bg-indigo-200"
            data-testid="create-poll-add-option-btn"
          >
            ➕ {$_("common.add")}
          </button>
        </div>
        <p class="mt-1 text-xs text-gray-500">
          {$_("polls.createForm.optionsHint")}
        </p>
      </div>
    {:else if formData.poll_type === PollType.Rating}
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-2">
          {$_("polls.createForm.ratingScale")}
        </label>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="text-xs text-gray-500">{$_("polls.createForm.minRating")}</label>
            <input
              type="number"
              value="1"
              disabled
              min="1"
              max="5"
              class="mt-1 block w-full rounded-md border-gray-300"
            />
          </div>
          <div>
            <label class="text-xs text-gray-500">{$_("polls.createForm.maxRating")}</label>
            <input
              type="number"
              value="5"
              disabled
              min="1"
              max="10"
              class="mt-1 block w-full rounded-md border-gray-300"
            />
          </div>
        </div>
        <p class="mt-1 text-xs text-gray-500">
          {$_("polls.createForm.ratingExample")}
        </p>
      </div>
    {/if}

    <!-- Dates -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div>
        <label for="starts_at" class="block text-sm font-medium text-gray-700">
          {$_("polls.createForm.startDate")} <span class="text-red-500">*</span>
        </label>
        <input
          type="date"
          id="starts_at"
          bind:value={startsAt}
          on:change={setDefaultEndDate}
          required
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
          data-testid="create-poll-start-date-input"
        />
      </div>
      <div>
        <label for="ends_at" class="block text-sm font-medium text-gray-700">
          {$_("polls.createForm.endDate")} <span class="text-red-500">*</span>
        </label>
        <input
          type="date"
          id="ends_at"
          bind:value={endsAtDate}
          required
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
          data-testid="create-poll-end-date-input"
        />
      </div>
    </div>
    <p class="text-xs text-gray-500">
      {$_("polls.createForm.durationRecommendation")}
    </p>

    <!-- Options -->
    <div class="space-y-3">
      <label class="flex items-center">
        <input
          type="checkbox"
          bind:checked={formData.is_anonymous}
          class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
          data-testid="create-poll-anonymous-checkbox"
        />
        <span class="ml-2 text-sm text-gray-700">
          🔒 {$_("polls.createForm.anonymousVote")}
        </span>
      </label>

      {#if formData.poll_type === PollType.MultipleChoice}
        <label class="flex items-center">
          <input
            type="checkbox"
            bind:checked={formData.allow_multiple_votes}
            class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
            data-testid="create-poll-multiple-votes-checkbox"
          />
          <span class="ml-2 text-sm text-gray-700">
            ☑️ {$_("polls.createForm.multipleSelection")}
          </span>
        </label>
      {/if}
    </div>

    <!-- Legal Notice -->
    <div class="p-4 bg-yellow-50 border border-yellow-200 rounded-md">
      <h4 class="text-sm font-medium text-yellow-900 mb-2">
        ⚖️ {$_("polls.createForm.legalFramework")}
      </h4>
      <p class="text-xs text-yellow-800">
        <strong>{$_("polls.createForm.legalReference")}:</strong> {$_("polls.createForm.legalText")}
      </p>
    </div>

    <!-- Submit Button -->
    <div class="flex justify-end space-x-3">
      <button
        type="button"
        on:click={() => dispatch("cancel")}
        class="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
        data-testid="create-poll-cancel-btn"
      >
        {$_("common.cancel")}
      </button>
      <button
        type="submit"
        disabled={loading}
        class="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed"
        data-testid="create-poll-submit-btn"
      >
        {#if loading}
          <span class="inline-block animate-spin mr-2">⏳</span>
          {$_("polls.createForm.creatingButton")}
        {:else}
          ✅ {$_("polls.createForm.submitButton")}
        {/if}
      </button>
    </div>
  </form>
</div>
