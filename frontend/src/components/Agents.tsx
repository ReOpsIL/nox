import React, { useState, useEffect } from 'react';
import { WebSocketClient } from '../api/websocket.ts';

interface AgentsProps {
  wsClient: WebSocketClient | null;
}

interface Agent {
  id: string;
  name: string;
  description: string;
  status: 'active' | 'idle' | 'stopped' | 'error';
  capabilities: string[];
  createdAt: string;
  lastActiveAt?: string;
  tasksCompleted: number;
  currentTask?: {
    id: string;
    title: string;
    status: string;
    progress: number;
    startedAt: string;
  } | string | null;
}

const Agents: React.FC<AgentsProps> = ({ wsClient }) => {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [showCreateTaskForm, setShowCreateTaskForm] = useState(false);
  const [selectedAgentForTask, setSelectedAgentForTask] = useState<string | null>(null);
  const [newAgent, setNewAgent] = useState({
    name: '',
    description: '',
    capabilities: ''
  });
  const [newTask, setNewTask] = useState({
    title: '',
    description: '',
    priority: 'MEDIUM',
    deadline: ''
  });

  useEffect(() => {
    fetchAgents();
    
    if (wsClient) {
      wsClient.on('agent_status_changed', handleAgentUpdate);
      wsClient.on('agent_created', fetchAgents);
      wsClient.on('agent_deleted', fetchAgents);
    }

    return () => {
      if (wsClient) {
        wsClient.off('agent_status_changed', handleAgentUpdate);
        wsClient.off('agent_created', fetchAgents);
        wsClient.off('agent_deleted', fetchAgents);
      }
    };
  }, [wsClient]);

  const fetchAgents = async () => {
    try {
      setLoading(true);
      const response = await fetch('/api/agents');
      if (response.ok) {
        const data = await response.json();
        // Handle API response format with success/data wrappers
        const agentsArray = data.success ? (data.agents || data.data || []) : [];
        setAgents(Array.isArray(agentsArray) ? agentsArray : []);
        setError(null);
      } else {
        setError('Failed to fetch agents');
      }
    } catch (err) {
      setError('Error fetching agents');
      console.error('Error fetching agents:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleAgentUpdate = (data: any) => {
    setAgents(prev => prev.map(agent => 
      agent.id === data.id ? { ...agent, ...data } : agent
    ));
  };

  const createAgent = async () => {
    try {
      const response = await fetch('/api/agents', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          name: newAgent.name,
          description: newAgent.description,
          capabilities: newAgent.capabilities.split(',').map(c => c.trim()).filter(c => c)
        }),
      });

      if (response.ok) {
        setShowCreateForm(false);
        setNewAgent({ name: '', description: '', capabilities: '' });
        fetchAgents();
      } else if (response.status === 501) {
        setError('Agent creation is not yet implemented');
      } else {
        setError('Failed to create agent');
      }
    } catch (err) {
      setError('Error creating agent');
      console.error('Error creating agent:', err);
    }
  };

  const createTask = async () => {
    if (!selectedAgentForTask) {
      setError('No agent selected for task creation');
      return;
    }

    try {
      const taskData = {
        agentId: selectedAgentForTask,
        title: newTask.title,
        description: newTask.description,
        priority: newTask.priority,
        deadline: newTask.deadline ? new Date(newTask.deadline).toISOString() : undefined,
        requestedBy: 'user'
      };

      const response = await fetch('/api/tasks', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(taskData),
      });

      if (response.ok) {
        setShowCreateTaskForm(false);
        setSelectedAgentForTask(null);
        setNewTask({ title: '', description: '', priority: 'MEDIUM', deadline: '' });
        // Optionally refresh agents to show updated task count
        fetchAgents();
      } else {
        const errorData = await response.json().catch(() => ({}));
        setError(errorData.message || 'Failed to create task');
      }
    } catch (err) {
      setError('Error creating task');
      console.error('Error creating task:', err);
    }
  };

  const openCreateTaskForm = (agentId: string) => {
    setSelectedAgentForTask(agentId);
    setShowCreateTaskForm(true);
  };

  const controlAgent = async (agentId: string, action: 'start' | 'stop') => {
    try {
      const response = await fetch(`/api/agents/${agentId}/${action}`, {
        method: 'POST',
      });

      if (!response.ok) {
        setError(`Failed to ${action} agent`);
      }
    } catch (err) {
      setError(`Error ${action}ing agent`);
      console.error(`Error ${action}ing agent:`, err);
    }
  };

  const deleteAgent = async (agentId: string) => {
    if (!window.confirm('Are you sure you want to delete this agent?')) {
      return;
    }

    try {
      const response = await fetch(`/api/agents/${agentId}`, {
        method: 'DELETE',
      });

      if (response.ok) {
        fetchAgents();
      } else {
        setError('Failed to delete agent');
      }
    } catch (err) {
      setError('Error deleting agent');
      console.error('Error deleting agent:', err);
    }
  };

  const deleteAllAgents = async () => {
    if (agents.length === 0) {
      setError('No agents to delete');
      return;
    }

    const confirmMessage = `Are you sure you want to delete ALL ${agents.length} agents and their tasks? This action cannot be undone.`;
    if (!window.confirm(confirmMessage)) {
      return;
    }

    try {
      setLoading(true);
      const response = await fetch('/api/agents/delete-all', {
        method: 'DELETE',
      });

      if (response.ok) {
        setAgents([]);
        setError(null);
      } else {
        const errorData = await response.json().catch(() => ({}));
        setError(errorData.message || 'Failed to delete all agents');
      }
    } catch (err) {
      setError('Error deleting all agents');
      console.error('Error deleting all agents:', err);
    } finally {
      setLoading(false);
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'active': return '🟢';
      case 'idle': return '🟡';
      case 'stopped': return '🔴';
      case 'error': return '❌';
      default: return '⚪';
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'green';
      case 'idle': return 'orange';
      case 'stopped': return 'red';
      case 'error': return 'red';
      default: return 'gray';
    }
  };

  if (loading) {
    return <div className="loading">Loading agents...</div>;
  }

  return (
    <div className="agents-page">
      <div className="page-header">
        <h1>👥 Agent Management</h1>
        <div className="header-actions">
          <button 
            className="btn btn-primary"
            onClick={() => setShowCreateForm(true)}
          >
            + Create Agent
          </button>
          {agents.length > 0 && (
            <button 
              className="btn btn-danger"
              onClick={deleteAllAgents}
              title="Delete all agents and their tasks"
            >
              🗑️ Delete All ({agents.length})
            </button>
          )}
        </div>
      </div>

      {error && (
        <div className="error-banner">
          {error}
          <button onClick={() => setError(null)}>×</button>
        </div>
      )}

      {showCreateForm && (
        <div className="modal-overlay">
          <div className="modal">
            <div className="modal-header">
              <h2>Create New Agent</h2>
              <button onClick={() => setShowCreateForm(false)}>×</button>
            </div>
            <div className="modal-body">
              <div className="form-group">
                <label>Name:</label>
                <input
                  type="text"
                  value={newAgent.name}
                  onChange={(e) => setNewAgent(prev => ({ ...prev, name: e.target.value }))}
                  placeholder="Agent name"
                />
              </div>
              <div className="form-group">
                <label>Description:</label>
                <textarea
                  value={newAgent.description}
                  onChange={(e) => setNewAgent(prev => ({ ...prev, description: e.target.value }))}
                  placeholder="What does this agent do?"
                />
              </div>
              <div className="form-group">
                <label>Capabilities (comma-separated):</label>
                <input
                  type="text"
                  value={newAgent.capabilities}
                  onChange={(e) => setNewAgent(prev => ({ ...prev, capabilities: e.target.value }))}
                  placeholder="code-review, testing, documentation"
                />
              </div>
            </div>
            <div className="modal-footer">
              <button className="btn btn-secondary" onClick={() => setShowCreateForm(false)}>
                Cancel
              </button>
              <button className="btn btn-primary" onClick={createAgent}>
                Create Agent
              </button>
            </div>
          </div>
        </div>
      )}

      {showCreateTaskForm && (
        <div className="modal-overlay">
          <div className="modal">
            <div className="modal-header">
              <h2>Create New Task</h2>
              <button onClick={() => {
                setShowCreateTaskForm(false);
                setSelectedAgentForTask(null);
              }}>×</button>
            </div>
            <div className="modal-body">
              <div className="form-group">
                <label>Agent:</label>
                <input
                  type="text"
                  value={agents.find(a => a.id === selectedAgentForTask)?.name || ''}
                  disabled
                  className="disabled"
                />
              </div>
              <div className="form-group">
                <label>Task Title:</label>
                <input
                  type="text"
                  value={newTask.title}
                  onChange={(e) => setNewTask(prev => ({ ...prev, title: e.target.value }))}
                  placeholder="Task title"
                  required
                />
              </div>
              <div className="form-group">
                <label>Description:</label>
                <textarea
                  value={newTask.description}
                  onChange={(e) => setNewTask(prev => ({ ...prev, description: e.target.value }))}
                  placeholder="Describe what the agent should do..."
                  rows={3}
                />
              </div>
              <div className="form-group">
                <label>Priority:</label>
                <select
                  value={newTask.priority}
                  onChange={(e) => setNewTask(prev => ({ ...prev, priority: e.target.value }))}
                >
                  <option value="LOW">Low</option>
                  <option value="MEDIUM">Medium</option>
                  <option value="HIGH">High</option>
                  <option value="CRITICAL">Critical</option>
                </select>
              </div>
              <div className="form-group">
                <label>Deadline (optional):</label>
                <input
                  type="datetime-local"
                  value={newTask.deadline}
                  onChange={(e) => setNewTask(prev => ({ ...prev, deadline: e.target.value }))}
                />
              </div>
            </div>
            <div className="modal-footer">
              <button 
                className="btn btn-secondary" 
                onClick={() => {
                  setShowCreateTaskForm(false);
                  setSelectedAgentForTask(null);
                }}
              >
                Cancel
              </button>
              <button 
                className="btn btn-primary" 
                onClick={createTask}
                disabled={!newTask.title.trim()}
              >
                Create Task
              </button>
            </div>
          </div>
        </div>
      )}

      <div className="agents-grid">
        {agents.length === 0 ? (
          <div className="empty-state">
            <h3>No agents found</h3>
            <p>Create your first agent to get started</p>
          </div>
        ) : (
          agents.map((agent) => (
            <div key={agent.id} className="agent-card">
              <div className="agent-header">
                <div className="agent-info">
                  <h3>{agent.name}</h3>
                  <span className={`status-badge ${getStatusColor(agent.status)}`}>
                    {getStatusIcon(agent.status)} {agent.status}
                  </span>
                </div>
                <div className="agent-actions">
                  <button 
                    className="btn btn-primary btn-sm"
                    onClick={() => openCreateTaskForm(agent.id)}
                    title="Create new task for this agent"
                  >
                    + Task
                  </button>
                  {agent.status === 'stopped' ? (
                    <button 
                      className="btn btn-success btn-sm"
                      onClick={() => controlAgent(agent.id, 'start')}
                    >
                      Start
                    </button>
                  ) : (
                    <button 
                      className="btn btn-warning btn-sm"
                      onClick={() => controlAgent(agent.id, 'stop')}
                    >
                      Stop
                    </button>
                  )}
                  <button 
                    className="btn btn-danger btn-sm"
                    onClick={() => deleteAgent(agent.id)}
                  >
                    Delete
                  </button>
                </div>
              </div>

              <div className="agent-body">
                <p className="agent-description">{agent.description}</p>
                
                <div className="agent-capabilities">
                  <strong>Capabilities:</strong>
                  <div className="capabilities-list">
                    {agent.capabilities.map((cap, index) => (
                      <span key={index} className="capability-tag">
                        {cap}
                      </span>
                    ))}
                  </div>
                </div>

                <div className="agent-stats">
                  <div className="stat">
                    <span className="stat-label">Tasks Completed:</span>
                    <span className="stat-value">{agent.tasksCompleted}</span>
                  </div>
                  <div className="stat">
                    <span className="stat-label">Created:</span>
                    <span className="stat-value">
                      {new Date(agent.createdAt).toLocaleDateString()}
                    </span>
                  </div>
                  {agent.lastActiveAt && (
                    <div className="stat">
                      <span className="stat-label">Last Active:</span>
                      <span className="stat-value">
                        {new Date(agent.lastActiveAt).toLocaleString()}
                      </span>
                    </div>
                  )}
                </div>

                {agent.currentTask && (
                  <div className="current-task">
                    <strong>Current Task:</strong>
                    <span>
                      {typeof agent.currentTask === 'object' && agent.currentTask !== null
                        ? (agent.currentTask.title || 'Unknown Task')
                        : String(agent.currentTask || 'No task')}
                    </span>
                  </div>
                )}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default Agents;