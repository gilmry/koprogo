import { toast } from "../../stores/toast";

export interface ErrorHandlingOptions<T> {
  /** The async action to execute */
  action: () => Promise<T>;
  /** Callback to set loading state (Svelte 4 vars can't be passed by ref) */
  setLoading?: (loading: boolean) => void;
  /** Toast message on success. If undefined, no success toast shown. */
  successMessage?: string;
  /**
   * Titre du message d'erreur. **Prioritaire sur `err.message`.**
   *
   * Le JSDoc disait « Fallback error message if err.message is empty », ce
   * qui décrivait l'inverse du code : `opts.errorMessage || err?.message`
   * fait toujours gagner le libellé de l'appelant. Environ 197 appelants en
   * fournissent un, donc le message du serveur était écrasé partout.
   *
   * On conserve ce comportement — ces libellés sont en français, ceux du
   * serveur souvent en anglais — mais le DÉTAIL du serveur est désormais
   * ajouté en seconde ligne du toast. Voir l'issue #782.
   */
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
/**
 * Le détail à afficher sous le titre d'un toast d'erreur.
 *
 * `apiFetch` lève une `ApiError` qui porte le corps de la réponse. Quand le
 * serveur nomme le champ fautif — « missing field `acp_id` » — c'est cette
 * chaîne qu'on montre, sous le libellé français de l'appelant.
 *
 * Rendu `undefined` si le détail n'est pas une chaîne : les erreurs métier
 * typées (`{code, ...}`) ont leurs gestionnaires dédiés dans `conformity.ts`
 * et `meetingCompletion.ts`, et un objet brut ne dirait rien à personne.
 */
function detailDuServeur(err: any): string | undefined {
  if (typeof err?.details === "string") return err.details;
  if (typeof err?.details?.message === "string") return err.details.message;
  return undefined;
}

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
    // `apiFetch` a déjà émis un toast pour les 4xx et 5xx : en émettre un
    // second, au libellé différent, en affichait deux que la déduplication ne
    // fusionnait pas. On ne parle que si personne n'a parlé.
    if (!err?.body?.__toastEmis) {
      const message = opts.errorMessage || err?.message || "An error occurred";
      toast.error(message, 7000, detailDuServeur(err));
    }
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
    const message = opts.errorMessage || err?.message || "An error occurred";
    opts.setError(message);
    if (!err?.body?.__toastEmis) {
      toast.error(message, 7000, detailDuServeur(err));
    }
  } finally {
    opts.setLoading(false);
  }
}
