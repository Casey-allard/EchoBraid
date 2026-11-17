//! `cargo xtask engine tick|status`: one autonomous engine tick, and its state.
//!
//! The decisions are `echobraid_engine`'s and are pure; this module does the
//! file work around them. A tick plans (Rule 30 abduction, Hessian sorting),
//! writes the admitted rhythm into the live model
//! (`app/src/Braid/Frontier.lex.tex`), regenerates the acceptance vectors from
//! the reference oracle, bumps the application's version, and, with
//! `--prove`, has the Lean kernel check the model (`lexlean verify`). A
//! candidate the kernel refuses is rolled back and remembered as refuted, so
//! it is never proposed again. The workflow `engine.yml` commits what a tick
//! leaves and redeploys the page, which rebuilds and re-verifies the holo.

use std::path::Path;
use std::process::Command;

use echobraid_engine::{
    frontier_module, plan, Driver, Ranked, Rhythm, Strategy, Weights, FRONTIER_CAPACITY,
};
use serde::{Deserialize, Serialize};

use crate::Fail;

const STATE: &str = "model/engine.toml";
const FRONTIER: &str = "frontier.toml";
const MODULE: &str = "app/src/Braid/Frontier.lex.tex";
const APP: &str = "app/src/App.lex.tex";
const LOCK: &str = "app/lexlean.lock";
const OUTPUT: &str = ".engine-output.env";
const STATUS: &str = ".engine-status.md";
/// The genesis ring: "EchoBraid" folded into one word.
const GENESIS: u64 = 0x4563_686f_4272_6169;

#[derive(Serialize, Deserialize)]
struct State {
    spec: String,
    rule30_state: String,
    rule30_generation: u64,
    tick: u64,
    #[serde(default)]
    weight: Vec<WeightRow>,
    #[serde(default)]
    attempt: Vec<Attempt>,
}

#[derive(Serialize, Deserialize)]
struct WeightRow {
    strategy: String,
    value: u32,
}

#[derive(Serialize, Deserialize)]
struct Attempt {
    tick: u64,
    kappa: String,
    rhythm: [u32; 4],
    strategy: String,
    outcome: String,
}

#[derive(Serialize, Deserialize, Default)]
struct Frontier {
    spec: String,
    #[serde(default)]
    claim: Vec<Claim>,
}

#[derive(Serialize, Deserialize)]
struct Claim {
    id: String,
    status: String,
    family: String,
    kappa: String,
    rhythm: [u32; 4],
    strategy: String,
    tick: u64,
    score: f64,
    curvature: f64,
    robust: f64,
    lean: String,
}

fn fail(message: impl Into<String>) -> Fail {
    message.into().into()
}

fn read(root: &Path, path: &str) -> Result<String, Fail> {
    std::fs::read_to_string(root.join(path)).map_err(|e| fail(format!("{path}: {e}")))
}

fn write(root: &Path, path: &str, text: &str) -> Result<(), Fail> {
    std::fs::write(root.join(path), text).map_err(|e| fail(format!("{path}: {e}")))
}

fn rhythm(r: [u32; 4]) -> Rhythm {
    Rhythm {
        inhale: r[0],
        hold: r[1],
        exhale: r[2],
        rest: r[3],
    }
}

fn array(r: &Rhythm) -> [u32; 4] {
    [r.inhale, r.hold, r.exhale, r.rest]
}

fn load(root: &Path) -> Result<(State, Frontier), Fail> {
    let state = if root.join(STATE).exists() {
        toml::from_str(&read(root, STATE)?).map_err(|e| fail(format!("{STATE}: {e}")))?
    } else {
        State {
            spec: "echobraid/engine/1".into(),
            rule30_state: format!("{GENESIS:#018x}"),
            rule30_generation: 0,
            tick: 0,
            weight: Vec::new(),
            attempt: Vec::new(),
        }
    };
    let frontier = if root.join(FRONTIER).exists() {
        toml::from_str(&read(root, FRONTIER)?).map_err(|e| fail(format!("{FRONTIER}: {e}")))?
    } else {
        Frontier {
            spec: "echobraid/frontier/1".into(),
            claim: Vec::new(),
        }
    };
    Ok((state, frontier))
}

