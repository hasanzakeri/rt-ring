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
///
/// # Examples
///
/// ```
/// let (p, c) = rt_ring::new(4);
/// p.push(1.0);
/// p.push(2.0);
/// assert_eq!(c.pop(), Some(1.0));
/// assert_eq!(c.pop(), Some(2.0));
/// assert_eq!(c.pop(), None);
/// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// let (p, c) = rt_ring::new(4);
    /// p.push(0.5);
    /// assert_eq!(c.pop(), Some(0.5));
    /// ```
    pub fn push(&self, sample: f32) {
        self.shared.push(sample);
    }

    /// Push a slice of samples into the buffer. Samples that cause overflow
    /// overwrite the oldest data.
    ///
    /// # Examples
    ///
    /// ```
    /// let (p, c) = rt_ring::new(4);
    /// p.push_slice(&[1.0, 2.0, 3.0]);
    /// let mut buf = [0.0f32; 3];
    /// let n = c.pop_slice(&mut buf);
    /// assert_eq!(n, 3);
    /// assert_eq!(&buf, &[1.0, 2.0, 3.0]);
    /// ```
    pub fn push_slice(&self, samples: &[f32]) {
        self.shared.push_slice(samples);
    }

    /// Returns the number of samples currently available for reading.
    ///
    /// # Examples
    ///
    /// ```
    /// let (p, c) = rt_ring::new(4);
    /// assert_eq!(p.available(), 0);
    /// p.push(1.0);
    /// p.push(2.0);
    /// assert_eq!(p.available(), 2);
    /// c.pop();
    /// assert_eq!(p.available(), 1);
    /// ```
    pub fn available(&self) -> usize {
        self.shared.available()
    }

    /// Returns the buffer capacity (always a power of two).
    ///
    /// # Examples
    ///
    /// ```
    /// let (p, _c) = rt_ring::new(5);
    /// assert_eq!(p.capacity(), 8); // rounded up to next power of two
    /// ```
    pub fn capacity(&self) -> usize {
        self.shared.capacity()
    }

    /// Returns the total number of samples that were overwritten since creation.
    ///
    /// Equivalent to [`Consumer::overwrite_count`] — both read the same atomic counter.
    ///
    /// # Examples
    ///
    /// ```
    /// let (p, c) = rt_ring::new(4);
    /// for i in 0..10 {
    ///     p.push(i as f32);
    /// }
    /// assert_eq!(p.overwrite_count(), 6);
    /// assert_eq!(p.overwrite_count(), c.overwrite_count());
    /// ```
    pub fn overwrite_count(&self) -> u64 {
        self.shared.overwrite_count()
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
    ///
    /// # Examples
    ///
    /// ```
    /// let (p, c) = rt_ring::new(4);
    /// p.push_slice(&[10.0, 20.0, 30.0]);
    /// let mut buf = [0.0f32; 4];
    /// let n = c.pop_slice(&mut buf);
    /// assert_eq!(n, 3);
    /// assert_eq!(&buf[..n], &[10.0, 20.0, 30.0]);
    /// ```
    pub fn pop_slice(&self, buf: &mut [f32]) -> usize {
        self.shared.pop_slice(buf)
    }

    /// Returns the number of samples currently available for reading.
    ///
    /// # Examples
    ///
    /// ```
    /// let (p, c) = rt_ring::new(4);
    /// assert_eq!(c.available(), 0);
    /// p.push(1.0);
    /// assert_eq!(c.available(), 1);
    /// ```
    pub fn available(&self) -> usize {
        self.shared.available()
    }

    /// Returns the total number of samples that were overwritten since creation.
    ///
    /// # Examples
    ///
    /// ```
    /// let (p, c) = rt_ring::new(4);
    /// for i in 0..10 {
    ///     p.push(i as f32);
    /// }
    /// assert_eq!(c.overwrite_count(), 6);
    /// ```
    pub fn overwrite_count(&self) -> u64 {
        self.shared.overwrite_count()
    }

    /// Returns the buffer capacity (always a power of two).
    ///
    /// # Examples
    ///
    /// ```
    /// let (_p, c) = rt_ring::new(3);
    /// assert_eq!(c.capacity(), 4); // rounded up to next power of two
    /// ```
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
