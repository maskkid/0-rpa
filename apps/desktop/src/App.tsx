import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { Layout } from './components/Layout';
import { Plugins } from './pages/Plugins';
import { PluginDetail } from './pages/PluginDetail';
import { Workflows } from './pages/Workflows';
import { WorkflowEditor } from './pages/WorkflowEditor';
import { Tasks } from './pages/Tasks';
import { Debug } from './pages/Debug';
import { Settings } from './pages/Settings';

export function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<Layout />}>
          <Route path="/" element={<Navigate to="/plugins" replace />} />
          <Route path="/plugins" element={<Plugins />} />
          <Route path="/plugins/:id" element={<PluginDetail />} />
          <Route path="/workflows" element={<Workflows />} />
          <Route path="/workflows/:id/edit" element={<WorkflowEditor />} />
          <Route path="/tasks" element={<Tasks />} />
          <Route path="/debug" element={<Debug />} />
          <Route path="/settings" element={<Settings />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
