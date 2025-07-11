import React, { useState, useEffect } from 'react';
import apiClient from '../api/api-client';
import './Settings.css';

interface SystemConfig {
  dashboard: {
    port: number;
    frontendPath: string;
  };
  server: {
    port: number;
    websocketEnabled: boolean;
  };
  metrics: {
    retentionPeriodDays: number;
    collectionIntervalMs: number;
    maxDataPoints: number;
  };
  agents: {
    autoStart: boolean;
    maxConcurrent: number;
    restartOnCrash: boolean;
    healthCheckIntervalMs: number;
  };
  logging: {
    level: 'debug' | 'info' | 'warn' | 'error';
    logToFile: boolean;
    logFilePath: string;
  };
}

/**
 * Settings page component
 */
const Settings: React.FC = () => {
  const [config, setConfig] = useState<SystemConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [isDirty, setIsDirty] = useState(false);

  // Load system configuration
  useEffect(() => {
    const fetchConfig = async () => {
      try {
        setLoading(true);
        const data = await apiClient.getSystemConfig();
        setConfig(data);
        setError(null);
      } catch (err) {
        console.error('Error fetching system configuration:', err);
        setError('Failed to load system configuration. Please try again later.');
      } finally {
        setLoading(false);
      }
    };
    
    fetchConfig();
  }, []);

  // Handle form field changes
  const handleChange = (section: keyof SystemConfig, field: string, value: any) => {
    if (!config) return;
    
    // Create a deep copy of the config
    const newConfig = JSON.parse(JSON.stringify(config));
    
    // Update the field
    newConfig[section][field] = value;
    
    // Update state
    setConfig(newConfig);
    setIsDirty(true);
    
    // Clear success message when form is changed
    if (success) {
      setSuccess(null);
    }
  };

  // Handle form submission
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!config) return;
    
    try {
      setLoading(true);
      await apiClient.updateSystemConfig(config);
      setSuccess('System configuration updated successfully');
      setIsDirty(false);
      setError(null);
    } catch (err) {
      console.error('Error updating system configuration:', err);
      setError('Failed to update system configuration. Please try again later.');
      setSuccess(null);
    } finally {
      setLoading(false);
    }
  };

  // Reset form to original values
  const handleReset = () => {
    const fetchConfig = async () => {
      try {
        setLoading(true);
        const data = await apiClient.getSystemConfig();
        setConfig(data);
        setIsDirty(false);
        setError(null);
        setSuccess(null);
      } catch (err) {
        console.error('Error fetching system configuration:', err);
        setError('Failed to reset form. Please try again later.');
      } finally {
        setLoading(false);
      }
    };
    
    fetchConfig();
  };

  if (loading && !config) {
    return (
      <div className="settings-loading">
        <div className="spinner"></div>
        <p>Loading system configuration...</p>
      </div>
    );
  }

  if (!config) {
    return (
      <div className="settings-error">
        <h2>Error</h2>
        <p>{error || 'Failed to load system configuration'}</p>
        <button onClick={() => window.location.reload()}>Retry</button>
      </div>
    );
  }

  return (
    <div className="settings-container">
      <div className="settings-header">
        <h1>System Settings</h1>
        <div className="settings-actions">
          <button 
            className="reset-button" 
            onClick={handleReset}
            disabled={loading || !isDirty}
          >
            Reset
          </button>
          <button 
            className="save-button" 
            onClick={handleSubmit}
            disabled={loading || !isDirty}
          >
            {loading ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </div>
      
      {error && (
        <div className="settings-error-message">
          <p>{error}</p>
          <button onClick={() => setError(null)}>Dismiss</button>
        </div>
      )}
      
      {success && (
        <div className="settings-success-message">
          <p>{success}</p>
          <button onClick={() => setSuccess(null)}>Dismiss</button>
        </div>
      )}
      
      <form className="settings-form" onSubmit={handleSubmit}>
        <div className="settings-section">
          <h2>Dashboard Settings</h2>
          <div className="settings-grid">
            <div className="form-group">
              <label htmlFor="dashboard-port">Dashboard Port</label>
              <input
                id="dashboard-port"
                type="number"
                value={config.dashboard.port}
                onChange={(e) => handleChange('dashboard', 'port', parseInt(e.target.value))}
                min="1024"
                max="65535"
              />
              <small>Port for the dashboard web server (default: 3001)</small>
            </div>
            <div className="form-group">
              <label htmlFor="frontend-path">Frontend Path</label>
              <input
                id="frontend-path"
                type="text"
                value={config.dashboard.frontendPath}
                onChange={(e) => handleChange('dashboard', 'frontendPath', e.target.value)}
              />
              <small>Path to the frontend build directory</small>
            </div>
          </div>
        </div>
        
        <div className="settings-section">
          <h2>Server Settings</h2>
          <div className="settings-grid">
            <div className="form-group">
              <label htmlFor="server-port">WebSocket Port</label>
              <input
                id="server-port"
                type="number"
                value={config.server.port}
                onChange={(e) => handleChange('server', 'port', parseInt(e.target.value))}
                min="1024"
                max="65535"
              />
              <small>Port for the WebSocket server (default: 3000)</small>
            </div>
            <div className="form-group">
              <label htmlFor="websocket-enabled">WebSocket Enabled</label>
              <div className="toggle-switch">
                <input
                  id="websocket-enabled"
                  type="checkbox"
                  checked={config.server.websocketEnabled}
                  onChange={(e) => handleChange('server', 'websocketEnabled', e.target.checked)}
                />
                <label htmlFor="websocket-enabled"></label>
              </div>
              <small>Enable/disable WebSocket server for real-time updates</small>
            </div>
          </div>
        </div>
        
        <div className="settings-section">
          <h2>Metrics Settings</h2>
          <div className="settings-grid">
            <div className="form-group">
              <label htmlFor="retention-period">Retention Period (days)</label>
              <input
                id="retention-period"
                type="number"
                value={config.metrics.retentionPeriodDays}
                onChange={(e) => handleChange('metrics', 'retentionPeriodDays', parseInt(e.target.value))}
                min="1"
                max="365"
              />
              <small>Number of days to keep metrics data (default: 7)</small>
            </div>
            <div className="form-group">
              <label htmlFor="collection-interval">Collection Interval (ms)</label>
              <input
                id="collection-interval"
                type="number"
                value={config.metrics.collectionIntervalMs}
                onChange={(e) => handleChange('metrics', 'collectionIntervalMs', parseInt(e.target.value))}
                min="1000"
                max="3600000"
                step="1000"
              />
              <small>Interval between metrics collections in milliseconds (default: 60000)</small>
            </div>
            <div className="form-group">
              <label htmlFor="max-data-points">Max Data Points</label>
              <input
                id="max-data-points"
                type="number"
                value={config.metrics.maxDataPoints}
                onChange={(e) => handleChange('metrics', 'maxDataPoints', parseInt(e.target.value))}
                min="100"
                max="100000"
              />
              <small>Maximum number of data points to store (default: 10080)</small>
            </div>
          </div>
        </div>
        
        <div className="settings-section">
          <h2>Agent Settings</h2>
          <div className="settings-grid">
            <div className="form-group">
              <label htmlFor="auto-start">Auto Start Agents</label>
              <div className="toggle-switch">
                <input
                  id="auto-start"
                  type="checkbox"
                  checked={config.agents.autoStart}
                  onChange={(e) => handleChange('agents', 'autoStart', e.target.checked)}
                />
                <label htmlFor="auto-start"></label>
              </div>
              <small>Automatically start agents when system starts</small>
            </div>
            <div className="form-group">
              <label htmlFor="max-concurrent">Max Concurrent Agents</label>
              <input
                id="max-concurrent"
                type="number"
                value={config.agents.maxConcurrent}
                onChange={(e) => handleChange('agents', 'maxConcurrent', parseInt(e.target.value))}
                min="1"
                max="100"
              />
              <small>Maximum number of concurrent agents (default: 10)</small>
            </div>
            <div className="form-group">
              <label htmlFor="restart-on-crash">Restart on Crash</label>
              <div className="toggle-switch">
                <input
                  id="restart-on-crash"
                  type="checkbox"
                  checked={config.agents.restartOnCrash}
                  onChange={(e) => handleChange('agents', 'restartOnCrash', e.target.checked)}
                />
                <label htmlFor="restart-on-crash"></label>
              </div>
              <small>Automatically restart agents when they crash</small>
            </div>
            <div className="form-group">
              <label htmlFor="health-check-interval">Health Check Interval (ms)</label>
              <input
                id="health-check-interval"
                type="number"
                value={config.agents.healthCheckIntervalMs}
                onChange={(e) => handleChange('agents', 'healthCheckIntervalMs', parseInt(e.target.value))}
                min="1000"
                max="60000"
                step="1000"
              />
              <small>Interval between agent health checks in milliseconds (default: 5000)</small>
            </div>
          </div>
        </div>
        
        <div className="settings-section">
          <h2>Logging Settings</h2>
          <div className="settings-grid">
            <div className="form-group">
              <label htmlFor="log-level">Log Level</label>
              <select
                id="log-level"
                value={config.logging.level}
                onChange={(e) => handleChange('logging', 'level', e.target.value)}
              >
                <option value="debug">Debug</option>
                <option value="info">Info</option>
                <option value="warn">Warning</option>
                <option value="error">Error</option>
              </select>
              <small>Minimum log level to record (default: info)</small>
            </div>
            <div className="form-group">
              <label htmlFor="log-to-file">Log to File</label>
              <div className="toggle-switch">
                <input
                  id="log-to-file"
                  type="checkbox"
                  checked={config.logging.logToFile}
                  onChange={(e) => handleChange('logging', 'logToFile', e.target.checked)}
                />
                <label htmlFor="log-to-file"></label>
              </div>
              <small>Save logs to a file on disk</small>
            </div>
            <div className="form-group">
              <label htmlFor="log-file-path">Log File Path</label>
              <input
                id="log-file-path"
                type="text"
                value={config.logging.logFilePath}
                onChange={(e) => handleChange('logging', 'logFilePath', e.target.value)}
                disabled={!config.logging.logToFile}
              />
              <small>Path to the log file</small>
            </div>
          </div>
        </div>
        
        <div className="settings-footer">
          <button 
            type="button" 
            className="reset-button" 
            onClick={handleReset}
            disabled={loading || !isDirty}
          >
            Reset
          </button>
          <button 
            type="submit" 
            className="save-button" 
            disabled={loading || !isDirty}
          >
            {loading ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </form>
    </div>
  );
};

export default Settings;