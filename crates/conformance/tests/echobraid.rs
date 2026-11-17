//! The `echobraid` suite: one test per scenario in `features/suites/echobraid.feature`.
//!
//! The Lean kernel proves every theorem when `just vv` runs `lexlean verify`
//! on `app/` (the holo's own kernel-decided theorems, checked again by
//! `prismpm verify`) and on the root proofs project (`proofs/Laws`: the
//! arithmetic theorems, stated over the very `app/src` definitions the guest
//! runs). What these tests hold fixed is the claim the proofs are about: each
//! theorem's exact statement, that it rests on no axiom beyond Lean's standard
//! three, and that the guest's `dispatch` reaches the very definitions the
//! theorems quantify over. A weakened statement, an extra axiom, or a reply
//! decided some other way would still verify, and would no longer be the
//! claim its row makes.

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/conformance is two below the root")
        .to_path_buf()
}

fn source(path: &str) -> String {
    std::fs::read_to_string(root().join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
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

/// Asserts the theorem states exactly `statement`, is proved rather than
/// assumed, and uses only Lean's standard axioms; and, for the proofs project,
/// that the project verifies the module it lives in.
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
    let project = if path.starts_with("proofs/") {
        source("lexlean.toml")
    } else {
        source("app/lexlean.toml")
    };
    let entry = path.strip_prefix("app/").unwrap_or(path);
    let reachable =
        project.contains(&format!("\"{entry}\"")) || project.contains("\"src/App.lex.tex\"");
    assert!(
        reachable,
        "{path} is not an entrypoint of the project that verifies it"
    );
}

/// Asserts the guest's dispatch reaches the proved definition through `term`.
fn dispatched(term: &str) {
    let app = source("app/src/App.lex.tex");
    assert!(
        app.contains(term),
        "dispatch no longer decides through {term}"
    );
}

/// `EB-01`: Semantic drift stays within 0 to 100 under every message and every rest, and a rest never raises it
#[test]
fn echobraid_eb_01() {
    pinned(
        "proofs/Laws/ResonanceLaws.lex.tex",
        "driftStepBounded",
        "((Braid.Resonance::driftStep drift length) <= 100)",
    );
    pinned(
        "proofs/Laws/ResonanceLaws.lex.tex",
        "restStepBounded",
        "((Braid.Resonance::restStep drift) <= 100)",
    );
    pinned(
        "proofs/Laws/ResonanceLaws.lex.tex",
        "restNeverRaisesDrift",
        "((drift <= 100) -> ((Braid.Resonance::restStep drift) <= drift))",
    );
    dispatched("(driftReply (Braid.Resonance::driftStep (arg value 1) (arg value 2)))");
    dispatched("(driftReply (Braid.Resonance::restStep (arg value 1)))");
}

/// `EB-02`: A pause is offered only when drift is divergent, and when it is offered the guide answers with the pause and nothing else
#[test]
fn echobraid_eb_02() {
    pinned("proofs/Laws/ResonanceLaws.lex.tex", "pauseOnlyWhenDivergent", "(((Braid.Resonance::offersPause drift) = true) -> ((Braid.Resonance::isDivergent drift) = true))");
    pinned("proofs/Laws/ReflectionLaws.lex.tex", "pauseComesFirst", "(((Braid.Resonance::offersPause drift) = true) -> ((Braid.Reflection::guideIndex drift variant) = 0))");
    dispatched("(Braid.Reflection::guideIndex drift variant)");
    dispatched("(if (Braid.Resonance::offersPause drift) then \"pause\" else \"flow\")");
}

/// `EB-03`: Coherence rises monotonically with drift and is divergent exactly when drift is at least 70
#[test]
fn echobraid_eb_03() {
    pinned("proofs/Laws/ResonanceLaws.lex.tex", "divergentDecides", "(((Braid.Resonance::isDivergent drift) = true) <-> ((Braid.Resonance::coherenceRank drift) = 2))");
    pinned("proofs/Laws/ResonanceLaws.lex.tex", "coherenceMonotone", "((lower <= upper) -> ((Braid.Resonance::coherenceRank lower) <= (Braid.Resonance::coherenceRank upper)))");
    dispatched("(Braid.Resonance::coherenceName drift)");
}

/// `EB-04`: Every breath countdown is at least one and within its cycle, and the 4-7-8 rhythm is safe with a 19-tick cycle
#[test]
fn echobraid_eb_04() {
    pinned("proofs/Laws/BreathLaws.lex.tex", "countPositive", "((position < (Braid.Breath::cycleLength rhythm)) -> (1 <= (Braid.Breath::countAt rhythm position)))");
    pinned(
        "proofs/Laws/BreathLaws.lex.tex",
        "countWithinPhase",
        "((Braid.Breath::countAt rhythm position) <= (Braid.Breath::cycleLength rhythm))",
    );
    pinned(
        "proofs/Laws/BreathLaws.lex.tex",
        "phaseBounded",
        "((Braid.Breath::phaseAt rhythm position) <= 3)",
    );
    pinned(
        "app/src/Braid/Breath.lex.tex",
        "relaxCycleIsNineteen",
        "((cycleLength (relax478)) = 19)",
    );
    pinned(
        "app/src/Braid/Breath.lex.tex",
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
    pinned("proofs/Laws/ConsentLaws.lex.tex", "noSilentGrant", "(((action =? grant) = false) -> ((current = 0) -> ((Braid.Consent::nextFlag current grant revoke action) = 0)))");
    pinned(
        "proofs/Laws/ConsentLaws.lex.tex",
        "revokeAllClears",
        "((Braid.Consent::nextFlag current grant revoke 7) = 0)",
    );
    pinned(
        "proofs/Laws/ConsentLaws.lex.tex",
        "flagIsBinary",
        "((Braid.Consent::nextFlag current grant revoke action) <= 1)",
    );
    pinned("proofs/Laws/ConsentLaws.lex.tex", "revokeWins", "((grant < revoke) -> ((revoke < 7) -> ((Braid.Consent::nextFlag current grant revoke revoke) = 0)))");
    dispatched("(Braid.Consent::nextFlag (arg value 1) 1 4 (arg value 4))");
    dispatched("(Braid.Consent::nextFlag (arg value 2) 2 5 (arg value 4))");
    dispatched("(Braid.Consent::nextFlag (arg value 3) 3 6 (arg value 4))");
}

/// `EB-06`: The memory game's Rule 30 tiles are always one of four, its prime-length levels never exceed 23, a wrong tile ends the round and victory comes only at level 7
#[test]
fn echobraid_eb_06() {
    pinned(
        "proofs/Laws/MemoryLaws.lex.tex",
        "tileBounded",
        "((Braid.Memory::tileOf state) < 4)",
    );
    pinned(
        "proofs/Laws/MemoryLaws.lex.tex",
        "levelLengthBounded",
        "((Braid.Memory::levelLength level) <= 23)",
    );
    pinned("proofs/Laws/MemoryLaws.lex.tex", "victoryOnlyAtFinalLevel", "(((level =? 7) = false) -> ((Braid.Memory::verdictCode level index expected chosen) <= 2))");
    pinned("proofs/Laws/MemoryLaws.lex.tex", "wrongTileEndsRound", "(((chosen =? expected) = false) -> ((Braid.Memory::verdictCode level index expected chosen) = 0))");
    pinned(
        "proofs/Laws/MemoryLaws.lex.tex",
        "seedNeverDegenerate",
        "(1 <= (Braid.Memory::seedState seed))",
    );
    dispatched("(Braid.Memory::verdictCode (arg value 1) (arg value 3) (Braid.Memory::tileAt (arg value 2) (arg value 3)) (arg value 4))");
}

/// `EB-07`: Every reflection prompt is indexed by one of the primes 2, 3, 5 and 7, and every guide reply is one of ten fixed, non-directive texts
#[test]
fn echobraid_eb_07() {
    pinned(
        "proofs/Laws/ReflectionLaws.lex.tex",
        "guideIndexBounded",
        "((Braid.Reflection::guideIndex drift variant) <= 9)",
    );
    pinned("proofs/Laws/ReflectionLaws.lex.tex", "promptPrimeBounded", "((2 <= (Braid.Reflection::promptPrime index)) /\\ ((Braid.Reflection::promptPrime index) <= 7))");
    pinned(
        "proofs/Laws/ReflectionLaws.lex.tex",
        "promptPrimeNotFour",
        "(((Braid.Reflection::promptPrime index) =? 4) = false)",
    );
    pinned(
        "proofs/Laws/ReflectionLaws.lex.tex",
        "promptPrimeNotSix",
        "(((Braid.Reflection::promptPrime index) =? 6) = false)",
    );
    dispatched("(Braid.Reflection::promptPrime index)");
    // The ten guide texts: short, and never an instruction.
    let reflection = source("app/src/Braid/Reflection.lex.tex");
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
    pinned("proofs/Laws/LogicLaws.lex.tex", "quorumIntersection", "(((members + (faults + 1)) <= (quorumA + quorumB)) -> ((joined <= members) -> ((Braid.Logic::quorumHolds faults quorumA quorumB joined) = true)))");
    pinned("proofs/Laws/LogicLaws.lex.tex", "byzantineQuorum", "((((faults + faults) + 1) <= quorumA) -> ((((faults + faults) + 1) <= quorumB) -> ((joined <= (((faults + faults) + faults) + 1)) -> ((Braid.Logic::quorumHolds faults quorumA quorumB joined) = true))))");
    pinned(
        "proofs/Laws/LogicLaws.lex.tex",
        "ringStaysOrdered",
        "((read <= write) -> (read <= (Braid.Logic::ringWrite capacity read write)))",
    );
    pinned("proofs/Laws/LogicLaws.lex.tex", "ringNeverOverflows", "((read <= write) -> (((prim subtract write read : Nat) <= capacity) -> ((prim subtract (Braid.Logic::ringWrite capacity read write) read : Nat) <= capacity)))");
    dispatched(
        "(Braid.Logic::quorumHolds (arg value 2) (arg value 3) (arg value 4) (arg value 5))",
    );
    dispatched("(Braid.Logic::ringWrite capacity read write)");
}

/// `EB-09`: Data kept under the 24-hour policy is erased once it is 24 hours old, immediate data is always erased and kept data never is
#[test]
fn echobraid_eb_09() {
    pinned(
        "proofs/Laws/RetentionLaws.lex.tex",
        "dayPolicyErases",
        "((24 <= age) -> ((Braid.Retention::shouldErase 1 age) = true))",
    );
    pinned(
        "app/src/Braid/Retention.lex.tex",
        "immediatePolicyAlwaysErases",
        "((shouldErase 0 age) = true)",
    );
    pinned(
        "app/src/Braid/Retention.lex.tex",
        "keepPolicyNeverErases",
        "((shouldErase 3 age) = false)",
    );
    dispatched("(Braid.Retention::shouldErase (arg value 1) (arg value 2))");
}

/// `EB-10`: Every breathing rhythm the engine has written into the frontier is proved safe by the Lean kernel
#[test]
fn echobraid_eb_10() {
    pinned(
        "app/src/Braid/Frontier.lex.tex",
        "frontierSafe",
        "((frontierSafeFrom 0 (frontierCount)) = true)",
    );
    dispatched("(Braid.Frontier::frontierAt (prim subtract index 1 : Nat))");
}
