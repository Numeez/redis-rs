# redis-rs

A Redis-like in-memory key-value server written from scratch in Rust, with **no dependencies**: no async runtime and no networking crates. The main data structures (the hash table, the AVL tree and the heap) are implemented by hand.

It follows the book [*Build Your Own Redis with C/C++*](https://build-your-own.org/redis/) by James Smith, translated to Rust. The book's design maps to Rust's ownership model in some interesting ways, described in [Design notes](#design-notes).

## Features

- **Event loop:** a single-threaded server built directly on the `poll(2)` system call, with non-blocking sockets that serve many clients at once
- **Custom hash table:** chaining, power-of-two capacity, FNV-1a hashing, and **incremental rehashing** so resizing never pauses the server
- **Sorted sets:** an **AVL tree** with subtree counts for O(log n) rank and offset queries, paired with a hash map for O(1) score lookups
- **Key expiry (TTL):** a generic **min-heap** of deadlines with lazy deletion, wired into `poll`'s timeout
- **Idle connection timeouts:** connections silent for 5 seconds are closed
- **Thread pool:** large sorted sets are freed on a background worker so the event loop never stalls
- **A binary protocol** with length-prefixed requests and typed responses (nil, error, string, integer, double, array)

## Quick start

Requires a recent stable Rust toolchain (1.88+, edition 2024). Runs on macOS and Linux.

```bash
cargo build

# terminal 1: start the server (listens on 127.0.0.1:6969)
./target/debug/redis-rs

# terminal 2: send commands with the bundled client
./target/debug/client set name numeez
./target/debug/client get name
```

## Commands

| Command | Description | Reply |
|---|---|---|
| `set key value` | Set a string value (clears any TTL) | nil |
| `get key` | Get a string value | string, or nil |
| `del key` | Delete a key of any type | `1` if deleted, `0` if not found |
| `keys` | List all keys | array of strings |
| `zadd key score name` | Add a member to a sorted set, or update its score | `1` if added, `0` if updated |
| `zrem key name` | Remove a member from a sorted set | `1` if removed, `0` if not found |
| `zscore key name` | Get a member's score | double, or nil |
| `zquery key score name offset limit` | Range query: start at the first member ≥ `(score, name)`, skip `offset`, return up to `limit` | array of `name, score` pairs |
| `pexpire key ms` | Set a time-to-live in milliseconds | `1` if set, `0` if the key doesn't exist |
| `pttl key` | Milliseconds left | number, `-1` if no TTL, `-2` if no key |

Sorted sets are ordered by score, then by name. Using the wrong type for a key (for example `get` on a sorted set) returns an error.

### Example session

```text
$ client zadd board 10 alice
(integer) 1
$ client zadd board 20 bob
(integer) 1
$ client zquery board 0 "" 0 10
1) alice
2) (double) 10
3) bob
4) (double) 20
$ client pexpire board 5000
(integer) 1
$ client pttl board
(integer) 4998
$ client get board
(error 1) expect string
```

## Protocol

All integers are little-endian.

**Request:** `u32 total_length`, then `u32 nstr`, then `nstr` strings, each encoded as `u32 length` followed by the bytes.

**Response:** `u32 total_length`, then one tagged value:

| Tag | Type | Payload |
|---|---|---|
| 0 | nil | none |
| 1 | error | `u32 code`, `u32 length`, message bytes |
| 2 | string | `u32 length`, bytes |
| 3 | integer | `i64` |
| 4 | double | `f64` |
| 5 | array | `u32 count`, followed by `count` tagged values |

## Project structure

```text
src/
├── main.rs        event loop, connection handling, timers
├── poll.rs        FFI binding to poll(2)
├── buffer.rs      input/output buffers and response framing
├── protocol.rs    response serialization
├── operation.rs   command parsing and execution
├── hash.rs        hash table with incremental rehashing
├── avl.rs         AVL tree with rank/select/range queries
├── key.rs         sorted-set key (score, name) and TTL heap entry
├── zset.rs        sorted set: hash map + AVL tree kept in sync
├── min_heap.rs    generic binary min-heap
├── threadpool.rs  worker pool for freeing large values
└── bin/client.rs  command-line client
```

## Design notes

Some parts of the book's C design don't map directly onto safe Rust. These are the choices made here:

- **No intrusive data structures.** The C version embeds tree and list nodes inside its data and uses parent pointers. Here, the tree and hash chains own their nodes through `Option<Box<Node>>`, and all operations are recursive functions that take a subtree and return the new one.
- **Rank and select, not parent-pointer walks.** Without parent pointers, range queries use each node's subtree `count`: `rank(key)` finds a position and `select(n)` finds the key at a position, both in O(log n).
- **Lazy deletion for timers.** Updating or removing a TTL never touches the heap. A new deadline pushes a new entry, and stale entries are discarded when they reach the top by comparing them with the key's current deadline. The idle-connection queue works the same way, using each client's last-active time.
- **One timeout for `poll`.** Before each `poll`, the loop computes the soonest deadline across idle connections and TTLs, capped at one hour, so timers fire on time even when no client is active.
- **Freeing off the event loop.** When `del`, `set` or expiry removes a sorted set with 1000 or more members, the value is moved into a closure and dropped on a worker thread. Rust's `Send` bound guarantees at compile time that this is safe.

## Testing

```bash
cargo test
```

The unit tests cover the AVL tree (insert and delete at every position, random operations checked against `BTreeSet`), rank, select and range queries, sorted-set ordering, and the min-heap.

`test.sh` is an end-to-end test of key expiry against a running server. Start the server, then run `bash test.sh`, and the results are written to `ttl_results.txt`.

## Limitations

This is a learning project, not a Redis replacement:

- it uses the book's own binary protocol, not RESP, so `redis-cli` and standard Redis clients can't connect
- there is no persistence: all data is lost when the server stops
- only strings and sorted sets are supported, with a small command set
- expired keys are removed by the timer loop only; there is no check when a key is accessed

## Acknowledgements

Based on [*Build Your Own Redis with C/C++*](https://build-your-own.org/redis/) by James Smith.