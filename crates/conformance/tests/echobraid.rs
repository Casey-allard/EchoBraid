//! The `echobraid` suite: one test per scenario in `features/suites/echobraid.feature`.
//!
//! The Lean kernel proves every theorem when `just vv` runs `prismpm verify`
//! (and `lexlean verify`) on `app/`. What these tests hold fixed is the claim the
//! proofs are about: each theorem's exact statement, that it rests on no axiom
//! beyond Lean's standard three, and that the guest's `dispatch` reaches the
//! very definitions the theorems quantify over. A weakened statement, an extra
//! axiom, or a reply decided some other way would still verify, and would no
//! longer be the claim its row makes.

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/conformance is two below the root")
        .to_path_buf()
}

fn source(path: &str) -> String {
    std::fs::read_to_string(root().join("app").join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The body of the theorem named `name`, up to its closing brace.
fn theorem<'a>(model: &'a str, name: &str) -> &'a str {
    let start = model
        .find(&format!("theorem{{\n  name: {name},\n"))
        .unwrap_or_else(|| panic!("{name} is not declared"));
    let end = model[start..]
        .find("\n}\n")
        .unwrap_or_else(|| panic!("{name} is not closed"));
    &model[start..start + end]
}

/// Asserts the theorem states exactly `statement` and uses only Lean's
/// standard axioms.
fn pinned(path: &str, name: &str, statement: &str) {
    let model = source(path);
    let body = theorem(&model, name);
    assert!(
        body.contains(&format!("  statement: {statement},\n")),
        "{name} in {path} no longer states {statement}:\n{body}"
    );
    assert!(
        !body.contains("sorry"),
        "{name} must be proved, not assumed"
    );
    if let Some(line) = body.lines().find(|l| l.trim_start().starts_with("axioms:")) {
        for axiom in line.split(['[', ']', ',']).map(str::trim).skip(1) {
            assert!(
                axiom.is_empty() || ["propext", "Quot.sound", "Classical.choice"].contains(&axiom),
                "{name} rests on a non-standard axiom {axiom}"
            );
        }
    }
}

/// Asserts the guest's dispatch reaches the proved definition through `term`.
fn dispatched(term: &str) {
    let app = source("src/App.lex.tex");
    assert!(
        app.contains(term),
        "dispatch no longer decides through {term}"
    );
}

/// `EB-01`: Semantic drift stays within 0 to 100 under every message and every rest, and a rest never raises it
#[test]
fn echobraid_eb_01() {
    pinned(
        "src/Braid/Resonance.lex.tex",
        "driftStepBounded",
        "((driftStep drift length) <= 100)",
    );
    pinned(
        "src/Braid/Resonance.lex.tex",
        "restStepBounded",
        "((restStep drift) <= 100)",
    );
    pinned(
        "src/Braid/Resonance.lex.tex",
        "restNeverRaisesDrift",
        "((drift <= 100) -> ((restStep drift) <= drift))",
    );
    dispatched("(driftReply (Braid.Resonance::driftStep (arg value 1) (arg value 2)))");
    dispatched("(driftReply (Braid.Resonance::restStep (arg value 1)))");
}

/// `EB-02`: A pause is offered only when drift is divergent, and when it is offered the guide answers with the pause and nothing else
#[test]
fn echobraid_eb_02() {
    pinned(
        "src/Braid/Resonance.lex.tex",
        "pauseOnlyWhenDivergent",
        "(((offersPause drift) = true) -> ((isDivergent drift) = true))",
    );
    pinned(
        "src/Braid/Reflection.lex.tex",
        "pauseComesFirst",
        "(((Braid.Resonance::offersPause drift) = true) -> ((guideIndex drift variant) = 0))",
    );
    dispatched("(Braid.Reflection::guideIndex drift variant)");
    dispatched("(if (Braid.Resonance::offersPause drift) then \"pause\" else \"flow\")");
}

/// `EB-03`: Coherence rises monotonically with drift and is divergent exactly when drift is at least 70
#[test]
fn echobraid_eb_03() {
    pinned(
        "src/Braid/Resonance.lex.tex",
        "divergentDecides",
        "(((isDivergent drift) = true) <-> ((coherenceRank drift) = 2))",
    );
    pinned(
        "src/Braid/Resonance.lex.tex",
        "coherenceMonotone",
        "((lower <= upper) -> ((coherenceRank lower) <= (coherenceRank upper)))",
    );
    dispatched("(Braid.Resonance::coherenceName drift)");
}

