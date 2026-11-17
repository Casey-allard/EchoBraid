Feature: repo-conformance

  The features uorust admitted for the repo-conformance package, each with the
  interface calls that realize it and the acceptance rows that accept it.

  @FT-02 @build
  Scenario: repo_conformance::meta: The honesty meta-gate (R2).
    Given the record Interface::featureRepoConformanceMeta
    When the interface call Interface::callRepoConformanceMetaHonestyReportIsClean is made
    And the interface call Interface::callRepoConformanceMetaCheckHonesty is made
    Then the acceptance row Interface::acRepoConformanceBddEveryIdHasAScenarioAndATestCm02 holds
    And the acceptance row Interface::acRepoConformanceBddTheMetaGateIsFalsifiableCm02 holds
    And the acceptance row Interface::acXtaskXtaskScaffoldScaffoldedCapabilitiesSatisfyTheMetaGate holds

  @FT-03 @build
  Scenario: repo_conformance::runner: Reading features/suites/*.feature (R3).
    Given the record Interface::featureRepoConformanceRunner
    When the interface call Interface::callRepoConformanceRunnerScenariosIn is made
    And the interface call Interface::callRepoConformanceRunnerSuiteReportIds is made
    And the interface call Interface::callRepoConformanceRunnerWorkspaceTestNames is made
    Then the acceptance row Interface::acRepoConformanceBddEveryIdHasAScenarioAndATestCm02 holds
    And the acceptance row Interface::acRepoConformanceBddNoScenarioIsPendingCm02 holds
    And the acceptance row Interface::acRepoConformanceBddTheMetaGateIsFalsifiableCm02 holds
