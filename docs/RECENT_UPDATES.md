# Recent Updates

## 2025-07-12: Agent Deletion Fix

### Issue Fixed
- **Problem**: Dashboard agent deletion was returning 404 errors
- **Cause**: Missing DELETE `/api/agents/:agentId` endpoint in the API routes
- **Impact**: Users could not delete agents from the web dashboard

### Solution Implemented
1. **Added DELETE endpoint** in `/src/dashboard/routes/agent-routes.ts`:
   - Validates agent exists in registry
   - Stops running agent process using `agentManager.killAgent()`
   - Removes agent from registry using `registryManager.deleteAgent()`
   - Returns proper JSON responses

2. **Fixed method name bug**:
   - Corrected `stopAgent()` call to `killAgent()` (actual method in AgentManager)

3. **Updated documentation**:
   - Enhanced API specification with DELETE endpoint details
   - Updated CLI reference with current commands
   - Fixed port references in README (dashboard on 3001, websocket on 3000)

### Verification
- ✅ Successfully deleted test agent `agent-1752298204931` (NewsHunter)
- ✅ Agent count properly decremented from 5 to 4
- ✅ Remaining agents intact and functional
- ✅ Dashboard deletion now returns proper success responses

### Technical Details
```typescript
// New DELETE endpoint
agentRouter.delete('/:agentId', async (req: Request, res: Response) => {
  const agentId = req.params.agentId as string;
  
  // Check if agent exists
  const agents = await registryManager.listAgents();
  const agent = agents.find(a => a.id === agentId);
  
  if (!agent) {
    return res.status(404).json({
      success: false,
      error: 'Agent not found',
      message: `Agent ${agentId} not found`
    });
  }

  // Stop running agent
  try {
    await agentManager.killAgent(agentId);
  } catch (error) {
    // Agent might not be running, continue with deletion
  }

  // Remove from registry
  await registryManager.deleteAgent(agentId);
  
  return res.json({
    success: true,
    message: `Agent ${agentId} deleted successfully`
  });
});
```

### Related Files Updated
- `/src/dashboard/routes/agent-routes.ts` - Added DELETE endpoint
- `/docs/API_SPECIFICATION.md` - Documented DELETE endpoint
- `/docs/CLI_REFERENCE.md` - Current CLI commands
- `/README.md` - Fixed port references and troubleshooting
- `/docs/RECENT_UPDATES.md` - This changelog

## Previous Updates

### Task Execution System
- Implemented complete task execution pipeline
- Connected TaskManager to AgentManager via event-driven architecture
- Tasks now properly transition from 'todo' → 'inprogress' → 'done'
- Claude CLI integration functional for real task execution

### System Reliability
- Fixed TypeScript compilation errors
- Resolved port conflicts in development/production modes
- Enhanced error handling and logging
- Improved system startup and shutdown procedures