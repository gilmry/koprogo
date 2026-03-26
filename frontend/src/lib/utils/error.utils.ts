import { toast } from "../../stores/toast";

export interface ErrorHandlingOptions<T> {
  /** The async action to execute */
  action: () => Promise<T>;
  /** Callback to set loading state (Svelte 4 vars can't be passed by ref) */
  setLoading?: (loading: boolean) => void;
  /** Toast message on success. If undefined, no success toast shown. */
  successMessage?: string;
  /** Fallback error message if err.message is empty */
  errorMessage?: string;
  /** Called with result on success, before returning */
  onSuccess?: (result: T) => void;
}

/**
 * Execute an async action with loading state, error handling, and toast notifications.
 * Replaces the ubiquitous try/catch/toast pattern found across 92+ components.
 *
 * Returns the result on success, or undefined on failure.
 *
 * @example
 * const ticket = await withErrorHandling({
 *   action: () => ticketsApi.assign(id, contractorId),
 *   setLoading: (v) => actionLoading = v,
 *   successMessage: $_("tickets.assigned_successfully"),
 *   errorMessage: $_("tickets.assign_failed"),
 * });
 * if (ticket) dispatch("updated", ticket);
 */
export async function withErrorHandling<T>(
  opts: ErrorHandlingOptions<T>,
): Promise<T | undefined> {
  try {
    opts.setLoading?.(true);
    const result = await opts.action();
    if (opts.successMessage) {
      toast.success(opts.successMessage);
    }
    opts.onSuccess?.(result);
    return result;
  } catch (err: any) {
    const message = err?.message || opts.errorMessage || "An error occurred";
    toast.error(message);
    return undefined;
  } finally {
    opts.setLoading?.(false);
  }
}

/**
 * Execute an async load action with loading and error state management.
 * Designed for onMount data fetching patterns.
 *
 * @example
 * onMount(() => withLoadingState({
 *   action: () => ticketsApi.listByBuilding(buildingId),
 *   setLoading: (v) => loading = v,
 *   setError: (v) => error = v,
 *   onSuccess: (data) => tickets = data,
 * }));
 */
export async function withLoadingState<T>(opts: {
  action: () => Promise<T>;
  setLoading: (loading: boolean) => void;
  setError: (error: string) => void;
  onSuccess: (result: T) => void;
  errorMessage?: string;
}): Promise<void> {
  try {
    opts.setLoading(true);
    opts.setError("");
    const result = await opts.action();
    opts.onSuccess(result);
  } catch (err: any) {
    const message = err?.message || opts.errorMessage || "An error occurred";
    opts.setError(message);
    toast.error(message);
  } finally {
    opts.setLoading(false);
  }
}
