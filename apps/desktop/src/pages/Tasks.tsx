import { useEffect, useState } from 'react';
import { useTaskStore } from '../store/useTaskStore';

export function Tasks() {
  const { tasks, fetchTasks, fetchTaskLogs, logs } =
    useTaskStore();
  const [selectedTask, setSelectedTask] = useState<number | null>(null);
  const [expandedTask, setExpandedTask] = useState<number | null>(null);

  useEffect(() => {
    fetchTasks();
    // Refresh every 5 seconds
    const interval = setInterval(fetchTasks, 5000);
    return () => clearInterval(interval);
  }, []);

  const handleSelectTask = async (taskId: number) => {
    setSelectedTask(taskId);
    await fetchTaskLogs(taskId);
  };

  const handleExpandTask = async (taskId: number) => {
    if (expandedTask === taskId) {
      setExpandedTask(null);
    } else {
      setExpandedTask(taskId);
      await fetchTaskLogs(taskId);
    }
  };

  const statusBadge = (status: string) => {
    const styles = {
      running: 'bg-blue-100 text-blue-800',
      completed: 'bg-green-100 text-green-800',
      failed: 'bg-red-100 text-red-800',
      cancelled: 'bg-gray-100 text-gray-800',
    };
    const labels = {
      running: '运行中',
      completed: '已完成',
      failed: '失败',
      cancelled: '已取消',
    };
    return (
      <span
        className={`px-2 py-1 rounded text-xs ${styles[status as keyof typeof styles] || styles.cancelled}`}
      >
        {labels[status as keyof typeof labels] || status}
      </span>
    );
  };

  const formatDuration = (start: string, end?: string) => {
    const startDate = new Date(start);
    const endDate = end ? new Date(end) : new Date();
    const diff = Math.floor((endDate.getTime() - startDate.getTime()) / 1000);
    if (diff < 60) return `${diff}秒`;
    if (diff < 3600) return `${Math.floor(diff / 60)}分${diff % 60}秒`;
    return `${Math.floor(diff / 3600)}时${Math.floor((diff % 3600) / 60)}分`;
  };

  const getEventIcon = (type: string) => {
    const icons: Record<string, string> = {
      highlight: '🎯',
      screenshot: '📸',
      action: '👉',
      heartbeat: '💓',
      error: '❌',
    };
    return icons[type] || '📝';
  };

  return (
    <div>
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">任务监控</h1>
        <button
          onClick={() => fetchTasks()}
          className="px-4 py-2 bg-gray-200 text-gray-700 rounded hover:bg-gray-300"
        >
          刷新
        </button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="bg-white rounded-lg shadow overflow-hidden">
          <div className="px-4 py-3 border-b bg-gray-50">
            <h2 className="font-medium">任务列表</h2>
          </div>
          <div className="divide-y max-h-[600px] overflow-y-auto">
            {tasks.length === 0 ? (
              <div className="p-8 text-center text-gray-500">暂无任务</div>
            ) : (
              tasks.map((task) => (
                <div
                  key={task.id}
                  className={`p-4 hover:bg-gray-50 cursor-pointer ${
                    selectedTask === task.id ? 'bg-blue-50' : ''
                  }`}
                  onClick={() => handleSelectTask(task.id)}
                >
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="font-medium">任务 #{task.id}</div>
                      <div className="text-sm text-gray-500">
                        工作流 #{task.workflow_id}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      {statusBadge(task.status)}
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleExpandTask(task.id);
                        }}
                        className="px-2 py-1 text-xs bg-gray-100 rounded hover:bg-gray-200"
                      >
                        {expandedTask === task.id ? '收起' : '展开'}
                      </button>
                    </div>
                  </div>

                  {expandedTask === task.id && (
                    <div className="mt-4 border-t pt-4">
                      <div className="text-sm text-gray-500 mb-2">
                        开始时间: {new Date(task.started_at).toLocaleString()}
                      </div>
                      {task.finished_at && (
                        <div className="text-sm text-gray-500 mb-2">
                          结束时间: {new Date(task.finished_at).toLocaleString()}
                        </div>
                      )}
                      <div className="text-sm text-gray-500 mb-2">
                        持续时间: {formatDuration(task.started_at, task.finished_at)}
                      </div>
                      {task.error_message && (
                        <div className="text-sm text-red-600 mb-2">
                          错误: {task.error_message}
                        </div>
                      )}
                    </div>
                  )}
                </div>
              ))
            )}
          </div>
        </div>

        <div className="bg-white rounded-lg shadow overflow-hidden">
          <div className="px-4 py-3 border-b bg-gray-50">
            <h2 className="font-medium">执行日志</h2>
          </div>
          <div className="p-4 max-h-[600px] overflow-y-auto">
            {!selectedTask ? (
              <div className="text-center text-gray-500 py-8">
                选择一个任务查看日志
              </div>
            ) : logs.length === 0 ? (
              <div className="text-center text-gray-500 py-8">暂无日志</div>
            ) : (
              <div className="space-y-1 font-mono text-sm">
                {logs.map((log, idx) => (
                  <div key={idx} className="flex gap-2 py-1">
                    <span className="text-gray-400 whitespace-nowrap">
                      {new Date(log.timestamp).toLocaleTimeString()}
                    </span>
                    <span>{getEventIcon(log.event_type)}</span>
                    <span className="text-gray-600">
                      {JSON.stringify(log.data)}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
