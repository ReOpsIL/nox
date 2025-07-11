export interface AgentMessage {
  id: string;
  from: string;
  to: string | 'broadcast';
  type: MessageType;
  content: string;
  priority: MessagePriority;
  timestamp: Date;
  requiresApproval: boolean;
  metadata?: MessageMetadata;
}

export type MessageType = 
  | 'task_request' 
  | 'task_response' 
  | 'broadcast' 
  | 'direct' 
  | 'system' 
  | 'approval_request'
  | 'capability_query'
  | 'collaboration_invite';

export type MessagePriority = 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';

export interface MessageMetadata {
  taskId?: string;
  deadline?: Date;
  dependencies?: string[];
  replyTo?: string;
  requestId?: string;
  attachments?: MessageAttachment[];
}

export interface MessageAttachment {
  type: 'file' | 'url' | 'data';
  name: string;
  content: string;
  size?: number;
  mimeType?: string;
}

export interface SystemMessage {
  type: SystemMessageType;
  source: 'user' | 'agent' | 'system';
  target?: string;
  payload: unknown;
  timestamp: Date;
}

export type SystemMessageType = 
  | 'agent_spawn' 
  | 'agent_kill' 
  | 'task_create' 
  | 'approval_request'
  | 'resource_alert'
  | 'health_check'
  | 'registry_update';

export interface MessageQueue {
  id: string;
  messages: AgentMessage[];
  maxSize: number;
  processingRate: number;
  lastProcessed: Date;
}

export interface ConversationLog {
  id: string;
  participants: string[];
  messages: AgentMessage[];
  startTime: Date;
  endTime?: Date;
  archived: boolean;
}