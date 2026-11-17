Feature: echobraid-engine

  The features uorust admitted for the echobraid-engine package, each with the
  interface calls that realize it and the acceptance rows that accept it.

  @FT-01 @build
  Scenario: echobraid_engine: The EchoBraid self-improvement engine.
    Given the record Interface::featureEchobraidEngine
    When the interface call Interface::callEchobraidEngineStepRule30U64 is made
    And the interface call Interface::callEchobraidEngineDriverNextWord is made
    And the interface call Interface::callEchobraidEngineRhythmCycle is made
    And the interface call Interface::callEchobraidEngineRhythmSafe is made
    And the interface call Interface::callEchobraidEngineRhythmKappa is made
    And the interface call Interface::callEchobraidEngineRhythmLabel is made
    And the interface call Interface::callEchobraidEngineCalmScore is made
    And the interface call Interface::callEchobraidEngineHessian is made
    And the interface call Interface::callEchobraidEngineFrobenius is made
    And the interface call Interface::callEchobraidEngineEigenvalues is made
    And the interface call Interface::callEchobraidEngineRobustScore is made
    And the interface call Interface::callEchobraidEngineHessianSort is made
    And the interface call Interface::callEchobraidEngineStrategyName is made
    And the interface call Interface::callEchobraidEngineStrategyNamed is made
    And the interface call Interface::callEchobraidEngineStrategyPropose is made
    And the interface call Interface::callEchobraidEngineWeightsOf is made
    And the interface call Interface::callEchobraidEngineWeightsSchedule is made
    And the interface call Interface::callEchobraidEngineWeightsAllocation is made
    And the interface call Interface::callEchobraidEngineWeightsLearn is made
    And the interface call Interface::callEchobraidEnginePlan is made
    And the interface call Interface::callEchobraidEngineFrontierModule is made
    Then the acceptance row Interface::acEchobraidEngineEchobraidEngineRelaxIsSafeAndHessianIsSymmetric holds
    And the acceptance row Interface::acEchobraidEngineEchobraidEngineRule30MatchesTheCellRule holds
    And the acceptance row Interface::acRepoConformanceEngineEngineEb11 holds
    And the acceptance row Interface::acRepoConformanceEngineEngineEb12 holds
    And the acceptance row Interface::acRepoConformanceEngineEngineEb13 holds