/// `EB-04`: Every breath countdown is at least one and within its cycle, and the 4-7-8 rhythm is safe with a 19-tick cycle
#[test]
fn echobraid_eb_04() {
    pinned(
        "src/Braid/Breath.lex.tex",
        "countPositive",
        "((position < (cycleLength rhythm)) -> (1 <= (countAt rhythm position)))",
    );
    pinned(
        "src/Braid/Breath.lex.tex",
        "countWithinPhase",
        "((countAt rhythm position) <= (cycleLength rhythm))",
    );
    pinned(
        "src/Braid/Breath.lex.tex",
        "phaseBounded",
        "((phaseAt rhythm position) <= 3)",
    );
    pinned(
        "src/Braid/Breath.lex.tex",
        "relaxCycleIsNineteen",
        "((cycleLength (relax478)) = 19)",
    );
    pinned(
        "src/Braid/Breath.lex.tex",
        "relaxIsSafe",
        "((rhythmSafe (relax478)) = true)",
    );
    dispatched(
        "(Braid.Breath::countAt (rhythmAt index) (Braid.Breath::positionOf (rhythmAt index) tick))",
    );
}

/// `EB-05`: No sensor consent turns on without its own grant, revoke-all clears every flag, and a flag is always 0 or 1
#[test]
fn echobraid_eb_05() {
    pinned("src/Braid/Consent.lex.tex", "noSilentGrant", "(((action =? grant) = false) -> ((current = 0) -> ((nextFlag current grant revoke action) = 0)))");
    pinned(
        "src/Braid/Consent.lex.tex",
        "revokeAllClears",
        "((nextFlag current grant revoke 7) = 0)",
    );
    pinned(
        "src/Braid/Consent.lex.tex",
        "flagIsBinary",
        "((nextFlag current grant revoke action) <= 1)",
    );
    pinned(
        "src/Braid/Consent.lex.tex",
        "revokeWins",
        "((grant < revoke) -> ((revoke < 7) -> ((nextFlag current grant revoke revoke) = 0)))",
    );
    dispatched("(Braid.Consent::nextFlag (arg value 1) 1 4 (arg value 4))");
    dispatched("(Braid.Consent::nextFlag (arg value 2) 2 5 (arg value 4))");
    dispatched("(Braid.Consent::nextFlag (arg value 3) 3 6 (arg value 4))");
}

/// `EB-06`: The memory game's Rule 30 tiles are always one of four, its prime-length levels never exceed 23, a wrong tile ends the round and victory comes only at level 7
#[test]
fn echobraid_eb_06() {
    pinned(
        "src/Braid/Memory.lex.tex",
        "tileBounded",
        "((tileOf state) < 4)",
    );
    pinned(
        "src/Braid/Memory.lex.tex",
        "levelLengthBounded",
        "((levelLength level) <= 23)",
    );
    pinned(
        "src/Braid/Memory.lex.tex",
        "victoryOnlyAtFinalLevel",
        "(((level =? 7) = false) -> ((verdictCode level index expected chosen) <= 2))",
    );
    pinned(
        "src/Braid/Memory.lex.tex",
        "wrongTileEndsRound",
        "(((chosen =? expected) = false) -> ((verdictCode level index expected chosen) = 0))",
    );
    pinned(
        "src/Braid/Memory.lex.tex",
        "seedNeverDegenerate",
        "(1 <= (seedState seed))",
    );
    dispatched("(Braid.Memory::verdictCode (arg value 1) (arg value 3) (Braid.Memory::tileAt (arg value 2) (arg value 3)) (arg value 4))");
}

