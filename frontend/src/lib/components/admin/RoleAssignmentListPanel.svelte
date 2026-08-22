<script lang="ts">
  // Story B1 (Phase B FE) — Panel orchestrant Form + List + refresh.
  //
  // Découplé du <RoleAssignmentForm> (modal CRUD) et <RoleAssignmentList>
  // (table). Ce panel sert de "scenario glue" pour la page Astro :
  //   - clic CTA "Nouvelle assignation" → ouvre le modal Form.
  //   - submit OK Form → ferme + bump `refreshTrigger` → List re-fetch.
  //
  // Volontairement minimal : pas de testing 4-cat ici (les sous-composants
  // sont testés). Pattern Svelte 5 runes only.

  import RoleAssignmentForm from "./RoleAssignmentForm.svelte";
  import RoleAssignmentList from "./RoleAssignmentList.svelte";

  let isFormOpen = $state(false);
  let refreshTrigger = $state(0);

  function openForm() {
    isFormOpen = true;
  }

  function closeForm() {
    isFormOpen = false;
  }

  function onAssignmentCreated() {
    isFormOpen = false;
    refreshTrigger += 1;
  }
</script>

<div class="space-y-4">
  <div class="flex justify-end">
    <button
      type="button"
      onclick={openForm}
      data-testid="role-assignment-new-button"
      class="min-h-[44px] inline-flex items-center px-4 py-2 rounded-lg bg-primary-600 text-white font-medium hover:bg-primary-700 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary-500"
    >
      <svg
        class="w-5 h-5 mr-2"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
        aria-hidden="true"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M12 4v16m8-8H4"
        />
      </svg>
      Nouvelle assignation
    </button>
  </div>

  <RoleAssignmentList {refreshTrigger} />

  <RoleAssignmentForm
    isOpen={isFormOpen}
    onclose={closeForm}
    onsuccess={onAssignmentCreated}
  />
</div>
