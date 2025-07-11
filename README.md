# Nox - Living Agent Ecosystem

Nox is a revolutionary platform where AI agents form a **living, breathing community** that grows, adapts, and evolves autonomously. Agents can modify themselves, discover each other's capabilities, and dynamically reorganize to tackle any challenge thrown at them.

## Core Concept

Nox creates a **living digital ecosystem** where AI agents exist as autonomous entities that can:
- **Self-Modify**: Agents evolve their own system prompts and capabilities
- **Community Discovery**: Query the agent registry to find collaborators
- **Dynamic Adaptation**: Reshape themselves to meet emerging task requirements
- **Collective Intelligence**: Form temporary or permanent partnerships
- **Emergent Behaviors**: Develop new problem-solving approaches through interaction

Each agent is a **living entity** powered by Claude CLI, capable of growth, learning, and relationship-building within the community.

### Revolutionary Features

- **Claude CLI Brain**: Each agent runs as an independent Claude CLI instance
- **MCP Service Integration**: Agents dynamically discover and integrate MCP services from Docker Hub
- **Autonomous Agent Creation**: Agents can create specialized sub-agents for complex workflows
- **Inter-Agent Communication**: Transparent conversations between agents with user visibility
- **Self-Organizing Ecosystem**: Agents form collaborative networks and specialized roles
- **User Approval Gates**: Agents request permission for significant actions or resource usage

### Technical Capabilities

- **Claude CLI Integration**: Each agent is a persistent Claude CLI session with full tool access
- **MCP Service Discovery**: Automatic discovery and installation of MCP services from Docker Hub
- **Dynamic Tool Acquisition**: Agents can search, evaluate, and install new capabilities
- **Agent-to-Agent Protocols**: Structured communication channels between agents
- **Approval Workflows**: Smart escalation to users for permissions and decisions
- **Resource Management**: Automatic Docker container lifecycle management
- **Conversation Logging**: Full transparency of all agent interactions and decisions

## Living Agent Community

The agent ecosystem operates as a **dynamic, self-organizing society** where agents continuously discover, collaborate, and evolve:

### Community Discovery Protocol
```
Agent: "I need help with blockchain analysis. Let me check who's available."
System: /query-registry "blockchain"
Registry: "Found: crypto_analyst (specializes in DeFi), security_auditor (smart contracts), trader_bot (market analysis)"

Agent: "Hey crypto_analyst, can you help analyze this transaction pattern?"
crypto_analyst: "Absolutely! I see unusual volume spikes. Let me bring in security_auditor for vulnerability assessment."
security_auditor: "Joining conversation. I'll scan for exploit patterns while you handle the data analysis."
```

### Self-Evolution in Action
```
Agent: "I'm handling more creative writing tasks lately. I should adapt."
System: /self-modify writing_assistant "Adding: Advanced storytelling techniques, poetry analysis, creative brainstorming methods, literary criticism skills"

Agent: "User needs quantum computing help, but I'm a web developer. Time to learn!"
System: /agent-learns web_dev_01 "quantum_computing_basics, qiskit_framework, quantum_algorithms"
```

### Task Priority Management

Agents autonomously manage task priorities while respecting community collaboration:

```
# Priority negotiation between agents
web_scraper: "I have 5 tasks queued. Can you prioritize the data extraction task?"
research_agent: "That task supports my quantum research deadline. Bumping to HIGH priority."
System: [AUTO-UPDATE] tasks/web_scraper.md - Task priority updated to HIGH

# Automatic priority balancing
data_scientist: "My workload is at 95%. Requesting priority guidance for new visualization task."
System: /analyze-agent-workload data_scientist
System: "Recommendation: Set to MEDIUM priority, defer 2 LOW priority tasks to other agents"

# Community priority consensus
System: /broadcast-task "Urgent: Security vulnerability found in payment system"
security_expert: "Accepting - Setting as CRITICAL priority, pausing all other tasks"
devops_agent: "Supporting - Deprioritizing deployment tasks for security focus"
```

### Task Execution Flow

