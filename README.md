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

## Semantics

- **FIFO when not full:** if the consumer keeps up, samples arrive in exact push order with no gaps
- **Overwrite-oldest when full:** the producer never blocks; it discards the oldest unread sample to make room
- **Monotonic with gaps:** popped values are always in push order, but consecutive values may skip by up to `capacity` samples when overwrites occur
- **Accounting invariant:** `overwrite_count() + popped == pushed` — always, in every interleaving
- **SPSC only:** exactly one producer thread and one consumer thread; violating this is undefined behavior at the application level (no panic, just wrong results)
- **Capacity is a power of two:** the requested capacity is rounded up, enabling fast modular indexing

## Features

- Overwrite counter tracks how many samples were dropped
- Bulk `push_slice` / `pop_slice` operations
- `available()` to query current buffer occupancy
- Capacity automatically rounds up to the next power of two

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
