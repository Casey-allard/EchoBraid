Feature: repo-model

  The features uorust admitted for the repo-model package, each with the
  interface calls that realize it and the acceptance rows that accept it.

  @FT-04 @build
  Scenario: repo_model: Typed registries parsed from model/*.toml.
    Given the record Interface::featureRepoModel
    When the interface call Interface::callRepoModelModelLoad is made
    And the interface call Interface::callRepoModelModelLoadFromRepoRoot is made
    And the interface call Interface::callRepoModelModelCheck is made
    And the interface call Interface::callRepoModelRepoRoot is made
    Then the acceptance row Interface::acRepoConformanceBddEverySomeTrueClaimCitesAnAuthorityCm03 holds
    And the acceptance row Interface::acRepoConformanceBddNoScenarioIsPendingCm02 holds
    And the acceptance row Interface::acRepoConformanceBddTheMetaGateIsFalsifiableCm02 holds
    And the acceptance row Interface::acRepoModelRepoModelEverySomeTrueClaimCitesAnAuthorityCm03 holds
    And the acceptance row Interface::acRepoModelRepoModelModelIsConsistentCm01 holds
    And the acceptance row Interface::acRepoModelRepoModelTheAuthorityRuleIsOneRule holds
    And the acceptance row Interface::acRepoModelRepoModelTheIdRegisterIsWellFormedCm02 holds
    And the acceptance row Interface::acXtaskXtaskAuditTheTemplateItselfPasses holds
    And the acceptance row Interface::acXtaskXtaskScaffoldScaffoldedCapabilitiesSatisfyTheMetaGate holds
    And the acceptance row Interface::acXtaskXtaskTheTemplateItselfPasses holds

  @FT-05 @build
  Scenario: repo_model::artifact: The checksum over an authority's committed artifact (R6).
    Given the record Interface::featureRepoModelArtifact
    When the interface call Interface::callRepoModelArtifactArtifactChecksum is made
    And the interface call Interface::callRepoModelArtifactModelCheckArtifacts is made
    Then the acceptance row Interface::acRepoConformanceBddEverySomeTrueClaimCitesAnAuthorityCm03 holds

  @FT-06 @build
  Scenario: repo_model::codegen: Generation of the documentation the model owns (R1, CM-01).
    Given the record Interface::featureRepoModelCodegen
    When the interface call Interface::callRepoModelCodegenRenderConformance is made
    Then the acceptance row Interface::acRepoModelRepoModelTheAuthorityRuleIsOneRule holds
