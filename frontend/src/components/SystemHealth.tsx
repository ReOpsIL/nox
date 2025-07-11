import React, { useState, useEffect } from 'react';
import { WebSocketClient } from '../api/websocket.ts';

interface SystemHealthProps {
  wsClient: WebSocketClient | null;
  systemStatus: any;
}

interface HealthMetrics {
  cpu: number;
  memory: number;
  uptime: number;
  activeAgents: number;
  runningTasks: number;
  completedTasks: number;
  failedTasks: number;
  errorRate: number;
  responseTime: number;
}

const SystemHealth: React.FC<SystemHealthProps> = ({ wsClient, systemStatus }) => {
  const [metrics, setMetrics] = useState<HealthMetrics>({
    cpu: 0,
    memory: 0,
    uptime: 0,
    activeAgents: 0,
    runningTasks: 0,
    completedTasks: 0,
    failedTasks: 0,
    errorRate: 0,
    responseTime: 0
  });
  
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [logs, setLogs] = useState<any[]>([]);

  useEffect(() => {
    fetchHealthMetrics();
    fetchSystemLogs();
    
    // Set up polling for metrics
    const interval = setInterval(fetchHealthMetrics, 5000);
    
    if (wsClient) {
      wsClient.on('system_health_update', handleHealthUpdate);
      wsClient.on('system_error', handleSystemError);
    }

    return () => {
      clearInterval(interval);
      if (wsClient) {
        wsClient.off('system_health_update', handleHealthUpdate);
        wsClient.off('system_error', handleSystemError);
      }
    };
  }, [wsClient]);

  const fetchHealthMetrics = async () => {
    try {
      const response = await fetch('/api/metrics/system');
      if (response.ok) {
        const data = await response.json();
        // Handle API response format with success/data wrappers
        const metricsData = data.success ? (data.metrics || data.data || data) : data;
        setMetrics({
          cpu: metricsData.cpu || 0,
          memory: metricsData.memory || 0,
          uptime: metricsData.uptime || 0,
          activeAgents: metricsData.activeAgents || 0,
          runningTasks: metricsData.runningTasks || 0,
          completedTasks: metricsData.completedTasks || 0,
          failedTasks: metricsData.failedTasks || 0,
          errorRate: metricsData.errorRate || 0,
          responseTime: metricsData.responseTime || 0
        });
        setError(null);
      } else {
        setError('Failed to fetch health metrics');
      }
    } catch (err) {
      setError('Error fetching health metrics');
      console.error('Error fetching health metrics:', err);
    } finally {
      setLoading(false);
    }
  };

  const fetchSystemLogs = async () => {
    try {
      const response = await fetch('/api/system/logs?limit=50');
      if (response.ok) {
        const data = await response.json();
        // Handle API response format with success/data wrappers
        const logsArray = data.success ? (data.logs || data.data || []) : (Array.isArray(data) ? data : []);
        setLogs(Array.isArray(logsArray) ? logsArray : []);
      }
    } catch (err) {
      console.error('Error fetching system logs:', err);
    }
  };

  const handleHealthUpdate = (data: any) => {
    setMetrics(prev => ({ ...prev, ...data }));
  };

  const handleSystemError = (data: any) => {
    setLogs(prev => [data, ...prev.slice(0, 49)]);
  };

  const formatUptime = (seconds: number) => {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    
    if (days > 0) {
      return `${days}d ${hours}h ${minutes}m`;
    } else if (hours > 0) {
      return `${hours}h ${minutes}m`;
    } else {
      return `${minutes}m`;
    }
  };

  const getHealthStatus = () => {
    if (metrics.cpu > 90 || metrics.memory > 90 || metrics.errorRate > 5) {
      return { status: 'critical', color: 'red', icon: '🔴' };
    } else if (metrics.cpu > 70 || metrics.memory > 70 || metrics.errorRate > 2) {
      return { status: 'warning', color: 'orange', icon: '🟡' };
    } else {
      return { status: 'healthy', color: 'green', icon: '🟢' };
    }
  };

  const health = getHealthStatus();

  if (loading) {
    return <div className="loading">Loading system health...</div>;
  }

  return (
    <div className="system-health-page">
      <div className="page-header">
        <h1>🔧 System Health</h1>
        <div className={`overall-status ${health.color}`}>
          {health.icon} {health.status.toUpperCase()}
        </div>
      </div>

      {error && (
        <div className="error-banner">
          {error}
          <button onClick={() => setError(null)}>×</button>
        </div>
      )}

      <div className="health-overview">
        <div className="health-grid">
          <div className="health-card">
            <div className="health-icon">💻</div>
            <div className="health-content">
              <h3>CPU Usage</h3>
              <div className="metric-value">{metrics.cpu.toFixed(1)}%</div>
              <div className="progress-bar">
                <div 
                  className="progress-fill"
                  style={{ 
                    width: `${metrics.cpu}%`,
                    backgroundColor: metrics.cpu > 80 ? '#f56565' : metrics.cpu > 60 ? '#ed8936' : '#48bb78'
                  }}
                ></div>
              </div>
            </div>
          </div>

          <div className="health-card">
            <div className="health-icon">🧠</div>
            <div className="health-content">
              <h3>Memory Usage</h3>
              <div className="metric-value">{metrics.memory.toFixed(1)}%</div>
              <div className="progress-bar">
                <div 
                  className="progress-fill"
                  style={{ 
                    width: `${metrics.memory}%`,
                    backgroundColor: metrics.memory > 80 ? '#f56565' : metrics.memory > 60 ? '#ed8936' : '#48bb78'
                  }}
                ></div>
              </div>
            </div>
          </div>

          <div className="health-card">
            <div className="health-icon">⏰</div>
            <div className="health-content">
              <h3>Uptime</h3>
              <div className="metric-value">{formatUptime(metrics.uptime)}</div>
              <div className="metric-subtitle">System running</div>
            </div>
          </div>

          <div className="health-card">
            <div className="health-icon">👥</div>
            <div className="health-content">
              <h3>Active Agents</h3>
              <div className="metric-value">{metrics.activeAgents}</div>
              <div className="metric-subtitle">Currently active</div>
            </div>
          </div>

          <div className="health-card">
            <div className="health-icon">⚡</div>
            <div className="health-content">
              <h3>Running Tasks</h3>
              <div className="metric-value">{metrics.runningTasks}</div>
              <div className="metric-subtitle">In progress</div>
            </div>
          </div>

          <div className="health-card">
            <div className="health-icon">✅</div>
            <div className="health-content">
              <h3>Completed Tasks</h3>
              <div className="metric-value">{metrics.completedTasks}</div>
              <div className="metric-subtitle">Total completed</div>
            </div>
          </div>

          <div className="health-card">
            <div className="health-icon">❌</div>
            <div className="health-content">
              <h3>Failed Tasks</h3>
              <div className="metric-value">{metrics.failedTasks}</div>
              <div className="metric-subtitle">Total failures</div>
            </div>
          </div>

          <div className="health-card">
            <div className="health-icon">📊</div>
            <div className="health-content">
              <h3>Error Rate</h3>
              <div className="metric-value">{metrics.errorRate.toFixed(2)}%</div>
              <div className="metric-subtitle">Last hour</div>
            </div>
          </div>

          <div className="health-card">
            <div className="health-icon">🚀</div>
            <div className="health-content">
              <h3>Response Time</h3>
              <div className="metric-value">{metrics.responseTime}ms</div>
              <div className="metric-subtitle">Average API response</div>
            </div>
          </div>
        </div>
      </div>

      <div className="health-details">
        <div className="system-info">
          <h2>📋 System Information</h2>
          <div className="info-grid">
            <div className="info-item">
              <span className="info-label">Health Score:</span>
              <span className={`info-value ${systemStatus?.health > 80 ? 'healthy' : systemStatus?.health > 60 ? 'warning' : 'critical'}`}>
                {systemStatus?.health > 80 ? '🟢' : systemStatus?.health > 60 ? '🟡' : '🔴'} 
                {systemStatus?.health || 0}/100
              </span>
            </div>
            <div className="info-item">
              <span className="info-label">System Uptime:</span>
              <span className="info-value">{formatUptime(systemStatus?.system?.uptime || 0)}</span>
            </div>
            <div className="info-item">
              <span className="info-label">WebSocket:</span>
              <span className={`info-value ${wsClient?.isConnected() ? 'connected' : 'disconnected'}`}>
                {wsClient?.isConnected() ? '🟢 Connected' : '🔴 Disconnected'}
              </span>
            </div>
            <div className="info-item">
              <span className="info-label">Client ID:</span>
              <span className="info-value">{wsClient?.getClientId() || 'Not connected'}</span>
            </div>
          </div>
        </div>

        <div className="system-logs">
          <h2>📄 Recent System Events</h2>
          <div className="logs-container">
            {logs.length === 0 ? (
              <div className="no-logs">No recent events</div>
            ) : (
              logs.map((log, index) => (
                <div key={index} className={`log-entry ${log.level || 'info'}`}>
                  <div className="log-timestamp">
                    {new Date(log.timestamp).toLocaleString()}
                  </div>
                  <div className="log-level">{log.level?.toUpperCase() || 'INFO'}</div>
                  <div className="log-message">{log.message}</div>
                  {log.details && (
                    <div className="log-details">{JSON.stringify(log.details, null, 2)}</div>
                  )}
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default SystemHealth;