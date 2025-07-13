import { logger } from '../utils/logger';
import { AgentConfig } from '../types';
import { 
  ClaudeInterfaceBase, 
  ClaudeSession, 
  ClaudeMessage, 
  ClaudeResponse, 
  ClaudeHealthStatus 
} from './claude-interface-base';

/**
 * Mock Claude Interface - Simulates Claude CLI for testing purposes
 * This allows testing task execution without requiring actual Claude CLI authentication
 */
export class MockClaudeInterface extends ClaudeInterfaceBase {
  private session: ClaudeSession | null = null;
  private conversationHistory: ClaudeMessage[] = [];
  private isInitialized = false;

  constructor(agentConfig: AgentConfig, workingDir: string) {
    super(agentConfig, workingDir);
  }

  /**
   * Initialize mock Claude interface
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) {
      logger.warn(`Mock Claude interface for agent ${this.agentConfig.id} already initialized`);
      return;
    }

    try {
      // Create mock session
      this.session = {
        id: `mock_session_${this.agentConfig.id}_${Date.now()}`,
        agentId: this.agentConfig.id,
        startTime: new Date(),
        lastActivity: new Date(),
        conversationPath: '',
        status: 'starting'
      };

      // Simulate initialization delay
      await new Promise(resolve => setTimeout(resolve, 100));

      this.isInitialized = true;
      this.session.status = 'ready';
      
      logger.info(`Mock Claude interface initialized for agent: ${this.agentConfig.id}`);
      this.emit('initialized', this.session);

    } catch (error) {
      const errorMsg = `Failed to initialize mock Claude interface for agent ${this.agentConfig.id}: ${error}`;
      logger.error(errorMsg);
      if (this.session) {
        this.session.status = 'error';
      }
      throw new Error(errorMsg);
    }
  }

  /**
   * Send message to mock Claude and get simulated response
   */
  async sendMessage(message: string): Promise<ClaudeResponse> {
    if (!this.isInitialized || !this.session) {
      throw new Error(`Mock Claude interface not initialized for agent: ${this.agentConfig.id}`);
    }

    if (this.session.status !== 'ready') {
      throw new Error(`Mock Claude interface not ready for agent: ${this.agentConfig.id}. Status: ${this.session.status}`);
    }

    try {
      this.session.status = 'busy';
      this.session.lastActivity = new Date();

      // Create user message
      const userMessage: ClaudeMessage = {
        id: `msg_${Date.now()}_user`,
        timestamp: new Date(),
        role: 'user',
        content: message
      };

      this.conversationHistory.push(userMessage);

      // Simulate processing delay
      await new Promise(resolve => setTimeout(resolve, 500 + Math.random() * 2000));

      // Generate mock response based on the task
      const mockResponse = this.generateMockResponse(message);

      // Create assistant message
      const assistantMessage: ClaudeMessage = {
        id: `msg_${Date.now()}_assistant`,
        timestamp: new Date(),
        role: 'assistant',
        content: mockResponse
      };

      this.conversationHistory.push(assistantMessage);

      this.session.status = 'ready';

      const response: ClaudeResponse = {
        success: true,
        content: mockResponse,
        sessionId: this.session.id,
        timestamp: new Date()
      };

      logger.info(`Mock Claude response generated for agent ${this.agentConfig.id}`);
      this.emit('response', response);

      return response;

    } catch (error) {
      this.session.status = 'error';
      const response: ClaudeResponse = {
        success: false,
        error: error instanceof Error ? error.message : String(error),
        sessionId: this.session.id,
        timestamp: new Date()
      };
      
      logger.error(`Mock Claude interface error for agent ${this.agentConfig.id}:`, error);
      return response;
    }
  }

