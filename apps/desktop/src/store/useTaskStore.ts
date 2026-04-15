import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export interface Task {
  id: number;
  workflow_id: number;
  status: 'running' | 'completed' | 'failed' | 'cancelled';
  started_at: string;
  finished_at?: string;
  error_message?: string;
}

export interface ExecutionEvent {
  timestamp: string;
  event_type: string;
  data: Record<string, unknown>;
}

interface TaskStore {
  tasks: Task[];
  currentTask: Task | null;
  logs: ExecutionEvent[];
  loading: boolean;
  error: string | null;
  fetchTasks: () => Promise<void>;
  fetchTaskLogs: (taskId: number) => Promise<void>;
  streamTaskEvents: (taskId: number) => void;
}

export const useTaskStore = create<TaskStore>((set) => ({
  tasks: [],
  currentTask: null,
  logs: [],
  loading: false,
  error: null,
  fetchTasks: async () => {
    set({ loading: true });
    try {
      const tasks = await invoke<Task[]>('list_tasks');
      set({ tasks, loading: false, error: null });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  fetchTaskLogs: async (taskId) => {
    try {
      const logs = await invoke<ExecutionEvent[]>('get_task_logs', { taskId });
      set({ logs });
    } catch (e) {
      set({ error: String(e) });
    }
  },
  streamTaskEvents: (taskId) => {
    // TODO: Implement SSE streaming
    console.log('Streaming events for task:', taskId);
  },
}));
