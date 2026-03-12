#[test]
fn empty_buffer_returns_none() {
    let (_p, c) = rt_ring::new(4);
    assert_eq!(c.pop(), None);
}

#[test]
fn push_pop_single() {
    let (p, c) = rt_ring::new(4);
    p.push(1.0);
    assert_eq!(c.pop(), Some(1.0));
    assert_eq!(c.pop(), None);
}

#[test]
fn push_pop_fifo_order() {
    let (p, c) = rt_ring::new(8);
    for i in 0..5 {
        p.push(i as f32);
    }
    for i in 0..5 {
        assert_eq!(c.pop(), Some(i as f32));
    }
    assert_eq!(c.pop(), None);
}

#[test]
fn capacity_rounds_to_power_of_two() {
    let (p, _c) = rt_ring::new(3);
    assert_eq!(p.capacity(), 4);

    let (p, _c) = rt_ring::new(5);
    assert_eq!(p.capacity(), 8);

    let (p, _c) = rt_ring::new(8);
    assert_eq!(p.capacity(), 8);

    let (p, _c) = rt_ring::new(1);
    assert_eq!(p.capacity(), 1);
}

#[test]
fn push_slice_pop_slice() {
    let (p, c) = rt_ring::new(8);
    let input = [1.0, 2.0, 3.0, 4.0, 5.0];
    p.push_slice(&input);

    let mut output = [0.0f32; 8];
    let n = c.pop_slice(&mut output);
    assert_eq!(n, 5);
    assert_eq!(&output[..5], &input);
}

#[test]
fn pop_slice_partial() {
    let (p, c) = rt_ring::new(8);
    p.push_slice(&[1.0, 2.0, 3.0]);

    let mut buf = [0.0f32; 2];
    let n = c.pop_slice(&mut buf);
    assert_eq!(n, 2);
    assert_eq!(buf, [1.0, 2.0]);

    assert_eq!(c.pop(), Some(3.0));
    assert_eq!(c.pop(), None);
}

#[test]
fn available_tracking() {
    let (p, c) = rt_ring::new(8);
    assert_eq!(c.available(), 0);
    assert_eq!(p.available(), 0);

    p.push(1.0);
    p.push(2.0);
    assert_eq!(c.available(), 2);

    c.pop();
    assert_eq!(c.available(), 1);

    c.pop();
    assert_eq!(c.available(), 0);
}

#[test]
#[should_panic(expected = "capacity must be greater than zero")]
fn zero_capacity_panics() {
    rt_ring::new(0);
}

#[test]
fn producer_consumer_are_send() {
    fn assert_send<T: Send>() {}
    assert_send::<rt_ring::Producer>();
    assert_send::<rt_ring::Consumer>();
}

// Clone is intentionally not implemented on Producer/Consumer.
// This is enforced at compile time by the absence of #[derive(Clone)].
// The static assertions in lib.rs verify Send.
