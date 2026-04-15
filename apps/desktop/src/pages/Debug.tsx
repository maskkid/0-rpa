import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface DebugEvent {
  timestamp: string;
  event_type: string;
  bounds?: { x: number; y: number; width: number; height: number };
  label?: string;
  color?: string;
  duration_ms?: number;
  screenshot_path?: string;
}

export function Debug() {
  const [enabled, setEnabled] = useState(false);
  const [highlightDuration, setHighlightDuration] = useState(500);
  const [slowMotion, setSlowMotion] = useState(0);
  const [screenshotOnStep, setScreenshotOnStep] = useState(false);
  const [events] = useState<DebugEvent[]>([]);
  const [selectedScreenshot, setSelectedScreenshot] = useState<string | null>(null);

  const colorMap: Record<string, string> = {
    red: '#ff4444',
    green: '#44ff44',
    blue: '#4444ff',
    yellow: '#ffff44',
  };

  const handleToggle = () => {
    setEnabled(!enabled);
    // TODO: Send command to engine to toggle debug mode
  };

  const handleLoadScreenshot = async (path: string) => {
    try {
      const url = await invoke<string>('get_debug_screenshot', { path });
      setSelectedScreenshot(url);
    } catch (e) {
      console.error('Failed to load screenshot:', e);
    }
  };

  return (
    <div>
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">调试可视化</h1>
        <button
          onClick={handleToggle}
          className={`px-4 py-2 rounded font-medium ${
            enabled
              ? 'bg-red-500 text-white hover:bg-red-600'
              : 'bg-green-500 text-white hover:bg-green-600'
          }`}
        >
          {enabled ? '关闭调试' : '开启调试'}
        </button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="space-y-6">
          <div className="bg-white rounded-lg shadow p-6">
            <h2 className="text-lg font-medium mb-4">调试配置</h2>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  高亮持续时间 (ms)
                </label>
                <input
                  type="number"
                  value={highlightDuration}
                  onChange={(e) => setHighlightDuration(Number(e.target.value))}
                  className="w-full px-3 py-2 border rounded"
                  min={100}
                  max={5000}
                  step={100}
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  慢动作延迟 (ms)
                </label>
                <input
                  type="number"
                  value={slowMotion}
                  onChange={(e) => setSlowMotion(Number(e.target.value))}
                  className="w-full px-3 py-2 border rounded"
                  min={0}
                  max={5000}
                  step={100}
                />
              </div>
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="screenshotOnStep"
                  checked={screenshotOnStep}
                  onChange={(e) => setScreenshotOnStep(e.target.checked)}
                  className="w-4 h-4"
                />
                <label htmlFor="screenshotOnStep" className="text-sm font-medium text-gray-700">
                  每步截图
                </label>
              </div>
              <button
                onClick={() => {
                  // TODO: Send config to engine
                }}
                className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
              >
                应用配置
              </button>
            </div>
          </div>

          <div className="bg-white rounded-lg shadow p-6">
            <h2 className="text-lg font-medium mb-4">事件记录</h2>
            <div className="space-y-2 max-h-[400px] overflow-y-auto">
              {events.length === 0 ? (
                <div className="text-center text-gray-500 py-4">
                  暂无事件，开启调试后会自动记录
                </div>
              ) : (
                events.map((event, idx) => (
                  <div key={idx} className="flex items-center gap-3 p-2 border rounded">
                    {event.event_type === 'highlight' && (
                      <>
                        <div
                          className="w-4 h-4 rounded"
                          style={{
                            backgroundColor: colorMap[event.color || 'red'] || '#ff4444',
                          }}
                        />
                        <div className="flex-1">
                          <div className="text-sm font-medium">
                            {event.label || '高亮'}
                          </div>
                          {event.bounds && (
                            <div className="text-xs text-gray-500">
                              ({event.bounds.x}, {event.bounds.y}){' '}
                              {event.bounds.width}x{event.bounds.height}
                            </div>
                          )}
                        </div>
                        <div className="text-xs text-gray-400">
                          {event.duration_ms}ms
                        </div>
                      </>
                    )}
                    {event.event_type === 'action' && (
                      <>
                        <span>👉</span>
                        <div className="flex-1">
                          <div className="text-sm font-medium">{event.label}</div>
                          {event.bounds && (
                            <div className="text-xs text-gray-500">
                              ({event.bounds.x}, {event.bounds.y})
                            </div>
                          )}
                        </div>
                      </>
                    )}
                    {event.event_type === 'screenshot' && (
                      <>
                        <span>📸</span>
                        <div className="flex-1">
                          <div className="text-sm">截图</div>
                          {event.screenshot_path && (
                            <button
                              onClick={() => handleLoadScreenshot(event.screenshot_path!)}
                              className="text-xs text-blue-600 hover:underline"
                            >
                              查看
                            </button>
                          )}
                        </div>
                      </>
                    )}
                    <span className="text-xs text-gray-400">
                      {new Date(event.timestamp).toLocaleTimeString()}
                    </span>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg shadow">
          <div className="px-4 py-3 border-b">
            <h2 className="font-medium">截图预览</h2>
          </div>
          <div className="p-4">
            {!selectedScreenshot ? (
              <div className="text-center text-gray-500 py-8">
                选择一个截图查看预览
              </div>
            ) : (
              <img
                src={selectedScreenshot}
                alt="Debug screenshot"
                className="max-w-full h-auto"
              />
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
