import React, { useState, useEffect } from 'react';
import { Line, Bar } from 'react-chartjs-2';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  Title,
  Tooltip,
  Legend,
  ChartData,
} from 'chart.js';
import apiClient from '../api/api-client';
import { connectWebSocket } from '../api/websocket';
import './SystemHealth.css';

// Register Chart.js components
ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  Title,
  Tooltip,
  Legend
);

/**
 * SystemHealth page component
 */
const SystemHealth: React.FC = () => {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [systemMetrics, setSystemMetrics] = useState<any>(null);
  const [historicalMetrics, setHistoricalMetrics] = useState<any[]>([]);
  const [timeRange, setTimeRange] = useState<string>('hour');
  const [interval, setInterval] = useState<'minute' | 'hour' | 'day'>('minute');
  
  // Load initial data
  useEffect(() => {
    const fetchData = async () => {
      try {
        setLoading(true);
        
        // Get latest system metrics
        const latestMetrics = await apiClient.getLatestSystemMetrics();
        setSystemMetrics(latestMetrics);
        
        // Get historical metrics based on selected time range
        await fetchHistoricalMetrics();
        
        setError(null);
      } catch (err) {
        console.error('Error fetching system health data:', err);
        setError('Failed to load system health data. Please try again later.');
      } finally {
        setLoading(false);
      }
    };
    
    fetchData();
    
    // Set up interval to refresh data
    const intervalId = setInterval(fetchData, 60000); // Refresh every minute
    
    return () => clearInterval(intervalId);
  }, []);

  // Fetch historical metrics based on selected time range
  const fetchHistoricalMetrics = async () => {
    try {
      const endTime = new Date().toISOString();
      let startTime: string;
      let newInterval: 'minute' | 'hour' | 'day' = 'minute';
      
      // Calculate start time based on selected time range
      switch (timeRange) {
        case 'hour':
          startTime = new Date(Date.now() - 60 * 60 * 1000).toISOString();
          newInterval = 'minute';
          break;
        case 'day':
          startTime = new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString();
          newInterval = 'hour';
          break;
        case 'week':
          startTime = new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString();
          newInterval = 'hour';
          break;
        case 'month':
          startTime = new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString();
          newInterval = 'day';
          break;
        default:
          startTime = new Date(Date.now() - 60 * 60 * 1000).toISOString();
          newInterval = 'minute';
      }
      
      setInterval(newInterval);
      
      // Get historical metrics
      const metrics = await apiClient.getSystemMetrics(startTime, endTime, newInterval);
      setHistoricalMetrics(metrics);
      
    } catch (err) {
      console.error('Error fetching historical metrics:', err);
      setError('Failed to load historical metrics. Please try again later.');
    }
  };

  // Handle time range change
  const handleTimeRangeChange = async (range: string) => {
    setTimeRange(range);
    try {
      setLoading(true);
      
      // Update time range
      setTimeRange(range);
      
      // Fetch historical metrics with new time range
      await fetchHistoricalMetrics();
      
      setError(null);
    } catch (err) {
      console.error('Error updating time range:', err);
      setError('Failed to update time range. Please try again later.');
    } finally {
      setLoading(false);
    }
  };

  // Connect to WebSocket for real-time updates
  useEffect(() => {
    const socket = connectWebSocket();
    
    // Listen for system metrics updates
    socket.on('system_metrics', (data) => {
      setSystemMetrics(data);
    });
    
    return () => {
      // No need to disconnect as the WebSocket client is a singleton
    };
  }, []);

  // Prepare CPU usage chart data
  const cpuChartData: ChartData<'line'> = {
    labels: historicalMetrics.map(metric => {
      const date = new Date(metric.timestamp);
      return interval === 'minute' 
        ? date.toLocaleTimeString() 
        : interval === 'hour'
          ? `${date.getHours()}:00`
          : date.toLocaleDateString();
    }),
    datasets: [
      {
        label: 'CPU Usage (%)',
        data: historicalMetrics.map(metric => metric.system.cpuUsage),
        borderColor: 'rgb(75, 192, 192)',
        backgroundColor: 'rgba(75, 192, 192, 0.5)',
        tension: 0.2,
      },
    ],
  };

  // Prepare memory usage chart data
  const memoryChartData: ChartData<'line'> = {
    labels: historicalMetrics.map(metric => {
      const date = new Date(metric.timestamp);
      return interval === 'minute' 
        ? date.toLocaleTimeString() 
        : interval === 'hour'
          ? `${date.getHours()}:00`
          : date.toLocaleDateString();
    }),
    datasets: [
      {
        label: 'Memory Usage (%)',
        data: historicalMetrics.map(metric => metric.system.memoryUsage),
        borderColor: 'rgb(153, 102, 255)',
        backgroundColor: 'rgba(153, 102, 255, 0.5)',
        tension: 0.2,
      },
    ],
  };

  // Prepare message throughput chart data
  const messageChartData: ChartData<'line'> = {
    labels: historicalMetrics.map(metric => {
      const date = new Date(metric.timestamp);
      return interval === 'minute' 
        ? date.toLocaleTimeString() 
        : interval === 'hour'
          ? `${date.getHours()}:00`
          : date.toLocaleDateString();
    }),
    datasets: [
      {
        label: 'Messages per Minute',
        data: historicalMetrics.map(metric => metric.messages.processedPerMinute),
        borderColor: 'rgb(255, 159, 64)',
        backgroundColor: 'rgba(255, 159, 64, 0.5)',
        tension: 0.2,
      },
    ],
  };

  // Prepare agent status chart data
  const agentChartData: ChartData<'bar'> = {
    labels: ['Active', 'Inactive', 'Error'],
    datasets: [
      {
        label: 'Agent Status',
        data: systemMetrics ? [
          systemMetrics.agents.active,
          systemMetrics.agents.inactive,
          systemMetrics.agents.error
        ] : [0, 0, 0],
        backgroundColor: [
          'rgba(75, 192, 192, 0.5)',
          'rgba(153, 102, 255, 0.5)',
          'rgba(255, 99, 132, 0.5)'
        ],
        borderColor: [
          'rgb(75, 192, 192)',
          'rgb(153, 102, 255)',
          'rgb(255, 99, 132)'
        ],
        borderWidth: 1,
      },
    ],
  };

  // Chart options
  const lineChartOptions = {
    responsive: true,
    maintainAspectRatio: false,
    scales: {
      y: {
        beginAtZero: true,
        ticks: {
          callback: (value: any) => `${value}${value === 'Messages per Minute' ? '' : '%'}`,
        },
      },
    },
    plugins: {
      legend: {
        position: 'top' as const,
      },
    },
  };

  const barChartOptions = {
    responsive: true,
    maintainAspectRatio: false,
    scales: {
      y: {
        beginAtZero: true,
      },
    },
    plugins: {
      legend: {
        display: false,
      },
    },
  };

  // Format bytes to human-readable format
  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  // Format uptime
  const formatUptime = (seconds?: number): string => {
    if (!seconds) return 'N/A';
    
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

  if (loading && !systemMetrics) {
    return (
      <div className="system-loading">
        <div className="spinner"></div>
        <p>Loading system health data...</p>
      </div>
    );
  }

  return (
    <div className="system-container">
      <div className="system-header">
        <h1>System Health</h1>
        <div className="time-range-selector">
          <button 
            className={timeRange === 'hour' ? 'active' : ''}
            onClick={() => handleTimeRangeChange('hour')}
          >
            Last Hour
          </button>
          <button 
            className={timeRange === 'day' ? 'active' : ''}
            onClick={() => handleTimeRangeChange('day')}
          >
            Last 24 Hours
          </button>
          <button 
            className={timeRange === 'week' ? 'active' : ''}
            onClick={() => handleTimeRangeChange('week')}
          >
            Last Week
          </button>
          <button 
            className={timeRange === 'month' ? 'active' : ''}
            onClick={() => handleTimeRangeChange('month')}
          >
            Last Month
          </button>
        </div>
      </div>
      
      {error && (
        <div className="system-error">
          <p>{error}</p>
          <button onClick={() => setError(null)}>Dismiss</button>
        </div>
      )}
      
      <div className="system-overview">
        <div className="system-card">
          <h2>System Overview</h2>
          <div className="system-stats">
            <div className="stat-item">
              <span className="stat-label">CPU Usage</span>
              <span className="stat-value">
                {systemMetrics?.system?.cpuUsage?.toFixed(1)}%
              </span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Memory Usage</span>
              <span className="stat-value">
                {systemMetrics?.system?.memoryUsage?.toFixed(1)}%
              </span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Total Memory</span>
              <span className="stat-value">
                {systemMetrics?.system?.memoryTotal ? formatBytes(systemMetrics.system.memoryTotal) : 'N/A'}
              </span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Uptime</span>
              <span className="stat-value">
                {formatUptime(systemMetrics?.system?.uptime)}
              </span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Load Average</span>
              <span className="stat-value">
                {systemMetrics?.system?.loadAverage?.map((load: number) => load.toFixed(2)).join(', ')}
              </span>
            </div>
          </div>
        </div>
        
        <div className="system-card">
          <h2>Agent Status</h2>
          <div className="agent-stats">
            <div className="stat-item">
              <span className="stat-label">Total Agents</span>
              <span className="stat-value">
                {systemMetrics?.agents?.total || 0}
              </span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Active</span>
              <span className="stat-value status-active">
                {systemMetrics?.agents?.active || 0}
              </span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Inactive</span>
              <span className="stat-value status-inactive">
                {systemMetrics?.agents?.inactive || 0}
              </span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Error</span>
              <span className="stat-value status-error">
                {systemMetrics?.agents?.error || 0}
              </span>
            </div>
          </div>
          <div className="agent-chart">
            <Bar data={agentChartData} options={barChartOptions} />
          </div>
        </div>
        
        <div className="system-card">
          <h2>Message Queue</h2>
          <div className="message-stats">
            <div className="stat-item">
              <span className="stat-label">Queue Size</span>
              <span className="stat-value">
                {systemMetrics?.messages?.queueSize || 0}
              </span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Messages/Minute</span>
              <span className="stat-value">
                {systemMetrics?.messages?.processedPerMinute || 0}
              </span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Total Processed</span>
              <span className="stat-value">
                {systemMetrics?.messages?.totalProcessed || 0}
              </span>
            </div>
          </div>
        </div>
        
        <div className="system-card">
          <h2>Task Status</h2>
          <div className="task-stats">
            <div className="stat-item">
              <span className="stat-label">Total Tasks</span>
              <span className="stat-value">
                {systemMetrics?.tasks?.total || 0}
              </span>
            </div>
            <div className="stat-item">
              <span className="stat-label">To Do</span>
              <span className="stat-value status-todo">
                {systemMetrics?.tasks?.todo || 0}
              </span>
            </div>
            <div className="stat-item">
              <span className="stat-label">In Progress</span>
              <span className="stat-value status-inprogress">
                {systemMetrics?.tasks?.inProgress || 0}
              </span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Done</span>
              <span className="stat-value status-done">
                {systemMetrics?.tasks?.done || 0}
              </span>
            </div>
            <div className="stat-item">
              <span className="stat-label">Blocked</span>
              <span className="stat-value status-blocked">
                {systemMetrics?.tasks?.blocked || 0}
              </span>
            </div>
          </div>
        </div>
      </div>
      
      <div className="system-charts">
        <div className="chart-container">
          <h2>CPU Usage History</h2>
          <div className="chart-wrapper">
            <Line data={cpuChartData} options={lineChartOptions} />
          </div>
        </div>
        
        <div className="chart-container">
          <h2>Memory Usage History</h2>
          <div className="chart-wrapper">
            <Line data={memoryChartData} options={lineChartOptions} />
          </div>
        </div>
        
        <div className="chart-container">
          <h2>Message Throughput</h2>
          <div className="chart-wrapper">
            <Line data={messageChartData} options={lineChartOptions} />
          </div>
        </div>
      </div>
      
      <div className="system-actions">
        <h2>System Actions</h2>
        <div className="action-buttons">
          <button className="action-button refresh" onClick={() => window.location.reload()}>
            Refresh Data
          </button>
          <button className="action-button export" onClick={() => alert('Export functionality not implemented yet')}>
            Export Metrics
          </button>
        </div>
      </div>
    </div>
  );
};

export default SystemHealth;