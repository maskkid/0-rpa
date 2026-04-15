import { useEffect, useState } from 'react';
import { useParams, Link, useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import type { Plugin } from '../store/usePluginStore';
import { useWorkflowStore } from '../store/useWorkflowStore';
import type { Workflow } from '../store/useWorkflowStore';

export function PluginDetail() {
  const { id } = useParams<{ id: string }>();
  const [plugin, setPlugin] = useState<Plugin | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'workflows' | 'info'>('workflows');
  const { workflows, fetchWorkflows } = useWorkflowStore();

  useEffect(() => {
    const loadPlugin = async () => {
      try {
        // For now, we don't have get_plugin command, so we list and find
        const plugins = await invoke<Plugin[]>('list_plugins');
        const found = plugins.find((p) => p.id === Number(id));
        setPlugin(found || null);
        if (found) {
          fetchWorkflows(found.id);
        }
      } catch (e) {
        setError(String(e));
      } finally {
        setLoading(false);
      }
    };
    loadPlugin();
  }, [id]);

  const handleExport = async () => {
    if (!plugin) return;
    const path = await save({
      defaultPath: `${plugin.name}.zip`,
      filters: [{ name: 'ZIP Archives', extensions: ['zip'] }],
    });
    if (path) {
      await invoke('export_plugin', { id: plugin.id, outputPath: path });
    }
  };

  if (loading) {
    return <div className="text-center py-8 text-gray-500">加载中...</div>;
  }

  if (!plugin) {
    return (
      <div className="text-center py-8 text-gray-500">
        插件不存在{' '}
        <Link to="/plugins" className="text-blue-600 hover:text-blue-800">
          返回列表
        </Link>
      </div>
    );
  }

  return (
    <div>
      <div className="flex items-center gap-4 mb-6">
        <Link
          to="/plugins"
          className="text-gray-500 hover:text-gray-700"
        >
          ← 返回
        </Link>
        <h1 className="text-2xl font-bold flex-1">{plugin.name}</h1>
        <button
          onClick={handleExport}
          className="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600"
        >
          导出
        </button>
      </div>

      {error && (
        <div className="mb-4 p-4 bg-red-100 border border-red-400 text-red-700 rounded">
          {error}
        </div>
      )}

      <div className="bg-white rounded-lg shadow mb-6">
        <div className="border-b">
          <nav className="flex">
            <button
              onClick={() => setActiveTab('workflows')}
              className={`px-6 py-3 text-sm font-medium ${
                activeTab === 'workflows'
                  ? 'border-b-2 border-blue-500 text-blue-600'
                  : 'text-gray-500 hover:text-gray-700'
              }`}
            >
              工作流列表
            </button>
            <button
              onClick={() => setActiveTab('info')}
              className={`px-6 py-3 text-sm font-medium ${
                activeTab === 'info'
                  ? 'border-b-2 border-blue-500 text-blue-600'
                  : 'text-gray-500 hover:text-gray-700'
              }`}
            >
              插件信息
            </button>
          </nav>
        </div>

        <div className="p-6">
          {activeTab === 'workflows' ? (
            <WorkflowsTab workflows={workflows} pluginId={plugin.id} />
          ) : (
            <InfoTab plugin={plugin} />
          )}
        </div>
      </div>
    </div>
  );
}

function WorkflowsTab({
  workflows,
  pluginId,
}: {
  workflows: Workflow[];
  pluginId: number;
}) {
  const { createWorkflow } = useWorkflowStore();
  const navigate = useNavigate();

  const handleCreate = async () => {
    const name = prompt('请输入工作流名称：');
    if (name) {
      const workflow = await createWorkflow(pluginId, name);
      if (workflow) {
        navigate(`/workflows/${workflow.id}/edit`);
      }
    }
  };

  return (
    <div>
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-lg font-medium">工作流</h2>
        <button
          onClick={handleCreate}
          className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        >
          新建工作流
        </button>
      </div>

      {workflows.length === 0 ? (
        <div className="text-center py-8 text-gray-500">
          暂无工作流，点击"新建工作流"开始
        </div>
      ) : (
        <div className="space-y-2">
          {workflows.map((wf) => (
            <div
              key={wf.id}
              className="flex items-center justify-between p-4 border rounded hover:bg-gray-50"
            >
              <div>
                <div className="font-medium">{wf.name}</div>
                <div className="text-sm text-gray-500">
                  {wf.description || '无描述'} · 运行次数: {wf.run_count}
                </div>
              </div>
              <Link
                to={`/workflows/${wf.id}/edit`}
                className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
              >
                编辑
              </Link>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function InfoTab({ plugin }: { plugin: Plugin }) {
  return (
    <div className="space-y-4">
      <div>
        <label className="block text-sm font-medium text-gray-500">名称</label>
        <div className="mt-1">{plugin.name}</div>
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-500">版本</label>
        <div className="mt-1">{plugin.version}</div>
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-500">作者</label>
        <div className="mt-1">{plugin.author || '-'}</div>
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-500">
          描述
        </label>
        <div className="mt-1">{plugin.description || '-'}</div>
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-500">状态</label>
        <div className="mt-1">
          <span
            className={`px-2 py-1 rounded text-xs ${
              plugin.is_enabled
                ? 'bg-green-100 text-green-800'
                : 'bg-gray-100 text-gray-800'
            }`}
          >
            {plugin.is_enabled ? '已启用' : '已禁用'}
          </span>
        </div>
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-500">
          安装时间
        </label>
        <div className="mt-1">
          {new Date(plugin.installed_at).toLocaleString()}
        </div>
      </div>
    </div>
  );
}
