import React, { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import apiClient from '../api/api-client';
import { connectWebSocket } from '../api/websocket';
import './Agents.css';

interface Agent {
  id: string;
  name: string;
  status: string;
  systemPrompt: string;
  createdAt: string;
  lastModified: string;
  cpuUsage?: number;
  memoryUsage?: number;
  uptime?: number;
}

/**
 * Agents page component
 */
const Agents: React.FC = () => {
  const { agentId } = useParams<{ agentId: string }>();
  const navigate = useNavigate();
  const [agents, setAgents] = useState<Agent[]>([]);
  const [selectedAgent, setSelectedAgent] = useState<Agent | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [actionInProgress, setActionInProgress] = useState<string | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newAgentData, setNewAgentData] = useState({
    name: '',
    systemPrompt: ''
  });

  // Load agents data
  useEffect(() => {
    const fetchAgents = async () => {
      try {
        setLoading(true);
        const data = await apiClient.getAgents();
        setAgents(data);
        
        // If an agent ID is specified in the URL, select that agent
        if (agentId) {
          const agent = data.find(a => a.id === agentId);
          if (agent) {
            setSelectedAgent(agent);
          } else {
            setError(`Agent with ID ${agentId} not found`);
          }
        }
        
        setError(null);
      } catch (err) {
        console.error('Error fetching agents:', err);
        setError('Failed to load agents. Please try again later.');
      } finally {
        setLoading(false);
      }
    };
    
    fetchAgents();
    
    // Set up interval to refresh data
    const intervalId = setInterval(fetchAgents, 30000); // Refresh every 30 seconds
    
    return () => clearInterval(intervalId);
  }, [agentId]);

  // Connect to WebSocket for real-time updates
  useEffect(() => {
    const socket = connectWebSocket();
    
    // Listen for agent status updates
    socket.on('agent_status_list', (data) => {
      setAgents(prevAgents => {
        // Update existing agents with new status
        const updatedAgents = prevAgents.map(agent => {
          const updatedAgent = data.find((a: any) => a.id === agent.id);
          if (updatedAgent) {
            return { ...agent, ...updatedAgent };
          }
          return agent;
        });
        
        // Update selected agent if needed
        if (selectedAgent) {
          const updatedSelectedAgent = data.find((a: any) => a.id === selectedAgent.id);
          if (updatedSelectedAgent) {
            setSelectedAgent({ ...selectedAgent, ...updatedSelectedAgent });
          }
        }
        
        return updatedAgents;
      });
    });
    
    // Listen for agent created event
    socket.on('agent_created', (data) => {
      setAgents(prevAgents => {
        // Check if agent already exists
        if (prevAgents.some(agent => agent.id === data.agentId)) {
          return prevAgents;
        }
        
        // Add new agent
        return [...prevAgents, {
          id: data.agentId,
          name: data.name,
          status: 'inactive',
          systemPrompt: '',
          createdAt: data.timestamp,
          lastModified: data.timestamp
        }];
      });
    });
    
    // Listen for agent deleted event
    socket.on('agent_deleted', (data) => {
      setAgents(prevAgents => prevAgents.filter(agent => agent.id !== data.agentId));
      
      // If the deleted agent is selected, deselect it
      if (selectedAgent && selectedAgent.id === data.agentId) {
        setSelectedAgent(null);
        navigate('/agents');
      }
    });
    
    return () => {
      // No need to disconnect as the WebSocket client is a singleton
    };
  }, [selectedAgent, navigate]);

  // Handle agent selection
  const handleSelectAgent = (agent: Agent) => {
    setSelectedAgent(agent);
    navigate(`/agents/${agent.id}`);
  };

  // Handle agent actions
  const handleAgentAction = async (action: string, agent: Agent) => {
    try {
      setActionInProgress(action);
      
      switch (action) {
        case 'start':
          await apiClient.startAgent(agent.id);
          break;
        case 'stop':
          await apiClient.stopAgent(agent.id);
          break;
        case 'restart':
          await apiClient.restartAgent(agent.id);
          break;
        case 'delete':
          if (window.confirm(`Are you sure you want to delete agent "${agent.name}"?`)) {
            await apiClient.deleteAgent(agent.id);
            if (selectedAgent && selectedAgent.id === agent.id) {
              setSelectedAgent(null);
              navigate('/agents');
            }
          }
          break;
        default:
          console.warn(`Unknown action: ${action}`);
      }
      
      // Refresh agents after action
      const updatedAgents = await apiClient.getAgents();
      setAgents(updatedAgents);
      
      // Update selected agent if needed
      if (selectedAgent && selectedAgent.id === agent.id) {
        const updatedAgent = updatedAgents.find(a => a.id === agent.id);
        if (updatedAgent) {
          setSelectedAgent(updatedAgent);
        }
      }
      
    } catch (err) {
      console.error(`Error performing action ${action}:`, err);
      setError(`Failed to ${action} agent. Please try again later.`);
    } finally {
      setActionInProgress(null);
    }
  };

  // Handle create agent
  const handleCreateAgent = async () => {
    try {
      setActionInProgress('create');
      
      // Validate input
      if (!newAgentData.name.trim()) {
        setError('Agent name is required');
        return;
      }
      
      if (!newAgentData.systemPrompt.trim()) {
        setError('System prompt is required');
        return;
      }
      
      // Create agent
      const createdAgent = await apiClient.createAgent(newAgentData);
      
      // Refresh agents
      const updatedAgents = await apiClient.getAgents();
      setAgents(updatedAgents);
      
      // Select the new agent
      setSelectedAgent(createdAgent);
      navigate(`/agents/${createdAgent.id}`);
      
      // Reset form and close modal
      setNewAgentData({ name: '', systemPrompt: '' });
      setShowCreateModal(false);
      setError(null);
      
    } catch (err) {
      console.error('Error creating agent:', err);
      setError('Failed to create agent. Please try again later.');
    } finally {
      setActionInProgress(null);
    }
  };

  // Format uptime
  const formatUptime = (seconds?: number): string => {
    if (!seconds) return 'N/A';
    
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
  };

  // Get status class
  const getStatusClass = (status: string): string => {
    switch (status) {
      case 'running':
        return 'status-running';
      case 'stopped':
        return 'status-stopped';
      case 'crashed':
        return 'status-error';
      default:
        return 'status-unknown';
    }
  };

  if (loading && agents.length === 0) {
    return (
      <div className="agents-loading">
        <div className="spinner"></div>
        <p>Loading agents...</p>
      </div>
    );
  }

  return (
    <div className="agents-container">
      <div className="agents-header">
        <h1>Agents</h1>
        <button 
          className="create-agent-btn"
          onClick={() => setShowCreateModal(true)}
          disabled={actionInProgress === 'create'}
        >
          Create Agent
        </button>
      </div>
      
      {error && (
        <div className="agents-error">
          <p>{error}</p>
          <button onClick={() => setError(null)}>Dismiss</button>
        </div>
      )}
      
      <div className="agents-content">
        <div className="agents-list">
          {agents.length === 0 ? (
            <div className="no-agents">
              <p>No agents found</p>
              <button 
                onClick={() => setShowCreateModal(true)}
                disabled={actionInProgress === 'create'}
              >
                Create your first agent
              </button>
            </div>
          ) : (
            <ul>
              {agents.map(agent => (
                <li 
                  key={agent.id} 
                  className={`agent-item ${selectedAgent?.id === agent.id ? 'selected' : ''}`}
                  onClick={() => handleSelectAgent(agent)}
                >
                  <div className="agent-item-header">
                    <span className="agent-name">{agent.name}</span>
                    <span className={`agent-status ${getStatusClass(agent.status)}`}>
                      {agent.status}
                    </span>
                  </div>
                  <div className="agent-item-details">
                    <span className="agent-id">ID: {agent.id}</span>
                    <span className="agent-created">
                      Created: {new Date(agent.createdAt).toLocaleString()}
                    </span>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </div>
        
        <div className="agent-details">
          {selectedAgent ? (
            <>
              <div className="agent-details-header">
                <h2>{selectedAgent.name}</h2>
                <div className="agent-actions">
                  {selectedAgent.status !== 'running' && (
                    <button 
                      onClick={() => handleAgentAction('start', selectedAgent)}
                      disabled={actionInProgress !== null}
                      className="action-start"
                    >
                      Start
                    </button>
                  )}
                  
                  {selectedAgent.status === 'running' && (
                    <button 
                      onClick={() => handleAgentAction('stop', selectedAgent)}
                      disabled={actionInProgress !== null}
                      className="action-stop"
                    >
                      Stop
                    </button>
                  )}
                  
                  {selectedAgent.status === 'running' && (
                    <button 
                      onClick={() => handleAgentAction('restart', selectedAgent)}
                      disabled={actionInProgress !== null}
                      className="action-restart"
                    >
                      Restart
                    </button>
                  )}
                  
                  <button 
                    onClick={() => handleAgentAction('delete', selectedAgent)}
                    disabled={actionInProgress !== null}
                    className="action-delete"
                  >
                    Delete
                  </button>
                </div>
              </div>
              
              <div className="agent-details-content">
                <div className="agent-info-section">
                  <h3>Agent Information</h3>
                  <div className="agent-info-grid">
                    <div className="info-item">
                      <span className="info-label">ID</span>
                      <span className="info-value">{selectedAgent.id}</span>
                    </div>
                    <div className="info-item">
                      <span className="info-label">Status</span>
                      <span className={`info-value ${getStatusClass(selectedAgent.status)}`}>
                        {selectedAgent.status}
                      </span>
                    </div>
                    <div className="info-item">
                      <span className="info-label">Created</span>
                      <span className="info-value">
                        {new Date(selectedAgent.createdAt).toLocaleString()}
                      </span>
                    </div>
                    <div className="info-item">
                      <span className="info-label">Last Modified</span>
                      <span className="info-value">
                        {new Date(selectedAgent.lastModified).toLocaleString()}
                      </span>
                    </div>
                    {selectedAgent.status === 'running' && (
                      <>
                        <div className="info-item">
                          <span className="info-label">CPU Usage</span>
                          <span className="info-value">
                            {selectedAgent.cpuUsage?.toFixed(1)}%
                          </span>
                        </div>
                        <div className="info-item">
                          <span className="info-label">Memory Usage</span>
                          <span className="info-value">
                            {selectedAgent.memoryUsage?.toFixed(1)}%
                          </span>
                        </div>
                        <div className="info-item">
                          <span className="info-label">Uptime</span>
                          <span className="info-value">
                            {formatUptime(selectedAgent.uptime)}
                          </span>
                        </div>
                      </>
                    )}
                  </div>
                </div>
                
                <div className="agent-prompt-section">
                  <h3>System Prompt</h3>
                  <div className="system-prompt">
                    {selectedAgent.systemPrompt || 'No system prompt defined'}
                  </div>
                </div>
                
                <div className="agent-metrics-section">
                  <h3>Recent Activity</h3>
                  <div className="activity-empty">
                    <p>No recent activity to display</p>
                  </div>
                </div>
              </div>
            </>
          ) : (
            <div className="no-agent-selected">
              <p>Select an agent to view details</p>
            </div>
          )}
        </div>
      </div>
      
      {/* Create Agent Modal */}
      {showCreateModal && (
        <div className="modal-overlay">
          <div className="modal-content">
            <div className="modal-header">
              <h2>Create New Agent</h2>
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
                <label htmlFor="agent-name">Agent Name</label>
                <input
                  id="agent-name"
                  type="text"
                  value={newAgentData.name}
                  onChange={(e) => setNewAgentData({ ...newAgentData, name: e.target.value })}
                  placeholder="Enter agent name"
                  disabled={actionInProgress === 'create'}
                />
              </div>
              <div className="form-group">
                <label htmlFor="system-prompt">System Prompt</label>
                <textarea
                  id="system-prompt"
                  value={newAgentData.systemPrompt}
                  onChange={(e) => setNewAgentData({ ...newAgentData, systemPrompt: e.target.value })}
                  placeholder="Enter system prompt"
                  rows={6}
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
                onClick={handleCreateAgent}
                disabled={actionInProgress === 'create'}
              >
                {actionInProgress === 'create' ? 'Creating...' : 'Create Agent'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default Agents;