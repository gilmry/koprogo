<script lang="ts">
  import { _ } from "svelte-i18n";
  import { noticesApi, type CreateNoticeDto, NoticeType, NoticeCategory } from "../../lib/api/notices";
  import { toast } from "../../stores/toast";

  export let isOpen = false;
  export let buildingId: string;
  export let authorId: string = "";
  export let onClose: () => void;
  export let onSuccess: () => void;

  let submitting = false;
  let formData: CreateNoticeDto = {
    building_id: buildingId,
    notice_type: NoticeType.Announcement,
    category: NoticeCategory.General,
    title: "",
    content: "",
    contact_info: "",
  };

  let expiresEnabled = false;
  let expiresDate = "";
  let eventDate = "";
  let eventLocation = "";

  async function handleSubmit() {
    if (!formData.title.trim()) {
      toast.error($_("notices.enter_title"));
      return;
    }
    if (!formData.content.trim()) {
      toast.error($_("notices.enter_content"));
      return;
    }

    try {
      submitting = true;
      const payload: CreateNoticeDto = { ...formData };

      if (expiresEnabled && expiresDate) {
        payload.expires_at = new Date(expiresDate).toISOString();
      }
      if (formData.notice_type === NoticeType.Event) {
        if (eventDate) payload.event_date = new Date(eventDate).toISOString();
        if (eventLocation) payload.event_location = eventLocation;
      }

      await noticesApi.create(payload);
      toast.success($_("notices.created_successfully"));
      resetForm();
      onSuccess();
      onClose();
    } catch (err: any) {
      toast.error(err.message || $_("notices.create_failed"));
    } finally {
      submitting = false;
    }
  }

  function resetForm() {
    formData = {
      building_id: buildingId,
      notice_type: NoticeType.Announcement,
      category: NoticeCategory.General,
      title: "",
      content: "",
      contact_info: "",
    };
    expiresEnabled = false;
    expiresDate = "";
    eventDate = "";
    eventLocation = "";
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
        <h2 class="text-2xl font-bold text-gray-900 mb-6">{$_("notices.create_notice")}</h2>

        <form on:submit|preventDefault={handleSubmit} class="space-y-4">
          <!-- Notice Type -->
          <div>
            <label for="notice_type" class="block text-sm font-medium text-gray-700 mb-1">
              {$_("notices.type")}
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
              {$_("notices.category")}
            </label>
            <select
              id="category"
              bind:value={formData.category}
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
            >
              {#each Object.values(NoticeCategory) as cat}
                <option value={cat}>{cat}</option>
              {/each}
            </select>
          </div>

          <!-- Title -->
          <div>
            <label for="title" class="block text-sm font-medium text-gray-700 mb-1">
              {$_("notices.title")} <span class="text-red-500">*</span>
            </label>
            <input
              type="text"
              id="title"
              bind:value={formData.title}
              required
              minlength="5"
              maxlength="255"
              placeholder={$_("notices.title_placeholder")}
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
            />
          </div>

          <!-- Content -->
          <div>
            <label for="content" class="block text-sm font-medium text-gray-700 mb-1">
              {$_("notices.content")} <span class="text-red-500">*</span>
            </label>
            <textarea
              id="content"
              bind:value={formData.content}
              required
              rows="6"
              placeholder={$_("notices.content_placeholder")}
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
            />
          </div>

          <!-- Event Fields (shown only for Event type) -->
          {#if formData.notice_type === NoticeType.Event}
            <div>
              <label for="event_date" class="block text-sm font-medium text-gray-700 mb-1">
                {$_("notices.event_date")}
              </label>
              <input
                type="datetime-local"
                id="event_date"
                bind:value={eventDate}
                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
              />
            </div>
            <div>
              <label for="event_location" class="block text-sm font-medium text-gray-700 mb-1">
                {$_("notices.event_location")}
              </label>
              <input
                type="text"
                id="event_location"
                bind:value={eventLocation}
                placeholder={$_("notices.event_location_placeholder")}
                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
              />
            </div>
          {/if}

          <!-- Contact Info -->
          <div>
            <label for="contact_info" class="block text-sm font-medium text-gray-700 mb-1">
              {$_("notices.contact_info")}
            </label>
            <input
              type="text"
              id="contact_info"
              bind:value={formData.contact_info}
              placeholder={$_("notices.contact_info_placeholder")}
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
                {$_("notices.set_expiration_date")}
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
              {submitting ? $_("notices.creating") : $_("notices.create_notice")}
            </button>
            <button
              type="button"
              on:click={handleCancel}
              disabled={submitting}
              class="flex-1 bg-gray-200 text-gray-700 px-4 py-2 rounded-md hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 disabled:opacity-50"
            >
              {$_("common.cancel")}
            </button>
          </div>
        </form>
      </div>
    </div>
  </div>
{/if}
