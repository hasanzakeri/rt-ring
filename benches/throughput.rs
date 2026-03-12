use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::{Arc, Barrier};
use std::thread;

const N: usize = 1_000_000;

fn bench_single_thread_push(c: &mut Criterion) {
    c.bench_function("rt-ring push 1M", |b| {
        b.iter(|| {
            let (p, _c) = rt_ring::new(1024);
            for i in 0..N {
                p.push(black_box(i as f32));
            }
        });
    });
}

fn bench_single_thread_push_pop(c: &mut Criterion) {
    c.bench_function("rt-ring push+pop 1M", |b| {
        b.iter(|| {
            let (p, consumer) = rt_ring::new(1024);
            for i in 0..N {
                p.push(i as f32);
                black_box(consumer.pop());
            }
        });
    });
}

fn bench_cross_thread(c: &mut Criterion) {
    c.bench_function("rt-ring cross-thread 1M", |b| {
        b.iter(|| {
            let (p, consumer) = rt_ring::new(1024);
            let barrier = Arc::new(Barrier::new(2));

            let b1 = Arc::clone(&barrier);
            let producer = thread::spawn(move || {
                b1.wait();
                for i in 0..N {
                    p.push(i as f32);
                }
            });

            let b2 = Arc::clone(&barrier);
            let cons = thread::spawn(move || {
                b2.wait();
                let mut count = 0usize;
                loop {
                    if consumer.pop().is_some() {
                        count += 1;
                    } else if count > 0 && consumer.available() == 0 {
                        // Try once more to avoid race
                        if consumer.pop().is_none() {
                            break;
                        }
                        count += 1;
                    }
                }
                count
            });

            producer.join().unwrap();
            let popped = cons.join().unwrap();
            black_box(popped);
        });
    });
}

fn bench_ringbuf_comparison(c: &mut Criterion) {
    use ringbuf::traits::{Consumer as _, Producer as _, Split};
    use ringbuf::HeapRb;

    c.bench_function("ringbuf push+pop 1M", |b| {
        b.iter(|| {
            let rb = HeapRb::<f32>::new(1024);
            let (mut prod, mut cons) = rb.split();
            for i in 0..N {
                let _ = prod.try_push(i as f32);
                black_box(cons.try_pop());
            }
        });
    });

    c.bench_function("ringbuf cross-thread 1M", |b| {
        b.iter(|| {
            let rb = HeapRb::<f32>::new(1024);
            let (mut prod, mut cons) = rb.split();
            let barrier = Arc::new(Barrier::new(2));

            let b1 = Arc::clone(&barrier);
            let producer = thread::spawn(move || {
                b1.wait();
                for i in 0..N {
                    let _ = prod.try_push(i as f32);
                }
            });

            let b2 = Arc::clone(&barrier);
            let consumer = thread::spawn(move || {
                b2.wait();
                let mut count = 0usize;
                loop {
                    if cons.try_pop().is_some() {
                        count += 1;
                    } else if count > 0 {
                        if cons.try_pop().is_none() {
                            break;
                        }
                        count += 1;
                    }
                }
                count
            });

            producer.join().unwrap();
            let popped = consumer.join().unwrap();
            black_box(popped);
        });
    });
}

criterion_group!(
    benches,
    bench_single_thread_push,
    bench_single_thread_push_pop,
    bench_cross_thread,
    bench_ringbuf_comparison,
);
criterion_main!(benches);
