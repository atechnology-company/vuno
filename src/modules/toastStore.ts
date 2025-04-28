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
export function addToast(message: string, type: ToastType = 'info', duration: number = 3000) {
  const id = Date.now().toString();
  const toast = { id, message, type, duration };
  
  toasts.update(items => [...items, toast]);
  
  // Automatically remove toast after duration
  setTimeout(() => {
    removeToast(id);
  }, duration);
  
  return id;
}

export function removeToast(id: string) {
  toasts.update(items => items.filter(t => t.id !== id));
}

export function clearToasts() {
  toasts.set([]);
} 