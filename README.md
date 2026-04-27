A high-performance, async key-value store written in Rust with Redis-compatible RESP protocol support.

## Features

**Core Commands**
- `GET key` - Retrieve a value
- `SET key value [EX seconds]` - Store a value with optional TTL
- `DEL key [key ...]` - Delete one or more keys
- `PING` - Health check
- `ECHO message` - Echo a message
**Advanced Features**
- **TTL/Key Expiration** - Automatic key expiration with `SET key value EX seconds`
- **Async Architecture** - Built with Tokio for high concurrency
- **RESP Protocol** - Redis-compatible serialization protocol for compatibility
- **In-Memory Storage** - Fast, thread-safe HashMap with RwLock
- **Background Cleanup** - Automatic expiration cleanup every 60 seconds

## Building

```bash
cargo build --release
```

## Running the Server

```bash
cargo run --release
```

The server listens on `127.0.0.1:6379` by default.

## Usage Examples

### Using the Included Test Client

```bash
cargo run --release --bin client
```

### Using redis-cli

```bash
# Connect to kvstore
redis-cli -p 6379

# Basic commands
> PING
PONG

> SET mykey "Hello"
OK

> GET mykey
"Hello"

> SET mykey "World" EX 10
OK

> DEL mykey
(integer) 1
```

### Using nc/telnet (raw RESP protocol)

```bash
# Connect and send PING
echo -ne "*1\r\n\$4\r\nPING\r\n" | nc localhost 6379

# SET key value
echo -ne "*3\r\n\$3\r\nSET\r\n\$3\r\nkey\r\n\$5\r\nvalue\r\n" | nc localhost 6379

# GET key
echo -ne "*2\r\n\$3\r\nGET\r\n\$3\r\nkey\r\n" | nc localhost 6379
```

## Architecture

### Core Components

1. **KvStore** - Thread-safe in-memory storage
   - Uses `Arc<RwLock<HashMap>>` for concurrent access
   - Stores values with optional expiration timestamps

2. **RespCodec** - RESP protocol parser/encoder
   - Parses Redis Serialization Protocol arrays
   - Encodes responses (simple strings, bulk strings, integers, errors, nil)

3. **TCP Server** - Async connection handler
   - Spawns new task per connection
   - Streams processed line-by-line
   - Graceful error handling

4. **Cleanup Task** - Background maintenance
   - Runs every 60 seconds
   - Removes expired keys from store

## Performance Characteristics

- **Read/Write**: O(1) average case (HashMap operations)
- **Expiration Check**: O(1) per get, full scan every 60 seconds
- **Concurrency**: Unlimited concurrent connections via Tokio
- **Memory**: In-memory only (no persistence by default)

## Future Enhancements

- [ ] Persistent storage (RocksDB/SQLite)
- [ ] TTL as a separate priority queue for immediate expiration
- [ ] Additional commands (INCR, APPEND, LPUSH, RPUSH, etc.)
- [ ] Pub/Sub support
- [ ] Cluster mode
- [ ] Replication
