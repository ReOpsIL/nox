import React from 'react';
import { Link, useLocation } from 'react-router-dom';
import './Sidebar.css';

const Sidebar: React.FC = () => {
  const location = useLocation();

  const isActive = (path: string) => {
    return location.pathname === path;
  };

  return (
    <nav className="sidebar">
      <div className="sidebar-content">
        <ul className="nav-menu">
          <li className="nav-item">
            <Link to="/" className={`nav-link ${isActive('/') ? 'active' : ''}`}>
              <span className="nav-icon">📊</span>
              <span className="nav-text">Dashboard</span>
            </Link>
          </li>
          
          <li className="nav-item">
            <Link to="/agents" className={`nav-link ${isActive('/agents') ? 'active' : ''}`}>
              <span className="nav-icon">👥</span>
              <span className="nav-text">Agents</span>
            </Link>
          </li>
          
          <li className="nav-item">
            <Link to="/tasks" className={`nav-link ${isActive('/tasks') ? 'active' : ''}`}>
              <span className="nav-icon">📋</span>
              <span className="nav-text">Tasks</span>
            </Link>
          </li>
          
          <li className="nav-item">
            <Link to="/health" className={`nav-link ${isActive('/health') ? 'active' : ''}`}>
              <span className="nav-icon">🔧</span>
              <span className="nav-text">System Health</span>
            </Link>
          </li>
          
          <li className="nav-item">
            <Link to="/settings" className={`nav-link ${isActive('/settings') ? 'active' : ''}`}>
              <span className="nav-icon">⚙️</span>
              <span className="nav-text">Settings</span>
            </Link>
          </li>
        </ul>
      </div>
    </nav>
  );
};

export default Sidebar;