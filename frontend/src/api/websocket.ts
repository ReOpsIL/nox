/**
 * Simple event emitter for browser environments
 */
class EventEmitter {
  private listeners: { [event: string]: Function[] } = {};

  on(event: string, listener: Function) {
    if (!this.listeners[event]) {
      this.listeners[event] = [];
    }
    this.listeners[event].push(listener);
  }

  emit(event: string, ...args: any[]) {
    if (this.listeners[event]) {
      this.listeners[event].forEach(listener => listener(...args));
    }
  }

  off(event: string, listener: Function) {
    if (this.listeners[event]) {
      this.listeners[event] = this.listeners[event].filter(l => l !== listener);
    }
  }
}

/**
 * WebSocket API client for real-time updates
 */
export class WebSocketClient extends EventEmitter {
  private socket: WebSocket | null = null;
  private reconnectTimeout: number | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 10;
  private reconnectDelay = 2000; // Start with 2 seconds
  private clientId: string | null = null;
  private connected = false;
  private url: string;

  // WebSocket-like interface for compatibility
  onopen: ((event: Event) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;

  constructor() {
    super();
    this.url = this.getWebSocketUrl();
  }

  /**
   * Connect to the WebSocket server
   */
  connect(): void {
    if (this.socket) {
      return;
    }

    try {
      this.socket = new WebSocket(this.url);
      
      this.socket.onopen = this.handleOpen.bind(this);
      this.socket.onmessage = this.handleMessage.bind(this);
      this.socket.onclose = this.handleClose.bind(this);
      this.socket.onerror = this.handleError.bind(this);
      
      console.log('WebSocket connecting to', this.url);
    } catch (error) {
      console.error('WebSocket connection error:', error);
      this.scheduleReconnect();
    }
  }

  /**
   * Disconnect from the WebSocket server
   */
  disconnect(): void {
    if (this.socket) {
      this.socket.close();
      this.socket = null;
    }

    if (this.reconnectTimeout) {
      window.clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }

    this.connected = false;
    this.clientId = null;
  }

  /**
   * Send a ping to the server
   */
  ping(): void {
    this.send('ping', {});
  }

  /**
   * Send a message to the server
   */
  send(type: string, data: any): void {
    if (!this.socket || this.socket.readyState !== WebSocket.OPEN) {
      console.warn('WebSocket not connected, cannot send message');
      return;
    }

    const message = JSON.stringify({
      type,
      data,
      timestamp: new Date().toISOString()
    });

    this.socket.send(message);
  }

  /**
   * Check if the client is connected
   */
  isConnected(): boolean {
    return this.connected;
  }

  /**
   * Get the client ID
   */
  getClientId(): string | null {
    return this.clientId;
  }

  /**
   * Close the WebSocket connection
   */
  close(): void {
    if (this.socket) {
      this.socket.close();
    }
  }

  /**
   * Handle WebSocket open event
   */
  private handleOpen(event: Event): void {
    console.log('WebSocket connected');
    this.connected = true;
    this.reconnectAttempts = 0;
    this.emit('connected');
    
    // Call compatibility callback
    if (this.onopen) {
      this.onopen(event);
    }
  }

  /**
   * Handle WebSocket message event
   */
  private handleMessage(event: MessageEvent): void {
    try {
      const message = JSON.parse(event.data);
      
      // Handle connection established message
      if (message.type === 'connection_established') {
        this.clientId = message.data.clientId;
        console.log('WebSocket connection established, client ID:', this.clientId);
      }
      
      // Emit the message event
      this.emit('message', message);
      
      // Call compatibility callback
      if (this.onmessage) {
        this.onmessage(event);
      }
      
      // Also emit an event specific to the message type
      this.emit(message.type, message.data);
      
    } catch (error) {
      console.error('Error parsing WebSocket message:', error);
    }
  }

  /**
   * Handle WebSocket close event
   */
  private handleClose(event: CloseEvent): void {
    console.log('WebSocket disconnected:', event.code, event.reason);
    this.socket = null;
    this.connected = false;
    this.emit('disconnected', { code: event.code, reason: event.reason });
    
    // Call compatibility callback
    if (this.onclose) {
      this.onclose(event);
    }
    
    // Attempt to reconnect
    this.scheduleReconnect();
  }

  /**
   * Handle WebSocket error event
   */
  private handleError(error: Event): void {
    console.error('WebSocket error:', error);
    this.emit('error', error);
    
    // Call compatibility callback
    if (this.onerror) {
      this.onerror(error);
    }
  }

  /**
   * Schedule a reconnection attempt
   */
  private scheduleReconnect(): void {
    if (this.reconnectTimeout) {
      window.clearTimeout(this.reconnectTimeout);
    }

    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('Maximum WebSocket reconnection attempts reached');
      this.emit('reconnect_failed');
      return;
    }

    const delay = Math.min(30000, this.reconnectDelay * Math.pow(1.5, this.reconnectAttempts));
    console.log(`Scheduling WebSocket reconnect in ${delay}ms (attempt ${this.reconnectAttempts + 1}/${this.maxReconnectAttempts})`);
    
    this.reconnectTimeout = window.setTimeout(() => {
      this.reconnectAttempts++;
      this.connect();
    }, delay) as unknown as number;
  }

  /**
   * Get the WebSocket URL
   */
  private getWebSocketUrl(): string {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = window.location.hostname;
    const port = 3000; // Default WebSocket port
    
    return `${protocol}//${host}:${port}`;
  }
}

// Create a singleton instance
const websocketClient = new WebSocketClient();

/**
 * Connect to the WebSocket server and return the client
 */
export function connectWebSocket(): WebSocketClient {
  websocketClient.connect();
  return websocketClient;
}