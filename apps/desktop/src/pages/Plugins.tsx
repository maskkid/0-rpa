import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { open } from '@tauri-apps/plugin-dialog';
import { usePluginStore } from '../store/usePluginStore';

export function Plugins() {
  const { plugins, loading, error, fetchPlugins, importPlugin, deletePlugin } =
    usePluginStore();
  const [confirmDelete, setConfirmDelete] = useState<number | null>(null);

  useEffect(() => {
    fetchPlugins();
  }, []);

  const handleImport = async () => {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'ZIP Archives', extensions: ['zip'] }],
    });
    if (selected) {
      await importPlugin(selected as string);
      fetchPlugins();
    }
  };

  const handleDelete = async (id: number) => {
    await deletePlugin(id);
    setConfirmDelete(null);
    fetchPlugins();
  };

  const statusBadge = (enabled: boolean) => (
    <span
      className={`px-2 py-1 rounded text-xs ${
        enabled ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'
      }`}
    >
      {enabled ? '已启用' : '已禁用'}
    </span>
  );

  return (
    <div>
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">插件管理</h1>
        <button
          onClick={handleImport}
          className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        >
          导入插件
        </button>
      </div>

      {error && (
        <div className="mb-4 p-4 bg-red-100 border border-red-400 text-red-700 rounded">
          {error}
        </div>
      )}

      {loading ? (
        <div className="text-center py-8 text-gray-500">加载中...</div>
      ) : plugins.length === 0 ? (
        <div className="text-center py-8 text-gray-500">
          暂无插件，点击"导入插件"开始
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
                  版本
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  作者
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  状态
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  安装时间
                </th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  操作
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {plugins.map((plugin) => (
                <tr key={plugin.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4 whitespace-nowrap">
                    <Link
                      to={`/plugins/${plugin.id}`}
                      className="text-blue-600 hover:text-blue-800 font-medium"
                    >
                      {plugin.name}
                    </Link>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {plugin.version}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {plugin.author || '-'}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    {statusBadge(plugin.is_enabled)}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {new Date(plugin.installed_at).toLocaleDateString()}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-right text-sm">
                    <Link
                      to={`/plugins/${plugin.id}`}
                      className="text-blue-600 hover:text-blue-800 mr-4"
                    >
                      查看
                    </Link>
                    {confirmDelete === plugin.id ? (
                      <span className="mr-2">
                        <button
                          onClick={() => handleDelete(plugin.id)}
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
                        onClick={() => setConfirmDelete(plugin.id)}
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
