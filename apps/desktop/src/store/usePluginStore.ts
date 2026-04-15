import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export interface Plugin {
  id: number;
  name: string;
  version: string;
  description?: string;
  author?: string;
  zip_path: string;
  installed_at: string;
  is_enabled: boolean;
}

interface PluginStore {
  plugins: Plugin[];
  loading: boolean;
  error: string | null;
  fetchPlugins: () => Promise<void>;
  importPlugin: (zipPath: string) => Promise<void>;
  exportPlugin: (id: number, outputPath: string) => Promise<void>;
  deletePlugin: (id: number) => Promise<void>;
}

export const usePluginStore = create<PluginStore>((set) => ({
  plugins: [],
  loading: false,
  error: null,
  fetchPlugins: async () => {
    set({ loading: true });
    try {
      const plugins = await invoke<Plugin[]>('list_plugins');
      set({ plugins, loading: false, error: null });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  importPlugin: async (zipPath) => {
    try {
      await invoke('import_plugin', { zipPath });
      await invoke('list_plugins');
    } catch (e) {
      set({ error: String(e) });
    }
  },
  exportPlugin: async (id, outputPath) => {
    try {
      await invoke('export_plugin', { id, outputPath });
    } catch (e) {
      set({ error: String(e) });
    }
  },
  deletePlugin: async (id) => {
    try {
      await invoke('delete_plugin', { id });
      await invoke('list_plugins');
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));
