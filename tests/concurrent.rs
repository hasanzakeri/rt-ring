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
                    if popped.is_multiple_of(10) {
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

#[test]
fn steady_state_no_overwrite_when_consumer_keeps_up() {
    use std::sync::mpsc;

    let count: usize = if cfg!(miri) { 200 } else { 20_000 };
    let (p, c) = rt_ring::new(128);

    let (tx_ready, rx_ready) = mpsc::channel::<usize>();
    let (tx_ack, rx_ack) = mpsc::channel::<()>();

    let producer = thread::spawn(move || {
        for i in 0..count {
            p.push(i as f32);
            tx_ready.send(i).unwrap();
            rx_ack.recv().unwrap();
        }
    });

    let consumer = thread::spawn(move || {
        for expected in 0..count {
            let produced = rx_ready.recv().unwrap();
            assert_eq!(produced, expected);

            let v = loop {
                if let Some(v) = c.pop() {
                    break v;
                }
                thread::yield_now();
            };

            assert_eq!(v as usize, expected);
            tx_ack.send(()).unwrap();
        }

        assert_eq!(c.overwrite_count(), 0, "no overwrites expected in lockstep");
        assert_eq!(c.available(), 0, "buffer should be drained");
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}

#[test]
fn available_never_exceeds_capacity_under_pressure() {
    let cap: usize = 32;
    let total: usize = if cfg!(miri) { 3_000 } else { 300_000 };
    let (p, c) = rt_ring::new(cap);
    let actual_cap = p.capacity();

    let barrier = Arc::new(Barrier::new(2));

    let b = Arc::clone(&barrier);
    let producer = thread::spawn(move || {
        b.wait();
        for i in 0..total {
            p.push(i as f32);
        }
    });

    let b = Arc::clone(&barrier);
    let consumer = thread::spawn(move || {
        b.wait();
        let mut last: Option<usize> = None;
        let mut popped = 0usize;

        loop {
            match c.pop() {
                Some(v) => {
                    let val = v as usize;
                    if let Some(prev) = last {
                        assert!(val > prev, "non-monotonic: prev={prev}, got={val}");
                    }
                    last = Some(val);
                    popped += 1;

                    let avail = c.available();
                    assert!(
                        avail <= actual_cap,
                        "available {avail} exceeded capacity {actual_cap}"
                    );

                    if popped.is_multiple_of(4) {
                        thread::yield_now();
                    }
                }
                None => {
                    let avail = c.available();
                    assert!(
                        avail <= actual_cap,
                        "available {avail} exceeded capacity {actual_cap}"
                    );
                    if last == Some(total - 1) {
                        break;
                    }
                    thread::yield_now();
                }
            }
        }

        let overwrites = c.overwrite_count() as usize;
        assert!(overwrites > 0, "pressure test should produce overwrites");
        assert_eq!(overwrites + popped, total);
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
