//! The EchoBraid self-improvement engine.
//!
//! One engine tick is one attempt to discover, prove and ship a new mechanism.
//! The design follows `Casey-allard/lexlean-r4`'s engine (itself modeled on
//! `afflom/rule30`): a Rule 30 automaton is the only source of variation,
//! learned Hebbian weights decide which strategy proposes next, and nothing is
//! admitted that the Lean kernel has not proved. EchoBraid's frontier is
//! breathing rhythms (the regulation mechanism its breath overlay runs); each
//! admitted rhythm becomes a row in `frontier.toml`, a record in
//! `app/src/Braid/Frontier.lex.tex` that the guest serves, and an acceptance
//! vector the holo replays.
//!
//! The tick, in order:
//!
//! 1. **Abduction.** The Rule 30 driver proposes [`CANDIDATES`] rhythms, each
//!    by a strategy scheduled by its Hebbian weight (heaviest first).
//! 2. **Hessian sorting.** Every candidate is scored by [`calm_score`] and
//!    ranked by [`robust_score`]: the score minus the Frobenius norm of the
//!    score's Hessian at the candidate, so a rhythm that stays calm when a
//!    person's timing slips by a tick outranks a sharper optimum.
//! 3. **Admission.** The best-ranked candidate that is safe ([`Rhythm::safe`],
//!    the mirror of the model's `rhythmSafe`), novel, and above
//!    [`ADMISSION_FLOOR`] is the one candidate the tick may admit.
//! 4. **Deduction.** The caller writes it into the model and has the Lean
//!    kernel prove `frontierSafe`; only a proved candidate stays.
//! 5. **Learning.** [`Weights::learn`] moves the strategies' weights by the
//!    outcome, clamped to `[1, 16]`.
//!
//! Everything here is pure and deterministic: the same state gives the same
//! tick, byte for byte, on any machine.

/// The Rule 30 step on a 64-cell ring held in one word, as lexlean-r4's
/// `step_rule30_u64`. A zero ring would stay zero forever, so zero is first
/// replaced by the golden-ratio constant.
pub fn step_rule30_u64(state: u64) -> u64 {
    let s = if state == 0 {
        0x9e37_79b9_7f4a_7c15
    } else {
        state
    };
    let left = s.rotate_right(1);
    let right = s.rotate_left(1);
    left ^ (s | right)
}

/// The engine's deterministic source of variation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Driver {
    /// The current 64-cell ring.
    pub state: u64,
    /// How many words the driver has produced since genesis.
    pub generation: u64,
}

impl Driver {
    /// Produces the next word: one Rule 30 step of the ring mixed with its
    /// generation, so two drivers on the same ring diverge by generation.
    pub fn next_word(&mut self) -> u64 {
        let mixed = self.state ^ self.generation.wrapping_mul(0x517c_c1b7_2722_0a95);
        self.state = step_rule30_u64(mixed);
        if self.state == 0 {
            self.state = step_rule30_u64(0);
        }
        self.generation += 1;
        self.state
    }
}

/// A breathing rhythm, in ticks of one second.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rhythm {
    /// Seconds breathing in.
    pub inhale: u32,
    /// Seconds holding.
    pub hold: u32,
    /// Seconds breathing out.
    pub exhale: u32,
    /// Seconds resting before the next breath.
    pub rest: u32,
}

/// The rhythm every EchoBraid session starts with.
pub const RELAX_478: Rhythm = Rhythm {
    inhale: 4,
    hold: 7,
    exhale: 8,
    rest: 0,
};

impl Rhythm {
    /// The number of ticks in one cycle.
    pub fn cycle(&self) -> u32 {
        self.inhale + self.hold + self.exhale + self.rest
    }

    /// The model's `rhythmSafe`, line for line: an inhale of 2 to 8 ticks, a
    /// hold of at most 8, an exhale at least as long as the inhale and at most
    /// 10 (a longer out-breath calms), a rest of at most 6, and a cycle of at
    /// most 30. The engine filters by it; the kernel decides it.
    pub fn safe(&self) -> bool {
        (2..=8).contains(&self.inhale)
            && self.hold <= 8
            && self.inhale <= self.exhale
            && self.exhale <= 10
            && self.rest <= 6
            && self.cycle() <= 30
    }

