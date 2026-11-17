Feature: engine

  The self-improvement engine: each tick a Rule 30 automaton proposes
  breathing rhythms, Hessian sorting ranks them, the best safe and novel one is
  written into app/src/Braid/Frontier.lex.tex, and the Lean kernel must prove
  frontierSafe before the tick is committed and the page redeployed.

  @EB-11 @build
  Scenario: The engine's Rule 30 automaton never reaches the all-zero state and is the 64-cell rule 30 step
    Given the engine's Rule 30 driver seeded with the genesis ring
    When the suite exercises EB-11
    Then every cell of every step is left XOR (centre OR right), and ten thousand words contain no zero ring

  @EB-12 @build
  Scenario: Hessian sorting ranks every candidate rhythm by its calm score minus the curvature of the score around it, most robust first
    Given a lattice of candidate rhythms
    When the suite exercises EB-12
    Then they are ordered by calm score minus the Frobenius norm of its symmetric Hessian, most robust first

  @EB-13 @build
  Scenario: An engine tick admits at most one new rhythm, only a safe and novel one, and its Hebbian strategy weights stay within 1 to 16
    Given forty consecutive engine ticks from genesis
    When the suite exercises EB-13
    Then each tick admits at most one safe, novel rhythm and every Hebbian weight stays within 1 to 16
