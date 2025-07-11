import React from 'react';
import { Link } from 'react-router-dom';
import './Header.css';

interface HeaderProps {
  connectionStatus: 'disconnected' | 'connecting' | 'connected';
  systemStatus: any;
}

const Header: React.FC<HeaderProps> = ({ connectionStatus, systemStatus }) => {
  const getConnectionIcon = () => {
    switch (connectionStatus) {
      case 'connected': return '🟢';
      case 'connecting': return '🟡';
      default: return '🔴';
    }
  };

  const getConnectionText = () => {
    switch (connectionStatus) {
      case 'connected': return 'Connected';
      case 'connecting': return 'Connecting...';
      default: return 'Disconnected';
    }
  };

  return (
    <header className="header">
      <div className="header-left">
        <Link to="/" className="logo">
          <h1>🚀 NOX Agent Ecosystem</h1>
        </Link>
        <span className="version">v1.0.0</span>
      </div>
      
      <div className="header-center">
        <nav className="main-nav">
          <Link to="/" className="nav-link">Dashboard</Link>
          <Link to="/agents" className="nav-link">Agents</Link>
          <Link to="/tasks" className="nav-link">Tasks</Link>
          <Link to="/health" className="nav-link">System Health</Link>
        </nav>
      </div>
      
      <div className="header-right">
        <div className="status-indicators">
          <div className="status-item">
            <span className="status-label">Health:</span>
            <span className={`status-value ${systemStatus?.health > 80 ? 'healthy' : systemStatus?.health > 60 ? 'warning' : 'critical'}`}>
              {systemStatus?.health > 80 ? '🟢' : systemStatus?.health > 60 ? '🟡' : '🔴'} 
              {systemStatus?.health || 0}/100
            </span>
          </div>
          
          <div className="status-item">
            <span className="status-label">WebSocket:</span>
            <span className={`status-value ${connectionStatus}`}>
              {getConnectionIcon()} {getConnectionText()}
            </span>
          </div>
        </div>
        
        <div className="header-actions">
          <Link to="/settings" className="settings-button" aria-label="Settings">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="3"></circle>
              <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
            </svg>
          </Link>
        </div>
      </div>
    </header>
  );
};

export default Header;