    /// The content address of the rhythm in the frontier.
    pub fn kappa(&self) -> String {
        format!(
            "eb:v1:breath:{}-{}-{}-{}",
            self.inhale, self.hold, self.exhale, self.rest
        )
    }

    /// The rhythm as the four numbers a person reads.
    pub fn label(&self) -> String {
        format!(
            "{}-{}-{}-{}",
            self.inhale, self.hold, self.exhale, self.rest
        )
    }

    fn as_point(&self) -> [f64; 4] {
        [self.inhale, self.hold, self.exhale, self.rest].map(f64::from)
    }
}

/// How calm a rhythm is expected to feel, as a smooth function of its four
/// phase lengths (higher is calmer). The terms follow the regulation guidance
/// the breath overlay is built on: an out-breath about twice the in-breath, a
/// cycle near 11 seconds (about five and a half breaths a minute, the
/// resonance band), a hold near three quarters of the in-breath, and little
/// idle rest.
pub fn calm_score(x: [f64; 4]) -> f64 {
    let [inhale, hold, exhale, rest] = x;
    let inhale = inhale.max(0.5);
    let ratio = exhale / inhale - 2.0;
    let pace = (inhale + hold + exhale + rest - 11.0) / 4.0;
    let holding = hold / inhale - 0.75;
    -(ratio * ratio) - pace * pace - 0.5 * holding * holding - 0.05 * rest * rest
}

/// The Hessian of [`calm_score`] at `x`, by central differences with a step of
/// half a tick: the curvature a person meets when their timing slips.
pub fn hessian(x: [f64; 4]) -> [[f64; 4]; 4] {
    const H: f64 = 0.5;
    let at = |dx: [f64; 4]| calm_score([x[0] + dx[0], x[1] + dx[1], x[2] + dx[2], x[3] + dx[3]]);
    let unit = |i: usize, s: f64| {
        let mut d = [0.0; 4];
        d[i] = s;
        d
    };
    let mut out = [[0.0; 4]; 4];
    for (i, row) in out.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = if i == j {
                (at(unit(i, H)) - 2.0 * at([0.0; 4]) + at(unit(i, -H))) / (H * H)
            } else {
                let pp = at({
                    let mut d = unit(i, H);
                    d[j] = H;
                    d
                });
                let pm = at({
                    let mut d = unit(i, H);
                    d[j] = -H;
                    d
                });
                let mp = at({
                    let mut d = unit(i, -H);
                    d[j] = H;
                    d
                });
                let mm = at({
                    let mut d = unit(i, -H);
                    d[j] = -H;
                    d
                });
                (pp - pm - mp + mm) / (4.0 * H * H)
            };
        }
    }
    out
}

/// The Frobenius norm of a 4 by 4 matrix: the total curvature.
pub fn frobenius(m: &[[f64; 4]; 4]) -> f64 {
    m.iter().flatten().map(|v| v * v).sum::<f64>().sqrt()
}

/// The eigenvalues of a symmetric 4 by 4 matrix by cyclic Jacobi rotation,
/// in descending order. The largest magnitude is the sharpest direction.
pub fn eigenvalues(mut a: [[f64; 4]; 4]) -> [f64; 4] {
    for _ in 0..64 {
        let mut off = 0.0;
        for (i, row) in a.iter().enumerate() {
            for (j, v) in row.iter().enumerate() {
                if i != j {
                    off += v * v;
                }
            }
        }
        if off < 1e-18 {
            break;
        }
        for p in 0..4 {
            for q in (p + 1)..4 {
                if a[p][q].abs() < 1e-15 {
                    continue;
                }
                let theta = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
                let t = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
                let t = if theta == 0.0 { 1.0 } else { t };
                let c = 1.0 / (t * t + 1.0).sqrt();
                let s = t * c;
                for row in a.iter_mut() {
                    let (akp, akq) = (row[p], row[q]);
                    row[p] = c * akp - s * akq;
                    row[q] = s * akp + c * akq;
                }
                let (rp, rq) = (a[p], a[q]);
                for (k, (apk, aqk)) in rp.iter().zip(rq.iter()).enumerate() {
                    a[p][k] = c * apk - s * aqk;
                    a[q][k] = s * apk + c * aqk;
                }
            }
        }
    }
    let mut out = [a[0][0], a[1][1], a[2][2], a[3][3]];
    out.sort_by(|x, y| y.total_cmp(x));
    out
}

