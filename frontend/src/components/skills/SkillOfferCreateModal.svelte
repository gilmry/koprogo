<script lang="ts">
  import {
    skillsApi,
    type CreateSkillOfferDto,
    SkillCategory,
    ExpertiseLevel,
  } from "../../lib/api/skills";
  import { toast } from "../../stores/toast";

  export let isOpen = false;
  export let buildingId: string;
  export let onClose: () => void;
  export let onSuccess: () => void;

  let submitting = false;
  let formData: CreateSkillOfferDto = {
    building_id: buildingId,
    skill_category: SkillCategory.Other,
    skill_name: "",
    description: "",
    expertise_level: ExpertiseLevel.Intermediate,
    is_available_for_help: true,
  };

  let certificationInput = "";

  async function handleSubmit() {
    if (!formData.skill_name.trim()) {
      toast.error("Please enter skill name");
      return;
    }
    if (!formData.description.trim()) {
      toast.error("Please enter description");
      return;
    }

    try {
      submitting = true;
      const payload = { ...formData };
      if (certificationInput.trim()) {
        payload.certifications = certificationInput.trim();
      }

      await skillsApi.createOffer(payload);
      toast.success("Skill offer created successfully");
      resetForm();
      onSuccess();
      onClose();
    } catch (err: any) {
      toast.error(err.message || "Failed to create skill offer");
    } finally {
      submitting = false;
    }
  }

  function resetForm() {
    formData = {
      building_id: buildingId,
      skill_category: SkillCategory.Other,
      skill_name: "",
      description: "",
      expertise_level: ExpertiseLevel.Intermediate,
      is_available_for_help: true,
    };
    certificationInput = "";
  }

  function handleCancel() {
    resetForm();
    onClose();
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center z-50 p-4">
    <div class="bg-white rounded-lg shadow-xl max-w-2xl w-full max-h-[90vh] overflow-y-auto">
      <div class="p-6">
        <h2 class="text-2xl font-bold text-gray-900 mb-6">Offer Your Skill</h2>

        <form on:submit|preventDefault={handleSubmit} class="space-y-4">
          <!-- Category -->
          <div>
            <label for="category" class="block text-sm font-medium text-gray-700 mb-1">
              Category <span class="text-red-500">*</span>
            </label>
            <select
              id="category"
              bind:value={formData.skill_category}
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
            >
              {#each Object.values(SkillCategory) as category}
                <option value={category}>{category}</option>
              {/each}
            </select>
          </div>

          <!-- Skill Name -->
          <div>
            <label for="skill_name" class="block text-sm font-medium text-gray-700 mb-1">
              Skill Name <span class="text-red-500">*</span>
            </label>
            <input
              type="text"
              id="skill_name"
              bind:value={formData.skill_name}
              required
              minlength="3"
              maxlength="100"
              placeholder="e.g., Plumbing Repairs, Piano Lessons"
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
            />
          </div>

          <!-- Description -->
          <div>
            <label for="description" class="block text-sm font-medium text-gray-700 mb-1">
              Description <span class="text-red-500">*</span>
            </label>
            <textarea
              id="description"
              bind:value={formData.description}
              required
              rows="4"
              maxlength="1000"
              placeholder="Describe what you can offer..."
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
            ></textarea>
          </div>

          <!-- Expertise Level -->
          <div>
            <label for="expertise" class="block text-sm font-medium text-gray-700 mb-1">
              Expertise Level <span class="text-red-500">*</span>
            </label>
            <select
              id="expertise"
              bind:value={formData.expertise_level}
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
            >
              {#each Object.values(ExpertiseLevel) as level}
                <option value={level}>{level}</option>
              {/each}
            </select>
          </div>

          <!-- Hourly Rate (Credits) -->
          <div>
            <label for="hourly_rate" class="block text-sm font-medium text-gray-700 mb-1">
              Hourly Rate (SEL Credits) - Optional
            </label>
            <input
              type="number"
              id="hourly_rate"
              bind:value={formData.hourly_rate_credits}
              min="0"
              max="100"
              placeholder="Leave empty for free"
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
            />
            <p class="text-xs text-gray-500 mt-1">1 hour = 1 credit (Belgian SEL system). 0 or empty = free/volunteer.</p>
          </div>

          <!-- Years Experience -->
          <div>
            <label for="years_exp" class="block text-sm font-medium text-gray-700 mb-1">
              Years of Experience (optional)
            </label>
            <input
              type="number"
              id="years_exp"
              bind:value={formData.years_of_experience}
              min="0"
              max="50"
              placeholder="e.g., 5"
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
            />
          </div>

          <!-- Certifications -->
          <div>
            <label for="certifications" class="block text-sm font-medium text-gray-700 mb-1">
              Certifications (optional)
            </label>
            <input
              type="text"
              id="certifications"
              bind:value={certificationInput}
              placeholder="e.g., Certified Plumber, First Aid Certificate"
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
            />
          </div>

          <!-- Actions -->
          <div class="flex gap-3 pt-4">
            <button
              type="submit"
              disabled={submitting}
              class="flex-1 bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {submitting ? "Creating..." : "Create Offer"}
            </button>
            <button
              type="button"
              on:click={handleCancel}
              disabled={submitting}
              class="flex-1 bg-gray-200 text-gray-700 px-4 py-2 rounded-md hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 disabled:opacity-50"
            >
              Cancel
            </button>
          </div>
        </form>
      </div>
    </div>
  </div>
{/if}
