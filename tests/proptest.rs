use proptest::prelude::*;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
enum Op {
    Push(f32),
    Pop,
}

fn op_strategy() -> impl Strategy<Value = Op> {
    prop_oneof![any::<f32>().prop_map(Op::Push), Just(Op::Pop),]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn matches_reference_model(
        capacity in 1usize..=64,
        ops in prop::collection::vec(op_strategy(), 0..=500),
    ) {
        let (p, c) = rt_ring::new(capacity);
        let actual_cap = p.capacity();
        let mut reference: VecDeque<f32> = VecDeque::new();

        for op in ops {
            match op {
                Op::Push(val) => {
                    p.push(val);
                    reference.push_back(val);
                    while reference.len() > actual_cap {
                        reference.pop_front();
                    }
                }
                Op::Pop => {
                    let got = c.pop();
                    let expected = reference.pop_front();
                    match (got, expected) {
                        (Some(g), Some(e)) => {
                            // Compare via to_bits to handle NaN correctly
                            prop_assert_eq!(
                                g.to_bits(), e.to_bits(),
                                "mismatch: got bits {:08x}, expected bits {:08x}",
                                g.to_bits(), e.to_bits()
                            );
                        }
                        (None, None) => {}
                        (g, e) => {
                            prop_assert!(false, "pop mismatch: got {:?}, expected {:?}", g, e);
                        }
                    }
                }
            }
        }
    }
}
