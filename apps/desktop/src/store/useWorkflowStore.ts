import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export interface Workflow {
  id: number;
  plugin_id: number;
  name: string;
  description?: string;
  script: string;
  created_at: string;
  updated_at: string;
  last_run_at?: string;
  run_count: number;
}

interface WorkflowStore {
  workflows: Workflow[];
  currentWorkflow: Workflow | null;
  loading: boolean;
  error: string | null;
  fetchWorkflows: (pluginId?: number) => Promise<void>;
  fetchWorkflow: (id: number) => Promise<void>;
  saveWorkflow: (id: number, script: string) => Promise<void>;
  deleteWorkflow: (id: number) => Promise<void>;
  createWorkflow: (pluginId: number, name: string) => Promise<Workflow | null>;
}

export const useWorkflowStore = create<WorkflowStore>((set) => ({
  workflows: [],
  currentWorkflow: null,
  loading: false,
  error: null,
  fetchWorkflows: async (pluginId) => {
    set({ loading: true });
    try {
      const workflows = await invoke<Workflow[]>('list_workflows', {
        pluginId: pluginId ?? null,
      });
      set({ workflows, loading: false, error: null });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  fetchWorkflow: async (id) => {
    set({ loading: true });
    try {
      const workflow = await invoke<Workflow>('get_workflow', { id });
      set({ currentWorkflow: workflow, loading: false, error: null });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  saveWorkflow: async (id, script) => {
    try {
      await invoke('save_workflow', { id, script });
    } catch (e) {
      set({ error: String(e) });
    }
  },
  deleteWorkflow: async (id) => {
    try {
      await invoke('delete_workflow', { id });
    } catch (e) {
      set({ error: String(e) });
    }
  },
  createWorkflow: async (pluginId, name) => {
    try {
      const workflow = await invoke<Workflow>('create_workflow', {
        pluginId,
        name,
      });
      return workflow;
    } catch (e) {
      set({ error: String(e) });
      return null;
    }
  },
}));
