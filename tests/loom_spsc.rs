#![cfg(feature = "loom-tests")]

use loom::model::Builder;
use loom::thread;

fn assert_monotonic(vals: &[usize]) {
    for w in vals.windows(2) {
        assert!(w[1] > w[0], "non-monotonic sequence: {:?}", vals);
    }
}

#[test]
fn loom_small_interleavings_accounting_and_order() {
    let mut builder = Builder::new();
    builder.max_permutations = Some(5_000);

    builder.check(|| {
        let capacity = 4usize;
        let pushed = 6usize;
        let (p, c) = rt_ring::new(capacity);

        let producer = thread::spawn(move || {
            for i in 0..pushed {
                p.push(i as f32);
                thread::yield_now();
            }
        });

        let consumer = thread::spawn(move || {
            let mut out = Vec::new();
            for _ in 0..pushed {
                if let Some(v) = c.pop() {
                    out.push(v as usize);
                }
                thread::yield_now();
            }
            (c, out)
        });

        producer.join().unwrap();
        let (c, mut out) = consumer.join().unwrap();

        while let Some(v) = c.pop() {
            out.push(v as usize);
        }

        assert_monotonic(&out);
        let overwrites = c.overwrite_count() as usize;
        assert_eq!(
            out.len() + overwrites,
            pushed,
            "accounting mismatch: popped={} overwrites={} pushed={}",
            out.len(),
            overwrites,
            pushed
        );
        assert!(c.available() <= c.capacity());
    });
}

#[test]
fn loom_capacity_one_keeps_latest_value() {
    let mut builder = Builder::new();
    builder.max_permutations = Some(2_000);

    builder.check(|| {
        let (p, c) = rt_ring::new(1);

        let producer = thread::spawn(move || {
            p.push(10.0);
            thread::yield_now();
            p.push(11.0);
            thread::yield_now();
            p.push(12.0);
        });

        let consumer = thread::spawn(move || {
            let mut seen = Vec::new();
            for _ in 0..3 {
                if let Some(v) = c.pop() {
                    seen.push(v as usize);
                }
                thread::yield_now();
            }
            (c, seen)
        });

        producer.join().unwrap();
        let (c, mut seen) = consumer.join().unwrap();
        while let Some(v) = c.pop() {
            seen.push(v as usize);
        }

        assert_monotonic(&seen);
        let overwrites = c.overwrite_count() as usize;
        assert_eq!(seen.len() + overwrites, 3);
        assert!(seen.last().copied().unwrap_or(12) <= 12);
    });
}
