import { writable } from "svelte/store";

export type ToastType = "success" | "error" | "info" | "warning";

export interface ToastMessage {
  id: number;
  message: string;
  type: ToastType;
  duration?: number;
  /**
   * Seconde ligne, plus discrète : le détail servi par le serveur.
   *
   * Le backend nomme le champ fautif de ses réponses 400 (« missing field
   * `acp_id` »), et cette information n'atteignait pas l'écran. Le titre
   * reste le libellé de l'appelant, en français ; le détail s'ajoute
   * dessous, sans le remplacer. Voir l'issue #782.
   */
  details?: string;
}

function createToastStore() {
  const { subscribe, update } = writable<ToastMessage[]>([]);
  let nextId = 1;

  return {
    subscribe,
    show: (
      message: string,
      type: ToastType = "info",
      duration = 5000,
      details?: string,
    ) => {
      // STORY-P7-402: dedupe identical toasts (same message + type) to avoid
      // cascades when several parallel API calls fail with the same error.
      //
      // Le DÉTAIL entre dans la clé de déduplication : deux erreurs de
      // validation sur des champs différents portent souvent le même titre
      // générique (« Invalid request body ») et seraient sinon fusionnées en
      // un seul message, ce qui masquerait le second champ fautif.
      let reusedId: number | null = null;
      update((toasts) => {
        const existing = toasts.find(
          (t) =>
            t.message === message && t.type === type && t.details === details,
        );
        if (existing) {
          reusedId = existing.id;
        }
        return toasts;
      });
      if (reusedId !== null) return reusedId;

      const id = nextId++;
      const toast: ToastMessage = { id, message, type, duration, details };

      update((toasts) => [...toasts, toast]);

      if (duration > 0) {
        setTimeout(() => {
          update((toasts) => toasts.filter((t) => t.id !== id));
        }, duration);
      }

      return id;
    },
    success: function (message: string, duration = 5000) {
      return this.show(message, "success", duration);
    },
    error: function (message: string, duration = 7000, details?: string) {
      return this.show(message, "error", duration, details);
    },
    info: function (message: string, duration = 5000) {
      return this.show(message, "info", duration);
    },
    warning: function (message: string, duration = 6000) {
      return this.show(message, "warning", duration);
    },
    dismiss: (id: number) => {
      update((toasts) => toasts.filter((t) => t.id !== id));
    },
    clear: () => {
      update(() => []);
    },
  };
}

export const toast = createToastStore();
