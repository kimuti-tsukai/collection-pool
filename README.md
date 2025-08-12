# Collection Pool

A high-performance object pool library for Rust collections that helps reduce memory allocation overhead by reusing collection instances.

## Features

- **Zero-allocation reuse**: Reuse collections instead of allocating new ones
- **Generic design**: Works with any type that implements the `Clearable` trait
- **Built-in support**: Pre-configured pools for common Rust collections
- **Thread-safe variants**: Multi-threaded support with `Mutex`-based pools
- **Memory efficient**: Preserves allocated capacity when returning objects to the pool
- **Easy to use**: Simple API with automatic cleanup via RAII

## Supported Collections

- `Vec<T>` - Dynamic arrays
- `HashMap<K, V>` - Hash maps
- `HashSet<T>` - Hash sets
- `String` - Owned strings
- `VecDeque<T>` - Double-ended queues
- `BinaryHeap<T>` - Priority queues

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
collection-pool = { git = "https://github.com/kimuti-tsukai/collection-pool" }
```

## Quick Start

### Basic Usage

```rust
use collection_pool::{VecPool, HashMapPool, StringPool};

// Create a pool for Vec<i32>
let vec_pool = VecPool::new();

{
    // Borrow a Vec from the pool
    let mut vec = vec_pool.get();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    println!("Vector: {:?}", &*vec); // [1, 2, 3]
} // Vec is automatically returned to pool and cleared

// The same Vec instance is reused
{
    let vec = vec_pool.get();
    println!("Length: {}", vec.len()); // 0 (cleared, but capacity preserved)
}
```

### HashMap Example

```rust
use collection_pool::HashMapPool;

let map_pool = HashMapPool::new();

{
    let mut map = map_pool.get();
    map.insert("hello".to_string(), 42);
    map.insert("world".to_string(), 24);
    println!("Map size: {}", map.len()); // 2
}

// Map is cleared but hash table capacity is preserved
{
    let map = map_pool.get();
    println!("Map size: {}", map.len()); // 0
}
```

### Thread-Safe Usage

```rust
use collection_pool::sync::ThreadSafeVecPool;
use std::sync::Arc;
use std::thread;

let pool = Arc::new(ThreadSafeVecPool::new());

let handles: Vec<_> = (0..10)
    .map(|i| {
        let pool = Arc::clone(&pool);
        thread::spawn(move || {
            let mut vec = pool.get();
            vec.push(i);
            println!("Thread {} pushed {}", i, i);
        })
    })
    .collect();

for handle in handles {
    handle.join().unwrap();
}
```

### Pre-warming the Pool

You can pre-allocate objects in the pool to avoid allocation during hot paths:

```rust
use collection_pool::VecPool;

let pool = VecPool::new();

// Pre-allocate 10 Vec instances
pool.prewarm(10).unwrap();

println!("Pool size: {}", pool.pool_size().unwrap()); // 10
```

## API Reference

### Pool Operations

- `Pool::new()` - Create a new empty pool
- `pool.get()` - Borrow an object from the pool (creates new if pool is empty)
- `pool.prewarm(count)` - Pre-allocate objects in the pool
- `pool.pool_size()` - Get the number of available objects in the pool

### Available Pool Types

#### Single-threaded
- `VecPool<T>` - Pool of `Vec<T>`
- `HashMapPool<K, V>` - Pool of `HashMap<K, V>`
- `HashSetPool<T>` - Pool of `HashSet<T>`
- `StringPool` - Pool of `String`
- `VecDequePool<T>` - Pool of `VecDeque<T>`
- `BinaryHeapPool<T>` - Pool of `BinaryHeap<T>`

#### Thread-safe (in `sync` module)
- `ThreadSafeVecPool<T>`
- `ThreadSafeHashMapPool<K, V>`
- `ThreadSafeHashSetPool<T>`
- `ThreadSafeStringPool`
- `ThreadSafeVecDequePool<T>`
- `ThreadSafeBinaryHeapPool<T>`

## Custom Types

You can create pools for your own types by implementing the `Clearable` trait:

```rust
use collection_pool::{Pool, Clearable};
use std::cell::RefCell;

struct MyBuffer {
    data: Vec<u8>,
    metadata: String,
}

impl Clearable for MyBuffer {
    fn clear(&mut self) {
        self.data.clear();
        self.metadata.clear();
    }
}

impl Default for MyBuffer {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            metadata: String::new(),
        }
    }
}

// Create a pool for your custom type
type MyBufferPool = Pool<MyBuffer, RefCell<Vec<MyBuffer>>>;

let pool = MyBufferPool::new();
let mut buffer = pool.get();
buffer.data.extend_from_slice(b"Hello, World!");
```

## Performance Benefits

Object pools are particularly beneficial in scenarios with:

- **High-frequency allocations**: Tight loops that create and destroy collections
- **Large collections**: Collections that allocate significant memory
- **Hot paths**: Performance-critical code where allocation overhead matters
- **Server applications**: Long-running applications that benefit from reduced GC pressure

## Memory Management

- Objects are automatically returned to the pool when the `Pooled` guard is dropped
- Collections are cleared but their allocated capacity is preserved
- If the pool's mutex is poisoned (thread-safe variants), objects are safely destroyed
- No memory leaks: unreturnable objects are properly cleaned up

## Thread Safety

- Single-threaded pools use `RefCell` for interior mutability
- Thread-safe pools use `Mutex` and can be shared across threads with `Arc`
- All operations are safe and panic-free under normal conditions

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.