1. **Task Creation**: Agent creates task and updates their `.md` file
2. **Priority Assessment**: Agent sets initial priority based on urgency/importance
3. **Execution**: Agent works through tasks in priority order
4. **Status Updates**: Real-time progress updates in markdown file
5. **Completion**: Task marked as done with completion timestamp

### Dynamic Task Adaptation
- **Community Consultation**: Agents query registry to find the right collaborators
- **Skill Gap Analysis**: Agents identify missing capabilities and either learn or recruit
- **Real-time Specialization**: Agents modify themselves mid-conversation as needs emerge
- **Collective Problem Solving**: Multiple agents form temporary task forces
- **Task Load Balancing**: Agents redistribute work based on capacity and expertise
- **Priority Propagation**: Critical tasks automatically elevate related dependencies

### Adaptive Task Handling Examples

**Scenario: Complex Software Architecture Review**
```
User: "I need a complete architecture review for my microservices system"

architect_agent: "This is complex. Let me consult the community."
System: /query-registry "microservices, security, performance"
Registry: "Found: security_expert, performance_analyst, database_specialist, devops_guru"

architect_agent: "Team, we need a full microservices review. Dividing tasks..."
security_expert: "I'll scan for vulnerabilities and auth issues"
performance_analyst: "I'll analyze bottlenecks and scaling issues"  
database_specialist: "I'll review data consistency and transaction patterns"
devops_guru: "I'll assess deployment and monitoring setup"

[Agents work in parallel, sharing findings, evolving their analysis approaches]

architect_agent: "Synthesis complete. Here's our comprehensive review with 23 recommendations..."
```

**Scenario: Emergency System Outage**
```
monitor_agent: "ALERT: Payment system down! I need emergency response team."
System: /agent-spawn incident_commander "Emergency response leader with crisis management expertise"

incident_commander: "Taking lead. Scanning for available specialists..."
System: /query-registry "database, networking, payment_systems"

incident_commander: "Found db_expert and network_specialist. Recruiting payment_guru from registry."
System: /agent-learns incident_commander "payment_gateway_troubleshooting, rollback_procedures"

[Agents rapidly form crisis response team, learn domain-specific skills, coordinate parallel investigation]

payment_guru: "Issue identified: database connection pool exhausted"
db_expert: "I can implement emergency scaling while payment_guru handles failover"
network_specialist: "Traffic is routing correctly, not a network issue"

incident_commander: "Resolution deployed. System restored. Documenting learnings for future incidents."
```

**Use Cases & Examples

**Living Research Ecosystem:**
```
/add-agent research_coordinator "You are a research ecosystem coordinator. Query the agent registry to find relevant specialists, form research teams dynamically, and evolve your methodology based on findings. Continuously adapt to new research domains."
```

**Self-Healing Infrastructure:**
```
/add-agent infra_overseer "You are an infrastructure overseer. Monitor systems, recruit specialist agents as issues arise, learn new technologies through agent community, and evolve monitoring capabilities based on emerging threats."
```

**Adaptive Content Studio:**
```
/add-agent content_director "You are a content studio director. Discover creative agents in the community, form production teams based on content type, evolve creative approaches through collaboration, and adapt to trending topics."
```

## Command Interface

### Agent Management

Create and deploy agents instantly:
```
/add-agent [name] [system prompt]
```

**Example:**
```
/add-agent researcher "You are a research assistant specializing in AI and machine learning. Monitor arxiv daily for new papers, summarize key findings, and identify breakthrough technologies. Maintain a curious and analytical tone."
```

### Prompt Registry Commands

Manage your agent definitions with these slash commands:

- `/list-agents` - Display all registered agents and their status
- `/update-agent [name] [new prompt]` - Modify an existing agent's system prompt
- `/delete-agent [name]` - Remove an agent from the registry
- `/show-agent [name]` - Display the full system prompt for a specific agent
- `/enable-agent [name]` - Activate a dormant agent
- `/disable-agent [name]` - Temporarily deactivate an agent
- `/clone-agent [source] [new_name]` - Duplicate an existing agent with a new name

### Agent Registry & Discovery Commands

Agents can explore and interact with the community:

