#!/usr/bin/env node

/**
 * Script to directly send a message to the test agent via API
 */

const http = require('http');

const postData = JSON.stringify({
  message: "Please search for the latest news about Google and provide a comprehensive summary. Use web search to find current information."
});

const options = {
  hostname: 'localhost',
  port: 3001,
  path: '/api/agents/test-agent-001/message',
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Content-Length': Buffer.byteLength(postData)
  }
};

console.log('🔗 Sending direct message to test-agent-001...');

const req = http.request(options, (res) => {
  console.log(`📊 Status: ${res.statusCode}`);
  console.log(`📋 Headers:`, res.headers);
  
  let data = '';
  res.on('data', (chunk) => {
    data += chunk;
  });
  
  res.on('end', () => {
    console.log('📤 Response:');
    try {
      const response = JSON.parse(data);
      console.log(JSON.stringify(response, null, 2));
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

console.log('✅ Message sent! Check the conversation file for response.');