/// The weight of curvature against calm in the ranking.
pub const CURVATURE_WEIGHT: f64 = 0.25;

/// The calm score minus the weighted curvature: the quantity Hessian sorting
/// ranks by.
pub fn robust_score(rhythm: &Rhythm) -> f64 {
    let x = rhythm.as_point();
    calm_score(x) - CURVATURE_WEIGHT * frobenius(&hessian(x))
}

/// A candidate after Hessian sorting.
#[derive(Clone, Debug, PartialEq)]
pub struct Ranked {
    /// The proposed rhythm.
    pub rhythm: Rhythm,
    /// The strategy that proposed it.
    pub strategy: Strategy,
    /// Its calm score.
    pub score: f64,
    /// The Frobenius norm of the score's Hessian at it.
    pub curvature: f64,
    /// The largest-magnitude eigenvalue of that Hessian.
    pub sharpest: f64,
    /// `score - CURVATURE_WEIGHT * curvature`.
    pub robust: f64,
}

/// Ranks candidates by descending robust score, the sharpest curvature
/// breaking ties, then the content address, so the order is total.
pub fn hessian_sort(candidates: &[(Rhythm, Strategy)]) -> Vec<Ranked> {
    let mut ranked: Vec<Ranked> = candidates
        .iter()
        .map(|(rhythm, strategy)| {
            let x = rhythm.as_point();
            let h = hessian(x);
            let curvature = frobenius(&h);
            let eig = eigenvalues(h);
            let sharpest = eig
                .iter()
                .copied()
                .fold(0.0_f64, |m, v| if v.abs() > m.abs() { v } else { m });
            let score = calm_score(x);
            Ranked {
                rhythm: *rhythm,
                strategy: *strategy,
                score,
                curvature,
                sharpest,
                robust: score - CURVATURE_WEIGHT * curvature,
            }
        })
        .collect();
    ranked.sort_by(|a, b| {
        b.robust
            .total_cmp(&a.robust)
            .then(a.sharpest.abs().total_cmp(&b.sharpest.abs()))
            .then(a.rhythm.kappa().cmp(&b.rhythm.kappa()))
    });
    ranked
}

/// A way of proposing a rhythm from a Rule 30 word.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Strategy {
    /// Read all four phase lengths straight from the word.
    Direct,
    /// Move each phase of the best known rhythm by at most one tick.
    Mutate,
    /// Take each phase from one of two known rhythms.
    Crossover,
}

impl Strategy {
    /// Every strategy, in a fixed order.
    pub const ALL: [Strategy; 3] = [Strategy::Direct, Strategy::Mutate, Strategy::Crossover];

    /// The strategy's name in the engine state.
    pub fn name(&self) -> &'static str {
        match self {
            Strategy::Direct => "rule30-direct",
            Strategy::Mutate => "mutate-best",
            Strategy::Crossover => "crossover",
        }
    }

    /// The strategy named `name`.
    pub fn named(name: &str) -> Option<Strategy> {
        Strategy::ALL.into_iter().find(|s| s.name() == name)
    }

    /// The rhythm this strategy proposes from `word`, given the rhythms known
    /// so far (never empty: 4-7-8 is always known).
    pub fn propose(&self, word: u64, known: &[Rhythm], best: Rhythm) -> Rhythm {
        let nibble = |k: u32| ((word >> (4 * k)) & 0xf) as u32;
        match self {
            Strategy::Direct => Rhythm {
                inhale: 1 + nibble(0) % 9,
                hold: nibble(1) % 10,
                exhale: 1 + nibble(2) % 12,
                rest: nibble(3) % 8,
            },
            Strategy::Mutate => {
                let shift = |v: u32, k: u32| match (word >> (2 * k + 16)) & 3 {
                    0 => v.saturating_sub(1),
                    1 => v + 1,
                    _ => v,
                };
                Rhythm {
                    inhale: shift(best.inhale, 0),
                    hold: shift(best.hold, 1),
                    exhale: shift(best.exhale, 2),
                    rest: shift(best.rest, 3),
                }
            }
            Strategy::Crossover => {
                let n = known.len() as u64;
                let a = known[(word % n) as usize];
                let b = known[((word >> 8) % n) as usize];
                let pick = |k: u32, x: u32, y: u32| if (word >> (24 + k)) & 1 == 0 { x } else { y };
                Rhythm {
                    inhale: pick(0, a.inhale, b.inhale),
                    hold: pick(1, a.hold, b.hold),
                    exhale: pick(2, a.exhale, b.exhale),
                    rest: pick(3, a.rest, b.rest),
                }
            }
        }
    }
}

