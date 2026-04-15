import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { useWorkflowStore } from '../store/useWorkflowStore';
import { usePluginStore } from '../store/usePluginStore';

export function Workflows() {
  const { workflows, fetchWorkflows, deleteWorkflow } = useWorkflowStore();
  const { plugins, fetchPlugins } = usePluginStore();
  const [filterPluginId, setFilterPluginId] = useState<number | null>(null);
  const [confirmDelete, setConfirmDelete] = useState<number | null>(null);

  useEffect(() => {
    fetchWorkflows(filterPluginId ?? undefined);
    fetchPlugins();
  }, [filterPluginId]);

  const handleDelete = async (id: number) => {
    await deleteWorkflow(id);
    setConfirmDelete(null);
    fetchWorkflows(filterPluginId ?? undefined);
  };

  const getPluginName = (pluginId: number) => {
    const plugin = plugins.find((p) => p.id === pluginId);
    return plugin?.name || `插件 #${pluginId}`;
  };

  const statusBadge = (lastRun?: string, runCount?: number) => {
    if (!lastRun && runCount === 0) {
      return <span className="text-gray-400 text-xs">从未运行</span>;
    }
    return (
      <span className="text-gray-500 text-xs">
        运行 {runCount} 次，最后于{' '}
        {lastRun ? new Date(lastRun).toLocaleDateString() : '-'}
      </span>
    );
  };

  return (
    <div>
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">工作流</h1>
      </div>

      <div className="mb-4 flex gap-2">
        <button
          onClick={() => setFilterPluginId(null)}
          className={`px-4 py-2 rounded text-sm ${
            filterPluginId === null
              ? 'bg-blue-500 text-white'
              : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
          }`}
        >
          全部
        </button>
        {plugins.map((plugin) => (
          <button
            key={plugin.id}
            onClick={() => setFilterPluginId(plugin.id)}
            className={`px-4 py-2 rounded text-sm ${
              filterPluginId === plugin.id
                ? 'bg-blue-500 text-white'
                : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
            }`}
          >
            {plugin.name}
          </button>
        ))}
      </div>

      {workflows.length === 0 ? (
        <div className="text-center py-8 text-gray-500">
          {filterPluginId
            ? '该插件下暂无工作流'
            : '暂无工作流，请从插件详情页创建'}
        </div>
      ) : (
        <div className="bg-white rounded-lg shadow overflow-hidden">
          <table className="min-w-full">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  名称
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  所属插件
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  描述
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  运行统计
                </th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  操作
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {workflows.map((workflow) => (
                <tr key={workflow.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4 whitespace-nowrap">
                    <Link
                      to={`/workflows/${workflow.id}/edit`}
                      className="text-blue-600 hover:text-blue-800 font-medium"
                    >
                      {workflow.name}
                    </Link>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {getPluginName(workflow.plugin_id)}
                  </td>
                  <td className="px-6 py-4 text-sm text-gray-500">
                    {workflow.description || '-'}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    {statusBadge(workflow.last_run_at, workflow.run_count)}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-right text-sm">
                    <Link
                      to={`/workflows/${workflow.id}/edit`}
                      className="text-blue-600 hover:text-blue-800 mr-4"
                    >
                      编辑
                    </Link>
                    {confirmDelete === workflow.id ? (
                      <span className="mr-2">
                        <button
                          onClick={() => handleDelete(workflow.id)}
                          className="text-red-600 hover:text-red-800 mr-2"
                        >
                          确认
                        </button>
                        <button
                          onClick={() => setConfirmDelete(null)}
                          className="text-gray-600 hover:text-gray-800"
                        >
                          取消
                        </button>
                      </span>
                    ) : (
                      <button
                        onClick={() => setConfirmDelete(workflow.id)}
                        className="text-red-600 hover:text-red-800"
                      >
                        删除
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
