import React, { useState, useEffect } from 'react';

interface SystemSettings {
  security: {
    maxAgents: number;
    spawnRateLimit: number;
    requireApprovalFor: string[];
    resourceLimits: {
      memoryPerAgent: string;
      claudeCallsPerMinute: number;
      maxConcurrentTasks: number;
      maxDockerContainers: number;
      diskSpaceLimit: string;
    };
    sandboxMode: boolean;
    allowExternalCommunication: boolean;
  };
  server: {
    port: number;
    dashboardPort: number;
    host: string;
    websocketEnabled: boolean;
    dashboardEnabled: boolean;
    apiEnabled: boolean;
    corsOrigins: string[];
    rateLimiting: {
      windowMs: number;
      maxRequests: number;
      skipSuccessfulRequests: boolean;
    };
  };
  claudeCli: {
    sessionTimeout: number;
    autoRestartOnCrash: boolean;
    backupConversations: boolean;
    cliPath: string;
    defaultArgs: string[];
    healthCheckInterval: number;
  };
  logging: {
    level: string;
    format: string;
    retention: {
      days: number;
      maxSizeMB: number;
      compress: boolean;
    };
  };
}

const Settings: React.FC = () => {
  const [settings, setSettings] = useState<SystemSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);
  const [activeTab, setActiveTab] = useState('security');

  useEffect(() => {
    fetchSettings();
  }, []);

  const fetchSettings = async () => {
    try {
      setLoading(true);
      const response = await fetch('/api/system/config');
      if (response.ok) {
        const data = await response.json();
        // Handle API response format with success/data wrappers
        const settingsData = data.success ? (data.config || data.settings || data.data || data) : data;
        setSettings(settingsData);
        setError(null);
      } else {
        setError('Failed to fetch settings');
      }
    } catch (err) {
      setError('Error fetching settings');
      console.error('Error fetching settings:', err);
    } finally {
      setLoading(false);
    }
  };

  const saveSettings = async () => {
    if (!settings) return;

    try {
      setSaving(true);
      const response = await fetch('/api/system/config', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(settings),
      });

      if (response.ok) {
        setSaveSuccess(true);
        setTimeout(() => setSaveSuccess(false), 3000);
        setError(null);
      } else {
        setError('Failed to save settings');
      }
    } catch (err) {
      setError('Error saving settings');
      console.error('Error saving settings:', err);
    } finally {
      setSaving(false);
    }
  };

  const updateSetting = (path: string[], value: any) => {
    if (!settings) return;

    const newSettings = { ...settings };
    let current: any = newSettings;
    
    for (let i = 0; i < path.length - 1; i++) {
      current = current[path[i]];
    }
    
    current[path[path.length - 1]] = value;
    setSettings(newSettings);
  };

  const renderSecuritySettings = () => (
    <div className="settings-section">
      <h3>🔒 Security Settings</h3>
      
      <div className="setting-group">
        <label>Maximum Agents:</label>
        <input
          type="number"
          value={settings?.security.maxAgents || 0}
          onChange={(e) => updateSetting(['security', 'maxAgents'], parseInt(e.target.value))}
        />
      </div>

      <div className="setting-group">
        <label>Spawn Rate Limit:</label>
        <input
          type="number"
          value={settings?.security.spawnRateLimit || 0}
          onChange={(e) => updateSetting(['security', 'spawnRateLimit'], parseInt(e.target.value))}
        />
      </div>

      <div className="setting-group">
        <label>Memory Per Agent:</label>
        <input
          type="text"
          value={settings?.security.resourceLimits.memoryPerAgent || ''}
          onChange={(e) => updateSetting(['security', 'resourceLimits', 'memoryPerAgent'], e.target.value)}
        />
      </div>

      <div className="setting-group">
        <label>Claude Calls Per Minute:</label>
        <input
          type="number"
          value={settings?.security.resourceLimits.claudeCallsPerMinute || 0}
          onChange={(e) => updateSetting(['security', 'resourceLimits', 'claudeCallsPerMinute'], parseInt(e.target.value))}
        />
      </div>

      <div className="setting-group">
        <label>Max Concurrent Tasks:</label>
        <input
          type="number"
          value={settings?.security.resourceLimits.maxConcurrentTasks || 0}
          onChange={(e) => updateSetting(['security', 'resourceLimits', 'maxConcurrentTasks'], parseInt(e.target.value))}
        />
      </div>

      <div className="setting-group">
        <label>Max Docker Containers:</label>
        <input
          type="number"
          value={settings?.security.resourceLimits.maxDockerContainers || 0}
          onChange={(e) => updateSetting(['security', 'resourceLimits', 'maxDockerContainers'], parseInt(e.target.value))}
        />
      </div>

      <div className="setting-group">
        <label>Disk Space Limit:</label>
        <input
          type="text"
          value={settings?.security.resourceLimits.diskSpaceLimit || ''}
          onChange={(e) => updateSetting(['security', 'resourceLimits', 'diskSpaceLimit'], e.target.value)}
        />
      </div>

      <div className="setting-group checkbox">
        <label>
          <input
            type="checkbox"
            checked={settings?.security.sandboxMode || false}
            onChange={(e) => updateSetting(['security', 'sandboxMode'], e.target.checked)}
          />
          Sandbox Mode
        </label>
      </div>

      <div className="setting-group checkbox">
        <label>
          <input
            type="checkbox"
            checked={settings?.security.allowExternalCommunication || false}
            onChange={(e) => updateSetting(['security', 'allowExternalCommunication'], e.target.checked)}
          />
          Allow External Communication
        </label>
      </div>
    </div>
  );

  const renderServerSettings = () => (
    <div className="settings-section">
      <h3>🌐 Server Settings</h3>
      
      <div className="setting-group">
        <label>WebSocket Port:</label>
        <input
          type="number"
          value={settings?.server.port || 0}
          onChange={(e) => updateSetting(['server', 'port'], parseInt(e.target.value))}
        />
      </div>

      <div className="setting-group">
        <label>Dashboard Port:</label>
        <input
          type="number"
          value={settings?.server.dashboardPort || 0}
          onChange={(e) => updateSetting(['server', 'dashboardPort'], parseInt(e.target.value))}
        />
      </div>

      <div className="setting-group">
        <label>Host:</label>
        <input
          type="text"
          value={settings?.server.host || ''}
          onChange={(e) => updateSetting(['server', 'host'], e.target.value)}
        />
      </div>

      <div className="setting-group checkbox">
        <label>
          <input
            type="checkbox"
            checked={settings?.server.websocketEnabled || false}
            onChange={(e) => updateSetting(['server', 'websocketEnabled'], e.target.checked)}
          />
          WebSocket Enabled
        </label>
      </div>

      <div className="setting-group checkbox">
        <label>
          <input
            type="checkbox"
            checked={settings?.server.dashboardEnabled || false}
            onChange={(e) => updateSetting(['server', 'dashboardEnabled'], e.target.checked)}
          />
          Dashboard Enabled
        </label>
      </div>

      <div className="setting-group checkbox">
        <label>
          <input
            type="checkbox"
            checked={settings?.server.apiEnabled || false}
            onChange={(e) => updateSetting(['server', 'apiEnabled'], e.target.checked)}
          />
          API Enabled
        </label>
      </div>

      <div className="setting-group">
        <label>Rate Limit - Max Requests:</label>
        <input
          type="number"
          value={settings?.server.rateLimiting.maxRequests || 0}
          onChange={(e) => updateSetting(['server', 'rateLimiting', 'maxRequests'], parseInt(e.target.value))}
        />
      </div>

      <div className="setting-group">
        <label>Rate Limit - Window (ms):</label>
        <input
          type="number"
          value={settings?.server.rateLimiting.windowMs || 0}
          onChange={(e) => updateSetting(['server', 'rateLimiting', 'windowMs'], parseInt(e.target.value))}
        />
      </div>
    </div>
  );

  const renderClaudeSettings = () => (
    <div className="settings-section">
      <h3>🤖 Claude CLI Settings</h3>
      
      <div className="setting-group">
        <label>Session Timeout (seconds):</label>
        <input
          type="number"
          value={settings?.claudeCli.sessionTimeout || 0}
          onChange={(e) => updateSetting(['claudeCli', 'sessionTimeout'], parseInt(e.target.value))}
        />
      </div>

      <div className="setting-group">
        <label>CLI Path:</label>
        <input
          type="text"
          value={settings?.claudeCli.cliPath || ''}
          onChange={(e) => updateSetting(['claudeCli', 'cliPath'], e.target.value)}
        />
      </div>

      <div className="setting-group">
        <label>Health Check Interval (seconds):</label>
        <input
          type="number"
          value={settings?.claudeCli.healthCheckInterval || 0}
          onChange={(e) => updateSetting(['claudeCli', 'healthCheckInterval'], parseInt(e.target.value))}
        />
      </div>

      <div className="setting-group checkbox">
        <label>
          <input
            type="checkbox"
            checked={settings?.claudeCli.autoRestartOnCrash || false}
            onChange={(e) => updateSetting(['claudeCli', 'autoRestartOnCrash'], e.target.checked)}
          />
          Auto Restart on Crash
        </label>
      </div>

      <div className="setting-group checkbox">
        <label>
          <input
            type="checkbox"
            checked={settings?.claudeCli.backupConversations || false}
            onChange={(e) => updateSetting(['claudeCli', 'backupConversations'], e.target.checked)}
          />
          Backup Conversations
        </label>
      </div>
    </div>
  );

  const renderLoggingSettings = () => (
    <div className="settings-section">
      <h3>📝 Logging Settings</h3>
      
      <div className="setting-group">
        <label>Log Level:</label>
        <select
          value={settings?.logging.level || 'info'}
          onChange={(e) => updateSetting(['logging', 'level'], e.target.value)}
        >
          <option value="debug">Debug</option>
          <option value="info">Info</option>
          <option value="warn">Warning</option>
          <option value="error">Error</option>
        </select>
      </div>

      <div className="setting-group">
        <label>Log Format:</label>
        <select
          value={settings?.logging.format || 'json'}
          onChange={(e) => updateSetting(['logging', 'format'], e.target.value)}
        >
          <option value="json">JSON</option>
          <option value="text">Text</option>
        </select>
      </div>

      <div className="setting-group">
        <label>Retention Days:</label>
        <input
          type="number"
          value={settings?.logging.retention.days || 0}
          onChange={(e) => updateSetting(['logging', 'retention', 'days'], parseInt(e.target.value))}
        />
      </div>

      <div className="setting-group">
        <label>Max Size (MB):</label>
        <input
          type="number"
          value={settings?.logging.retention.maxSizeMB || 0}
          onChange={(e) => updateSetting(['logging', 'retention', 'maxSizeMB'], parseInt(e.target.value))}
        />
      </div>

      <div className="setting-group checkbox">
        <label>
          <input
            type="checkbox"
            checked={settings?.logging.retention.compress || false}
            onChange={(e) => updateSetting(['logging', 'retention', 'compress'], e.target.checked)}
          />
          Compress Logs
        </label>
      </div>
    </div>
  );

  if (loading) {
    return <div className="loading">Loading settings...</div>;
  }

  if (!settings) {
    return <div className="error">Failed to load settings</div>;
  }

  return (
    <div className="settings-page">
      <div className="page-header">
        <h1>⚙️ System Settings</h1>
        <div className="settings-actions">
          {saveSuccess && (
            <span className="save-success">✅ Settings saved successfully!</span>
          )}
          <button 
            className="btn btn-primary"
            onClick={saveSettings}
            disabled={saving}
          >
            {saving ? 'Saving...' : 'Save Settings'}
          </button>
        </div>
      </div>

      {error && (
        <div className="error-banner">
          {error}
          <button onClick={() => setError(null)}>×</button>
        </div>
      )}

      <div className="settings-content">
        <div className="settings-tabs">
          <button 
            className={`tab ${activeTab === 'security' ? 'active' : ''}`}
            onClick={() => setActiveTab('security')}
          >
            🔒 Security
          </button>
          <button 
            className={`tab ${activeTab === 'server' ? 'active' : ''}`}
            onClick={() => setActiveTab('server')}
          >
            🌐 Server
          </button>
          <button 
            className={`tab ${activeTab === 'claude' ? 'active' : ''}`}
            onClick={() => setActiveTab('claude')}
          >
            🤖 Claude CLI
          </button>
          <button 
            className={`tab ${activeTab === 'logging' ? 'active' : ''}`}
            onClick={() => setActiveTab('logging')}
          >
            📝 Logging
          </button>
        </div>

        <div className="settings-panel">
          {activeTab === 'security' && renderSecuritySettings()}
          {activeTab === 'server' && renderServerSettings()}
          {activeTab === 'claude' && renderClaudeSettings()}
          {activeTab === 'logging' && renderLoggingSettings()}
        </div>
      </div>
    </div>
  );
};

export default Settings;