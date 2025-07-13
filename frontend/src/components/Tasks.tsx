import React, { useState, useEffect, useCallback } from 'react';
import { WebSocketClient } from '../api/websocket.ts';

interface TasksProps {
  wsClient: WebSocketClient | null;
}

interface Task {
  id: string;
  title: string;
  description: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  priority: 'low' | 'medium' | 'high' | 'critical';
  assignedAgent?: string;
  createdAt: string;
  startedAt?: string;
  completedAt?: string;
  estimatedTime?: number;
  actualTime?: number;
  progress?: number;
  results?: string;
  error?: string;
}

const Tasks: React.FC<TasksProps> = ({ wsClient }) => {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<string>('all');
  const [selectedTask, setSelectedTask] = useState<Task | null>(null);
  const [autoRefresh, setAutoRefresh] = useState(false);
  const [refreshInterval, setRefreshInterval] = useState<NodeJS.Timeout | null>(null);

  const fetchTasks = async () => {
    try {
      setLoading(true);
      const response = await fetch('/api/tasks');
      if (response.ok) {
        const data = await response.json();
        // Handle API response format with success/data wrappers
        const tasksArray = data.success ? (data.tasks || data.data || []) : [];
        setTasks(Array.isArray(tasksArray) ? tasksArray : []);
        setError(null);
      } else {
        setError('Failed to fetch tasks');
      }
    } catch (err) {
      setError('Error fetching tasks');
      console.error('Error fetching tasks:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleTaskUpdate = useCallback((data: any) => {
    setTasks(prev => prev.map(task => 
      task.id === data.id ? { ...task, ...data } : task
    ));
    
    // Update selected task if it's currently being viewed
    if (selectedTask && selectedTask.id === data.id) {
      setSelectedTask(prev => prev ? { ...prev, ...data } : null);
    }
  }, [selectedTask]);

  useEffect(() => {
    fetchTasks();
    
    if (wsClient) {
      wsClient.on('task_started', handleTaskUpdate);
      wsClient.on('task_completed', handleTaskUpdate);
      wsClient.on('task_failed', handleTaskUpdate);
      wsClient.on('task_progress', handleTaskUpdate);
      wsClient.on('agent_deleted', fetchTasks); // Refresh tasks when agent is deleted
      wsClient.on('agents_deleted_all', fetchTasks); // Refresh tasks when all agents are deleted
      wsClient.on('task_created', fetchTasks); // Refresh tasks when new task is created
      wsClient.on('task_deleted', fetchTasks); // Refresh tasks when task is deleted
    }

    return () => {
      if (wsClient) {
        wsClient.off('task_started', handleTaskUpdate);
        wsClient.off('task_completed', handleTaskUpdate);
        wsClient.off('task_failed', handleTaskUpdate);
        wsClient.off('task_progress', handleTaskUpdate);
        wsClient.off('agent_deleted', fetchTasks);
        wsClient.off('agents_deleted_all', fetchTasks);
        wsClient.off('task_created', fetchTasks);
        wsClient.off('task_deleted', fetchTasks);
      }
    };
  }, [wsClient, handleTaskUpdate]);

  // Auto-refresh effect for task execution updates
  useEffect(() => {
    if (autoRefresh) {
      const interval = setInterval(() => {
        fetchTasks(); // Refresh task execution status
      }, 3000); // Refresh every 3 seconds for more responsive task updates
      setRefreshInterval(interval);
      
      return () => {
        clearInterval(interval);
      };
    } else {
      if (refreshInterval) {
        clearInterval(refreshInterval);
        setRefreshInterval(null);
      }
    }
  }, [autoRefresh, refreshInterval]);

  const cancelTask = async (taskId: string) => {
    try {
      const response = await fetch(`/api/tasks/${taskId}/cancel`, {
        method: 'POST',
      });

      if (!response.ok) {
        setError('Failed to cancel task');
      }
    } catch (err) {
      setError('Error cancelling task');
      console.error('Error cancelling task:', err);
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'pending': return '⏳';
      case 'running': return '⚡';
      case 'completed': return '✅';
      case 'failed': return '❌';
      case 'cancelled': return '🚫';
      default: return '❓';
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'pending': return 'orange';
      case 'running': return 'blue';
      case 'completed': return 'green';
      case 'failed': return 'red';
      case 'cancelled': return 'gray';
      default: return 'gray';
    }
  };

  const getPriorityIcon = (priority: string) => {
    switch (priority) {
      case 'low': return '🟢';
      case 'medium': return '🟡';
      case 'high': return '🟠';
      case 'critical': return '🔴';
      default: return '⚪';
    }
  };

  const formatDuration = (milliseconds: number) => {
    const seconds = Math.floor(milliseconds / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    
    if (hours > 0) {
      return `${hours}h ${minutes % 60}m`;
    } else if (minutes > 0) {
      return `${minutes}m ${seconds % 60}s`;
    } else {
      return `${seconds}s`;
    }
  };

  const filteredTasks = tasks.filter(task => {
    if (filter === 'all') return true;
    return task.status === filter;
  });

  if (loading) {
    return <div className="loading">Loading tasks...</div>;
  }

  return (
    <div className="tasks-page">
      <div className="page-header">
        <h1>📋 Task Management</h1>
        <div className="header-actions">
          <div className="refresh-controls">
            <label className="auto-refresh-label">
              <input
                type="checkbox"
                checked={autoRefresh}
                onChange={(e) => setAutoRefresh(e.target.checked)}
              />
              Auto-refresh (3s)
            </label>
            <button 
              className="btn btn-secondary btn-sm"
              onClick={fetchTasks}
              title="Manually refresh tasks"
              disabled={loading}
            >
              🔄 Refresh
            </button>
          </div>
        </div>
      </div>
      
      <div className="task-filters-section">
        <div className="task-filters">
          <button 
            className={`filter-btn ${filter === 'all' ? 'active' : ''}`}
            onClick={() => setFilter('all')}
          >
            All Tasks
          </button>
          <button 
            className={`filter-btn ${filter === 'pending' ? 'active' : ''}`}
            onClick={() => setFilter('pending')}
          >
            Pending
          </button>
          <button 
            className={`filter-btn ${filter === 'running' ? 'active' : ''}`}
            onClick={() => setFilter('running')}
          >
            Running
          </button>
          <button 
            className={`filter-btn ${filter === 'completed' ? 'active' : ''}`}
            onClick={() => setFilter('completed')}
          >
            Completed
          </button>
          <button 
            className={`filter-btn ${filter === 'failed' ? 'active' : ''}`}
            onClick={() => setFilter('failed')}
          >
            Failed
          </button>
        </div>
      </div>

      {error && (
        <div className="error-banner">
          {error}
          <button onClick={() => setError(null)}>×</button>
        </div>
      )}

      <div className="tasks-content">
        <div className="tasks-list">
          {filteredTasks.length === 0 ? (
            <div className="empty-state">
              <h3>No tasks found</h3>
              <p>
                {filter === 'all' 
                  ? 'No tasks have been created yet'
                  : `No ${filter} tasks found`
                }
              </p>
            </div>
          ) : (
            filteredTasks.map((task) => (
              <div 
                key={task.id} 
                className={`task-card ${selectedTask?.id === task.id ? 'selected' : ''}`}
                onClick={() => setSelectedTask(task)}
              >
                <div className="task-header">
                  <div className="task-title">
                    <h3>{task.title}</h3>
                    <div className="task-badges">
                      <span className={`status-badge ${getStatusColor(task.status)}`}>
                        {getStatusIcon(task.status)} {task.status}
                      </span>
                      <span className="priority-badge">
                        {getPriorityIcon(task.priority)} {task.priority}
                      </span>
                    </div>
                  </div>
                  {(task.status === 'pending' || task.status === 'running') && (
                    <button 
                      className="btn btn-danger btn-sm"
                      onClick={(e) => {
                        e.stopPropagation();
                        cancelTask(task.id);
                      }}
                    >
                      Cancel
                    </button>
                  )}
                </div>

                <div className="task-body">
                  <p className="task-description">{task.description}</p>
                  
                  {task.assignedAgent && (
                    <div className="task-agent">
                      <strong>Assigned to:</strong> {task.assignedAgent}
                    </div>
                  )}

                  {task.progress !== undefined && task.status === 'running' && (
                    <div className="task-progress">
                      <div className="progress-bar">
                        <div 
                          className="progress-fill" 
                          style={{ width: `${task.progress}%` }}
                        ></div>
                      </div>
                      <span className="progress-text">{task.progress}%</span>
                    </div>
                  )}

                  <div className="task-timestamps">
                    <span>Created: {new Date(task.createdAt).toLocaleString()}</span>
                    {task.startedAt && (
                      <span>Started: {new Date(task.startedAt).toLocaleString()}</span>
                    )}
                    {task.completedAt && (
                      <span>Completed: {new Date(task.completedAt).toLocaleString()}</span>
                    )}
                  </div>

                  {task.actualTime && (
                    <div className="task-duration">
                      Duration: {formatDuration(task.actualTime)}
                    </div>
                  )}

                  {/* Show task output preview */}
                  {task.results && (
                    <div className="task-output-preview">
                      <strong>Output:</strong>
                      <div className="output-snippet">
                        {task.results.length > 150 
                          ? `${task.results.substring(0, 150)}...` 
                          : task.results
                        }
                      </div>
                    </div>
                  )}

                  {task.error && (
                    <div className="task-error-preview">
                      <strong>Error:</strong>
                      <div className="error-snippet">
                        {task.error.length > 150 
                          ? `${task.error.substring(0, 150)}...` 
                          : task.error
                        }
                      </div>
                    </div>
                  )}
                </div>
              </div>
            ))
          )}
        </div>

        {selectedTask && (
          <div className="task-details">
            <div className="task-details-header">
              <h2>{selectedTask.title}</h2>
              <button onClick={() => setSelectedTask(null)}>×</button>
            </div>
            
            <div className="task-details-body">
              <div className="detail-section">
                <h3>Description</h3>
                <p>{selectedTask.description}</p>
              </div>

              <div className="detail-section">
                <h3>Status Information</h3>
                <div className="status-info">
                  <div className="status-item">
                    <span>Status:</span>
                    <span className={`status-badge ${getStatusColor(selectedTask.status)}`}>
                      {getStatusIcon(selectedTask.status)} {selectedTask.status}
                    </span>
                  </div>
                  <div className="status-item">
                    <span>Priority:</span>
                    <span className="priority-badge">
                      {getPriorityIcon(selectedTask.priority)} {selectedTask.priority}
                    </span>
                  </div>
                  {selectedTask.assignedAgent && (
                    <div className="status-item">
                      <span>Assigned Agent:</span>
                      <span>{selectedTask.assignedAgent}</span>
                    </div>
                  )}
                </div>
              </div>

              {selectedTask.results && (
                <div className="detail-section">
                  <h3>Results</h3>
                  <pre className="task-results">{selectedTask.results}</pre>
                </div>
              )}

              {selectedTask.error && (
                <div className="detail-section">
                  <h3>Error</h3>
                  <pre className="task-error">{selectedTask.error}</pre>
                </div>
              )}

              <div className="detail-section">
                <h3>Timing</h3>
                <div className="timing-info">
                  <div>Created: {new Date(selectedTask.createdAt).toLocaleString()}</div>
                  {selectedTask.startedAt && (
                    <div>Started: {new Date(selectedTask.startedAt).toLocaleString()}</div>
                  )}
                  {selectedTask.completedAt && (
                    <div>Completed: {new Date(selectedTask.completedAt).toLocaleString()}</div>
                  )}
                  {selectedTask.estimatedTime && (
                    <div>Estimated Time: {formatDuration(selectedTask.estimatedTime)}</div>
                  )}
                  {selectedTask.actualTime && (
                    <div>Actual Time: {formatDuration(selectedTask.actualTime)}</div>
                  )}
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default Tasks;