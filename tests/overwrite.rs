use rt_ring;

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
