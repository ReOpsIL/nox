import React, { useState, useEffect, useCallback } from 'react';
import { WebSocketClient } from '../api/websocket.ts';

interface DashboardProps {
  wsClient: WebSocketClient | null;
  systemStatus: any;
}

const Dashboard: React.FC<DashboardProps> = ({ wsClient, systemStatus }) => {
  const [stats, setStats] = useState({
    activeAgents: 0,
    runningTasks: 0,
    completedTasks: 0,
    uptime: 0
  });

  const [recentActivity, setRecentActivity] = useState<any[]>([]);

  const fetchDashboardStats = async () => {
    try {
      const [agentsRes, tasksRes, metricsRes] = await Promise.all([
        fetch('/api/agents'),
        fetch('/api/tasks'),
        fetch('/api/metrics/system')
      ]);

      if (agentsRes.ok && tasksRes.ok && metricsRes.ok) {
        const agentsData = await agentsRes.json();
        const tasksData = await tasksRes.json();
        const metricsData = await metricsRes.json();

        // Handle API response format with success/data wrappers
        const agents = agentsData.success ? (agentsData.agents || agentsData.data || []) : [];
        const tasks = tasksData.success ? (tasksData.tasks || tasksData.data || []) : [];
        const metrics = metricsData.success ? (metricsData.metrics || metricsData.data || metricsData) : {};

        setStats({
          activeAgents: Array.isArray(agents) ? agents.filter((a: any) => a.status === 'active' || a.status === 'running').length : 0,
          runningTasks: Array.isArray(tasks) ? tasks.filter((t: any) => t.status === 'running').length : 0,
          completedTasks: Array.isArray(tasks) ? tasks.filter((t: any) => t.status === 'completed').length : 0,
          uptime: metrics.uptime || 0
        });
      }
    } catch (error) {
      console.error('Failed to fetch dashboard stats:', error);
    }
  };

  const handleAgentUpdate = useCallback((data: any) => {
    setRecentActivity(prev => [{
      type: 'agent',
      message: `Agent ${data.name} status changed to ${data.status}`,
      timestamp: new Date().toISOString()
    }, ...prev.slice(0, 9)]);
    fetchDashboardStats();
  }, []);

  const handleTaskUpdate = useCallback((data: any) => {
    setRecentActivity(prev => [{
      type: 'task',
      message: `Task ${data.title} ${data.status}`,
      timestamp: new Date().toISOString()
    }, ...prev.slice(0, 9)]);
    fetchDashboardStats();
  }, []);

  useEffect(() => {
    fetchDashboardStats();
    
    if (wsClient) {
      wsClient.on('agent_status_changed', handleAgentUpdate);
      wsClient.on('task_completed', handleTaskUpdate);
      wsClient.on('task_started', handleTaskUpdate);
    }

    return () => {
      if (wsClient) {
        wsClient.off('agent_status_changed', handleAgentUpdate);
        wsClient.off('task_completed', handleTaskUpdate);
        wsClient.off('task_started', handleTaskUpdate);
      }
    };
  }, [wsClient, handleAgentUpdate, handleTaskUpdate]);

  const formatUptime = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${minutes}m`;
  };

  return (
    <div className="dashboard">
      <div className="dashboard-header">
        <h1>🚀 NOX Dashboard</h1>
        <p>Real-time monitoring and management</p>
      </div>

      <div className="stats-grid">
        <div className="stat-card">
          <div className="stat-icon">👥</div>
          <div className="stat-content">
            <h3>{stats.activeAgents}</h3>
            <p>Active Agents</p>
          </div>
        </div>

        <div className="stat-card">
          <div className="stat-icon">⚡</div>
          <div className="stat-content">
            <h3>{stats.runningTasks}</h3>
            <p>Running Tasks</p>
          </div>
        </div>

        <div className="stat-card">
          <div className="stat-icon">✅</div>
          <div className="stat-content">
            <h3>{stats.completedTasks}</h3>
            <p>Completed Tasks</p>
          </div>
        </div>

        <div className="stat-card">
          <div className="stat-icon">⏰</div>
          <div className="stat-content">
            <h3>{formatUptime(stats.uptime)}</h3>
            <p>System Uptime</p>
          </div>
        </div>
      </div>

      <div className="dashboard-content">
        <div className="activity-section">
          <h2>📊 Recent Activity</h2>
          <div className="activity-list">
            {recentActivity.length > 0 ? (
              recentActivity.map((activity, index) => (
                <div key={index} className="activity-item">
                  <div className={`activity-type ${activity.type}`}>
                    {activity.type === 'agent' ? '👤' : '📋'}
                  </div>
                  <div className="activity-content">
                    <p>{activity.message}</p>
                    <small>{new Date(activity.timestamp).toLocaleTimeString()}</small>
                  </div>
                </div>
              ))
            ) : (
              <p className="no-activity">No recent activity</p>
            )}</div>
        </div>

        <div className="system-overview">
          <h2>🔧 System Overview</h2>
          <div className="system-info">
            {systemStatus ? (
              <div className="system-details">
                <div className="info-item">
                  <strong>Health Score:</strong>
                  <span className={`status ${systemStatus.health > 80 ? 'healthy' : systemStatus.health > 60 ? 'warning' : 'critical'}`}>
                    {systemStatus.health > 80 ? '🟢' : systemStatus.health > 60 ? '🟡' : '🔴'} {systemStatus.health || 0}/100
                  </span>
                </div>
                <div className="info-item">
                  <strong>Active Agents:</strong>
                  <span>{systemStatus.agents?.active || 0}/{systemStatus.agents?.total || 0}</span>
                </div>
                <div className="info-item">
                  <strong>WebSocket:</strong>
                  <span className={wsClient?.isConnected() ? 'connected' : 'disconnected'}>
                    {wsClient?.isConnected() ? '🟢 Connected' : '🔴 Disconnected'}
                  </span>
                </div>
              </div>
            ) : (
              <p>Loading system information...</p>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default Dashboard;