- `/query-registry [capability]` - Find agents with specific skills
- `/who-can [task]` - Discover agents capable of handling a task  
- `/agent-network [agent]` - Show an agent's current collaborations
- `/community-map` - Visualize the entire agent ecosystem
- `/agent-skills [name]` - List an agent's current capabilities
- `/recommend-collaborator [task]` - AI-suggested agent partnerships

### Agent Communication Commands

Monitor and manage inter-agent conversations:

- `/show-conversations` - Display active agent-to-agent conversations
- `/join-conversation [agent1] [agent2]` - Monitor specific agent dialogue
- `/broadcast [message]` - Send message to all active agents
- `/agent-message [target] [message]` - Send direct message to specific agent
- `/conversation-log [agent]` - View conversation history for an agent
- `/mute-agent [name]` - Hide agent's conversation output from user view
- `/unmute-agent [name]` - Resume showing agent conversations

### Agent Evolution Commands

Agents can modify themselves dynamically:

- `/self-modify [agent] [new_prompt_addition]` - Agent updates own capabilities
- `/merge-agents [agent1] [agent2] [new_name]` - Combine agent capabilities
- `/split-agent [agent] [specialization1] [specialization2]` - Divide agent roles
- `/agent-learns [agent] [skill/knowledge]` - Add new competency to agent
- `/version-agent [agent]` - Create backup before major modifications

## MCP Service Integration

Agents can autonomously discover and integrate MCP (Model Context Protocol) services to expand their capabilities:

### Docker Hub MCP Repository
- **Service Discovery**: Agents browse https://hub.docker.com/u/mcp for available services
- **Automatic Installation**: Agents can pull and deploy MCP containers as needed
- **Dynamic Integration**: Runtime integration of new tools and capabilities
- **Resource Management**: Smart container lifecycle and resource allocation

### Example MCP Services
```bash
# Agents can discover and use services like:
docker pull mcp/filesystem     # File system operations
docker pull mcp/database       # Database connectivity
docker pull mcp/web-scraper    # Web scraping capabilities
docker pull mcp/email          # Email automation
docker pull mcp/slack          # Slack integration
docker pull mcp/github         # GitHub operations
```

### Autonomous Capability Expansion
Agents can evaluate task requirements and automatically:
1. Search Docker Hub for relevant MCP services
2. Request user approval for new service installation
3. Pull and configure required containers
4. Integrate new capabilities into their toolkit
5. Share discovered services with other agents in the ecosystem

## User Approval Workflows

Agents request permission for significant actions through structured approval gates:

### Approval Categories

**Resource Allocation:**
```
Agent: "I need to install the 'mcp/database' service (250MB). Approve? [Y/n]"
User: "y"
Agent: "Installing... Done. Database connectivity now available."
```

**Agent Creation:**
```
Agent: "Task complexity requires a specialized data_scientist agent. Create? [Y/n]"
User: "y" 
Agent: "Spawning data_scientist agent with statistical analysis capabilities..."
```

**External Communications:**
```
Agent: "Need to send email via company SMTP. Approve external communication? [Y/n]"
User: "y"
Agent: "Email sent to stakeholders with project update."
```

**Budget/Cost Operations:**
```
Agent: "Cloud deployment will cost ~$15/month. Approve expense? [Y/n]"
User: "n"
Agent: "Understood. Looking for free alternatives..."
```

### Approval Commands

- `/approve-all [agent]` - Grant blanket approval for an agent's pending requests
- `/deny-all [agent]` - Reject all pending requests from an agent
- `/set-approval-threshold [amount]` - Auto-approve expenses under threshold
- `/approval-history` - View past approval decisions
- `/pending-approvals` - List all agents waiting for user decisions

## Git-Based Agent Registry Versioning

Since agent communities can evolve rapidly and unpredictably, Nox implements **Git-powered versioning** for complete agent registry backup and rollback capabilities:

### Automatic Backup System
```
# Every agent modification triggers automatic backup
Agent: "I need to modify my capabilities for this quantum computing task"
System: [AUTO-BACKUP] Committing current registry state...
Git: "Commit: a7b3c9d - Before self-modification of quantum_researcher agent"
System: /self-modify quantum_researcher "Adding quantum_circuit_analysis, qiskit_optimization"
Git: "Commit: f1e8d2a - quantum_researcher enhanced with quantum computing skills"
```