  /**
   * Generate a realistic mock response based on the task content
   */
  private generateMockResponse(taskMessage: string): string {
    const taskId = this.extractTaskId(taskMessage);
    const agentName = this.agentConfig.name;

    // Create a realistic response based on agent type and task
    let response = `Hello! I'm ${agentName}, and I've received your task request.\n\n`;

    if (taskMessage.includes('NEWS') || taskMessage.includes('news')) {
      response += `📰 **News Analysis Task Completed**\n\n`;
      response += `I have successfully analyzed the latest news in the agentic coding space:\n\n`;
      response += `🔍 **Key Findings:**\n`;
      response += `- Found 15 relevant articles about autonomous coding tools\n`;
      response += `- Identified 3 major trend shifts in AI-powered development\n`;
      response += `- Discovered 5 new frameworks for agent-based programming\n\n`;
      response += `📊 **Sentiment Analysis:** 78% positive coverage\n`;
      response += `📈 **Trend Direction:** Growing interest in autonomous code generation\n\n`;
    } else if (taskMessage.includes('CODE') || taskMessage.includes('GitHub')) {
      response += `🔍 **Code Analysis Task Completed**\n\n`;
      response += `I have successfully scanned GitHub repositories for agentic coding tools:\n\n`;
      response += `📦 **Repositories Analyzed:** 127\n`;
      response += `⭐ **High-quality Projects Found:** 23\n`;
      response += `🆕 **New Frameworks Discovered:** 8\n`;
      response += `📚 **Documentation Quality:** 85% above average\n\n`;
      response += `🏆 **Top Recommendations:**\n`;
      response += `1. AutoAgent-Framework (⭐ 2.3k stars)\n`;
      response += `2. CodeGen-AI (⭐ 1.8k stars)\n`;
      response += `3. AgentScript (⭐ 1.2k stars)\n\n`;
    } else if (taskMessage.includes('RESEARCH') || taskMessage.includes('ArXiv')) {
      response += `📚 **Research Analysis Task Completed**\n\n`;
      response += `I have successfully analyzed recent academic papers on autonomous programming:\n\n`;
      response += `📄 **Papers Reviewed:** 34\n`;
      response += `🎯 **Highly Relevant Papers:** 12\n`;
      response += `🔗 **Citation Networks Mapped:** 5 major clusters\n`;
      response += `📊 **Research Trends Identified:** 3 emerging patterns\n\n`;
      response += `🧠 **Key Research Areas:**\n`;
      response += `1. Multi-agent code generation systems\n`;
      response += `2. Self-modifying programming agents\n`;
      response += `3. Collaborative AI development workflows\n\n`;
    } else if (taskMessage.includes('SOCIAL') || taskMessage.includes('sentiment')) {
      response += `📱 **Social Media Analysis Task Completed**\n\n`;
      response += `I have successfully monitored social platforms for agentic coding discussions:\n\n`;
      response += `💬 **Posts Analyzed:** 1,247\n`;
      response += `👥 **Unique Contributors:** 423\n`;
      response += `📈 **Engagement Metrics:** +34% increase this week\n`;
      response += `🎯 **Key Topics:** Agent frameworks, autonomous debugging, code review AI\n\n`;
      response += `😊 **Sentiment Breakdown:**\n`;
      response += `- Positive: 68%\n`;
      response += `- Neutral: 25%\n`;
      response += `- Negative: 7%\n\n`;
    } else {
      response += `✅ **General Task Completed**\n\n`;
      response += `I have successfully completed the assigned task using my specialized capabilities.\n\n`;
      response += `🎯 **Task Summary:**\n`;
      response += `- Analyzed the requested information thoroughly\n`;
      response += `- Applied domain-specific expertise\n`;
      response += `- Generated actionable insights\n`;
      response += `- Completed all required deliverables\n\n`;
    }

    response += `⏱️ **Completion Time:** ${Math.floor(Math.random() * 5) + 1} minutes\n`;
    response += `📋 **Status:** All objectives completed successfully\n\n`;
    response += `💡 **Next Steps:**\n`;
    response += `- Results have been documented\n`;
    response += `- Data is ready for further analysis\n`;
    response += `- Standing by for additional tasks\n\n`;
    
    if (taskId) {
      response += `TASK COMPLETED: ${taskId}\n\n`;
    }
    
    response += `Best regards,\n${agentName} 🤖`;

    return response;
  }

  /**
   * Extract task ID from the task message
   */
  private extractTaskId(message: string): string | null {
    const match = message.match(/Task ID: ([^\n\r]+)/);
    return match && match[1] ? match[1].trim() : null;
  }

  /**
   * Get conversation history
   */
  getConversationHistory(): ClaudeMessage[] {
    return [...this.conversationHistory];
  }

  /**
   * Get session information
   */
  getSession(): ClaudeSession | null {
    return this.session;
  }

  /**
   * Get health status
   */
  getHealthStatus(): ClaudeHealthStatus {
    return {
      healthy: this.session?.status === 'ready' || this.session?.status === 'busy',
      status: this.session?.status || 'not-initialized',
      lastActivity: this.session?.lastActivity || new Date(),
      messageCount: this.conversationHistory.length
    };
  }

  /**
   * Stop the mock interface
   */
  async stop(): Promise<void> {
    if (this.session) {
      this.session.status = 'stopped';
    }
    this.isInitialized = false;
    logger.info(`Mock Claude interface stopped for agent: ${this.agentConfig.id}`);
    this.emit('stopped');
  }

  /**
   * Shutdown the mock interface (alias for stop)
   */
  override async shutdown(): Promise<void> {
    return this.stop();
  }
}