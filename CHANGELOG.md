# Changelog

All notable changes to this project will be documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.1.0] - 2026-03-11

Initial release.

- Lock-free SPSC ring buffer with overwrite-oldest semantics
- `Producer::push()` and `Producer::push_slice()` — never block, overwrite oldest on full
- `Consumer::pop()` and `Consumer::pop_slice()` — FIFO order
- `overwrite_count()` on both `Producer` and `Consumer` — tracks total samples overwritten
- `available()` and `capacity()` on both halves
- Monotonic position counters (no full-vs-empty ambiguity)
- `AtomicU32` data storage — no `unsafe`, no `UnsafeCell`
- Cache-line-padded `write_pos` / `read_pos` — prevents false sharing in cross-thread use
- Zero runtime dependencies
- Property-based tests (proptest), exhaustive concurrency tests (loom), fuzz target (libFuzzer)
- Miri-compatible test suite

### Performance (Apple M-series, release build, 1M samples)

| Benchmark                   | rt-ring | ringbuf 0.4 |
|-----------------------------|---------|-------------|
| push 1M (single-thread)     | 4.11 ms | —           |
| push+pop 1M (single-thread) | 4.91 ms | 5.03 ms     |
| cross-thread 1M             | 5.85 ms | 1.12 ms     |

The cross-thread gap vs ringbuf is expected — rt-ring pays a per-push CAS for
overwrite-oldest semantics that ringbuf's non-overwriting path avoids.
