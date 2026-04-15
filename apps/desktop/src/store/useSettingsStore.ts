import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export interface Settings {
  database_url: string;
  engine_binary_path: string;
  engine_port: number;
  engine_timeout_ms: number;
  debug_enabled: boolean;
  debug_highlight: boolean;
  debug_highlight_duration_ms: number;
  debug_screenshot_on_step: boolean;
  debug_screenshot_dir: string;
  debug_slow_motion_ms: number;
}

interface SettingsStore {
  settings: Settings | null;
  loading: boolean;
  error: string | null;
  engineRunning: boolean;
  fetchSettings: () => Promise<void>;
  saveSettings: (settings: Settings) => Promise<void>;
  testConnection: (url: string) => Promise<boolean>;
  startEngine: (binaryPath: string) => Promise<string>;
  stopEngine: () => Promise<void>;
  executeWorkflow: (script: string) => Promise<string>;
  getTaskStatus: (taskId: string) => Promise<unknown>;
}

export const useSettingsStore = create<SettingsStore>((set) => ({
  settings: null,
  loading: false,
  error: null,
  engineRunning: false,
  fetchSettings: async () => {
    set({ loading: true });
    try {
      const settings = await invoke<Settings>('get_settings');
      set({ settings, loading: false, error: null });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  saveSettings: async (settings) => {
    try {
      await invoke('save_settings', { settings });
    } catch (e) {
      set({ error: String(e) });
    }
  },
  testConnection: async (url) => {
    try {
      return await invoke<boolean>('test_database_connection', { url });
    } catch (e) {
      set({ error: String(e) });
      return false;
    }
  },
  startEngine: async (binaryPath) => {
    try {
      const url = await invoke<string>('start_engine', { binaryPath });
      set({ engineRunning: true });
      return url;
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },
  stopEngine: async () => {
    try {
      await invoke('stop_engine');
      set({ engineRunning: false });
    } catch (e) {
      set({ error: String(e) });
    }
  },
  executeWorkflow: async (script) => {
    try {
      return await invoke<string>('execute_workflow', { script });
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },
  getTaskStatus: async (taskId) => {
    try {
      return await invoke('get_task_status', { taskId });
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },
}));
