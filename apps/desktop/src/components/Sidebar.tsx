import { NavLink } from 'react-router-dom';

const navItems = [
  { to: '/plugins', label: '插件管理', icon: '📦' },
  { to: '/workflows', label: '工作流', icon: '⚙️' },
  { to: '/tasks', label: '任务监控', icon: '📊' },
  { to: '/debug', label: '调试', icon: '🐛' },
  { to: '/settings', label: '设置', icon: '⚡' },
];

export function Sidebar() {
  return (
    <aside className="w-56 bg-gray-900 text-white flex flex-col">
      <div className="p-4 font-bold text-lg border-b border-gray-700">
        RPA Desktop
      </div>
      <nav className="flex-1 p-2">
        {navItems.map(({ to, label, icon }) => (
          <NavLink
            key={to}
            to={to}
            className={({ isActive }) =>
              `flex items-center gap-2 px-3 py-2 rounded hover:bg-gray-700 ${
                isActive ? 'bg-gray-700' : ''
              }`
            }
          >
            <span>{icon}</span> {label}
          </NavLink>
        ))}
      </nav>
    </aside>
  );
}
