# WebSocket Stability Improvements

## Current Issues

The current WebSocket implementation in `src/api/websocket.rs` has several issues that need to be addressed:

1. **API Compatibility**: The code uses `session.send()` method, but this method doesn't exist in actix-ws 0.2. The correct way to send messages is likely using `session.text()` or `session.binary()`.

2. **Formatting Issues**: There are formatting issues in the debug logs, particularly with the `CloseCode` and `Option<String>` types.

3. **Method Name Error**: The code uses `is_elapsed()` method, but the correct method is `elapsed()`.

4. **Type Mismatches**: The broadcast functions return `Result<usize, SendError<String>>`, but the function signatures expect `Result<(), SendError<String>>`.

5. **Queue Processing**: The message queue processing is not optimized for high throughput (processes one message every 100ms).

6. **No Rate Limiting**: There's no rate limiting for incoming messages, which could lead to resource exhaustion under high load.

7. **No Reconnection Mechanism**: There's no mechanism for clients to reconnect if the connection is lost.

8. **Limited Error Handling**: Error handling is minimal (mostly just breaking the connection).

## Recommended Improvements

### 1. Fix API Compatibility Issues

Update the code to use the correct methods for sending messages with actix-ws 0.2:

```
// Instead of
if session.send(Message::Text(msg.into())).await.is_err() {
    break;
}

// Use
if session.text(msg).await.is_err() {
    break;
}
```

### 2. Fix Formatting Issues

Update the debug logs to use the debug formatter for types that don't implement Display:

```
// Instead of
debug!("Connection closed with code {}: {}", reason.code, reason.description);

// Use
debug!("Connection closed with code {:?}: {:?}", reason.code, reason.description);
```

### 3. Fix Method Name Error

```
// Instead of
if heartbeat_interval.tick().await.is_elapsed() {

// Use
if heartbeat_interval.tick().await.elapsed() > Duration::from_secs(0) {
```

### 4. Fix Type Mismatches

```
// Instead of
WEBSOCKET_MANAGER.sender().send(message.to_string())

// Use
WEBSOCKET_MANAGER.sender().send(message.to_string()).map(|_| ())
```

### 5. Optimize Message Queue Processing

Improve the message queue processing for higher throughput:

- Process messages in batches instead of one at a time
- Use a more efficient interval (50ms instead of 100ms)
- Implement a priority queue for important messages

```
let mut interval = tokio::time::interval(Duration::from_millis(50));
loop {
    interval.tick().await;
    
    let mut queue = queue_processor.lock().await;
    let batch_size = std::cmp::min(QUEUE_BATCH_SIZE, queue.len());
    
    if batch_size > 0 {
        let batch: Vec<_> = queue.drain(0..batch_size).collect();
        drop(queue); // Release the lock before sending
        
        for msg in batch {
            if session.text(msg).await.is_err() {
                return;
            }
        }
    }
}
```

### 6. Add Rate Limiting

Implement rate limiting for incoming messages to prevent resource exhaustion:

```
// Add a rate limiter
let mut message_count = 0;
let mut rate_limit_interval = tokio::time::interval(Duration::from_secs(1));

// In the message handling loop
message_count += 1;
if message_count > MAX_MESSAGES_PER_SECOND {
    if rate_limit_interval.tick().await.elapsed() > Duration::from_secs(0) {
        message_count = 0;
    } else {
        // Rate limit exceeded, ignore or throttle
        continue;
    }
}
```

### 7. Implement Reconnection Mechanisms

Add support for client reconnection:

- Add a unique client ID for each connection
- Store session state that can be restored on reconnection
- Implement a reconnection protocol with exponential backoff

### 8. Enhance Error Handling

Improve error handling throughout the WebSocket code:

- Add more detailed error logging
- Implement graceful degradation under high load
- Add circuit breakers to prevent cascading failures

## Constants for WebSocket Stability

Add these constants to improve WebSocket stability:

```
// Constants for WebSocket stability
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_QUEUE_SIZE: usize = 10000;
const QUEUE_BATCH_SIZE: usize = 10;
const QUEUE_PROCESS_INTERVAL: Duration = Duration::from_millis(50);
const MAX_MESSAGES_PER_SECOND: usize = 100;
const RECONNECT_INTERVAL: Duration = Duration::from_secs(5);
const MAX_RECONNECT_ATTEMPTS: usize = 3;
```

## Conclusion

Implementing these improvements will significantly enhance the stability and performance of the WebSocket implementation, especially under high load. The changes should be made incrementally, with thorough testing after each change to ensure that existing functionality is not broken.