use std::sync::{Arc, Barrier};
use std::thread;

fn iteration_count() -> u64 {
    if cfg!(miri) {
        1_000
    } else {
        10_000_000
    }
}

#[test]
fn monotonic_values_cross_thread() {
    let count = iteration_count();
    let (p, c) = rt_ring::new(1024);

    let barrier = Arc::new(Barrier::new(2));

    let b = Arc::clone(&barrier);
    let producer = thread::spawn(move || {
        b.wait();
        for i in 0..count {
            p.push(i as f32);
        }
    });

    let b = Arc::clone(&barrier);
    let consumer = thread::spawn(move || {
        b.wait();
        let mut last: Option<u64> = None;
        let mut popped = 0u64;

        loop {
            match c.pop() {
                Some(v) => {
                    let val = v as u64;
                    if let Some(prev) = last {
                        assert!(val > prev, "non-monotonic: prev={prev}, got={val}");
                    }
                    last = Some(val);
                    popped += 1;
                }
                None => {
                    if let Some(prev) = last {
                        if prev == count - 1 {
                            break;
                        }
                    }
                    thread::yield_now();
                }
            }
        }

        let overwrites = c.overwrite_count();
        assert_eq!(
            overwrites + popped,
            count,
            "overwrites({overwrites}) + popped({popped}) != pushed({count})"
        );
        popped
    });

    producer.join().unwrap();
    let popped = consumer.join().unwrap();
    assert!(
        popped > 0,
        "consumer should have popped at least some values"
    );
}

#[test]
fn slow_consumer_forces_overwrites() {
    let count = if cfg!(miri) { 100 } else { 100_000 };
    let (p, c) = rt_ring::new(64);

    let barrier = Arc::new(Barrier::new(2));

    let b = Arc::clone(&barrier);
    let producer = thread::spawn(move || {
        b.wait();
        for i in 0..count {
            p.push(i as f32);
        }
    });

    let b = Arc::clone(&barrier);
    let consumer = thread::spawn(move || {
        b.wait();
        let mut last: Option<u64> = None;
        let mut popped = 0u64;
        let mut done = false;

        while !done {
            match c.pop() {
                Some(v) => {
                    let val = v as u64;
                    if let Some(prev) = last {
                        assert!(val > prev, "non-monotonic: prev={prev}, got={val}");
                    }
                    last = Some(val);
                    popped += 1;

                    // Slow down every 10 pops
                    if popped % 10 == 0 {
                        thread::yield_now();
                        thread::yield_now();
                    }
                }
                None => {
                    if last == Some((count - 1) as u64) {
                        done = true;
                    } else {
                        thread::yield_now();
                    }
                }
            }
        }

        let overwrites = c.overwrite_count();
        assert_eq!(overwrites + popped, count as u64);
        assert!(overwrites > 0, "slow consumer should cause some overwrites");
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}

#[test]
fn bursty_producer() {
    let total: usize = if cfg!(miri) { 500 } else { 1_000_000 };
    let (p, c) = rt_ring::new(256);

    let barrier = Arc::new(Barrier::new(2));

    let b = Arc::clone(&barrier);
    let producer = thread::spawn(move || {
        b.wait();
        let mut sent = 0usize;
        let mut burst_size = 1;
        while sent < total {
            let end = (sent + burst_size).min(total);
            let burst: Vec<f32> = (sent..end).map(|i| i as f32).collect();
            p.push_slice(&burst);
            sent = end;
            burst_size = (burst_size % 50) + 1;
        }
    });

    let b = Arc::clone(&barrier);
    let consumer = thread::spawn(move || {
        b.wait();
        let mut last: Option<u64> = None;
        let mut popped = 0u64;

        loop {
            match c.pop() {
                Some(v) => {
                    let val = v as u64;
                    if let Some(prev) = last {
                        assert!(val > prev, "non-monotonic: prev={prev}, got={val}");
                    }
                    last = Some(val);
                    popped += 1;
                }
                None => {
                    if last == Some((total - 1) as u64) {
                        break;
                    }
                    thread::yield_now();
                }
            }
        }

        let overwrites = c.overwrite_count();
        assert_eq!(overwrites + popped, total as u64);
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