/// The bounds of a Hebbian weight.
pub const WEIGHT_RANGE: (u32, u32) = (1, 16);

/// The learned Hebbian weight of each strategy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Weights(pub Vec<(Strategy, u32)>);

impl Default for Weights {
    fn default() -> Self {
        Weights(Strategy::ALL.iter().map(|s| (*s, 8)).collect())
    }
}

impl Weights {
    /// The weight of `strategy`.
    pub fn of(&self, strategy: Strategy) -> u32 {
        self.0
            .iter()
            .find(|(s, _)| *s == strategy)
            .map_or(8, |(_, w)| *w)
    }

    /// The strategies heaviest first, the fixed order breaking ties.
    pub fn schedule(&self) -> Vec<Strategy> {
        let mut order = Strategy::ALL.to_vec();
        order.sort_by(|a, b| self.of(*b).cmp(&self.of(*a)).then(a.cmp(b)));
        order
    }

    /// How many of `total` proposals each strategy makes: in proportion to its
    /// weight, at least one each, the remainder to the heaviest.
    pub fn allocation(&self, total: usize) -> Vec<(Strategy, usize)> {
        let sum: u32 = Strategy::ALL.iter().map(|s| self.of(*s)).sum();
        let mut out: Vec<(Strategy, usize)> = self
            .schedule()
            .into_iter()
            .map(|s| (s, ((self.of(s) as usize * total) / sum as usize).max(1)))
            .collect();
        let used: usize = out.iter().map(|(_, n)| n).sum();
        if used < total {
            out[0].1 += total - used;
        }
        out
    }

    /// The Hebbian update: the strategy whose proposal was proved gains two,
    /// every strategy all of whose proposals were refused loses one, and every
    /// weight stays within [`WEIGHT_RANGE`].
    pub fn learn(&mut self, proved: Option<Strategy>, refused: &[Strategy]) {
        let (lo, hi) = WEIGHT_RANGE;
        for (s, w) in self.0.iter_mut() {
            if Some(*s) == proved {
                *w = (*w + 2).min(hi);
            } else if refused.contains(s) {
                *w = w.saturating_sub(1).max(lo);
            }
            *w = (*w).clamp(lo, hi);
        }
        for s in Strategy::ALL {
            if !self.0.iter().any(|(x, _)| *x == s) {
                self.0.push((s, 8));
            }
        }
        self.0.sort_by_key(|(s, _)| *s);
    }
}

/// Proposals per tick.
pub const CANDIDATES: usize = 12;

/// The lowest robust score a rhythm may be admitted with.
pub const ADMISSION_FLOOR: f64 = -2.5;

/// The most rhythms the frontier holds; past it the engine is settled.
pub const FRONTIER_CAPACITY: usize = 32;

/// What one tick decided, before the kernel has proved anything.
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    /// Every candidate, Hessian sorted.
    pub ranked: Vec<Ranked>,
    /// The one candidate the tick proposes to admit, if any.
    pub admit: Option<Ranked>,
    /// Strategies none of whose proposals was admissible.
    pub refused: Vec<Strategy>,
    /// The driver after the tick.
    pub driver: Driver,
}

