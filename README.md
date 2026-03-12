# rt-ring

Lock-free SPSC (single-producer, single-consumer) ring buffer that **overwrites the oldest data** when full — designed for real-time audio and similar domains where freshness matters more than completeness.

## Key Properties

- **Overwrite-oldest semantics:** when the buffer is full, new writes discard the oldest sample instead of failing
- **Lock-free, real-time safe:** no allocations, locks, or IO in the hot path
- **Safe Rust API:** no `unsafe` code
- **Zero runtime dependencies**

## Usage

```rust
use rt_ring::{new, Producer, Consumer};

let (producer, consumer) = rt_ring::new(1024);

// Producer side (e.g., audio input callback)
producer.push(0.5);
producer.push_slice(&[0.1, 0.2, 0.3]);

// Consumer side (e.g., processing thread)
while let Some(sample) = consumer.pop() {
    // process sample
}

// Check how many samples were overwritten
println!("overwrites: {}", consumer.overwrite_count());
```

## Features

- Overwrite counter tracks how many samples were dropped
- Bulk `push_slice` / `pop_slice` operations
- `available()` to query current buffer occupancy
- Capacity automatically rounds up to the next power of two
