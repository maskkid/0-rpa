import { useEffect, useState, useCallback } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import MonacoEditor from '@monaco-editor/react';
import { useWorkflowStore } from '../store/useWorkflowStore';
import { useSettingsStore } from '../store/useSettingsStore';

export function WorkflowEditor() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const {
    currentWorkflow,
    fetchWorkflow,
    saveWorkflow,
    loading,
    error,
  } = useWorkflowStore();
  const { executeWorkflow, engineRunning } = useSettingsStore();
  const [script, setScript] = useState('');
  const [saving, setSaving] = useState(false);
  const [running, setRunning] = useState(false);

  useEffect(() => {
    if (id) {
      fetchWorkflow(Number(id));
    }
  }, [id]);

  useEffect(() => {
    if (currentWorkflow) {
      setScript(currentWorkflow.script);
    }
  }, [currentWorkflow]);

  const handleSave = useCallback(async () => {
    if (!id) return;
    setSaving(true);
    try {
      await saveWorkflow(Number(id), script);
      alert('保存成功');
    } catch (e) {
      alert(`保存失败: ${e}`);
    } finally {
      setSaving(false);
    }
  }, [id, script, saveWorkflow]);

  const handleRun = async () => {
    if (!id) return;
    setRunning(true);
    try {
      // First save
      await saveWorkflow(Number(id), script);
      // Then execute
      const taskId = await executeWorkflow(script);
      alert(`工作流已启动，任务ID: ${taskId}`);
      navigate('/tasks');
    } catch (e) {
      alert(`运行失败: ${e}`);
    } finally {
      setRunning(false);
    }
  };

  if (loading && !currentWorkflow) {
    return <div className="text-center py-8 text-gray-500">加载中...</div>;
  }

  if (!currentWorkflow) {
    return (
      <div className="text-center py-8 text-gray-500">
        工作流不存在{' '}
        <Link to="/workflows" className="text-blue-600 hover:text-blue-800">
          返回列表
        </Link>
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col -m-6">
      <div className="p-4 border-b flex justify-between items-center bg-white">
        <div className="flex items-center gap-4">
          <Link
            to="/workflows"
            className="text-gray-500 hover:text-gray-700"
          >
            ← 返回
          </Link>
          <h1 className="text-lg font-bold">{currentWorkflow.name}</h1>
        </div>
        <div className="flex gap-2">
          <button
            onClick={handleSave}
            disabled={saving}
            className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:bg-blue-300"
          >
            {saving ? '保存中...' : '保存'}
          </button>
          <button
            onClick={handleRun}
            disabled={running || !engineRunning}
            className="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600 disabled:bg-green-300"
            title={!engineRunning ? '请先在设置中启动引擎' : ''}
          >
            {running ? '运行中...' : '运行'}
          </button>
        </div>
      </div>

      {error && (
        <div className="p-4 bg-red-100 border-b border-red-400 text-red-700">
          {error}
        </div>
      )}

      {!engineRunning && (
        <div className="p-4 bg-yellow-100 border-b border-yellow-400 text-yellow-700">
          引擎未启动，请前往{' '}
          <Link to="/settings" className="underline">
            设置页面
          </Link>{' '}
          启动引擎后再运行工作流
        </div>
      )}

      <div className="flex-1">
        <MonacoEditor
          height="100%"
          defaultLanguage="json"
          value={script}
          onChange={(v) => setScript(v ?? '{}')}
          theme="vs-dark"
          options={{
            minimap: { enabled: false },
            fontSize: 14,
            wordWrap: 'on',
            automaticLayout: true,
            scrollBeyondLastLine: false,
            tabSize: 2,
          }}
        />
      </div>
    </div>
  );
}