/// Plans one tick: abduction and Hessian sorting over the known rhythms
/// (4-7-8 and the frontier), never readmitting a known or refuted rhythm.
pub fn plan(driver: Driver, weights: &Weights, frontier: &[Rhythm], refuted: &[Rhythm]) -> Plan {
    let mut driver = driver;
    let mut known = vec![RELAX_478];
    known.extend_from_slice(frontier);
    let best = *known
        .iter()
        .max_by(|a, b| {
            robust_score(a)
                .total_cmp(&robust_score(b))
                .then(b.kappa().cmp(&a.kappa()))
        })
        .unwrap_or(&RELAX_478);
    let mut candidates = Vec::with_capacity(CANDIDATES);
    for (strategy, count) in weights.allocation(CANDIDATES) {
        for _ in 0..count {
            let word = driver.next_word();
            candidates.push((strategy.propose(word, &known, best), strategy));
        }
    }
    let ranked = hessian_sort(&candidates);
    let admissible = |r: &Ranked| {
        r.rhythm.safe()
            && !known.contains(&r.rhythm)
            && !refuted.contains(&r.rhythm)
            && r.robust >= ADMISSION_FLOOR
    };
    let admit = if frontier.len() >= FRONTIER_CAPACITY {
        None
    } else {
        ranked.iter().find(|r| admissible(r)).cloned()
    };
    let refused = Strategy::ALL
        .into_iter()
        .filter(|s| {
            ranked.iter().any(|r| r.strategy == *s)
                && !ranked.iter().any(|r| r.strategy == *s && admissible(r))
        })
        .collect();
    Plan {
        ranked,
        admit,
        refused,
        driver,
    }
}

/// The LexLean source of `app/src/Braid/Frontier.lex.tex` for `frontier`: the
/// rhythms as records, their names, their count, and the theorem the kernel
/// must prove for the tick to stand.
pub fn frontier_module(frontier: &[Rhythm]) -> String {
    let mut rhythms = String::from("(nil Braid.Breath::Rhythm)");
    let mut names = String::from("(nil String)");
    for (i, r) in frontier.iter().enumerate().rev() {
        rhythms = format!(
            "(cons (record Braid.Breath::Rhythm {{inhale := {}, hold := {}, exhale := {}, rest := {}}}) {rhythms})",
            r.inhale, r.hold, r.exhale, r.rest
        );
        names = format!("(cons \"F{} {}\" {names})", i + 1, r.label());
    }
    format!(
        "\\begin{{lexlean}}{{Braid.Frontier}}\n\\useglossary{{lexlean.std.bool@1.2.0}}\n\\useglossary{{lexlean.std.nat@1.2.0}}\n\\useglossary{{prism.arch@1.0.0}}\n\\useglossary{{echobraid.domain@1.0.0}}\n\\importmodule{{Braid.Breath}}\n\\title{{EchoBraid learning companion model}}\n\\begin{{semanticmodule}}\ndefinition{{\n  name: frontierRhythms,\n  parameters: [],\n  result: (List Braid.Breath::Rhythm),\n  body: {rhythms}\n}}\n\ndefinition{{\n  name: frontierNames,\n  parameters: [],\n  result: (List String),\n  body: {names}\n}}\n\ndefinition{{name: frontierCount, parameters: [], result: Nat, body: {}}}\n\ntheorem{{\n  name: frontierSafe,\n  parameters: [],\n  statement: ((Braid.Breath::allSafe (frontierRhythms)) = true),\n  proof: decide{{}}\n}}\n\\end{{semanticmodule}}\n\\end{{lexlean}}\n",
        frontier.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule30_matches_the_cell_rule() {
        let s = 0b0001_0000u64;
        let n = step_rule30_u64(s);
        for i in 0..64 {
            let bit = |x: u64, k: i32| (x >> (k.rem_euclid(64))) & 1;
            let (l, c, r) = (bit(s, i + 1), bit(s, i), bit(s, i - 1));
            assert_eq!(bit(n, i), l ^ (c | r), "cell {i}");
        }
    }

    #[test]
    fn relax_is_safe_and_hessian_is_symmetric() {
        assert!(RELAX_478.safe());
        let h = hessian(RELAX_478.as_point());
        for (i, row) in h.iter().enumerate() {
            for (j, v) in row.iter().enumerate() {
                assert!((v - h[j][i]).abs() < 1e-9);
            }
        }
    }
}
