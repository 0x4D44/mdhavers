# network

A simulated, in-process network: messages, point-to-point channels, pub/sub topics, a broker, a router, an event emitter, a priority queue, a toy RPC server and a connection pool. Nothing touches a real socket; everything is plain mdhavers objects and callbacks. For real sockets see [network_real](./network_real.md).

Import: `fetch "stdlib/network"`

Pure mdhavers; runs on any backend that can run the interpreter. Loading the module prints `Network module loaded! Ready tae send messages across the Highlands!`. Helpers starting with `_` are internal and not documented.

Errors are returned as dicts (`{"success": nae, "error": "..."}`), never hurled.

## Constants

| Name | Value |
|---|---|
| `MSG_TYPE_DATA` | `"data"` |
| `MSG_TYPE_REQUEST` | `"request"` |
| `MSG_TYPE_RESPONSE` | `"response"` |
| `MSG_TYPE_EVENT` | `"event"` |
| `MSG_TYPE_ERROR` | `"error"` |

## Functions

### make_message(msg_type, payload, sender = "anonymous")
Returns a new `Message`.

### data_message(payload, sender = "anonymous")
`Message` of type `"data"`.

### request_message(payload, sender = "anonymous")
`Message` of type `"request"`.

### error_message(error_text, sender = "anonymous")
`Message` of type `"error"` whose payload is `{"error": error_text}`.

### make_channel(name, capacity = 100)
Returns a `Channel`.

### make_topic(name)
Returns a `Topic`.

### make_broker()
Returns a `MessageBroker`.

### make_router()
Returns a `Router`.

### make_emitter()
Returns an `EventEmitter`.

### make_queue(name)
Returns a `MessageQueue`.

### make_rpc_server(name)
Returns an `RPCServer`.

### make_pool(max_connections = 10)
Returns a `ConnectionPool`.

## Classes

### Message
`Message(msg_type, payload, sender = "anonymous")`. Fields: `id` (`"msg_1"`, `"msg_2"`, ... from a module-wide counter), `type`, `payload`, `sender`, `timestamp` (a module-wide logical counter, not wall-clock time), `headers` (dict), `replied` (`nae`).

- `with_header(key, value)`: sets a header; returns the message.
- `get_header(key)`: the value, or `naething`.
- `to_dict()`: dict of `id`, `type`, `payload`, `sender`, `timestamp`, `headers`.

### Channel
`Channel(name, capacity = 100)`. Fields: `name`, `capacity`, `messages`, `closed`, `listeners`. A FIFO queue.

- `send(message)`: returns `{"success": aye, "message_id": id}`, or `{"success": nae, "error": "Channel is closed"}` / `{"success": nae, "error": "Channel is full"}`. Calls every listener with the message after queueing it.
- `receive()`: removes and returns the oldest message, or `naething` if empty.
- `peek()`: oldest message without removing it, or `naething`.
- `size()`, `is_empty()`, `is_closed()`.
- `on_message(callback)`: registers a listener; returns the channel.
- `close()`: further sends fail; returns the channel.
- `clear()`: drops queued messages; returns the channel.

### Topic
`Topic(name)`. Pub/sub with history. Fields: `name`, `subscribers`, `message_history`, `max_history` (100), `retain_last` (`nae`), `last_message`.

- `subscribe(callback)`: adds a subscriber. With retention on, the new subscriber is immediately called with the last message. Returns the topic.
- `unsubscribe(callback)`: removes a subscriber; returns the topic.
- `publish(message)`: records history (trimmed to `max_history`) and calls every subscriber; returns the topic.
- `with_retention()`: turns on last-message retention; returns the topic.
- `subscriber_count()`, `get_history()`.

### MessageBroker
`MessageBroker()`. Holds named channels, topics and request handlers.

