use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};

pub(crate) struct Shared {
    data: Box<[AtomicU32]>,
    capacity: usize,
    mask: usize,
    write_pos: AtomicUsize,
    read_pos: AtomicUsize,
    overwrite_count: AtomicU64,
}

impl Shared {
    pub(crate) fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "capacity must be greater than zero");
        let capacity = capacity.next_power_of_two();
        let mask = capacity - 1;
        let data: Vec<AtomicU32> = (0..capacity).map(|_| AtomicU32::new(0)).collect();
        Self {
            data: data.into_boxed_slice(),
            capacity,
            mask,
            write_pos: AtomicUsize::new(0),
            read_pos: AtomicUsize::new(0),
            overwrite_count: AtomicU64::new(0),
        }
    }

    pub(crate) fn push(&self, sample: f32) {
        let wp = self.write_pos.load(Ordering::Relaxed);
        loop {
            let rp = self.read_pos.load(Ordering::Acquire);
            if wp - rp < self.capacity {
                break;
            }
            // Buffer full — try to advance read_pos to discard oldest.
            // CAS avoids moving read_pos backwards if the consumer already advanced it.
            if self
                .read_pos
                .compare_exchange_weak(rp, rp + 1, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                self.overwrite_count.fetch_add(1, Ordering::Relaxed);
                break;
            }
            // CAS failed: consumer popped (freeing space) or raced. Re-check.
        }

        self.data[wp & self.mask].store(sample.to_bits(), Ordering::Release);
        self.write_pos.store(wp + 1, Ordering::Release);
    }

    pub(crate) fn push_slice(&self, samples: &[f32]) {
        for &sample in samples {
            self.push(sample);
        }
    }

    pub(crate) fn pop(&self) -> Option<f32> {
        loop {
            let rp = self.read_pos.load(Ordering::Acquire);
            let wp = self.write_pos.load(Ordering::Acquire);

            if rp >= wp {
                return None;
            }

            let bits = self.data[rp & self.mask].load(Ordering::Acquire);
            // CAS to advance read_pos. If producer overwrote and moved read_pos
            // past rp, CAS fails and we retry from the new position.
            if self
                .read_pos
                .compare_exchange_weak(rp, rp + 1, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                return Some(f32::from_bits(bits));
            }
        }
    }

    pub(crate) fn pop_slice(&self, buf: &mut [f32]) -> usize {
        let mut count = 0;
        for slot in buf.iter_mut() {
            match self.pop() {
                Some(v) => {
                    *slot = v;
                    count += 1;
                }
                None => break,
            }
        }
        count
    }

    pub(crate) fn available(&self) -> usize {
        let wp = self.write_pos.load(Ordering::Acquire);
        let rp = self.read_pos.load(Ordering::Acquire);
        wp.saturating_sub(rp)
    }

    pub(crate) fn overwrite_count(&self) -> u64 {
        self.overwrite_count.load(Ordering::Relaxed)
    }

    pub(crate) fn capacity(&self) -> usize {
        self.capacity
    }
}