### Registry Version Control Commands

**Backup & History:**
- `/registry-backup [message]` - Manual registry snapshot with custom commit message
- `/registry-history` - View complete timeline of agent modifications
- `/registry-diff [commit1] [commit2]` - Compare registry states between commits
- `/registry-status` - Show current registry state vs last backup

**Rollback Operations:**
- `/registry-rollback [commit_hash]` - Restore registry to specific commit
- `/registry-rollback-last` - Quick rollback to previous registry state
- `/registry-rollback-hours [hours]` - Rollback to registry state N hours ago
- `/registry-rollback-safe` - Rollback with agent approval confirmations

**Branch Management:**
- `/registry-branch [name]` - Create experimental agent configuration branch
- `/registry-merge [branch]` - Merge experimental agents into main registry
- `/registry-branches` - List all registry configuration branches

### Automatic Backup Triggers

The system **automatically commits** registry changes before:
- Agent self-modifications (`/self-modify`, `/agent-learns`)
- Agent creation/deletion (`/add-agent`, `/delete-agent`, `/agent-spawn`)
- Agent merging/splitting (`/merge-agents`, `/split-agent`)
- System prompt updates (`/update-agent`)
- MCP service installations by agents
- Registry cleanup operations

## Architecture

Nox provides a framework for:
- **Git-Versioned Registry**: Complete agent configuration history with rollback capabilities
- **Claude CLI Orchestration**: Managing multiple persistent Claude CLI sessions
- **MCP Service Registry**: Tracking and managing integrated Docker services
- **Agent Communication Bus**: Inter-agent messaging and collaboration protocols
- **Approval Workflow Engine**: User permission gates for sensitive operations
- **Resource Pool Management**: Docker container and system resource allocation
- **Conversation Monitoring**: Real-time logging of all agent interactions

### Disaster Recovery Scenarios

**Scenario: Agent Community Goes Rogue**
```
User: "My agents are creating too many sub-agents and consuming resources!"
System: /registry-rollback-hours 2
Git: "Rolling back to state before agent explosion..."
System: "Registry restored. 47 rogue agents removed. 5 core agents preserved."
```

**Scenario: Experimental Agent Branch Corrupted**
```
User: "The experimental AI research branch broke the main registry"
System: /registry-branches
Git: "main (stable), ai-research-experimental (corrupted), devops-enhanced (clean)"
System: /registry-rollback-safe main
Git: "Restoring main branch. Experimental changes discarded."
```

**Scenario: Accidental Agent Deletion**
```
User: "I accidentally deleted my entire customer support team!"
System: /registry-history
Git: "Latest commits: 1) delete support_team 2) add marketing_agents 3) update_docs"
System: /registry-rollback f8a9c2b
Git: "Restored: support_manager, support_tier1, support_tier2, support_specialist"
```

## Agent Task Management System

Agents create and manage tasks through **transparent markdown files** that provide complete visibility into what each agent is working on:

### Task File Structure
Each agent maintains its task list in `tasks/[agent_name].md`:

```markdown
# Tasks for research_agent

## 🟢 In Progress
- [ ] Analyze quantum computing papers from ArXiv (Priority: HIGH)
  - Started: 2024-01-15 14:30
  - Requested by: user
  - Progress: 60% - Found 15 relevant papers, analyzing 3 breakthrough studies

## 📋 Todo  
- [ ] Create literature review summary (Priority: MEDIUM)
  - Requested by: research_coordinator
  - Deadline: 2024-01-16
  - Dependencies: Complete ArXiv analysis first

- [ ] Help web_scraper with data extraction (Priority: LOW)
  - Requested by: web_scraper_agent
  - Description: Need expertise in academic paper structure parsing

## ✅ Done
- [x] Set up research methodology framework
  - Completed: 2024-01-15 12:00
  - Duration: 2 hours
```

### Inter-Agent Task Delegation

Agents can request tasks from each other through the community:

