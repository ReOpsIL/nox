import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { Line } from 'react-chartjs-2';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  ChartData,
} from 'chart.js';
import apiClient from '../api/api-client';
import { connectWebSocket } from '../api/websocket';
import './Dashboard.css';

// Register Chart.js components
ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend
);

/**
 * Dashboard page component
 */
const Dashboard: React.FC = () => {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [systemMetrics, setSystemMetrics] = useState<any>(null);
  const [agentStats, setAgentStats] = useState<any>({
    total: 0,
    active: 0,
    inactive: 0,
    error: 0,
  });
  const [taskStats, setTaskStats] = useState<any>({
    total: 0,
    todo: 0,
    inProgress: 0,
    done: 0,
    blocked: 0,
  });
  const [cpuHistory, setCpuHistory] = useState<number[]>([]);
  const [memoryHistory, setMemoryHistory] = useState<number[]>([]);
  const [timeLabels, setTimeLabels] = useState<string[]>([]);

  // Load initial data
  useEffect(() => {
    const fetchData = async () => {
      try {
        setLoading(true);
        
        // Get latest system metrics
        const metrics = await apiClient.getLatestSystemMetrics();
        setSystemMetrics(metrics);
        
        if (metrics) {
          setAgentStats(metrics.agents);
          setTaskStats(metrics.tasks);
          
          // Add current metrics to history
          addMetricToHistory(metrics);
        }
        
        // Get historical metrics for charts
        const endTime = new Date().toISOString();
        const startTime = new Date(Date.now() - 60 * 60 * 1000).toISOString(); // Last hour
        const historicalMetrics = await apiClient.getSystemMetrics(startTime, endTime, 'minute');
        
        if (historicalMetrics && historicalMetrics.length > 0) {
          const cpuData: number[] = [];
          const memData: number[] = [];
          const labels: string[] = [];
          
          historicalMetrics.forEach(metric => {
            cpuData.push(metric.system.cpuUsage);
            memData.push(metric.system.memoryUsage);
            labels.push(new Date(metric.timestamp).toLocaleTimeString());
          });
          
          setCpuHistory(cpuData);
          setMemoryHistory(memData);
          setTimeLabels(labels);
        }
        
        setError(null);
      } catch (err) {
        console.error('Error fetching dashboard data:', err);
        setError('Failed to load dashboard data. Please try again later.');
      } finally {
        setLoading(false);
      }
    };
    
    fetchData();
    
    // Set up interval to refresh data
    const intervalId = setInterval(fetchData, 60000); // Refresh every minute
    
    return () => clearInterval(intervalId);
  }, []);

  // Connect to WebSocket for real-time updates
  useEffect(() => {
    const socket = connectWebSocket();
    
    // Listen for agent status updates
    socket.on('agent_status_list', (data) => {
      const stats = {
        total: data.length,
        active: data.filter((a: any) => a.status === 'running').length,
        inactive: data.filter((a: any) => a.status === 'stopped').length,
        error: data.filter((a: any) => a.status !== 'running' && a.status !== 'stopped').length,
      };
      setAgentStats(stats);
    });
    
    // Listen for task dashboard updates
    socket.on('task_dashboard', (data) => {
      setTaskStats({
        total: data.total,
        todo: data.byStatus.todo,
        inProgress: data.byStatus.inprogress,
        done: data.byStatus.done,
        blocked: data.blocked,
      });
    });
    
    // Listen for system metrics updates
    socket.on('system_metrics', (data) => {
      setSystemMetrics(data);
      addMetricToHistory(data);
    });
    
    return () => {
      // No need to disconnect as the WebSocket client is a singleton
    };
  }, []);

  // Add a new metric to the history
  const addMetricToHistory = (metric: any) => {
    if (!metric || !metric.system) return;
    
    setCpuHistory(prev => {
      const newHistory = [...prev, metric.system.cpuUsage];
      return newHistory.slice(-60); // Keep last 60 data points
    });
    
    setMemoryHistory(prev => {
      const newHistory = [...prev, metric.system.memoryUsage];
      return newHistory.slice(-60); // Keep last 60 data points
    });
    
    setTimeLabels(prev => {
      const newLabels = [...prev, new Date().toLocaleTimeString()];
      return newLabels.slice(-60); // Keep last 60 data points
    });
  };

  // Prepare chart data
  const cpuChartData: ChartData<'line'> = {
    labels: timeLabels,
    datasets: [
      {
        label: 'CPU Usage (%)',
        data: cpuHistory,
        borderColor: 'rgb(75, 192, 192)',
        backgroundColor: 'rgba(75, 192, 192, 0.5)',
        tension: 0.2,
      },
    ],
  };

  const memoryChartData: ChartData<'line'> = {
    labels: timeLabels,
    datasets: [
      {
        label: 'Memory Usage (%)',
        data: memoryHistory,
        borderColor: 'rgb(153, 102, 255)',
        backgroundColor: 'rgba(153, 102, 255, 0.5)',
        tension: 0.2,
      },
    ],
  };

  const chartOptions = {
    responsive: true,
    maintainAspectRatio: false,
    scales: {
      y: {
        beginAtZero: true,
        max: 100,
        ticks: {
          callback: (value: any) => `${value}%`,
        },
      },
    },
    plugins: {
      legend: {
        position: 'top' as const,
      },
    },
  };

  if (loading && !systemMetrics) {
    return (
      <div className="dashboard-loading">
        <div className="spinner"></div>
        <p>Loading dashboard data...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="dashboard-error">
        <h2>Error</h2>
        <p>{error}</p>
        <button onClick={() => window.location.reload()}>Retry</button>
      </div>
    );
  }

  return (
    <div className="dashboard-container">
      <h1>System Dashboard</h1>
      
      <div className="dashboard-overview">
        <div className="dashboard-card">
          <h2>Agents</h2>
          <div className="stat-grid">
            <div className="stat-item">
              <span className="stat-value">{agentStats.total}</span>
              <span className="stat-label">Total</span>
            </div>
            <div className="stat-item">
              <span className="stat-value">{agentStats.active}</span>
              <span className="stat-label">Active</span>
            </div>
            <div className="stat-item">
              <span className="stat-value">{agentStats.inactive}</span>
              <span className="stat-label">Inactive</span>
            </div>
            <div className="stat-item">
              <span className="stat-value">{agentStats.error}</span>
              <span className="stat-label">Error</span>
            </div>
          </div>
          <Link to="/agents" className="dashboard-link">View All Agents</Link>
        </div>
        
        <div className="dashboard-card">
          <h2>Tasks</h2>
          <div className="stat-grid">
            <div className="stat-item">
              <span className="stat-value">{taskStats.total}</span>
              <span className="stat-label">Total</span>
            </div>
            <div className="stat-item">
              <span className="stat-value">{taskStats.todo}</span>
              <span className="stat-label">To Do</span>
            </div>
            <div className="stat-item">
              <span className="stat-value">{taskStats.inProgress}</span>
              <span className="stat-label">In Progress</span>
            </div>
            <div className="stat-item">
              <span className="stat-value">{taskStats.done}</span>
              <span className="stat-label">Done</span>
            </div>
          </div>
          <Link to="/tasks" className="dashboard-link">View All Tasks</Link>
        </div>
        
        <div className="dashboard-card">
          <h2>System Health</h2>
          <div className="stat-grid">
            <div className="stat-item">
              <span className="stat-value">
                {systemMetrics?.system?.cpuUsage?.toFixed(1)}%
              </span>
              <span className="stat-label">CPU Usage</span>
            </div>
            <div className="stat-item">
              <span className="stat-value">
                {systemMetrics?.system?.memoryUsage?.toFixed(1)}%
              </span>
              <span className="stat-label">Memory Usage</span>
            </div>
            <div className="stat-item">
              <span className="stat-value">
                {systemMetrics?.messages?.processedPerMinute}
              </span>
              <span className="stat-label">Msgs/min</span>
            </div>
            <div className="stat-item">
              <span className="stat-value">
                {formatUptime(systemMetrics?.system?.uptime)}
              </span>
              <span className="stat-label">Uptime</span>
            </div>
          </div>
          <Link to="/system" className="dashboard-link">View System Health</Link>
        </div>
      </div>
      
      <div className="dashboard-charts">
        <div className="chart-container">
          <h2>CPU Usage</h2>
          <div className="chart-wrapper">
            <Line data={cpuChartData} options={chartOptions} />
          </div>
        </div>
        
        <div className="chart-container">
          <h2>Memory Usage</h2>
          <div className="chart-wrapper">
            <Line data={memoryChartData} options={chartOptions} />
          </div>
        </div>
      </div>
      
      <div className="dashboard-recent-activity">
        <h2>Recent Activity</h2>
        {/* This would be populated with real activity data */}
        <div className="activity-empty">
          <p>No recent activity to display</p>
        </div>
      </div>
    </div>
  );
};

// Helper function to format uptime
function formatUptime(seconds?: number): string {
  if (!seconds) return '0m';
  
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  
  if (days > 0) {
    return `${days}d ${hours}h`;
  } else if (hours > 0) {
    return `${hours}h ${minutes}m`;
  } else {
    return `${minutes}m`;
  }
}

export default Dashboard;