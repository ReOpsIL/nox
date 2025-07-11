import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';

/**
 * API client for interacting with the Nox backend
 */
export class ApiClient {
  private client: AxiosInstance;
  private baseUrl: string;

  constructor() {
    this.baseUrl = '/api'; // Relative URL, will be proxied by React dev server
    this.client = axios.create({
      baseURL: this.baseUrl,
      timeout: 10000,
      headers: {
        'Content-Type': 'application/json',
      },
    });

    // Add request interceptor for logging
    this.client.interceptors.request.use(
      (config) => {
        console.debug(`API Request: ${config.method?.toUpperCase()} ${config.url}`);
        return config;
      },
      (error) => {
        console.error('API Request Error:', error);
        return Promise.reject(error);
      }
    );

    // Add response interceptor for logging
    this.client.interceptors.response.use(
      (response) => {
        console.debug(`API Response: ${response.status} ${response.config.url}`);
        return response;
      },
      (error) => {
        if (error.response) {
          console.error(`API Error ${error.response.status}:`, error.response.data);
        } else {
          console.error('API Error:', error.message);
        }
        return Promise.reject(error);
      }
    );
  }

  /**
   * Get WebSocket server information
   */
  async getWebSocketInfo(): Promise<{ url: string }> {
    const response = await this.get<{ url: string }>('/websocket-info');
    return response.data;
  }

  /**
   * Get system health information
   */
  async getHealth(): Promise<any> {
    const response = await this.get<any>('/health');
    return response.data;
  }

  // Agent API

  /**
   * Get all agents
   */
  async getAgents(): Promise<any[]> {
    const response = await this.get<any[]>('/agents');
    return response.data;
  }

  /**
   * Get a specific agent
   */
  async getAgent(agentId: string): Promise<any> {
    const response = await this.get<any>(`/agents/${agentId}`);
    return response.data;
  }

  /**
   * Create a new agent
   */
  async createAgent(agentData: any): Promise<any> {
    const response = await this.post<any>('/agents', agentData);
    return response.data;
  }

  /**
   * Update an agent
   */
  async updateAgent(agentId: string, agentData: any): Promise<any> {
    const response = await this.put<any>(`/agents/${agentId}`, agentData);
    return response.data;
  }

  /**
   * Delete an agent
   */
  async deleteAgent(agentId: string): Promise<void> {
    await this.delete(`/agents/${agentId}`);
  }

  /**
   * Start an agent
   */
  async startAgent(agentId: string): Promise<any> {
    const response = await this.post<any>(`/agents/${agentId}/start`, {});
    return response.data;
  }

  /**
   * Stop an agent
   */
  async stopAgent(agentId: string): Promise<any> {
    const response = await this.post<any>(`/agents/${agentId}/stop`, {});
    return response.data;
  }

  /**
   * Restart an agent
   */
  async restartAgent(agentId: string): Promise<any> {
    const response = await this.post<any>(`/agents/${agentId}/restart`, {});
    return response.data;
  }

  // Task API

  /**
   * Get all tasks
   */
  async getTasks(): Promise<any[]> {
    const response = await this.get<any[]>('/tasks');
    return response.data;
  }

  /**
   * Get tasks for a specific agent
   */
  async getAgentTasks(agentId: string): Promise<any[]> {
    const response = await this.get<any[]>(`/agents/${agentId}/tasks`);
    return response.data;
  }

  /**
   * Get a specific task
   */
  async getTask(taskId: string): Promise<any> {
    const response = await this.get<any>(`/tasks/${taskId}`);
    return response.data;
  }

  /**
   * Create a new task
   */
  async createTask(taskData: any): Promise<any> {
    const response = await this.post<any>('/tasks', taskData);
    return response.data;
  }

  /**
   * Update a task
   */
  async updateTask(taskId: string, taskData: any): Promise<any> {
    const response = await this.put<any>(`/tasks/${taskId}`, taskData);
    return response.data;
  }

  /**
   * Delete a task
   */
  async deleteTask(taskId: string): Promise<void> {
    await this.delete(`/tasks/${taskId}`);
  }

  /**
   * Get task dashboard data
   */
  async getTaskDashboard(): Promise<any> {
    const response = await this.get<any>('/tasks/dashboard');
    return response.data;
  }

  // Metrics API

  /**
   * Get system metrics
   */
  async getSystemMetrics(
    startTime?: string,
    endTime?: string,
    interval?: 'minute' | 'hour' | 'day'
  ): Promise<any[]> {
    const params: Record<string, string> = {};
    if (startTime) params.startTime = startTime;
    if (endTime) params.endTime = endTime;
    if (interval) params.interval = interval;

    const response = await this.get<any[]>('/metrics/system', { params });
    return response.data;
  }

  /**
   * Get agent metrics
   */
  async getAgentMetrics(
    agentId: string,
    startTime?: string,
    endTime?: string,
    interval?: 'minute' | 'hour' | 'day'
  ): Promise<any[]> {
    const params: Record<string, string> = {};
    if (startTime) params.startTime = startTime;
    if (endTime) params.endTime = endTime;
    if (interval) params.interval = interval;

    const response = await this.get<any[]>(`/metrics/agents/${agentId}`, { params });
    return response.data;
  }

  /**
   * Get latest system metrics
   */
  async getLatestSystemMetrics(): Promise<any> {
    const response = await this.get<any>('/metrics/system/latest');
    return response.data;
  }

  /**
   * Get latest agent metrics
   */
  async getLatestAgentMetrics(agentId: string): Promise<any> {
    const response = await this.get<any>(`/metrics/agents/${agentId}/latest`);
    return response.data;
  }

  // System API

  /**
   * Get system configuration
   */
  async getSystemConfig(): Promise<any> {
    const response = await this.get<any>('/system/config');
    return response.data;
  }

  /**
   * Update system configuration
   */
  async updateSystemConfig(configData: any): Promise<any> {
    const response = await this.put<any>('/system/config', configData);
    return response.data;
  }

  /**
   * Get system status
   */
  async getSystemStatus(): Promise<any> {
    const response = await this.get<any>('/system/status');
    return response.data;
  }

  // Private helper methods

  private async get<T>(url: string, config?: AxiosRequestConfig): Promise<AxiosResponse<T>> {
    return this.client.get<T>(url, config);
  }

  private async post<T>(url: string, data: any, config?: AxiosRequestConfig): Promise<AxiosResponse<T>> {
    return this.client.post<T>(url, data, config);
  }

  private async put<T>(url: string, data: any, config?: AxiosRequestConfig): Promise<AxiosResponse<T>> {
    return this.client.put<T>(url, data, config);
  }

  private async delete<T>(url: string, config?: AxiosRequestConfig): Promise<AxiosResponse<T>> {
    return this.client.delete<T>(url, config);
  }
}

// Create a singleton instance
const apiClient = new ApiClient();

export default apiClient;