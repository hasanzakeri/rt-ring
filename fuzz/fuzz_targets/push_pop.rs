#![no_main]
use libfuzzer_sys::fuzz_target;
use std::collections::VecDeque;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    // First byte determines capacity (1..=64)
    let capacity = (data[0] as usize % 64) + 1;
    let (p, c) = rt_ring::new(capacity);
    let actual_cap = p.capacity();
    let mut reference: VecDeque<f32> = VecDeque::new();

    // Remaining bytes are operations: even byte = push(next byte as f32), odd byte = pop
    let mut i = 1;
    while i < data.len() {
        let op_byte = data[i];
        if op_byte % 2 == 0 {
            // Push operation — use next byte as the value
            i += 1;
            let val = if i < data.len() {
                data[i] as f32
            } else {
                0.0
            };
            p.push(val);
            reference.push_back(val);
            while reference.len() > actual_cap {
                reference.pop_front();
            }
        } else {
            // Pop operation
            let got = c.pop();
            let expected = reference.pop_front();
            match (got, expected) {
                (Some(g), Some(e)) => {
                    assert_eq!(g.to_bits(), e.to_bits(), "value mismatch");
                }
                (None, None) => {}
                _ => panic!("pop presence mismatch"),
            }
        }
        i += 1;
    }
});
