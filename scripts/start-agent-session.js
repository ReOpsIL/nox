#!/usr/bin/env node

/**
 * Script to start the Claude CLI session for the test agent
 */

const http = require('http');

const postData = JSON.stringify({});

const options = {
  hostname: 'localhost',
  port: 3001,
  path: '/api/agents/test-agent-001/start',
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Content-Length': Buffer.byteLength(postData)
  }
};

console.log('🚀 Starting Claude CLI session for test-agent-001...');

const req = http.request(options, (res) => {
  console.log(`📊 Status: ${res.statusCode}`);
  
  let data = '';
  res.on('data', (chunk) => {
    data += chunk;
  });
  
  res.on('end', () => {
    console.log('📤 Response:');
    try {
      const response = JSON.parse(data);
      console.log(JSON.stringify(response, null, 2));
      
      if (response.success) {
        console.log('✅ Agent session started successfully!');
        console.log('🔗 You can now send messages to the agent.');
      } else {
        console.log('❌ Failed to start agent session.');
      }
    } catch (error) {
      console.log(data);
    }
  });
});

req.on('error', (error) => {
  console.error('❌ Error:', error);
});

req.write(postData);
req.end();