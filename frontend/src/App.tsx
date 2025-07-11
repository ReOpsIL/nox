import React, { useState, useEffect } from 'react';
import { Routes, Route } from 'react-router-dom';
import { connectWebSocket, WebSocketClient } from './api/websocket.ts';
import './App.css';

// Components
import Dashboard from './components/Dashboard.tsx';
import Agents from './components/Agents.tsx';
import Tasks from './components/Tasks.tsx';
import SystemHealth from './components/SystemHealth.tsx';
import Settings from './components/Settings.tsx';
import Sidebar from './components/Sidebar.tsx';
import Header from './components/Header.tsx';

const App: React.FC = () => {
  const [wsClient, setWsClient] = useState<WebSocketClient | null>(null);
  const [connectionStatus, setConnectionStatus] = useState<'disconnected' | 'connecting' | 'connected'>('disconnected');
  const [systemStatus, setSystemStatus] = useState<any>(null);

  useEffect(() => {
    // Initialize WebSocket connection
    const client = connectWebSocket();
    setWsClient(client);
    setConnectionStatus('connecting');

    // Set up event listeners
    client.on('connected', () => {
      setConnectionStatus('connected');
      console.log('WebSocket connected');
    });

    client.on('disconnected', () => {
      setConnectionStatus('disconnected');
      console.log('WebSocket disconnected');
    });

    client.on('system_status_update', (data: any) => {
      setSystemStatus(data);
    });

    // Fetch initial system status
    fetchSystemStatus();

    // Clean up on unmount
    return () => {
      client.disconnect();
    };
  }, []);

  const fetchSystemStatus = async () => {
    try {
      const response = await fetch('/api/system/status');
      if (response.ok) {
        const data = await response.json();
        // Handle API response format with success/data wrappers
        const statusData = data.success ? (data.status || data.data || data) : data;
        setSystemStatus(statusData);
      }
    } catch (error) {
      console.error('Failed to fetch system status:', error);
    }
  };

  return (
    <div className="app">
      <Header 
        connectionStatus={connectionStatus}
        systemStatus={systemStatus}
      />
      
      <div className="app-body">
        <Sidebar />
        
        <main className="main-content">
          <Routes>
            <Route path="/" element={<Dashboard wsClient={wsClient} systemStatus={systemStatus} />} />
            <Route path="/agents" element={<Agents wsClient={wsClient} />} />
            <Route path="/tasks" element={<Tasks wsClient={wsClient} />} />
            <Route path="/health" element={<SystemHealth wsClient={wsClient} systemStatus={systemStatus} />} />
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </main>
      </div>
    </div>
  );
};

export default App;