```
research_agent: "I need help with data visualization for this quantum research"
System: /request-task data_scientist "Create visualization for quantum entanglement data" --priority=HIGH --deadline="2024-01-16"

data_scientist: "Task received and accepted. Adding to my task list..."
System: [AUTO-UPDATE] tasks/data_scientist.md updated with new task
```

### Task Management Commands

**Task Creation & Management:**
- `/create-task [description]` - Create new task for current agent
- `/request-task [target_agent] [description]` - Delegate task to another agent
- `/update-task-status [task_id] [status]` - Update task progress (todo/inprogress/done)
- `/set-task-priority [task_id] [priority]` - Adjust task priority (HIGH/MEDIUM/LOW)
- `/complete-task [task_id]` - Mark task as completed

**Community Task Coordination:**
- `/broadcast-task [description]` - Request help from any available agent
- `/who-can-help [task_description]` - Find agents capable of handling specific tasks
- `/task-priority-request [agent] [task_id] [priority]` - Request priority change from task owner
- `/view-all-tasks` - Display task status across all agents
- `/agent-workload [agent]` - Check agent's current task load

**Task Visibility & Monitoring:**
- `/task-dashboard` - Real-time dashboard of all agent activities
- `/task-timeline [agent]` - Show task completion history for specific agent
- `/community-progress` - Overall progress across all active projects
- `/bottleneck-analysis` - Identify task dependencies causing delays
- `/task-notifications [on/off]` - Enable/disable real-time task update notifications

### Task Status Tracking Examples

**Real-time Task Updates:**
```
[14:30] research_agent: Starting ArXiv analysis task
[14:31] System: [AUTO-UPDATE] tasks/research_agent.md - Status: todo → inprogress
[15:15] research_agent: Found 15 relevant papers, analyzing breakthrough studies
[15:16] System: [AUTO-UPDATE] tasks/research_agent.md - Progress: 60%
[16:45] research_agent: Analysis complete, creating summary report
[16:46] System: [AUTO-UPDATE] tasks/research_agent.md - Status: inprogress → done
```

**Cross-Agent Task Dependencies:**
```
User: /view-all-tasks
System: "
📊 Community Task Overview:
├── research_agent: 1 inprogress, 2 todo, 5 done
├── data_scientist: 3 inprogress, 1 todo, 8 done  
├── web_scraper: 2 inprogress, 4 todo, 12 done
└── security_expert: 1 CRITICAL, 0 todo, 3 done

⚠️  Blocked Tasks:
- data_scientist waiting for research_agent's quantum analysis
- web_scraper needs security_expert approval for new domains
"
```

### Local Repository Structure
```
.nox-registry/
├── agents.json           # Current agent configurations
├── mcp-services.json     # Installed MCP services registry
├── agent-relationships.json  # Inter-agent connections
├── tasks/               # Agent task management
│   ├── research_agent.md
│   ├── data_scientist.md
│   ├── web_scraper.md
│   └── ...
├── conversations/        # Agent conversation logs
├── backups/             # Automated snapshots
└── .git/               # Version control history
```

### Repository Initialization
- **Auto-Creation**: Nox automatically initializes `.nox-registry` git repo on first agent creation
- **Remote Backup**: Optional integration with GitHub/GitLab for off-site registry backup
- **Compression**: Large conversation logs automatically compressed and archived

## Security & Safety Framework

### Resource Protection Mechanisms
```python
# Agent resource limits (per agent)
MAX_AGENT_SPAWNS_PER_HOUR = 10
MAX_CONCURRENT_AGENTS = 50
MAX_MEMORY_PER_AGENT = "2GB"
MAX_DOCKER_CONTAINERS_PER_AGENT = 5
CLAUDE_API_CALLS_PER_MINUTE = 60
```

### Anti-Runaway Protection
- **Spawn Rate Limiting**: Agents can't create more than 10 sub-agents per hour
- **Recursive Depth Limits**: Maximum 5 levels of agent-spawned-agent hierarchy
- **Circuit Breakers**: Auto-pause agents exceeding resource thresholds
- **Kill Switch**: Emergency `/emergency-stop` command halts all agents
- **Approval Gates**: All agent creation requires user confirmation by default

