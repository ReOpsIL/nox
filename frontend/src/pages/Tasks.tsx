import React, { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import apiClient from '../api/api-client';
import { connectWebSocket } from '../api/websocket';
import './Tasks.css';

interface Task {
  id: string;
  agentId: string;
  title: string;
  description: string;
  status: 'todo' | 'inprogress' | 'done';
  priority: 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';
  createdAt: string;
  deadline?: string;
  dependencies: string[];
  requestedBy: string;
  progress?: number;
}

interface Agent {
  id: string;
  name: string;
}

/**
 * Tasks page component
 */
const Tasks: React.FC = () => {
  const { taskId } = useParams<{ taskId: string }>();
  const navigate = useNavigate();
  const [tasks, setTasks] = useState<Task[]>([]);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [selectedTask, setSelectedTask] = useState<Task | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [actionInProgress, setActionInProgress] = useState<string | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [filterStatus, setFilterStatus] = useState<string>('all');
  const [filterAgent, setFilterAgent] = useState<string>('all');
  const [filterPriority, setFilterPriority] = useState<string>('all');
  const [newTaskData, setNewTaskData] = useState({
    title: '',
    description: '',
    agentId: '',
    priority: 'MEDIUM' as 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL',
    deadline: ''
  });

  // Load tasks and agents data
  useEffect(() => {
    const fetchData = async () => {
      try {
        setLoading(true);
        
        // Get all tasks
        const tasksData = await apiClient.getTasks();
        setTasks(tasksData);
        
        // Get all agents for assignment
        const agentsData = await apiClient.getAgents();
        setAgents(agentsData);
        
        // If a task ID is specified in the URL, select that task
        if (taskId) {
          const task = tasksData.find(t => t.id === taskId);
          if (task) {
            setSelectedTask(task);
          } else {
            setError(`Task with ID ${taskId} not found`);
          }
        }
        
        setError(null);
      } catch (err) {
        console.error('Error fetching tasks:', err);
        setError('Failed to load tasks. Please try again later.');
      } finally {
        setLoading(false);
      }
    };
    
    fetchData();
    
    // Set up interval to refresh data
    const intervalId = setInterval(fetchData, 30000); // Refresh every 30 seconds
    
    return () => clearInterval(intervalId);
  }, [taskId]);

  // Connect to WebSocket for real-time updates
  useEffect(() => {
    const socket = connectWebSocket();
    
    // Listen for task created event
    socket.on('task_created', (data) => {
      setTasks(prevTasks => {
        // Check if task already exists
        if (prevTasks.some(task => task.id === data.taskId)) {
          return prevTasks;
        }
        
        // Add new task
        const newTask: Task = {
          id: data.taskId,
          agentId: data.agentId,
          title: data.title,
          description: '',
          status: data.status as 'todo' | 'inprogress' | 'done',
          priority: data.priority as 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL',
          createdAt: data.timestamp,
          dependencies: [],
          requestedBy: data.agentId,
          progress: 0
        };
        
        return [...prevTasks, newTask];
      });
    });
    
    // Listen for task updated event
    socket.on('task_updated', (data) => {
      setTasks(prevTasks => {
        return prevTasks.map(task => {
          if (task.id === data.taskId) {
            // Update task with new data
            return {
              ...task,
              title: data.title || task.title,
              status: data.status as 'todo' | 'inprogress' | 'done' || task.status,
              priority: data.priority as 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL' || task.priority,
              progress: data.progress !== undefined ? data.progress : task.progress
            };
          }
          return task;
        });
      });
      
      // Update selected task if needed
      if (selectedTask && selectedTask.id === data.taskId) {
        setSelectedTask(prevTask => {
          if (!prevTask) return null;
          
          return {
            ...prevTask,
            title: data.title || prevTask.title,
            status: data.status as 'todo' | 'inprogress' | 'done' || prevTask.status,
            priority: data.priority as 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL' || prevTask.priority,
            progress: data.progress !== undefined ? data.progress : prevTask.progress
          };
        });
      }
    });
    
    // Listen for task delegated event
    socket.on('task_delegated', (data) => {
      setTasks(prevTasks => {
        return prevTasks.map(task => {
          if (task.id === data.taskId) {
            // Update task with new agent
            return {
              ...task,
              agentId: data.toAgentId
            };
          }
          return task;
        });
      });
      
      // Update selected task if needed
      if (selectedTask && selectedTask.id === data.taskId) {
        setSelectedTask(prevTask => {
          if (!prevTask) return null;
          
          return {
            ...prevTask,
            agentId: data.toAgentId
          };
        });
      }
    });
    
    return () => {
      // No need to disconnect as the WebSocket client is a singleton
    };
  }, [selectedTask]);

  // Filter tasks based on selected filters
  const filteredTasks = tasks.filter(task => {
    // Filter by status
    if (filterStatus !== 'all' && task.status !== filterStatus) {
      return false;
    }
    
    // Filter by agent
    if (filterAgent !== 'all' && task.agentId !== filterAgent) {
      return false;
    }
    
    // Filter by priority
    if (filterPriority !== 'all' && task.priority !== filterPriority) {
      return false;
    }
    
    return true;
  });

  // Handle task selection
  const handleSelectTask = (task: Task) => {
    setSelectedTask(task);
    navigate(`/tasks/${task.id}`);
  };

  // Handle task actions
  const handleTaskAction = async (action: string, task: Task) => {
    try {
      setActionInProgress(action);
      
      switch (action) {
        case 'update-status': {
          // Cycle through statuses: todo -> inprogress -> done -> todo
          let newStatus: 'todo' | 'inprogress' | 'done';
          
          if (task.status === 'todo') {
            newStatus = 'inprogress';
          } else if (task.status === 'inprogress') {
            newStatus = 'done';
          } else {
            newStatus = 'todo';
          }
          
          await apiClient.updateTask(task.id, { ...task, status: newStatus });
          break;
        }
        case 'delete':
          if (window.confirm(`Are you sure you want to delete task "${task.title}"?`)) {
            await apiClient.deleteTask(task.id);
            if (selectedTask && selectedTask.id === task.id) {
              setSelectedTask(null);
              navigate('/tasks');
            }
          }
          break;
        default:
          console.warn(`Unknown action: ${action}`);
      }
      
      // Refresh tasks after action
      const updatedTasks = await apiClient.getTasks();
      setTasks(updatedTasks);
      
      // Update selected task if needed
      if (selectedTask && selectedTask.id === task.id) {
        const updatedTask = updatedTasks.find(t => t.id === task.id);
        if (updatedTask) {
          setSelectedTask(updatedTask);
        }
      }
      
    } catch (err) {
      console.error(`Error performing action ${action}:`, err);
      setError(`Failed to ${action} task. Please try again later.`);
    } finally {
      setActionInProgress(null);
    }
  };

  // Handle create task
  const handleCreateTask = async () => {
    try {
      setActionInProgress('create');
      
      // Validate input
      if (!newTaskData.title.trim()) {
        setError('Task title is required');
        return;
      }
      
      if (!newTaskData.agentId) {
        setError('Agent assignment is required');
        return;
      }
      
      // Create task
      const taskData = {
        ...newTaskData,
        status: 'todo' as 'todo',
        dependencies: [],
        requestedBy: 'user'
      };
      
      const createdTask = await apiClient.createTask(taskData);
      
      // Refresh tasks
      const updatedTasks = await apiClient.getTasks();
      setTasks(updatedTasks);
      
      // Select the new task
      setSelectedTask(createdTask);
      navigate(`/tasks/${createdTask.id}`);
      
      // Reset form and close modal
      setNewTaskData({
        title: '',
        description: '',
        agentId: '',
        priority: 'MEDIUM',
        deadline: ''
      });
      setShowCreateModal(false);
      setError(null);
      
    } catch (err) {
      console.error('Error creating task:', err);
      setError('Failed to create task. Please try again later.');
    } finally {
      setActionInProgress(null);
    }
  };

  // Get status class
  const getStatusClass = (status: string): string => {
    switch (status) {
      case 'todo':
        return 'status-todo';
      case 'inprogress':
        return 'status-inprogress';
      case 'done':
        return 'status-done';
      default:
        return '';
    }
  };

  // Get priority class
  const getPriorityClass = (priority: string): string => {
    switch (priority) {
      case 'LOW':
        return 'priority-low';
      case 'MEDIUM':
        return 'priority-medium';
      case 'HIGH':
        return 'priority-high';
      case 'CRITICAL':
        return 'priority-critical';
      default:
        return '';
    }
  };

  // Format date
  const formatDate = (dateString?: string): string => {
    if (!dateString) return 'N/A';
    return new Date(dateString).toLocaleString();
  };

  // Get agent name by ID
  const getAgentName = (agentId: string): string => {
    const agent = agents.find(a => a.id === agentId);
    return agent ? agent.name : `Agent ${agentId}`;
  };

  if (loading && tasks.length === 0) {
    return (
      <div className="tasks-loading">
        <div className="spinner"></div>
        <p>Loading tasks...</p>
      </div>
    );
  }

  return (
    <div className="tasks-container">
      <div className="tasks-header">
        <h1>Tasks</h1>
        <button 
          className="create-task-btn"
          onClick={() => setShowCreateModal(true)}
          disabled={actionInProgress === 'create'}
        >
          Create Task
        </button>
      </div>
      
      {error && (
        <div className="tasks-error">
          <p>{error}</p>
          <button onClick={() => setError(null)}>Dismiss</button>
        </div>
      )}
      
      <div className="tasks-filters">
        <div className="filter-group">
          <label htmlFor="status-filter">Status:</label>
          <select 
            id="status-filter" 
            value={filterStatus}
            onChange={(e) => setFilterStatus(e.target.value)}
          >
            <option value="all">All Statuses</option>
            <option value="todo">To Do</option>
            <option value="inprogress">In Progress</option>
            <option value="done">Done</option>
          </select>
        </div>
        
        <div className="filter-group">
          <label htmlFor="agent-filter">Agent:</label>
          <select 
            id="agent-filter" 
            value={filterAgent}
            onChange={(e) => setFilterAgent(e.target.value)}
          >
            <option value="all">All Agents</option>
            {agents.map(agent => (
              <option key={agent.id} value={agent.id}>{agent.name}</option>
            ))}
          </select>
        </div>
        
        <div className="filter-group">
          <label htmlFor="priority-filter">Priority:</label>
          <select 
            id="priority-filter" 
            value={filterPriority}
            onChange={(e) => setFilterPriority(e.target.value)}
          >
            <option value="all">All Priorities</option>
            <option value="LOW">Low</option>
            <option value="MEDIUM">Medium</option>
            <option value="HIGH">High</option>
            <option value="CRITICAL">Critical</option>
          </select>
        </div>
      </div>
      
      <div className="tasks-content">
        <div className="tasks-list">
          {filteredTasks.length === 0 ? (
            <div className="no-tasks">
              <p>No tasks found</p>
              <button 
                onClick={() => setShowCreateModal(true)}
                disabled={actionInProgress === 'create'}
              >
                Create your first task
              </button>
            </div>
          ) : (
            <ul>
              {filteredTasks.map(task => (
                <li 
                  key={task.id} 
                  className={`task-item ${selectedTask?.id === task.id ? 'selected' : ''}`}
                  onClick={() => handleSelectTask(task)}
                >
                  <div className="task-item-header">
                    <span className="task-title">{task.title}</span>
                    <span className={`task-priority ${getPriorityClass(task.priority)}`}>
                      {task.priority}
                    </span>
                  </div>
                  <div className="task-item-details">
                    <span className={`task-status ${getStatusClass(task.status)}`}>
                      {task.status}
                    </span>
                    <span className="task-agent">
                      {getAgentName(task.agentId)}
                    </span>
                  </div>
                  {task.progress !== undefined && task.progress > 0 && (
                    <div className="task-progress-bar">
                      <div 
                        className="task-progress-fill"
                        style={{ width: `${task.progress}%` }}
                      ></div>
                    </div>
                  )}
                </li>
              ))}
            </ul>
          )}
        </div>
        
        <div className="task-details">
          {selectedTask ? (
            <>
              <div className="task-details-header">
                <h2>{selectedTask.title}</h2>
                <div className="task-actions">
                  <button 
                    onClick={() => handleTaskAction('update-status', selectedTask)}
                    disabled={actionInProgress !== null}
                    className={`action-status ${getStatusClass(selectedTask.status)}`}
                  >
                    {selectedTask.status === 'todo' ? 'Start' : 
                     selectedTask.status === 'inprogress' ? 'Complete' : 'Reopen'}
                  </button>
                  
                  <button 
                    onClick={() => handleTaskAction('delete', selectedTask)}
                    disabled={actionInProgress !== null}
                    className="action-delete"
                  >
                    Delete
                  </button>
                </div>
              </div>
              
              <div className="task-details-content">
                <div className="task-info-section">
                  <h3>Task Information</h3>
                  <div className="task-info-grid">
                    <div className="info-item">
                      <span className="info-label">ID</span>
                      <span className="info-value">{selectedTask.id}</span>
                    </div>
                    <div className="info-item">
                      <span className="info-label">Status</span>
                      <span className={`info-value ${getStatusClass(selectedTask.status)}`}>
                        {selectedTask.status}
                      </span>
                    </div>
                    <div className="info-item">
                      <span className="info-label">Priority</span>
                      <span className={`info-value ${getPriorityClass(selectedTask.priority)}`}>
                        {selectedTask.priority}
                      </span>
                    </div>
                    <div className="info-item">
                      <span className="info-label">Assigned To</span>
                      <span className="info-value">
                        {getAgentName(selectedTask.agentId)}
                      </span>
                    </div>
                    <div className="info-item">
                      <span className="info-label">Created</span>
                      <span className="info-value">
                        {formatDate(selectedTask.createdAt)}
                      </span>
                    </div>
                    {selectedTask.deadline && (
                      <div className="info-item">
                        <span className="info-label">Deadline</span>
                        <span className="info-value">
                          {formatDate(selectedTask.deadline)}
                        </span>
                      </div>
                    )}
                    {selectedTask.progress !== undefined && (
                      <div className="info-item">
                        <span className="info-label">Progress</span>
                        <span className="info-value">
                          {selectedTask.progress}%
                        </span>
                      </div>
                    )}
                  </div>
                </div>
                
                <div className="task-description-section">
                  <h3>Description</h3>
                  <div className="task-description">
                    {selectedTask.description || 'No description provided'}
                  </div>
                </div>
                
                {selectedTask.dependencies.length > 0 && (
                  <div className="task-dependencies-section">
                    <h3>Dependencies</h3>
                    <ul className="dependencies-list">
                      {selectedTask.dependencies.map(depId => {
                        const depTask = tasks.find(t => t.id === depId);
                        return (
                          <li key={depId} className="dependency-item">
                            {depTask ? depTask.title : `Task ${depId}`}
                          </li>
                        );
                      })}
                    </ul>
                  </div>
                )}
              </div>
            </>
          ) : (
            <div className="no-task-selected">
              <p>Select a task to view details</p>
            </div>
          )}
        </div>
      </div>
      
      {/* Create Task Modal */}
      {showCreateModal && (
        <div className="modal-overlay">
          <div className="modal-content">
            <div className="modal-header">
              <h2>Create New Task</h2>
              <button 
                className="modal-close"
                onClick={() => setShowCreateModal(false)}
                disabled={actionInProgress === 'create'}
              >
                ×
              </button>
            </div>
            <div className="modal-body">
              <div className="form-group">
                <label htmlFor="task-title">Task Title</label>
                <input
                  id="task-title"
                  type="text"
                  value={newTaskData.title}
                  onChange={(e) => setNewTaskData({ ...newTaskData, title: e.target.value })}
                  placeholder="Enter task title"
                  disabled={actionInProgress === 'create'}
                />
              </div>
              <div className="form-group">
                <label htmlFor="task-description">Description</label>
                <textarea
                  id="task-description"
                  value={newTaskData.description}
                  onChange={(e) => setNewTaskData({ ...newTaskData, description: e.target.value })}
                  placeholder="Enter task description"
                  rows={4}
                  disabled={actionInProgress === 'create'}
                />
              </div>
              <div className="form-group">
                <label htmlFor="task-agent">Assign To</label>
                <select
                  id="task-agent"
                  value={newTaskData.agentId}
                  onChange={(e) => setNewTaskData({ ...newTaskData, agentId: e.target.value })}
                  disabled={actionInProgress === 'create'}
                >
                  <option value="">Select an agent</option>
                  {agents.map(agent => (
                    <option key={agent.id} value={agent.id}>{agent.name}</option>
                  ))}
                </select>
              </div>
              <div className="form-group">
                <label htmlFor="task-priority">Priority</label>
                <select
                  id="task-priority"
                  value={newTaskData.priority}
                  onChange={(e) => setNewTaskData({ 
                    ...newTaskData, 
                    priority: e.target.value as 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL'
                  })}
                  disabled={actionInProgress === 'create'}
                >
                  <option value="LOW">Low</option>
                  <option value="MEDIUM">Medium</option>
                  <option value="HIGH">High</option>
                  <option value="CRITICAL">Critical</option>
                </select>
              </div>
              <div className="form-group">
                <label htmlFor="task-deadline">Deadline (Optional)</label>
                <input
                  id="task-deadline"
                  type="datetime-local"
                  value={newTaskData.deadline}
                  onChange={(e) => setNewTaskData({ ...newTaskData, deadline: e.target.value })}
                  disabled={actionInProgress === 'create'}
                />
              </div>
            </div>
            <div className="modal-footer">
              <button
                className="modal-cancel"
                onClick={() => setShowCreateModal(false)}
                disabled={actionInProgress === 'create'}
              >
                Cancel
              </button>
              <button
                className="modal-submit"
                onClick={handleCreateTask}
                disabled={actionInProgress === 'create'}
              >
                {actionInProgress === 'create' ? 'Creating...' : 'Create Task'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default Tasks;