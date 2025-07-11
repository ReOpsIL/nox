export interface Task {
  id: string;
  agentId: string;
  title: string;
  description: string;
  status: TaskStatus;
  priority: TaskPriority;
  createdAt: Date;
  updatedAt: Date;
  startedAt?: Date;
  completedAt?: Date;
  deadline?: Date;
  dependencies: string[];
  requestedBy: string;
  assignedTo?: string;
  progress: number;
  metadata?: Record<string, unknown>;
}

export type TaskStatus = 'todo' | 'inprogress' | 'done' | 'blocked' | 'cancelled';

export type TaskPriority = 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';

export interface TaskDelegation {
  fromAgent: string;
  toAgent: string;
  task: Omit<Task, 'id' | 'createdAt' | 'updatedAt'>;
  delegationType: 'request' | 'assignment' | 'collaboration';
  requiresApproval: boolean;
  deadline?: Date;
}

export interface TaskUpdate {
  taskId: string;
  agentId: string;
  status?: TaskStatus;
  progress?: number;
  notes?: string;
  timestamp: Date;
}

export interface TaskMetrics {
  agentId: string;
  totalTasks: number;
  completedTasks: number;
  inProgressTasks: number;
  todoTasks: number;
  blockedTasks: number;
  averageCompletionTime: number;
  tasksByPriority: Record<TaskPriority, number>;
}