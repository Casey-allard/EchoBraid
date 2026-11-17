Feature: echobraid

  EchoBraid's own Prism application, in app/: every mechanism of the
  learning companion (drift, pause, breath, consent, memory, prompts, the
  AxiomScan logic and retention) is a LexLean definition the guest runs and
  a theorem the Lean kernel proves.

  @EB-01 @build
  Scenario: Semantic drift stays within 0 to 100 under every message and every rest, and a rest never raises it
    Given the EchoBraid model in app/src
    When the Lean kernel checks driftStepBounded, restStepBounded, restNeverRaisesDrift during lexlean verify
    Then semantic drift stays within 0 to 100 under every message and every rest, and a rest never raises it
    And the guest's dispatch decides through those same definitions

  @EB-02 @build
  Scenario: A pause is offered only when drift is divergent, and when it is offered the guide answers with the pause and nothing else
    Given the EchoBraid model in app/src
    When the Lean kernel checks pauseOnlyWhenDivergent, pauseComesFirst during lexlean verify
    Then a pause is offered only when drift is divergent, and when it is offered the guide answers with the pause and nothing else
    And the guest's dispatch decides through those same definitions

  @EB-03 @build
  Scenario: Coherence rises monotonically with drift and is divergent exactly when drift is at least 70
    Given the EchoBraid model in app/src
    When the Lean kernel checks divergentDecides, coherenceMonotone during lexlean verify
    Then coherence rises monotonically with drift and is divergent exactly when drift is at least 70
    And the guest's dispatch decides through those same definitions

  @EB-04 @build
  Scenario: Every breath countdown is at least one and within its cycle, and the 4-7-8 rhythm is safe with a 19-tick cycle
    Given the EchoBraid model in app/src
    When the Lean kernel checks countPositive, countWithinPhase, phaseBounded, relaxCycleIsNineteen, relaxIsSafe during lexlean verify
    Then every breath countdown is at least one and within its cycle, and the 4-7-8 rhythm is safe with a 19-tick cycle
    And the guest's dispatch decides through those same definitions

  @EB-05 @build
  Scenario: No sensor consent turns on without its own grant, revoke-all clears every flag, and a flag is always 0 or 1
    Given the EchoBraid model in app/src
    When the Lean kernel checks noSilentGrant, revokeAllClears, flagIsBinary, revokeWins during lexlean verify
    Then no sensor consent turns on without its own grant, revoke-all clears every flag, and a flag is always 0 or 1
    And the guest's dispatch decides through those same definitions

  @EB-06 @build
  Scenario: The memory game's Rule 30 tiles are always one of four, its prime-length levels never exceed 23, a wrong tile ends the round and victory comes only at level 7
    Given the EchoBraid model in app/src
    When the Lean kernel checks tileBounded, levelLengthBounded, victoryOnlyAtFinalLevel, wrongTileEndsRound, seedNeverDegenerate during lexlean verify
    Then the memory game's Rule 30 tiles are always one of four, its prime-length levels never exceed 23, a wrong tile ends the round and victory comes only at level 7
    And the guest's dispatch decides through those same definitions

  @EB-07 @build
  Scenario: Every reflection prompt is indexed by one of the primes 2, 3, 5 and 7, and every guide reply is one of ten fixed, non-directive texts
    Given the EchoBraid model in app/src
    When the Lean kernel checks guideIndexBounded, promptPrimeBounded, promptPrimeNotFour, promptPrimeNotSix during lexlean verify
    Then every reflection prompt is indexed by one of the primes 2, 3, 5 and 7, and every guide reply is one of ten fixed, non-directive texts
    And the guest's dispatch decides through those same definitions

  @EB-08 @build
  Scenario: Two quorums whose sizes sum past the membership plus the fault bound share at least one more member than the fault bound, and the ring buffer never overflows or reorders
    Given the EchoBraid model in app/src
    When the Lean kernel checks quorumIntersection, byzantineQuorum, ringStaysOrdered, ringNeverOverflows during lexlean verify
    Then two quorums whose sizes sum past the membership plus the fault bound share at least one more member than the fault bound, and the ring buffer never overflows or reorders
    And the guest's dispatch decides through those same definitions

  @EB-09 @build
  Scenario: Data kept under the 24-hour policy is erased once it is 24 hours old, immediate data is always erased and kept data never is
    Given the EchoBraid model in app/src
    When the Lean kernel checks dayPolicyErases, immediatePolicyAlwaysErases, keepPolicyNeverErases during lexlean verify
    Then data kept under the 24-hour policy is erased once it is 24 hours old, immediate data is always erased and kept data never is
    And the guest's dispatch decides through those same definitions

  @EB-10 @build
  Scenario: Every breathing rhythm the engine has written into the frontier is proved safe by the Lean kernel
    Given the EchoBraid model in app/src
    When the Lean kernel checks frontierSafe during lexlean verify
    Then every breathing rhythm the engine has written into the frontier is proved safe by the Lean kernel
    And the guest's dispatch decides through those same definitions
