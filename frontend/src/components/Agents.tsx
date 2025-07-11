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
  currentTask?: string;
}

const Agents: React.FC<AgentsProps> = ({ wsClient }) => {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newAgent, setNewAgent] = useState({
    name: '',
    description: '',
    capabilities: ''
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
        <button 
          className="btn btn-primary"
          onClick={() => setShowCreateForm(true)}
        >
          + Create Agent
        </button>
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
                    <span>{agent.currentTask}</span>
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