/// `EB-07`: Every reflection prompt is indexed by one of the primes 2, 3, 5 and 7, and every guide reply is one of ten fixed, non-directive texts
#[test]
fn echobraid_eb_07() {
    pinned(
        "src/Braid/Reflection.lex.tex",
        "guideIndexBounded",
        "((guideIndex drift variant) <= 9)",
    );
    pinned(
        "src/Braid/Reflection.lex.tex",
        "promptPrimeBounded",
        "((2 <= (promptPrime index)) /\\ ((promptPrime index) <= 7))",
    );
    pinned(
        "src/Braid/Reflection.lex.tex",
        "promptPrimeNotFour",
        "(((promptPrime index) =? 4) = false)",
    );
    pinned(
        "src/Braid/Reflection.lex.tex",
        "promptPrimeNotSix",
        "(((promptPrime index) =? 6) = false)",
    );
    dispatched("(Braid.Reflection::promptPrime index)");
    // The ten guide texts: short, and never an instruction.
    let reflection = source("src/Braid/Reflection.lex.tex");
    let guide = theorem(&reflection, "guideIndexBounded");
    assert!(guide.contains("<= 9"), "ten guide texts, indexes 0 to 9");
    let start = reflection.find("name: guideText,").expect("guideText");
    let body = &reflection[start..start + reflection[start..].find("\n}\n").expect("closed")];
    let texts: Vec<&str> = body.split('"').skip(1).step_by(2).collect();
    assert_eq!(texts.len(), 9, "nine reflections beside the pause offer");
    for text in texts {
        assert!(
            text.split_whitespace().count() <= 50,
            "over 50 words: {text}"
        );
        let lower = text.to_lowercase();
        for directive in ["you should", "you must", "you need to", "try to", "do this"] {
            assert!(!lower.contains(directive), "directive guide text: {text}");
        }
    }
}

/// `EB-08`: Two quorums whose sizes sum past the membership plus the fault bound share at least one more member than the fault bound, and the ring buffer never overflows or reorders
#[test]
fn echobraid_eb_08() {
    pinned("src/Braid/Logic.lex.tex", "quorumIntersection", "(((members + (faults + 1)) <= (quorumA + quorumB)) -> ((joined <= members) -> ((quorumHolds faults quorumA quorumB joined) = true)))");
    pinned("src/Braid/Logic.lex.tex", "byzantineQuorum", "((((faults + faults) + 1) <= quorumA) -> ((((faults + faults) + 1) <= quorumB) -> ((joined <= (((faults + faults) + faults) + 1)) -> ((quorumHolds faults quorumA quorumB joined) = true))))");
    pinned(
        "src/Braid/Logic.lex.tex",
        "ringStaysOrdered",
        "((read <= write) -> (read <= (ringWrite capacity read write)))",
    );
    pinned("src/Braid/Logic.lex.tex", "ringNeverOverflows", "((read <= write) -> (((prim subtract write read : Nat) <= capacity) -> ((prim subtract (ringWrite capacity read write) read : Nat) <= capacity)))");
    dispatched(
        "(Braid.Logic::quorumHolds (arg value 2) (arg value 3) (arg value 4) (arg value 5))",
    );
    dispatched("(Braid.Logic::ringWrite capacity read write)");
}

/// `EB-09`: Data kept under the 24-hour policy is erased once it is 24 hours old, immediate data is always erased and kept data never is
#[test]
fn echobraid_eb_09() {
    pinned(
        "src/Braid/Retention.lex.tex",
        "dayPolicyErases",
        "((24 <= age) -> ((shouldErase 1 age) = true))",
    );
    pinned(
        "src/Braid/Retention.lex.tex",
        "immediatePolicyAlwaysErases",
        "((shouldErase 0 age) = true)",
    );
    pinned(
        "src/Braid/Retention.lex.tex",
        "keepPolicyNeverErases",
        "((shouldErase 3 age) = false)",
    );
    dispatched("(Braid.Retention::shouldErase (arg value 1) (arg value 2))");
}

/// `EB-10`: Every breathing rhythm the engine has written into the frontier is proved safe by the Lean kernel
#[test]
fn echobraid_eb_10() {
    pinned(
        "src/Braid/Frontier.lex.tex",
        "frontierSafe",
        "((Braid.Breath::allSafe (frontierRhythms)) = true)",
    );
    dispatched(
        "(Braid.Breath::nthRhythm (Braid.Frontier::frontierRhythms) (prim subtract index 1 : Nat))",
    );
}
