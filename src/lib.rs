mod shared;

use shared::Shared;
use std::sync::Arc;

/// Creates a new SPSC ring buffer with the given capacity.
///
/// Capacity is rounded up to the next power of two.
/// Returns a `(Producer, Consumer)` pair.
///
/// # Panics
///
/// Panics if `capacity` is zero.
pub fn new(capacity: usize) -> (Producer, Consumer) {
    let shared = Arc::new(Shared::new(capacity));
    (
        Producer {
            shared: Arc::clone(&shared),
        },
        Consumer { shared },
    )
}

/// The producer half of the ring buffer. Not `Clone` — enforces single-producer.
pub struct Producer {
    shared: Arc<Shared>,
}

impl Producer {
    /// Push a sample into the buffer. If the buffer is full, the oldest sample
    /// is overwritten.
    pub fn push(&self, sample: f32) {
        self.shared.push(sample);
    }

    /// Push a slice of samples into the buffer. Samples that cause overflow
    /// overwrite the oldest data.
    pub fn push_slice(&self, samples: &[f32]) {
        self.shared.push_slice(samples);
    }

    /// Returns the number of samples currently available for reading.
    pub fn available(&self) -> usize {
        self.shared.available()
    }

    /// Returns the buffer capacity (always a power of two).
    pub fn capacity(&self) -> usize {
        self.shared.capacity()
    }
}

/// The consumer half of the ring buffer. Not `Clone` — enforces single-consumer.
pub struct Consumer {
    shared: Arc<Shared>,
}

impl Consumer {
    /// Pop the oldest sample from the buffer, or `None` if empty.
    pub fn pop(&self) -> Option<f32> {
        self.shared.pop()
    }

    /// Pop up to `buf.len()` samples into `buf`. Returns the number of samples read.
    pub fn pop_slice(&self, buf: &mut [f32]) -> usize {
        self.shared.pop_slice(buf)
    }

    /// Returns the number of samples currently available for reading.
    pub fn available(&self) -> usize {
        self.shared.available()
    }

    /// Returns the total number of samples that were overwritten since creation.
    pub fn overwrite_count(&self) -> u64 {
        self.shared.overwrite_count()
    }

    /// Returns the buffer capacity (always a power of two).
    pub fn capacity(&self) -> usize {
        self.shared.capacity()
    }
}

// Safety assertions: Producer and Consumer are Send (via Arc<Shared>), but not Clone.
// This is automatically derived — no manual unsafe impl needed.
// We add static assertions to catch regressions.
fn _assert_send<T: Send>() {}
const _: () = {
    let _ = _assert_send::<Producer>;
    let _ = _assert_send::<Consumer>;
};