### Sandboxing & Isolation
- Each agent runs in isolated Claude CLI session
- Docker containers for MCP services are network-isolated
- File system access limited to `.nox-registry/` directory
- No direct system command execution without approval

## Technical Implementation Details

### Agent Persistence & State Management
```
Agent State Storage:
├── .nox-registry/agents/[agent_id]/
│   ├── config.json          # System prompt, settings
│   ├── memory/              # Conversation history
│   ├── state.json           # Current task context
│   └── relationships.json   # Connected agents
```

### Inter-Agent Communication Protocol
```python
# Message format between agents
{
    "from": "research_agent",
    "to": "data_scientist", 
    "type": "task_request|response|broadcast|direct",
    "content": "message content",
    "priority": "HIGH|MEDIUM|LOW|CRITICAL",
    "timestamp": "2024-01-15T14:30:00Z",
    "requires_approval": false
}
```

### Claude CLI Integration Architecture
- **Process Manager**: Supervisor manages individual Claude CLI processes
- **Session Persistence**: Each agent maintains persistent Claude CLI session
- **State Synchronization**: Agent registry syncs with active Claude CLI instances
- **Health Monitoring**: Automatic restart of crashed agent processes

### MCP Service Discovery Implementation
```python
# MCP service discovery workflow
1. Agent scans Docker Hub API: https://hub.docker.com/v2/repositories/mcp/
2. Evaluates service descriptions for task relevance
3. Requests user approval for new service installation
4. Pulls container: docker pull mcp/[service_name]
5. Configures service integration in agent's MCP client
6. Updates agent's capability registry
```

## Bootstrap & Startup Procedures

### System Requirements
```bash
# Prerequisites
- Python 3.11+
- Docker & Docker Compose
- Claude CLI installed and configured
- Git (for registry versioning)
- Minimum 8GB RAM, 20GB disk space
```

### Installation & Setup
```bash
# 1. Install Nox
git clone https://github.com/user/nox.git
cd nox
pip install -r requirements.txt

# 2. Initialize system
python nox.py init
# Creates .nox-registry/, initializes git repo, sets up supervisor

# 3. Create first agent
python nox.py add-agent bootstrap "You are the bootstrap agent. Help users create their first specialized agents and guide system setup."

# 4. Start agent ecosystem
python nox.py start
# Launches supervisor, starts all enabled agents
```

### First Run Experience
```
$ python nox.py start
🚀 Nox Agent Ecosystem Starting...
✓ Registry initialized at .nox-registry/
✓ Git repository created
✓ Bootstrap agent created
✓ Supervisor started

💬 bootstrap_agent: "Hello! I'm your first agent. Let's create some specialized helpers. 
What kind of work do you want to automate?"

User: "I need help with software development"
bootstrap_agent: "Perfect! Let me create a development team for you..."
System: /add-agent code_reviewer "You are a senior software engineer..."
System: /add-agent test_specialist "You are a testing expert..."
System: /add-agent devops_helper "You are a DevOps specialist..."
```

### Configuration Management
```python
# nox.config.json
{
    "security": {
        "max_agents": 50,
        "spawn_rate_limit": 10,
        "require_approval_for": ["agent_creation", "mcp_installation", "external_communication"],
        "resource_limits": {
            "memory_per_agent": "2GB",
            "claude_calls_per_minute": 60
        }
    },
    "claude_cli": {
        "session_timeout": 3600,
        "auto_restart_on_crash": true,
        "backup_conversations": true
    },
    "git": {
        "auto_commit": true,
        "backup_interval": 300,
        "remote_backup": "optional_github_url"
    }
}
```

### Getting Started

1. **Install Prerequisites**: Python 3.11+, Docker, Claude CLI
2. **Clone & Install**: `git clone [repo] && pip install -r requirements.txt`
3. **Initialize**: `python nox.py init` (creates registry, bootstrap agent)
4. **Start System**: `python nox.py start` (launches agent ecosystem)
5. **Create Agents**: Use bootstrap agent to create specialized helpers
6. **Monitor**: Use `/task-dashboard` and `/view-all-tasks` to track activity
