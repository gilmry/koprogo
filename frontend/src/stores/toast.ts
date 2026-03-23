import { writable } from "svelte/store";

export type ToastType = "success" | "error" | "info" | "warning";

export interface ToastMessage {
  id: number;
  message: string;
  type: ToastType;
  duration?: number;
}

function createToastStore() {
  const { subscribe, update } = writable<ToastMessage[]>([]);
  let nextId = 1;

  return {
    subscribe,
    show: (message: string, type: ToastType = "info", duration = 5000) => {
      const id = nextId++;
      const toast: ToastMessage = { id, message, type, duration };

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
    error: function (message: string, duration = 7000) {
      return this.show(message, "error", duration);
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
