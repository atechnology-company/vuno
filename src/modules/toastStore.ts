import { writable } from 'svelte/store';

// Types
export type ToastType = 'info' | 'success' | 'error' | 'warning';

export interface ToastItem {
  id: string;
  message: string;
  type: ToastType;
  duration: number;
}

// Create a proper Svelte store for toasts
export const toasts = writable<ToastItem[]>([]);

// Toast management functions
export function addToast(_message: string, _type: ToastType = 'info', _duration: number = 3000) {
  // Toasts disabled: do nothing and ensure store stays empty.
  clearToasts();
  return '';
}

export function removeToast(id: string) {
  toasts.update(items => items.filter(t => t.id !== id));
}

export function clearToasts() {
  toasts.set([]);
} 

// Optional: close all toasts convenience
export function closeAllToasts() {
  toasts.update(items => []);
}

// Attach a single global ESC handler (idempotent) to guarantee toast closure even if component unmounted temporarily
if (typeof window !== 'undefined' && !(window as any).__vunoToastEsc) {
  (window as any).__vunoToastEsc = true;
  window.addEventListener('keydown', (e: KeyboardEvent) => {
    if (e.key === 'Escape') {
      toasts.update(items => items.slice(0, -1)); // pop last
    }
  }, true);
}