fn weights(state: &State) -> Weights {
    let mut w = Weights::default();
    for row in &state.weight {
        if let Some(s) = Strategy::named(&row.strategy) {
            if let Some(slot) = w.0.iter_mut().find(|(x, _)| *x == s) {
                slot.1 = row.value;
            }
        }
    }
    w
}

fn header(path: &str) -> String {
    match path {
        STATE => "# model/engine.toml: the EchoBraid engine's state. Written by `cargo xtask engine tick`; never edit by hand.\n".into(),
        _ => "# frontier.toml: every rhythm the engine has proved and shipped. Written by `cargo xtask engine tick`; never edit by hand.\n".into(),
    }
}

fn save(root: &Path, state: &State, frontier: &Frontier) -> Result<(), Fail> {
    let s = toml::to_string(state).map_err(|e| fail(e.to_string()))?;
    write(root, STATE, &(header(STATE) + &s))?;
    let f = toml::to_string(frontier).map_err(|e| fail(e.to_string()))?;
    write(root, FRONTIER, &(header(FRONTIER) + &f))
}

fn run(root: &Path, dir: &str, program: &str, args: &[&str]) -> Result<(), Fail> {
    let status = Command::new(program)
        .args(args)
        .current_dir(root.join(dir))
        .status()
        .map_err(|e| fail(format!("{program}: {e}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(fail(format!(
            "{program} {} exited with {status}",
            args.join(" ")
        )))
    }
}

fn bump_version(app: &str, tick: u64) -> String {
    let key = "cargoVersion := \"";
    match app.find(key) {
        Some(start) => {
            let from = start + key.len();
            let end = from + app[from..].find('"').unwrap_or(0);
            format!("{}0.2.{tick}{}", &app[..from], &app[end..])
        }
        None => app.to_string(),
    }
}

fn describe(r: &Ranked) -> String {
    format!(
        "{} by {} (calm {:.3}, curvature {:.3}, sharpest {:.3}, robust {:.3})",
        r.rhythm.label(),
        r.strategy.name(),
        r.score,
        r.curvature,
        r.sharpest,
        r.robust
    )
}

