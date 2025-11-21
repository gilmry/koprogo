<script lang="ts">
  import { LoanStatus } from "../../lib/api/sharing";

  export let status: LoanStatus;

  const statusConfig: Record<LoanStatus, { label: string; class: string }> = {
    [LoanStatus.Requested]: {
      label: "Requested",
      class: "bg-yellow-100 text-yellow-800",
    },
    [LoanStatus.Approved]: {
      label: "Approved",
      class: "bg-green-100 text-green-800",
    },
    [LoanStatus.Active]: {
      label: "Active",
      class: "bg-blue-100 text-blue-800",
    },
    [LoanStatus.Returned]: {
      label: "Returned",
      class: "bg-gray-100 text-gray-800",
    },
    [LoanStatus.Overdue]: {
      label: "Overdue",
      class: "bg-red-100 text-red-800",
    },
    [LoanStatus.Cancelled]: {
      label: "Cancelled",
      class: "bg-gray-100 text-gray-600",
    },
    [LoanStatus.Disputed]: {
      label: "Disputed",
      class: "bg-red-100 text-red-900",
    },
  };

  $: config = statusConfig[status];
</script>

<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.class}">
  {config.label}
</span>
