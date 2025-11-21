<script lang="ts">
  import { noticesApi, type CreateNoticeDto, NoticeType, NoticeVisibility } from "../../lib/api/notices";
  import { addToast } from "../../stores/toast";

  export let isOpen = false;
  export let buildingId: string;
  export let authorId: string;
  export let onClose: () => void;
  export let onSuccess: () => void;

  let submitting = false;
  let formData: CreateNoticeDto = {
    building_id: buildingId,
    author_id: authorId,
    notice_type: NoticeType.Announcement,
    category: "",
    title: "",
    content: "",
    visibility: NoticeVisibility.BuildingOnly,
    contact_info: "",
  };

  let expiresEnabled = false;
  let expiresDate = "";

  async function handleSubmit() {
    if (!formData.title.trim()) {
      addToast({ message: "Please enter a title", type: "error" });
      return;
    }
    if (!formData.content.trim()) {
      addToast({ message: "Please enter content", type: "error" });
      return;
    }

    try {
      submitting = true;
      const payload = { ...formData };

      if (expiresEnabled && expiresDate) {
        payload.expires_at = new Date(expiresDate).toISOString();
      }

      await noticesApi.create(payload);
      addToast({ message: "Notice created successfully", type: "success" });
      resetForm();
      onSuccess();
      onClose();
    } catch (err: any) {
      addToast({
        message: err.message || "Failed to create notice",
        type: "error",
      });
    } finally {
      submitting = false;
    }
  }

  function resetForm() {
    formData = {
      building_id: buildingId,
      author_id: authorId,
      notice_type: NoticeType.Announcement,
      category: "",
      title: "",
      content: "",
      visibility: NoticeVisibility.BuildingOnly,
      contact_info: "",
    };
    expiresEnabled = false;
    expiresDate = "";
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
        <h2 class="text-2xl font-bold text-gray-900 mb-6">Create Notice</h2>

        <form on:submit|preventDefault={handleSubmit} class="space-y-4">
          <!-- Notice Type -->
          <div>
            <label for="notice_type" class="block text-sm font-medium text-gray-700 mb-1">
              Type
            </label>
            <select
              id="notice_type"
              bind:value={formData.notice_type}
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
            >
              {#each Object.values(NoticeType) as type}
                <option value={type}>{type}</option>
              {/each}
            </select>
          </div>

          <!-- Category -->
          <div>
            <label for="category" class="block text-sm font-medium text-gray-700 mb-1">
              Category (optional)
            </label>
            <input
              type="text"
              id="category"
              bind:value={formData.category}
              placeholder="e.g., Electronics, Furniture, General"
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
            />
          </div>

          <!-- Title -->
          <div>
            <label for="title" class="block text-sm font-medium text-gray-700 mb-1">
              Title <span class="text-red-500">*</span>
            </label>
            <input
              type="text"
              id="title"
              bind:value={formData.title}
              required
              placeholder="Enter a clear title..."
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
            />
          </div>

          <!-- Content -->
          <div>
            <label for="content" class="block text-sm font-medium text-gray-700 mb-1">
              Content <span class="text-red-500">*</span>
            </label>
            <textarea
              id="content"
              bind:value={formData.content}
              required
              rows="6"
              placeholder="Describe your notice in detail..."
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
            />
          </div>

          <!-- Visibility -->
          <div>
            <label for="visibility" class="block text-sm font-medium text-gray-700 mb-1">
              Visibility
            </label>
            <select
              id="visibility"
              bind:value={formData.visibility}
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
            >
              <option value={NoticeVisibility.BuildingOnly}>🏢 Building Only</option>
              <option value={NoticeVisibility.OwnersOnly}>🔒 Owners Only</option>
              <option value={NoticeVisibility.Public}>🌍 Public</option>
            </select>
          </div>

          <!-- Contact Info -->
          <div>
            <label for="contact_info" class="block text-sm font-medium text-gray-700 mb-1">
              Contact Information (optional)
            </label>
            <input
              type="text"
              id="contact_info"
              bind:value={formData.contact_info}
              placeholder="Phone, email, or other contact details"
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
            />
          </div>

          <!-- Expiration Date -->
          <div>
            <div class="flex items-center mb-2">
              <input
                type="checkbox"
                id="expires_enabled"
                bind:checked={expiresEnabled}
                class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
              />
              <label for="expires_enabled" class="ml-2 text-sm font-medium text-gray-700">
                Set expiration date
              </label>
            </div>
            {#if expiresEnabled}
              <input
                type="datetime-local"
                id="expires_at"
                bind:value={expiresDate}
                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
              />
            {/if}
          </div>

          <!-- Actions -->
          <div class="flex gap-3 pt-4">
            <button
              type="submit"
              disabled={submitting}
              class="flex-1 bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {submitting ? "Creating..." : "Create Notice"}
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
