//! The `engine` suite: one test per scenario in `features/suites/engine.feature`.
//!
//! The engine proposes; only the Lean kernel admits. These tests hold the
//! engine's own promises fixed: its variation is Rule 30 and never dies out,
//! its ranking is Hessian sorting, and a tick admits at most one safe, novel
//! rhythm while its Hebbian weights stay bounded. The engine's safety filter is
//! also held to the model's `rhythmSafe` text, so the two cannot drift apart.

use echobraid_engine::{
    calm_score, eigenvalues, frobenius, hessian, hessian_sort, plan, robust_score, step_rule30_u64,
    Driver, Rhythm, Strategy, Weights, CURVATURE_WEIGHT, RELAX_478, WEIGHT_RANGE,
};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/conformance is two below the root")
        .to_path_buf()
}

/// `EB-11`: The engine's Rule 30 automaton never reaches the all-zero state and is the 64-cell rule 30 step
#[test]
fn engine_eb_11() {
    // Rule 30 cell by cell: new = left XOR (centre OR right), on a ring.
    for seed in [1u64, 0x8000_0000_0000_0000, 0x4563_686f_4272_6169, u64::MAX] {
        let next = step_rule30_u64(seed);
        for i in 0..64i32 {
            let bit = |x: u64, k: i32| (x >> k.rem_euclid(64)) & 1;
            assert_eq!(
                bit(next, i),
                bit(seed, i + 1) ^ (bit(seed, i) | bit(seed, i - 1)),
                "cell {i} of {seed:#x}"
            );
        }
    }
    assert_ne!(
        step_rule30_u64(0),
        0,
        "the zero ring is replaced before it steps"
    );
    let mut driver = Driver {
        state: 0x4563_686f_4272_6169,
        generation: 0,
    };
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..10_000 {
        let word = driver.next_word();
        assert_ne!(word, 0, "the driver reached the all-zero ring");
        seen.insert(word);
    }
    assert!(
        seen.len() > 9_900,
        "the driver cycles too soon: {} distinct of 10000",
        seen.len()
    );
}

/// `EB-12`: Hessian sorting ranks every candidate rhythm by its calm score minus the curvature of the score around it, most robust first
#[test]
fn engine_eb_12() {
    let candidates: Vec<(Rhythm, Strategy)> = (2..=8)
        .flat_map(|i| {
            (0..=4).map(move |h| {
                (
                    Rhythm {
                        inhale: i,
                        hold: h,
                        exhale: 2 * i,
                        rest: 1,
                    },
                    Strategy::Direct,
                )
            })
        })
        .collect();
    let ranked = hessian_sort(&candidates);
    assert_eq!(ranked.len(), candidates.len());
    for pair in ranked.windows(2) {
        assert!(pair[0].robust >= pair[1].robust, "not most robust first");
    }
    for r in &ranked {
        let x = [
            r.rhythm.inhale,
            r.rhythm.hold,
            r.rhythm.exhale,
            r.rhythm.rest,
        ]
        .map(f64::from);
        let h = hessian(x);
        for (i, row) in h.iter().enumerate() {
            for (j, v) in row.iter().enumerate() {
                assert!((v - h[j][i]).abs() < 1e-9, "the Hessian is symmetric");
            }
        }
        assert!((r.score - calm_score(x)).abs() < 1e-12);
        assert!((r.curvature - frobenius(&h)).abs() < 1e-12);
        assert!((r.robust - (r.score - CURVATURE_WEIGHT * r.curvature)).abs() < 1e-12);
        assert!((r.robust - robust_score(&r.rhythm)).abs() < 1e-12);
        // The eigenvalues of a symmetric matrix sum to its trace.
        let trace: f64 = (0..4).map(|i| h[i][i]).sum();
        assert!(
            (eigenvalues(h).iter().sum::<f64>() - trace).abs() < 1e-6,
            "Jacobi preserved the trace"
        );
    }
    // A flat optimum beats a sharper one of equal calm: curvature is penalised.
    let flat = [1.0f64, 0.0, 0.0, 0.0];
    assert!(frobenius(&[[0.0; 4]; 4]) < frobenius(&[flat, flat, flat, flat]));
}

/// `EB-13`: An engine tick admits at most one new rhythm, only a safe and novel one, and its Hebbian strategy weights stay within 1 to 16
#[test]
fn engine_eb_13() {
    // The engine's filter is the model's rhythmSafe, clause for clause.
    let breath =
        std::fs::read_to_string(root().join("app/src/Braid/Breath.lex.tex")).expect("Breath model");
    for clause in [
        "(2 <=? (project rhythm inhale))",
        "((project rhythm inhale) <=? 8)",
        "((project rhythm hold) <=? 8)",
        "((project rhythm inhale) <=? (project rhythm exhale))",
        "((project rhythm exhale) <=? 10)",
        "((project rhythm rest) <=? 6)",
        "((cycleLength rhythm) <=? 30)",
    ] {
        assert!(breath.contains(clause), "rhythmSafe no longer has {clause}");
    }
    assert!(RELAX_478.safe());
    assert!(
        !Rhythm {
            inhale: 4,
            hold: 7,
            exhale: 3,
            rest: 0
        }
        .safe(),
        "an exhale shorter than the inhale is unsafe"
    );

    let mut driver = Driver {
        state: 0x4563_686f_4272_6169,
        generation: 0,
    };
    let mut weights = Weights::default();
    let mut frontier: Vec<Rhythm> = Vec::new();
    for _ in 0..40 {
        let before = frontier.len();
        let p = plan(driver, &weights, &frontier, &[]);
        if let Some(a) = &p.admit {
            assert!(a.rhythm.safe(), "admitted an unsafe rhythm");
            assert!(
                !frontier.contains(&a.rhythm) && a.rhythm != RELAX_478,
                "admitted a known rhythm"
            );
            frontier.push(a.rhythm);
        }
        assert!(
            frontier.len() <= before + 1,
            "more than one rhythm in a tick"
        );
        weights.learn(p.admit.as_ref().map(|a| a.strategy), &p.refused);
        for (_, w) in &weights.0 {
            assert!(
                (WEIGHT_RANGE.0..=WEIGHT_RANGE.1).contains(w),
                "weight {w} out of range"
            );
        }
        driver = p.driver;
    }
    assert!(!frontier.is_empty(), "forty ticks discovered nothing");
}
