<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import {
    pollsApi,
    type CreatePollDto,
    type CreatePollOptionDto,
    PollType,
  } from "../../lib/api/polls";

  export let buildingId: string;

  const dispatch = createEventDispatcher();

  let formData: CreatePollDto = {
    building_id: buildingId,
    poll_type: PollType.YesNo,
    question: "",
    description: "",
    starts_at: new Date().toISOString().split("T")[0],
    ends_at: "",
    is_anonymous: false,
    allow_multiple_votes: false,
    min_rating: 1,
    max_rating: 5,
    options: [],
  };

  let loading = false;
  let error = "";
  let success = false;

  // For multiple choice options
  let newOptionText = "";

  function setDefaultEndDate() {
    if (formData.starts_at) {
      const startDate = new Date(formData.starts_at);
      startDate.setDate(startDate.getDate() + 7); // 7 days by default
      formData.ends_at = startDate.toISOString().split("T")[0];
    }
  }

  function onPollTypeChange() {
    formData.options = [];
    if (formData.poll_type === PollType.YesNo) {
      // Auto-create Yes/No options
      formData.options = [
        { option_text: "Oui", option_value: 1, display_order: 1 },
        { option_text: "Non", option_value: 0, display_order: 2 },
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
      if (!formData.question.trim()) {
        throw new Error("La question est obligatoire");
      }
      if (!formData.ends_at) {
        throw new Error("La date de fin est obligatoire");
      }
      if (formData.ends_at <= formData.starts_at!) {
        throw new Error("La date de fin doit être postérieure à la date de début");
      }
      if (
        (formData.poll_type === PollType.YesNo ||
          formData.poll_type === PollType.MultipleChoice) &&
        formData.options.length === 0
      ) {
        throw new Error("Au moins une option est requise");
      }

      const poll = await pollsApi.create(formData);
      success = true;
      dispatch("created", poll);

      // Reset form after 2 seconds
      setTimeout(() => {
        window.location.href = `/polls/${poll.id}`;
      }, 1500);
    } catch (err: any) {
      error = err.message || "Erreur lors de la création du sondage";
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
    ➕ Créer un sondage (consultation)
  </h3>

  <p class="text-sm text-gray-600 mb-6">
    Consultez les copropriétaires entre les assemblées générales sur une
    décision ponctuelle. Conforme à l'<strong>Article 577-8/4 §4 du Code Civil Belge</strong>.
  </p>

  {#if success}
    <div class="mb-4 p-4 bg-green-50 border border-green-200 rounded-md">
      <p class="text-sm text-green-800">
        ✅ Sondage créé avec succès ! Redirection...
      </p>
    </div>
  {/if}

  {#if error}
    <div class="mb-4 p-4 bg-red-50 border border-red-200 rounded-md">
      <p class="text-sm text-red-800">❌ {error}</p>
    </div>
  {/if}

  <form on:submit={handleSubmit} class="space-y-6">
    <!-- Poll Type -->
    <div>
      <label for="poll_type" class="block text-sm font-medium text-gray-700">
        Type de sondage <span class="text-red-500">*</span>
      </label>
      <select
        id="poll_type"
        bind:value={formData.poll_type}
        on:change={onPollTypeChange}
        required
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
      >
        <option value={PollType.YesNo}>👍👎 Oui/Non (décision binaire)</option>
        <option value={PollType.MultipleChoice}>☑️ Choix multiple (plusieurs options)</option>
        <option value={PollType.Rating}>⭐ Notation (1-5 étoiles)</option>
        <option value={PollType.OpenEnded}>💬 Texte libre (avis/suggestions)</option>
      </select>
    </div>

    <!-- Question -->
    <div>
      <label for="question" class="block text-sm font-medium text-gray-700">
        Question <span class="text-red-500">*</span>
      </label>
      <input
        type="text"
        id="question"
        bind:value={formData.question}
        required
        placeholder="Ex: Êtes-vous favorable à la rénovation de la façade ?"
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
      />
    </div>

    <!-- Description -->
    <div>
      <label for="description" class="block text-sm font-medium text-gray-700">
        Description (optionnelle)
      </label>
      <textarea
        id="description"
        bind:value={formData.description}
        rows="3"
        placeholder="Contexte, détails supplémentaires..."
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
      ></textarea>
    </div>

    <!-- Options (for YesNo and MultipleChoice) -->
    {#if formData.poll_type === PollType.YesNo}
      <div class="p-4 bg-blue-50 border border-blue-200 rounded-md">
        <p class="text-sm text-blue-800">
          ℹ️ Les options "Oui" et "Non" sont créées automatiquement pour un
          sondage Oui/Non.
        </p>
      </div>
    {:else if formData.poll_type === PollType.MultipleChoice}
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-2">
          Options de choix <span class="text-red-500">*</span>
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
                title="Supprimer"
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
            placeholder="Nouvelle option..."
            class="flex-1 rounded-md border-gray-300"
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
          >
            ➕ Ajouter
          </button>
        </div>
        <p class="mt-1 text-xs text-gray-500">
          Ajoutez au moins 2 options. Appuyez sur Entrée pour ajouter rapidement.
        </p>
      </div>
    {:else if formData.poll_type === PollType.Rating}
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-2">
          Échelle de notation
        </label>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="text-xs text-gray-500">Note minimale</label>
            <input
              type="number"
              bind:value={formData.min_rating}
              min="1"
              max="5"
              class="mt-1 block w-full rounded-md border-gray-300"
            />
          </div>
          <div>
            <label class="text-xs text-gray-500">Note maximale</label>
            <input
              type="number"
              bind:value={formData.max_rating}
              min="1"
              max="10"
              class="mt-1 block w-full rounded-md border-gray-300"
            />
          </div>
        </div>
        <p class="mt-1 text-xs text-gray-500">
          Ex: 1-5 étoiles (classique) ou 1-10 (notation détaillée)
        </p>
      </div>
    {/if}

    <!-- Dates -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div>
        <label for="starts_at" class="block text-sm font-medium text-gray-700">
          Date de début <span class="text-red-500">*</span>
        </label>
        <input
          type="date"
          id="starts_at"
          bind:value={formData.starts_at}
          on:change={setDefaultEndDate}
          required
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
        />
      </div>
      <div>
        <label for="ends_at" class="block text-sm font-medium text-gray-700">
          Date de fin <span class="text-red-500">*</span>
        </label>
        <input
          type="date"
          id="ends_at"
          bind:value={formData.ends_at}
          required
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
        />
      </div>
    </div>
    <p class="text-xs text-gray-500">
      Recommandation: 7-14 jours pour laisser le temps aux copropriétaires de répondre.
    </p>

    <!-- Options -->
    <div class="space-y-3">
      <label class="flex items-center">
        <input
          type="checkbox"
          bind:checked={formData.is_anonymous}
          class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
        />
        <span class="ml-2 text-sm text-gray-700">
          🔒 Vote anonyme (l'identité des votants est masquée)
        </span>
      </label>

      {#if formData.poll_type === PollType.MultipleChoice}
        <label class="flex items-center">
          <input
            type="checkbox"
            bind:checked={formData.allow_multiple_votes}
            class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
          />
          <span class="ml-2 text-sm text-gray-700">
            ☑️ Autoriser la sélection multiple (plusieurs options à la fois)
          </span>
        </label>
      {/if}
    </div>

    <!-- Legal Notice -->
    <div class="p-4 bg-yellow-50 border border-yellow-200 rounded-md">
      <h4 class="text-sm font-medium text-yellow-900 mb-2">
        ⚖️ Cadre légal belge
      </h4>
      <p class="text-xs text-yellow-800">
        <strong>Article 577-8/4 §4 du Code Civil:</strong> Le syndic peut consulter
        les copropriétaires entre les assemblées générales sur toute décision ne
        nécessitant pas de vote formel en AG. Les résultats doivent être documentés
        dans le procès-verbal de la prochaine assemblée générale.
      </p>
    </div>

    <!-- Submit Button -->
    <div class="flex justify-end space-x-3">
      <button
        type="button"
        on:click={() => dispatch("cancel")}
        class="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
      >
        Annuler
      </button>
      <button
        type="submit"
        disabled={loading}
        class="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {#if loading}
          <span class="inline-block animate-spin mr-2">⏳</span>
          Création en cours...
        {:else}
          ✅ Créer le sondage (brouillon)
        {/if}
      </button>
    </div>
  </form>
</div>
