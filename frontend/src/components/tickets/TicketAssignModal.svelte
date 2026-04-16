<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { _ } from '../../lib/i18n';
  import { api } from '../../lib/api';
  import { toast } from "../../stores/toast";
  import { withErrorHandling } from "../../lib/utils/error.utils";
  import Modal from "../ui/Modal.svelte";
  import Button from "../ui/Button.svelte";

  export let open = false;
  export let ticketId: string;

  const dispatch = createEventDispatcher();

  interface AssignableUser {
    id: string;
    first_name: string;
    last_name: string;
    role: string;
    profession?: string;
  }

  let assignableUsers: AssignableUser[] = [];
  let selectedUserId = "";
  let loadingUsers = false;
  let submitting = false;

  $: if (open && assignableUsers.length === 0) {
    loadAssignableUsers();
  }

  async function loadAssignableUsers() {
    loadingUsers = true;
    try {
      assignableUsers = await api.get<AssignableUser[]>('/tickets/assignable-users');
    } catch {
      toast.error($_("tickets.assign.loadFailed"));
    } finally {
      loadingUsers = false;
    }
  }

  function getRoleLabel(user: AssignableUser): string {
    if (user.profession) return user.profession;
    switch (user.role) {
      case 'contractor': return $_('roles.contractor');
      case 'syndic': return $_('roles.syndic');
      case 'board_member': return $_('roles.board_member');
      default: return user.role;
    }
  }

  async function handleSubmit() {
    if (!selectedUserId) {
      toast.error($_("tickets.assign.selectAssignee"));
      return;
    }

    await withErrorHandling({
      action: async () => {
        dispatch("assigned", { contractorId: selectedUserId });
      },
      setLoading: (v) => submitting = v,
      errorMessage: $_("tickets.assign_failed"),
    });
    handleClose();
  }

  function handleClose() {
    open = false;
    selectedUserId = "";
    dispatch("close");
  }
</script>

<Modal isOpen={open} on:close={handleClose} title={$_("tickets.assign_to_contractor")}>
  <form on:submit|preventDefault={handleSubmit} data-testid="ticket-assign-form">
    <div class="space-y-4">
      <p class="text-sm text-gray-600" data-testid="ticket-assign-description">
        {$_("tickets.assign_description")}
      </p>

      {#if loadingUsers}
        <p class="text-sm text-gray-500">{$_("common.loading")}</p>
      {:else if assignableUsers.length === 0}
        <p class="text-sm text-orange-600">{$_("tickets.assign.noAssignableUsers")}</p>
      {:else}
        <div>
          <label for="assignee-select" class="block text-sm font-medium text-gray-700 mb-1">
            {$_("tickets.assign.selectAssignee")} *
          </label>
          <select
            id="assignee-select"
            bind:value={selectedUserId}
            required
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            data-testid="ticket-assignee-select"
          >
            <option value="" disabled>{$_("tickets.assign.choosePerson")}</option>
            {#each assignableUsers as user (user.id)}
              <option value={user.id}>
                {user.first_name} {user.last_name} — {getRoleLabel(user)}
              </option>
            {/each}
          </select>
        </div>
      {/if}
    </div>

    <div class="mt-6 flex justify-end space-x-3">
      <Button type="button" variant="outline" on:click={handleClose} data-testid="ticket-assign-cancel-btn">
        {$_("common.cancel")}
      </Button>
      <Button
        type="submit"
        loading={submitting}
        disabled={!selectedUserId || loadingUsers}
        data-testid="ticket-assign-submit-btn"
      >
        {$_("tickets.assign_ticket")}
      </Button>
    </div>
  </form>
</Modal>
