use std::collections::VecDeque;

#[test]
fn producer_overwrite_count_matches_consumer() {
    let (p, c) = rt_ring::new(4);
    for i in 0..10 {
        p.push(i as f32);
    }
    assert_eq!(p.overwrite_count(), 6);
    assert_eq!(p.overwrite_count(), c.overwrite_count());
}

#[test]
fn overwrite_one_past_capacity() {
    // capacity = 4
    let (p, c) = rt_ring::new(4);
    // Fill it
    for i in 0..4 {
        p.push(i as f32);
    }
    // Push one more — oldest (0.0) should be dropped
    p.push(100.0);

    assert_eq!(c.overwrite_count(), 1);
    assert_eq!(c.pop(), Some(1.0));
    assert_eq!(c.pop(), Some(2.0));
    assert_eq!(c.pop(), Some(3.0));
    assert_eq!(c.pop(), Some(100.0));
    assert_eq!(c.pop(), None);
}

#[test]
fn overwrite_entire_buffer() {
    let cap = 4;
    let (p, c) = rt_ring::new(cap);

    // Push cap values
    for i in 0..cap {
        p.push(i as f32);
    }
    // Push cap more — all originals should be gone
    for i in 0..cap {
        p.push((i + cap) as f32);
    }

    assert_eq!(c.overwrite_count(), cap as u64);
    // Only the last `cap` values remain
    for i in 0..cap {
        assert_eq!(c.pop(), Some((i + cap) as f32));
    }
    assert_eq!(c.pop(), None);
}

#[test]
fn partial_drain_then_overflow() {
    let (p, c) = rt_ring::new(4);

    for i in 0..4 {
        p.push(i as f32);
    }
    assert_eq!(c.pop(), Some(0.0));
    assert_eq!(c.pop(), Some(1.0));

    // 2 items left (2.0, 3.0). Push 4 more.
    p.push(10.0);
    p.push(11.0);
    // Now full (4 items: 2,3,10,11)
    p.push(12.0);
    // Overwrite: oldest (2.0) dropped
    p.push(13.0);
    // Overwrite: oldest (3.0) dropped

    assert_eq!(c.overwrite_count(), 2);

    assert_eq!(c.pop(), Some(10.0));
    assert_eq!(c.pop(), Some(11.0));
    assert_eq!(c.pop(), Some(12.0));
    assert_eq!(c.pop(), Some(13.0));
    assert_eq!(c.pop(), None);
}

#[test]
fn order_preserved_after_overwrites() {
    let (p, c) = rt_ring::new(4);

    // Push 10 values (0..10), buffer only holds last 4
    for i in 0..10 {
        p.push(i as f32);
    }

    assert_eq!(c.overwrite_count(), 6);

    let mut out = Vec::new();
    while let Some(v) = c.pop() {
        out.push(v);
    }
    assert_eq!(out, vec![6.0, 7.0, 8.0, 9.0]);
}

#[test]
fn overwrite_capacity_one() {
    let (p, c) = rt_ring::new(1);
    assert_eq!(p.capacity(), 1);

    p.push(1.0);
    p.push(2.0);
    p.push(3.0);

    assert_eq!(c.overwrite_count(), 2);
    assert_eq!(c.pop(), Some(3.0));
    assert_eq!(c.pop(), None);
}

#[test]
fn long_run_overwrite_accounting_and_tail_integrity() {
    let cap = 8usize;
    let total = 2_000usize;
    let (p, c) = rt_ring::new(cap);

    for i in 0..total {
        p.push(i as f32);
    }

    assert_eq!(c.overwrite_count(), (total - cap) as u64);
    assert_eq!(c.available(), cap);

    for expected in (total - cap)..total {
        assert_eq!(c.pop(), Some(expected as f32));
    }
    assert_eq!(c.pop(), None);
}

#[test]
fn mixed_push_slice_pop_slice_matches_reference_and_overwrite_count() {
    fn apply_push_slice(
        p: &rt_ring::Producer,
        cap: usize,
        reference: &mut VecDeque<f32>,
        expected_overwrites: &mut u64,
        values: &[f32],
    ) {
        p.push_slice(values);
        for &v in values {
            reference.push_back(v);
            if reference.len() > cap {
                reference.pop_front();
                *expected_overwrites += 1;
            }
        }
    }

    let requested_cap = 8usize;
    let (p, c) = rt_ring::new(requested_cap);
    let cap = p.capacity();

    let mut reference: VecDeque<f32> = VecDeque::new();
    let mut expected_overwrites = 0u64;

    apply_push_slice(
        &p,
        cap,
        &mut reference,
        &mut expected_overwrites,
        &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0],
    );

    let mut tmp = [0.0f32; 3];
    let n = c.pop_slice(&mut tmp);
    assert_eq!(n, 3);
    for got in tmp.into_iter().take(n) {
        let expected = reference.pop_front().unwrap();
        assert_eq!(got.to_bits(), expected.to_bits());
    }

    apply_push_slice(
        &p,
        cap,
        &mut reference,
        &mut expected_overwrites,
        &[6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0],
    );

    let mut tmp = [0.0f32; 2];
    let n = c.pop_slice(&mut tmp);
    assert_eq!(n, 2);
    for got in tmp.into_iter().take(n) {
        let expected = reference.pop_front().unwrap();
        assert_eq!(got.to_bits(), expected.to_bits());
    }

    apply_push_slice(
        &p,
        cap,
        &mut reference,
        &mut expected_overwrites,
        &[14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0, 21.0],
    );

    let mut drained = Vec::new();
    while let Some(v) = c.pop() {
        drained.push(v);
    }
    let expected: Vec<f32> = reference.into_iter().collect();

    assert_eq!(drained.len(), expected.len());
    for (got, exp) in drained.iter().zip(expected.iter()) {
        assert_eq!(got.to_bits(), exp.to_bits());
    }
    assert_eq!(c.overwrite_count(), expected_overwrites);
}
