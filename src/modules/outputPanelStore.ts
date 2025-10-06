import { writable } from 'svelte/store';

export type OutputKind = 'ai' | 'shell' | 'file' | 'system' | 'info' | 'error';

export interface OutputItem {
  id: string;
  title?: string;
  kind: OutputKind;
  content: string;
  timestamp: number;
}

interface PanelState {
  visible: boolean;
  items: OutputItem[];
}

const initial: PanelState = { visible: false, items: [] };

export const outputPanel = writable<PanelState>(initial);

export function addOutput(content: string, kind: OutputKind = 'info', title?: string) {
  const id = `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
  const item: OutputItem = { id, content, kind, title, timestamp: Date.now() };
  outputPanel.update(s => ({ visible: true, items: [item, ...s.items].slice(0, 50) }));
  return id;
}

export function clearOutput(id?: string) {
  if (!id) {
    outputPanel.update(s => ({ ...s, items: [] }));
  } else {
    outputPanel.update(s => ({ ...s, items: s.items.filter(i => i.id !== id) }));
  }
}

export function hidePanel() {
  outputPanel.update(s => ({ ...s, visible: false }));
}

export function showPanel() {
  outputPanel.update(s => ({ ...s, visible: true }));
}
