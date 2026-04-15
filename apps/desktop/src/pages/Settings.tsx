import { useEffect, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { useSettingsStore } from '../store/useSettingsStore';
import type { Settings } from '../store/useSettingsStore';

export function Settings() {
  const {
    settings,
    loading,
    error,
    engineRunning,
    fetchSettings,
    saveSettings,
    testConnection,
    startEngine,
    stopEngine,
  } = useSettingsStore();

  const [formData, setFormData] = useState<Settings | null>(null);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<boolean | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    fetchSettings();
  }, []);

  useEffect(() => {
    if (settings) {
      setFormData(settings);
    }
  }, [settings]);

  const handleTestConnection = async () => {
    if (!formData) return;
    setTesting(true);
    setTestResult(null);
    try {
      const result = await testConnection(formData.database_url);
      setTestResult(result);
    } catch (e) {
      setTestResult(false);
    } finally {
      setTesting(false);
    }
  };

  const handleSave = async () => {
    if (!formData) return;
    setSaving(true);
    try {
      await saveSettings(formData);
      alert('设置已保存');
    } catch (e) {
      alert(`保存失败: ${e}`);
    } finally {
      setSaving(false);
    }
  };

  const handleSelectEngineBinary = async () => {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: 'Executables',
          extensions: [],
        },
      ],
    });
    if (selected && formData) {
      setFormData({ ...formData, engine_binary_path: selected as string });
    }
  };

  const handleStartEngine = async () => {
    if (!formData?.engine_binary_path) {
      alert('请先选择引擎二进制文件路径');
      return;
    }
    try {
      await startEngine(formData.engine_binary_path);
      alert('引擎已启动');
    } catch (e) {
      alert(`启动引擎失败: ${e}`);
    }
  };

  const handleStopEngine = async () => {
    try {
      await stopEngine();
      alert('引擎已停止');
    } catch (e) {
      alert(`停止引擎失败: ${e}`);
    }
  };

  if (loading || !formData) {
    return <div className="text-center py-8 text-gray-500">加载中...</div>;
  }

  const isMySQL = formData.database_url.startsWith('mysql://');
  const isSQLite = formData.database_url.startsWith('sqlite://');

  // Suppress unused variable warnings
  void isMySQL;
  void isSQLite;

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">设置</h1>

      {error && (
        <div className="mb-4 p-4 bg-red-100 border border-red-400 text-red-700 rounded">
          {error}
        </div>
      )}

      <div className="space-y-6">
        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-lg font-medium mb-4">数据库连接</h2>
          <div className="space-y-4">
            <div className="flex items-center gap-4">
              <span
                className={`px-3 py-1 rounded text-sm font-medium ${
                  isSQLite
                    ? 'bg-green-100 text-green-800'
                    : isMySQL
                    ? 'bg-blue-100 text-blue-800'
                    : 'bg-gray-100 text-gray-800'
                }`}
              >
                {isSQLite ? 'SQLite' : isMySQL ? 'MySQL' : '其他'}
              </span>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                连接 URL
              </label>
              <input
                type="text"
                value={formData.database_url}
                onChange={(e) =>
                  setFormData({ ...formData, database_url: e.target.value })
                }
                className="w-full px-3 py-2 border rounded"
                placeholder="sqlite:///path/to/database.db"
              />
              <p className="mt-1 text-xs text-gray-500">
                SQLite格式: sqlite:///path/to/data.db
                <br />
                MySQL格式: mysql://user:password@localhost:3306/database
              </p>
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={handleTestConnection}
                disabled={testing}
                className="px-4 py-2 bg-gray-200 text-gray-700 rounded hover:bg-gray-300 disabled:bg-gray-100"
              >
                {testing ? '测试中...' : '测试连接'}
              </button>
              {testResult === true && (
                <span className="text-green-600">✓ 连接成功</span>
              )}
              {testResult === false && (
                <span className="text-red-600">✗ 连接失败</span>
              )}
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-lg font-medium mb-4">引擎配置</h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                引擎二进制文件路径
              </label>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={formData.engine_binary_path}
                  onChange={(e) =>
                    setFormData({
                      ...formData,
                      engine_binary_path: e.target.value,
                    })
                  }
                  className="flex-1 px-3 py-2 border rounded"
                  placeholder="选择引擎可执行文件"
                />
                <button
                  onClick={handleSelectEngineBinary}
                  className="px-4 py-2 bg-gray-200 text-gray-700 rounded hover:bg-gray-300"
                >
                  浏览
                </button>
              </div>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                引擎端口
              </label>
              <input
                type="number"
                value={formData.engine_port}
                onChange={(e) =>
                  setFormData({ ...formData, engine_port: Number(e.target.value) })
                }
                className="w-32 px-3 py-2 border rounded"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                执行超时 (ms)
              </label>
              <input
                type="number"
                value={formData.engine_timeout_ms}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    engine_timeout_ms: Number(e.target.value),
                  })
                }
                className="w-32 px-3 py-2 border rounded"
              />
            </div>
            <div className="flex items-center gap-4 pt-2">
              {engineRunning ? (
                <button
                  onClick={handleStopEngine}
                  className="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600"
                >
                  停止引擎
                </button>
              ) : (
                <button
                  onClick={handleStartEngine}
                  className="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600"
                >
                  启动引擎
                </button>
              )}
              {engineRunning && (
                <span className="text-green-600">✓ 引擎运行中</span>
              )}
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-lg font-medium mb-4">调试选项</h2>
          <div className="space-y-4">
            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                id="debug_enabled"
                checked={formData.debug_enabled}
                onChange={(e) =>
                  setFormData({ ...formData, debug_enabled: e.target.checked })
                }
                className="w-4 h-4"
              />
              <label htmlFor="debug_enabled" className="text-sm font-medium text-gray-700">
                启用调试模式
              </label>
            </div>
            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                id="debug_highlight"
                checked={formData.debug_highlight}
                onChange={(e) =>
                  setFormData({ ...formData, debug_highlight: e.target.checked })
                }
                className="w-4 h-4"
              />
              <label htmlFor="debug_highlight" className="text-sm font-medium text-gray-700">
                显示高亮
              </label>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                高亮持续时间 (ms)
              </label>
              <input
                type="number"
                value={formData.debug_highlight_duration_ms}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    debug_highlight_duration_ms: Number(e.target.value),
                  })
                }
                className="w-32 px-3 py-2 border rounded"
              />
            </div>
            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                id="debug_screenshot_on_step"
                checked={formData.debug_screenshot_on_step}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    debug_screenshot_on_step: e.target.checked,
                  })
                }
                className="w-4 h-4"
              />
              <label htmlFor="debug_screenshot_on_step" className="text-sm font-medium text-gray-700">
                每步截图
              </label>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                截图保存目录
              </label>
              <input
                type="text"
                value={formData.debug_screenshot_dir}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    debug_screenshot_dir: e.target.value,
                  })
                }
                className="w-full px-3 py-2 border rounded"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                慢动作延迟 (ms)
              </label>
              <input
                type="number"
                value={formData.debug_slow_motion_ms}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    debug_slow_motion_ms: Number(e.target.value),
                  })
                }
                className="w-32 px-3 py-2 border rounded"
              />
            </div>
          </div>
        </div>

        <div className="flex justify-end">
          <button
            onClick={handleSave}
            disabled={saving}
            className="px-6 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:bg-blue-300"
          >
            {saving ? '保存中...' : '保存设置'}
          </button>
        </div>
      </div>
    </div>
  );
}