/// One engine tick. `--write` applies it to the tree; `--prove` also has the
/// Lean kernel verify the model and rolls the candidate back when it refuses.
pub fn tick(root: &Path, write_tree: bool, prove: bool) -> Result<(), Fail> {
    let (mut state, mut frontier) = load(root)?;
    let ring = u64::from_str_radix(state.rule30_state.trim_start_matches("0x"), 16)
        .map_err(|e| fail(format!("rule30_state: {e}")))?;
    let driver = Driver {
        state: ring,
        generation: state.rule30_generation,
    };
    let mut w = weights(&state);
    let known: Vec<Rhythm> = frontier.claim.iter().map(|c| rhythm(c.rhythm)).collect();
    let refuted: Vec<Rhythm> = state
        .attempt
        .iter()
        .filter(|a| a.outcome == "refuted")
        .map(|a| rhythm(a.rhythm))
        .collect();
    let p = plan(driver, &w, &known, &refuted);
    let tick = state.tick + 1;

    let mut report = format!("## Engine tick {tick}\n\nRule 30 ring `{:#018x}`, generation {}.\n\n| rank | rhythm | strategy | calm | curvature | robust | safe |\n| --- | --- | --- | --- | --- | --- | --- |\n", p.driver.state, p.driver.generation);
    for (i, r) in p.ranked.iter().enumerate() {
        report.push_str(&format!(
            "| {} | {} | {} | {:.3} | {:.3} | {:.3} | {} |\n",
            i + 1,
            r.rhythm.label(),
            r.strategy.name(),
            r.score,
            r.curvature,
            r.robust,
            if r.rhythm.safe() { "yes" } else { "no" }
        ));
    }

    let mut outcome = "idle".to_string();
    let mut title = format!(
        "Engine tick {tick}: 0 proved, {} candidates refused",
        p.ranked.len()
    );
    let mut proved = None;
    let mut refused = p.refused.clone();
    if let Some(candidate) = &p.admit {
        let id = format!("EB-F{:02}", frontier.claim.len() + 1);
        report.push_str(&format!(
            "\nAdmitted for proof: {id} {}.\n",
            describe(candidate)
        ));
        if write_tree {
            let module_before = read(root, MODULE)?;
            let app_before = read(root, APP)?;
            let lock_before = read(root, LOCK)?;
            let mut next = known.clone();
            next.push(candidate.rhythm);
            write(root, MODULE, &frontier_module(&next))?;
            write(root, APP, &bump_version(&app_before, tick))?;
            let result =
                run(root, ".", "node", &["scripts/reference.mjs", "--write"]).and_then(|()| {
                    if prove {
                        run(root, "app", "lexlean", &["fmt"])?;
                        run(root, "app", "lexlean", &["lock"])?;
                        run(root, "app", "lexlean", &["verify"])
                    } else {
                        Ok(())
                    }
                });
            match result {
                Ok(()) => {
                    outcome = if prove {
                        "proved".into()
                    } else {
                        "planned".into()
                    };
                    proved = Some(candidate.strategy);
                    frontier.claim.push(Claim {
                        id: id.clone(),
                        status: if prove {
                            "proved".into()
                        } else {
                            "observed".into()
                        },
                        family: "breath".into(),
                        kappa: candidate.rhythm.kappa(),
                        rhythm: array(&candidate.rhythm),
                        strategy: candidate.strategy.name().into(),
                        tick,
                        score: (candidate.score * 1e6).round() / 1e6,
                        curvature: (candidate.curvature * 1e6).round() / 1e6,
                        robust: (candidate.robust * 1e6).round() / 1e6,
                        lean: "EchoBraid.Braid.Frontier.frontierSafe".into(),
                    });
                    title = format!(
                        "Engine tick {tick}: 1 proved ({id} breath {}), {} strategies refused",
                        candidate.rhythm.label(),
                        refused.len()
                    );
                }
                Err(e) => {
                    write(root, MODULE, &module_before)?;
                    write(root, APP, &app_before)?;
                    write(root, LOCK, &lock_before)?;
                    run(root, ".", "node", &["scripts/reference.mjs", "--write"])?;
                    outcome = "refuted".into();
                    refused.push(candidate.strategy);
                    report.push_str(&format!(
                        "\nThe kernel refused it and it was rolled back: {}\n",
                        e
                    ));
                    title = format!(
                        "Engine tick {tick}: 0 proved, 1 refuted ({})",
                        candidate.rhythm.label()
                    );
                }
            }
            state.attempt.push(Attempt {
                tick,
                kappa: candidate.rhythm.kappa(),
                rhythm: array(&candidate.rhythm),
                strategy: candidate.strategy.name().into(),
                outcome: outcome.clone(),
            });
        }
    } else if known.len() >= FRONTIER_CAPACITY {
        outcome = "settled".into();
        title = format!("Engine tick {tick}: settled at {} rhythms", known.len());
    }

    w.learn(proved, &refused);
    report.push_str("\nHebbian weights after the tick: ");
    report.push_str(
        &w.0.iter()
            .map(|(s, v)| format!("{} {v}", s.name()))
            .collect::<Vec<_>>()
            .join(", "),
    );
    report.push('\n');
    println!("{title}\n\n{report}");

    if write_tree {
        state.rule30_state = format!("{:#018x}", p.driver.state);
        state.rule30_generation = p.driver.generation;
        state.tick = tick;
        state.weight =
            w.0.iter()
                .map(|(s, v)| WeightRow {
                    strategy: s.name().into(),
                    value: *v,
                })
                .collect();
        save(root, &state, &frontier)?;
        write(
            root,
            OUTPUT,
            &format!("outcome={outcome}\ntick={tick}\ntitle={title}\n"),
        )?;
        write(root, STATUS, &format!("{title}\n\n{report}"))?;
    }
    Ok(())
}

/// Prints the frontier and the weights.
pub fn status(root: &Path) -> Result<(), Fail> {
    let (state, frontier) = load(root)?;
    println!(
        "engine tick {} (Rule 30 generation {})",
        state.tick, state.rule30_generation
    );
    for row in &state.weight {
        println!("  weight {:14} {}", row.strategy, row.value);
    }
    println!(
        "frontier: {} of {FRONTIER_CAPACITY} rhythms",
        frontier.claim.len()
    );
    for c in &frontier.claim {
        println!(
            "  {} {} {:?} by {} at tick {} (robust {:.3})",
            c.id, c.status, c.rhythm, c.strategy, c.tick, c.robust
        );
    }
    let refuted = state
        .attempt
        .iter()
        .filter(|a| a.outcome == "refuted")
        .count();
    println!("refuted attempts: {refuted}");
    Ok(())
}
