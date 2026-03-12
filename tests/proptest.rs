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
    #![proptest_config({
        let mut cfg = ProptestConfig::default();

        // Miri runs tests under "isolation" where `getcwd` is not available.
        // Proptest's default failure persistence uses `current_dir()` to locate/write
        // its regressions file, so disable persistence only under Miri.
        if cfg!(miri) {
            cfg.failure_persistence = None;
        }

        // Miri is *very* slow; keep CI times reasonable.
        cfg.cases = if cfg!(miri) { 20 } else { 10_000 };

        cfg
    })]

    #[test]
    fn matches_reference_model(
        capacity in 1usize..=64,
        ops in prop::collection::vec(op_strategy(), 0..=if cfg!(miri) { 50 } else { 500 }),
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

    #[test]
    fn tracks_available_and_overwrites_correctly(
        capacity in 1usize..=64,
        ops in prop::collection::vec(op_strategy(), 0..=if cfg!(miri) { 50 } else { 500 }),
    ) {
        let (p, c) = rt_ring::new(capacity);
        let actual_cap = p.capacity();
        let mut reference: VecDeque<f32> = VecDeque::new();
        let mut expected_overwrites = 0u64;

        for op in ops {
            match op {
                Op::Push(val) => {
                    if reference.len() == actual_cap {
                        expected_overwrites += 1;
                        reference.pop_front();
                    }
                    reference.push_back(val);
                    p.push(val);
                }
                Op::Pop => {
                    let got = c.pop();
                    let expected = reference.pop_front();
                    match (got, expected) {
                        (Some(g), Some(e)) => prop_assert_eq!(g.to_bits(), e.to_bits()),
                        (None, None) => {}
                        (g, e) => prop_assert!(false, "pop mismatch: got {:?}, expected {:?}", g, e),
                    }
                }
            }

            prop_assert_eq!(c.available(), reference.len());
            prop_assert_eq!(p.available(), reference.len());
            prop_assert_eq!(c.overwrite_count(), expected_overwrites);
        }
    }
}
