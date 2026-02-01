# Redis Server in Rust

A Redis server implementation built from scratch in Rust, following the [CodeCrafters "Build Your Own Redis"](https://codecrafters.io/challenges/redis) challenge. The goal is to learn Rust through a real systems programming project: async networking, protocol parsing, concurrency, and ownership.

## What's implemented

- **RESP protocol parser** -- hand-written, zero-copy parser for Redis Serialization Protocol (simple strings, bulk strings, arrays, errors, null)
- **Commands**: `PING`, `ECHO`, `GET`, `SET` (with `EX`/`PX` expiry flags)
- **Async I/O** -- Tokio-based TCP server with per-connection task spawning
- **TCP framing** -- incremental read buffer with proper frame detection (handles partial reads and pipelined commands)
- **Concurrent key-value store** -- `RwLock`-protected `HashMap` with lazy expiry on read
- **Error handling** -- structured error types with `thiserror`, connection-level errors with `anyhow`, RESP error responses sent back to clients on malformed input

## Architecture

```
src/
  main.rs          -- TCP accept loop, connection handling, read/write buffering
  resp/
    parser.rs      -- RESP protocol parser (returns consumed byte count for framing)
    value.rs       -- RESP value types + wire format serialization (Display)
    error.rs       -- Parse error types
  command/
    raw.rs         -- Raw command extraction from RESP values
    command.rs     -- Typed command validation (Command enum)
    execute.rs     -- Command execution against the store
    mod.rs         -- Pipeline: RESP value -> raw command -> typed command -> response
  store.rs         -- Thread-safe key-value store with TTL support
```

## What I'd do differently

- **Use `bytes::BytesMut`** instead of `Vec<u8>` for the read buffer -- `drain(..n)` on a `Vec` shifts all remaining bytes left (O(n)), while `BytesMut` advances an internal cursor (O(1))
- **Use `Arc<str>` for stored values** -- currently every `GET` clones the full `String` while holding the lock; `Arc<str>` would make reads a cheap reference count bump
- **Separate parser errors** -- `UnexpectedEOF` currently means both "need more data" and "connection sent truncated garbage"; a dedicated `Incomplete` variant would make the framing loop more explicit

## What's next

- Remaining Redis commands (`DEL`, `EXISTS`, `INCR`, `TTL`, ...)
- Pub/Sub
- RDB persistence
- Command pipelining throughput benchmarks

## Running

```sh
cargo run
```

Listens on `127.0.0.1:6379`. Test with any Redis client:

```sh
redis-cli PING
redis-cli SET foo bar EX 10
redis-cli GET foo
```