- `create_channel(name, capacity = 100)`: creates or returns the existing channel.
- `get_channel(name)`: the channel or `naething`.
- `delete_channel(name)`: returns the broker.
- `create_topic(name)`, `get_topic(name)`: likewise for topics.
- `publish(topic_name, message)`: `aye` if the topic exists, else `nae`.
- `subscribe(topic_name, callback)`: `aye` if the topic exists, else `nae`.
- `register_handler(request_type, handler)`: returns the broker.
- `request(request_type, payload)`: `{"success": aye, "response": handler(payload)}` or `{"success": nae, "error": "No handler fer <type>"}`.
- `stats()`: `{"channels", "topics", "handlers"}` counts.

### Router
`Router()`. Maps exact path strings to handlers.

- `route(path, handler)`: handler is called as `handler(context)`; returns the router.
- `use(middleware)`: middleware is `mw(path, context)`; a falsy result blocks dispatch. Returns the router.
- `on_not_found(handler)`: called as `handler(path, context)`; returns the router.
- `dispatch(path, context = {})`: returns `{"handled": aye, "result": ...}`, or `{"handled": aye, "result": ..., "not_found": aye}` via the not-found handler, or `{"handled": nae, "reason": "Middleware blocked"}` / `{"handled": nae, "reason": "Route not found"}`.
- `list_routes()`: list of registered paths.

### EventEmitter
`EventEmitter()`. Node.js style.

- `on(event, callback)`, `once(event, callback)`: register; return the emitter.
- `emit(event, data = naething)`: calls each callback with `data`; once-listeners are then cleared. Returns the number of callbacks called.
- `off(event, callback = naething)`: removes one callback, or all regular listeners for the event if none is given. Does not touch `once` listeners. Returns the emitter.
- `listener_count(event)`: regular plus once listeners.
- `events()`: list of event names that have ever had a listener.

### MessageQueue
`MessageQueue(name)`. Priority queue: higher priority number leaves first. Fields: `name`, `queue`, `dead_letter`, `max_retries` (3), `processing`.

- `enqueue(message, priority = 5)`: returns the queue.
- `dequeue()`: the highest-priority message, or `naething`.
- `peek()`: the same without removing it.
- `requeue(message)`: finds the message (by `.id`) still in the queue, bumps its retry count, and moves it to the dead-letter list once retries reach `max_retries`. Returns the queue.
- `size()`, `dead_letter_count()`, `get_dead_letters()`.
- `clear_dead_letters()`: returns the queue.

### RPCServer
`RPCServer(name)`.

- `register(method_name, handler)`: returns the server.
- `call(method_name, params = [])`: `{"success": aye, "result": handler(params)}`; an unknown method gives `{"success": nae, "error": "Method '<name>' not found"}`; an error inside the handler is caught and returned as `{"success": nae, "error": <text>}`.
- `list_methods()`.

### ConnectionPool
`ConnectionPool(max_connections = 10)`. Simulated: connections are dicts with `id`, `created_at`, `last_used`.

- `acquire()`: `{"success": aye, "connection": conn}`, reusing an available one or creating a new one under the limit; otherwise `{"success": nae, "error": "Pool exhausted"}`.
- `release(conn)`: returns `aye` if the connection was in use, else `nae`.
- `stats()`: `{"total", "available", "in_use", "max"}`.

## Example

```mdhavers
fetch "stdlib/network"

ken topic = make_topic("news")
topic.subscribe(|m| { blether "got " + m.payload })
topic.publish(data_message("hello", "me"))

ken ch = make_channel("jobs", 1)
blether ch.send(data_message(1))["success"]
blether ch.send(data_message(2))["error"]
```

## Compared with network_real

`network` is a pure simulation for message-passing patterns inside one program: no sockets, no bytes, no DNS, and nothing leaves the process. [network_real](./network_real.md) wraps actual OS sockets (UDP/TCP) and DNS. Use `network` for tests, prototypes and in-process messaging; use `network_real` when data must cross a machine boundary. The two share no function names, so both can be fetched together.
